#!/usr/bin/env node
/**
 * compile_typescript.mjs
 *
 * Drop-in replacement for the per-type `json2ts` loop in
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 *
 * Takes the combined schemas.json (keyed by top-level type name) produced by
 * `cargo run --bin export_schemas` and emits one `.d.ts` per type into the
 * output directory, with proper cross-file `import type { X } from './X'`
 * statements for references to other top-level types. Inlined $defs for
 * non-top-level helper types are preserved as before.
 *
 * Why this exists: running `json2ts` on each type's schema in isolation
 * produces a stub reference type (`{ type, [k:string]: unknown }`) when the
 * same name is defined in a sibling schema (since the tool cannot infer the
 * link). tsup then bundles these stubs alongside the full definitions from
 * the sibling files, and rollup-dts disambiguates with `$1`/`$2`/`$3` name
 * suffixes. The result is an unusable bundled `.d.ts` where `UnifiedStep`
 * is constructed from stubs and consumers cannot narrow by the `type`
 * discriminator.
 *
 * The fix: for every `$ref: "#/$defs/X"` where `X` is itself a top-level
 * type, drop the ref in favour of json2ts's `tsType` escape hatch
 * (`{ tsType: "X" }`, which json2ts emits as the bare identifier `X`),
 * strip those externalized entries from the local `$defs`, then prepend
 * `import type { X } from './X'`. The bundled output then has exactly one
 * declaration per type.
 *
 * Usage:
 *   node compile_typescript.mjs --input SCHEMAS_JSON --output OUT_DIR
 */

import { readFileSync, writeFileSync, mkdirSync, rmSync, existsSync } from 'node:fs';
import { join, resolve, dirname } from 'node:path';
import { compile } from 'json-schema-to-typescript';
import { toSafeString } from 'json-schema-to-typescript/dist/src/utils.js';
import { format as prettierFormat } from 'prettier';

// ─── CLI ────────────────────────────────────────────────────────────────────

function parseArgs(argv) {
  const args = {};
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === '--input' || a === '-i') args.input = argv[++i];
    else if (a === '--output' || a === '-o') args.output = argv[++i];
    else if (a === '--help' || a === '-h') args.help = true;
  }
  return args;
}

const args = parseArgs(process.argv.slice(2));
if (args.help || !args.input || !args.output) {
  console.error('Usage: compile_typescript.mjs --input SCHEMAS_JSON --output OUT_DIR');
  process.exit(args.help ? 0 : 1);
}

const schemasPath = resolve(args.input);
const outDir = resolve(args.output);

if (!existsSync(schemasPath)) {
  console.error(`ERROR: input not found: ${schemasPath}`);
  process.exit(1);
}

// ─── Load ───────────────────────────────────────────────────────────────────

const schemas = JSON.parse(readFileSync(schemasPath, 'utf8'));
const typeNames = new Set(Object.keys(schemas));
console.log(`Processing ${typeNames.size} top-level types...`);

// json-schema-to-typescript's internal `toSafeString` normalization uppercases
// letters that follow digits (e.g. `A11yAction` → `A11YAction`). `customName`
// doesn't bypass this: the name returned from `customName` is still run
// through the same normalization before being emitted. For any top-level
// type whose name is mangled by this transform, we post-process the output
// to rename it back to the schema title. Keep the Rust type names canonical.
const renameMap = new Map();
for (const name of typeNames) {
  const normalized = toSafeString(name);
  if (normalized !== name) renameMap.set(normalized, name);
}
if (renameMap.size > 0) {
  console.log(
    `Will restore ${renameMap.size} type name(s) mangled by json2ts normalization: ` +
      [...renameMap.entries()].map(([n, o]) => `${n}→${o}`).join(', '),
  );
}

/** Word-boundary rename — keeps partial matches like `Foo` inside `FooBar` intact. */
function applyRenames(source) {
  if (renameMap.size === 0) return source;
  let out = source;
  for (const [normalized, original] of renameMap) {
    // Match only as a standalone identifier (not a substring of a longer one).
    const re = new RegExp(`\\b${normalized}\\b`, 'g');
    out = out.replace(re, original);
  }
  return out;
}

// ─── Rewrite pass ───────────────────────────────────────────────────────────
// For each top-level type T:
//   1. Walk its schema tree
//   2. Mark `$ref: "#/$defs/X"` as external when X is another top-level type
//      (but not T itself — self-references stay local)
//   3. Collect the set of externalized refs → used later for import injection
//   4. Strip externalized entries from T's local $defs
//   5. Turn each marked ref into `{ tsType: "X" }` (see externalRefsToTsType)
//
// Collision handling: when two Rust types share a short name (e.g.
// `tree_events::ActionType` and `execution::ActionType`), one of them is
// renamed at the JSON-Schema layer via `#[schemars(title = "X")]`. The
// renamed type appears as top-level key `X`, but inline `$defs[ActionType]`
// entries (emitted under the Rust path name) still reference it — so a naive
// lookup of `refName` against `typeNames` matches the non-renamed sibling
// and rewrites the ref to the wrong file. To disambiguate, we consult the
// inline `$defs[refName].title`: if that title itself is a top-level key,
// use it in preference to the raw `refName`.

/**
 * Resolve a local `$defs` ref name to the top-level type name to import,
 * honoring `title`-based schemars renames. Returns null if the ref does not
 * point at a top-level type.
 *
 * @param {string} refName
 * @param {Record<string, unknown> | undefined} localDefs
 * @returns {string | null}
 */
function resolveTopLevelRefName(refName, localDefs) {
  const def = localDefs?.[refName];
  if (def && typeof def === 'object' && typeof def.title === 'string' && typeNames.has(def.title)) {
    return def.title;
  }
  if (typeNames.has(refName)) return refName;
  return null;
}

/**
 * Internal-only `$ref` sentinel for "this points at a sibling type's own
 * file". It stays a `$ref` (rather than becoming `tsType` immediately) so
 * that `flattenRefSiblingsToAllOf` — which keys off `$ref` — still sees it
 * and can lift discriminator constraints out of the ref node. The sentinel
 * is erased by `externalRefsToTsType` before json2ts ever sees the schema.
 */
const EXTERNAL_REF_PREFIX = '#/$qontinui-external/';

/**
 * @param {unknown} node
 * @param {string} selfName
 * @param {Set<string>} collected
 * @param {Record<string, unknown> | undefined} localDefs
 * @returns {unknown}
 */
function rewriteRefs(node, selfName, collected, localDefs) {
  if (Array.isArray(node)) {
    return node.map((n) => rewriteRefs(n, selfName, collected, localDefs));
  }
  if (node === null || typeof node !== 'object') return node;
  if (typeof node.$ref === 'string') {
    const m = node.$ref.match(/^#\/\$defs\/(.+)$/);
    if (m) {
      const resolved = resolveTopLevelRefName(m[1], localDefs);
      if (resolved && resolved !== selfName) {
        collected.add(resolved);
        const out = { ...node, $ref: `${EXTERNAL_REF_PREFIX}${resolved}` };
        return out;
      }
    }
  }
  const out = {};
  for (const [k, v] of Object.entries(node)) {
    out[k] = rewriteRefs(v, selfName, collected, localDefs);
  }
  return out;
}

/**
 * Replace every externalized ref with json2ts's `tsType` escape hatch, which
 * emits the given string verbatim as the type.
 *
 * This runs LAST of the three schema passes, after `flattenRefSiblingsToAllOf`
 * has moved any discriminator constraint off the ref node — `tsType`
 * supersedes every other directive in json2ts (`typesOfSchema` returns
 * `CUSTOM_TYPE` and stops), so a `properties`/`required` sibling left in
 * place here would be silently dropped.
 *
 * ── WHY NOT A REAL `$ref` TO A SIBLING FILE ─────────────────────────────────
 *
 * The previous implementation wrote each rewritten schema to a temp directory
 * and pointed the ref at `./X.schema.json`, relying on
 * `declareExternallyReferenced: false` to stop json2ts redeclaring the target.
 * That shipped a whole class of dangling identifiers to npm:
 *
 *   PolicyEvaluation.d.ts       overallStatus: PolicyStatus1;
 *   TaskCompletionResult.d.ts   last_results?: IterationVerificationResults1 | null;
 *
 * json2ts caches parsed ASTs by schema-object IDENTITY and uniquifies names
 * with a `usedNames` counter, so the same type reached as two different
 * objects gets a second name — and `declareExternallyReferenced: false`
 * then declines to declare it, leaving a reference to a type that does not
 * exist. `$RefParser` hands back distinct objects for the same target
 * routinely:
 *
 *   * once per referencing FILE (`PolicyEvaluation` refs `PolicyStatus`
 *     directly and again through `ConjunctEvaluation.schema.json`), and
 *   * once per ref site that carries SIBLING KEYS — `{ $ref, description }`
 *     is merged into a fresh object, which is why `TaskCompletionResult`
 *     got one name for the ref site with a doc comment and another for the
 *     bare ones.
 *
 * `tsType` removes the mechanism rather than the symptom: no file is
 * resolved, no second object exists, no name is ever allocated for the
 * imported type, so a suffixed alias cannot be emitted. It also removes the
 * need for a temp directory of sibling schemas entirely.
 *
 * @param {unknown} node
 * @returns {unknown}
 */
function externalRefsToTsType(node) {
  if (Array.isArray(node)) return node.map(externalRefsToTsType);
  if (node === null || typeof node !== 'object') return node;
  if (typeof node.$ref === 'string' && node.$ref.startsWith(EXTERNAL_REF_PREFIX)) {
    const { $ref, ...rest } = node;
    return { ...rest, tsType: $ref.slice(EXTERNAL_REF_PREFIX.length) };
  }
  const out = {};
  for (const [k, v] of Object.entries(node)) {
    out[k] = externalRefsToTsType(v);
  }
  return out;
}

/**
 * Flatten `{ $ref, properties, required, type }` siblings into `allOf`.
 *
 * Schemars emits internally-tagged enum variants as:
 *   { "$ref": "#/$defs/CommandStep",
 *     "properties": { "type": { "const": "command", "type": "string" } },
 *     "required": ["type"],
 *     "type": "object" }
 *
 * That shape means "must match CommandStep AND have type='command'". But
 * json-schema-to-typescript treats `$ref + sibling properties` inconsistently
 * — the discriminator constraint is silently dropped, and CanonicalStep
 * lands in the generated TS as `CommandStep | PromptStep | …`, losing the
 * `type` field entirely. Consumers then cannot narrow by tag:
 * `(step: CanonicalStep) => step.type` fails because `type` is on neither
 * variant.
 *
 * The fix is a mechanical rewrite: any object that mixes `$ref` with
 * `properties`/`required`/`type` gets rewritten as `allOf: [{$ref}, {...}]`,
 * which json2ts handles correctly, producing
 * `(CommandStep & { type: "command" }) | (PromptStep & { type: "prompt" }) | …`
 * — a proper discriminated union.
 */
function flattenRefSiblingsToAllOf(node) {
  if (Array.isArray(node)) return node.forEach(flattenRefSiblingsToAllOf);
  if (node === null || typeof node !== 'object') return;
  const hasRef = typeof node.$ref === 'string';
  const siblings = Object.keys(node).filter(
    (k) => k !== '$ref' && k !== 'description' && k !== 'title',
  );
  if (hasRef && siblings.length > 0) {
    const inline = {};
    for (const k of siblings) {
      inline[k] = node[k];
      delete node[k];
    }
    // Force the inline constraint to be a closed object — without this,
    // json2ts emits `[k: string]: unknown` for both the `$ref` target and
    // the inline constraint, and the resulting intersection gets a
    // duplicate index signature that tsup's dts bundler rejects as
    // "Syntax not yet supported".
    inline.additionalProperties = false;
    const ref = node.$ref;
    delete node.$ref;
    node.allOf = [{ $ref: ref }, inline];
  }
  for (const v of Object.values(node)) flattenRefSiblingsToAllOf(v);
}

/**
 * Promote fields with a `default` to `required`.
 *
 * Rationale: in Rust, `#[serde(default)]` on a non-`Option<T>` field means
 * "substitute this default if the wire JSON omits the field." After
 * deserialization, the field is always present. schemars emits the JSON
 * Schema with `default: <value>` but does not add the field to `required`
 * — and json-schema-to-typescript renders that as `name?: T`, i.e.
 * `T | undefined`.
 *
 * That TS shape is a lie about the runtime contract: after any Rust→JSON
 * round trip, the field is always there. Consumers forced to write `x ??
 * []` at every access site are paying a cost for a condition that cannot
 * actually occur. So at codegen time, we walk each object schema and add
 * any default-having property to `required`. json2ts then emits `name: T`.
 *
 * This changes the schema contract from "wire-optional" to "runtime-
 * required". Lenient deserialization is unaffected — Rust still accepts
 * JSON that omits the field and substitutes the default. Producers that
 * want to emit minimal JSON can still omit defaulted fields on the wire.
 */
function promoteDefaultsToRequired(node) {
  if (Array.isArray(node)) return node.forEach(promoteDefaultsToRequired);
  if (node === null || typeof node !== 'object') return;
  if (node.properties && typeof node.properties === 'object') {
    const required = new Set(Array.isArray(node.required) ? node.required : []);
    for (const [propName, propSchema] of Object.entries(node.properties)) {
      if (propSchema && typeof propSchema === 'object' && 'default' in propSchema) {
        required.add(propName);
      }
    }
    if (required.size > 0) node.required = [...required];
  }
  for (const v of Object.values(node)) promoteDefaultsToRequired(v);
}

/** @type {Map<string, { schema: unknown, imports: Set<string>, declarable: Set<string> }>} */
const processed = new Map();

for (const [name, schema] of Object.entries(schemas)) {
  const imports = new Set();
  // Pass the schema's own $defs so title-renamed inline defs can be resolved
  // to their true top-level type names during ref rewriting.
  const localDefs = (schema && typeof schema === 'object') ? schema.$defs : undefined;
  let rewritten = rewriteRefs(schema, name, imports, localDefs);
  // Convert `$ref` + sibling property constraints into `allOf` so json2ts
  // preserves discriminator literals on tagged-union variants. Must run
  // before the compile call; order relative to `promoteDefaultsToRequired`
  // doesn't matter.
  flattenRefSiblingsToAllOf(rewritten);
  promoteDefaultsToRequired(rewritten);
  // Erase the external-ref sentinel in favour of `tsType`. Strictly after
  // flattenRefSiblingsToAllOf — `tsType` supersedes sibling constraints.
  rewritten = externalRefsToTsType(rewritten);
  // Strip externalized $defs — we don't want stubs emitted for them.
  // Match by both raw def-key and (when present) title, since the imports
  // set stores the resolved top-level name (which may be the title, not the
  // key). Without this, title-renamed inline defs linger as stub declarations.
  if (rewritten.$defs) {
    const kept = {};
    for (const [k, v] of Object.entries(rewritten.$defs)) {
      const defTitle = (v && typeof v === 'object') ? v.title : undefined;
      if (imports.has(k) || (typeof defTitle === 'string' && imports.has(defTitle))) {
        continue;
      }
      kept[k] = v;
    }
    if (Object.keys(kept).length === 0) delete rewritten.$defs;
    else rewritten.$defs = kept;
  }
  // Ensure the top-level type carries its name for json2ts output.
  if (!rewritten.title) rewritten.title = name;
  // The names json2ts is ALLOWED to declare in this file: the root type, plus
  // whatever local `$defs` survived the externalized strip above. Both the raw
  // key and the `title` are admissible (`customName` prefers the title), and
  // both are recorded in their `toSafeString` form too because that
  // normalization is applied to every emitted name and `applyRenames` only
  // reverses it for TOP-LEVEL types. Anything the emitted file declares that is
  // not in this set is a name json2ts INVENTED — see the fork guard below.
  const declarable = new Set([name]);
  if (rewritten.$defs) {
    for (const [defKey, defSchema] of Object.entries(rewritten.$defs)) {
      const defTitle =
        defSchema && typeof defSchema === 'object' && typeof defSchema.title === 'string'
          ? defSchema.title
          : undefined;
      for (const candidate of [defKey, defTitle]) {
        if (!candidate) continue;
        declarable.add(candidate);
        declarable.add(toSafeString(candidate));
      }
    }
  }
  processed.set(name, { schema: rewritten, imports, declarable });
}

// ─── Compile pass ───────────────────────────────────────────────────────────

mkdirSync(outDir, { recursive: true });

const BANNER = `/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with \`just generate-types\` or
 * \`qontinui-runner/src-tauri/scripts/generate_types.sh\`.
 */`;

const compileOpts = {
  // No schema reaches json2ts with a file-relative `$ref` any more (see
  // externalRefsToTsType), so nothing is ever resolved against this — it is
  // set to the output dir purely because json2ts requires a base path.
  cwd: outDir,
  // Must be TRUE, despite the name reading like the opposite of what we want.
  //
  // json2ts's flag does not mean "declare types that came from another file".
  // In `generator.ts` it is the escape hatch on a much blunter predicate:
  //
  //     ast.standaloneName === rootASTName || options.declareExternallyReferenced
  //
  // With the flag off, the ONLY named object json2ts declares in a file is the
  // root type itself. Every other named object — including a helper that lives
  // in this schema's OWN `$defs` and was never externalized — is referenced by
  // name and then silently not declared. That is how `IrPageSpec.d.ts` came to
  // say `apiAssertions?: IrApiCheck[] | null` with no `IrApiCheck` anywhere:
  // `IrApiCheck` is a `$defs`-only helper (not registered top-level in the
  // runner's `schema_export.rs`, so it gets no file of its own and no import),
  // and the flag suppressed its local declaration. Same for `SpecChanged` and
  // `ThresholdConfig`. Union and enum helpers escaped the bug because
  // `declareNamedTypes` emits those unconditionally — only object helpers were
  // dropped, which is why this went unnoticed for so long.
  //
  // Turning it on is safe precisely because `tsType` means there is nothing
  // "external" left in the schema json2ts sees: the reachable named closure is
  // the root type plus its genuinely-local `$defs` helpers, which is exactly
  // what this file should declare.
  declareExternallyReferenced: true,
  bannerComment: '',
  // Disable json2ts's *internal* Prettier pass and run our own pinned pass at
  // the end of each file instead (see PRETTIER_OPTS below). json2ts loads
  // Prettier lazily and silently no-ops the format step when its bundled
  // `typescript` parser fails to resolve in some environments — which is
  // exactly what happened in the `schema-drift` CI job: locally the union
  // members wrapped one-per-line (Prettier ran), but in CI json2ts emitted the
  // raw single-line `A | B | C` union (Prettier did NOT run), so the committed
  // multi-line files looked "drifted" on every PR even though no schema value
  // changed. Owning the format pass here makes codegen output byte-identical
  // across machines and CI.
  format: false,
  // Keep object shapes open (emit `[k: string]: unknown` catch-all index
  // signature when schemars does not specify `additionalProperties: false`).
  // This matches the Rust wire semantics (serde tolerates unknown keys by
  // default) and preserves round-trip-via-TS flexibility. Narrowing by the
  // `type` discriminator still works because `flattenRefSiblingsToAllOf`
  // rewrites `$ref + properties` into `allOf`, which json2ts turns into
  // `(CommandStep & { type: "command" }) | …` — intersection members carry
  // typed fields even though CommandStep alone has the open index sig.
  additionalProperties: true,
  enableConstEnums: false,
  strictIndexSignatures: false,
  // Preserve the exact schema `title` for the exported type name. Without
  // this, json2ts's internal `toSafeString` normalization applies `startCase`
  // logic that turns things like `A11yAction` into `A11YAction` (digit+lower
  // followed by upper → upper+upper). Returning the raw title keeps our
  // type names identical to the Rust type names that schemars emitted.
  customName(schema, keyNameFromDefinition) {
    return schema.title ?? keyNameFromDefinition;
  },
  // unreachableDefinitions keeps helper types that aren't reachable via refs;
  // default is fine.
};

// Pinned Prettier options for the explicit format pass. These are Prettier's
// own defaults written out longhand so the generated `.d.ts` formatting is
// reproducible regardless of which `^3.x` Prettier `npm install` happens to
// resolve (the lockfile is .gitignore'd). `parser: 'typescript'` is required
// because we format raw strings, not files with a `.ts` extension.
const PRETTIER_OPTS = {
  parser: 'typescript',
  printWidth: 80,
  tabWidth: 2,
  useTabs: false,
  semi: true,
  singleQuote: false,
  quoteProps: 'as-needed',
  trailingComma: 'all',
  bracketSpacing: true,
  arrowParens: 'always',
  endOfLine: 'lf',
};

/**
 * Strip `export type X = ...;` and `export interface X { ... }` blocks whose
 * name is in `importedNames` — a declaration of an imported name would shadow
 * the `import type { X } from './X'` we inject and re-create the duplicate
 * declarations this script exists to remove.
 *
 * Since the `tsType` rewrite this is a backstop rather than the load-bearing
 * pass it once was: json2ts is no longer handed any external schema whose
 * body it could redeclare. It still guards the one remaining way an imported
 * name could be declared locally — a `$defs` entry that survived the
 * externalized-defs strip below under a name that collides with an import.
 *
 * This scanner walks the file, finds each `export type|interface NAME`, and
 * deletes the full block when NAME is externally imported. The deletion
 * stays textual — no AST — which keeps the script dependency surface small
 * and the behavior predictable across json2ts versions.
 */
/**
 * Count net brace/paren depth change in a line, ignoring strings and comments.
 * Quick-and-dirty but sufficient for json-schema-to-typescript output, which
 * uses only straight quotes and no template literals in emitted types.
 */
function depthDelta(line) {
  let depth = 0;
  let inSingle = false;
  let inDouble = false;
  let inLineComment = false;
  let inBlockComment = false;
  for (let i = 0; i < line.length; i++) {
    const c = line[i];
    const next = line[i + 1];
    if (inLineComment) break;
    if (inBlockComment) {
      if (c === '*' && next === '/') {
        inBlockComment = false;
        i++;
      }
      continue;
    }
    if (inSingle) {
      if (c === '\\') i++;
      else if (c === "'") inSingle = false;
      continue;
    }
    if (inDouble) {
      if (c === '\\') i++;
      else if (c === '"') inDouble = false;
      continue;
    }
    if (c === '/' && next === '/') {
      inLineComment = true;
      continue;
    }
    if (c === '/' && next === '*') {
      inBlockComment = true;
      i++;
      continue;
    }
    if (c === "'") inSingle = true;
    else if (c === '"') inDouble = true;
    else if (c === '{' || c === '(' || c === '[') depth++;
    else if (c === '}' || c === ')' || c === ']') depth--;
  }
  return depth;
}

function stripDuplicateExports(source, importedNames) {
  if (importedNames.size === 0) return source;
  const lines = source.split('\n');
  const result = [];
  let i = 0;
  while (i < lines.length) {
    // Collect any JSDoc block that precedes an export, so we remove it with
    // the export it belongs to.
    let jsdocStart = -1;
    if ((lines[i] ?? '').trimStart().startsWith('/**')) {
      jsdocStart = i;
      while (i < lines.length && !lines[i].includes('*/')) i++;
      if (i < lines.length) i++; // consume closing */
      if (i >= lines.length) {
        // unterminated JSDoc — keep as-is
        result.push(...lines.slice(jsdocStart));
        break;
      }
    }
    const next = lines[i] ?? '';
    const typeM = next.match(/^export\s+type\s+([A-Za-z_$][\w$]*)\s*=/);
    const ifaceM = next.match(/^export\s+interface\s+([A-Za-z_$][\w$]*)\b/);
    const match = (typeM && importedNames.has(typeM[1])) ? typeM
      : (ifaceM && importedNames.has(ifaceM[1])) ? ifaceM
      : null;
    if (match) {
      // Consume the entire declaration. For `export interface X { ... }`,
      // brace depth opens and closes. For `export type X = ...;`, either
      // the RHS is a simple type that terminates on the same or next line
      // with a semicolon at depth 0, or it's a nested union/intersection
      // whose depth fluctuates and terminates with `;` at depth 0.
      // Termination rule: consume until depth returns to 0 AND the line
      // ends with `;` OR `}`. This covers both `export type = ...;` and
      // `export interface X { ... }` (where `}` at depth-0 is the end).
      let depth = 0;
      while (i < lines.length) {
        const line = lines[i];
        depth += depthDelta(line);
        const terminates = /[;}]\s*$/.test(line);
        i++;
        if (depth === 0 && terminates) break;
      }
      continue;
    }
    // Not a duplicate — restore the JSDoc if we captured one and keep the
    // next statement (which will be picked up on the next iteration).
    if (jsdocStart >= 0) result.push(...lines.slice(jsdocStart, i));
    if (i < lines.length) result.push(lines[i]);
    i++;
  }
  return result.join('\n');
}

/**
 * Every top-level exported declaration name in one emitted file body.
 *
 * Called on the per-file string json2ts produced — after `applyRenames` and
 * `stripDuplicateExports`, but BEFORE the banner and the `import type` lines
 * are prepended — so only json2ts's own declarations are in scope, never an
 * import this script injected. json2ts emits every declaration at column 0
 * (`export interface X {`, `export type X =`), which is what the `^` anchor
 * keys off; nested members are indented and cannot match.
 *
 * @param {string} source
 * @returns {string[]}
 */
function declaredNames(source) {
  const re = /^export\s+(?:declare\s+)?(?:interface|type|enum|const\s+enum)\s+([A-Za-z_$][\w$]*)/gm;
  return [...source.matchAll(re)].map((m) => m[1]);
}

/**
 * One emitted declaration: the name, and the top-level type whose file
 * declares it. Collected as each file is emitted; asserted over once
 * generation completes (see the fork guard).
 *
 * @type {{ decl: string, owner: string }[]}
 */
const declarations = [];

let emitted = 0;
let failed = 0;
for (const [name, { schema, imports }] of processed) {
  try {
    // `compile(schema, typeName)` — the name is used as the exported type
    // name. Passing the name matches json2ts CLI behavior which derives it
    // from the filename.
    let ts = await compile(schema, name, compileOpts);
    // Restore any type names mangled by json2ts's toSafeString
    // (e.g. `A11yAction` → `A11YAction`). This must happen BEFORE the
    // duplicate-exports strip so the match set uses the original names.
    ts = applyRenames(ts);
    // Remove any externally-imported type declarations that json2ts
    // re-emitted anyway (happens for union targets).
    ts = stripDuplicateExports(ts, imports);
    // Collapse runs of 2+ blank lines that the stripping may have left behind.
    ts = ts.replace(/\n{3,}/g, '\n\n');

    // Record what this file declares, for the fork guard below.
    for (const decl of declaredNames(ts)) declarations.push({ decl, owner: name });

    const importLines = [...imports]
      .sort()
      .map((dep) => `import type { ${dep} } from './${dep}';`)
      .join('\n');
    const out =
      BANNER +
      '\n\n' +
      (importLines ? importLines + '\n\n' : '') +
      ts.trimStart();
    // Explicit, pinned Prettier pass — the single source of truth for output
    // formatting now that json2ts's internal pass is disabled (`format: false`
    // above). PRETTIER_OPTS hard-codes the defaults so the output can't shift
    // when the transitive Prettier version floats (the lockfile is .gitignore'd,
    // so each `npm install` may resolve a different `^3.x`).
    const formatted = await prettierFormat(out, PRETTIER_OPTS);
    writeFileSync(join(outDir, `${name}.d.ts`), formatted);
    emitted++;
  } catch (err) {
    console.error(`  FAILED ${name}: ${err.message}`);
    failed++;
  }
}

// ─── Fork guard ─────────────────────────────────────────────────────────────
//
// `declareExternallyReferenced: true` (see compileOpts) converts one failure
// mode into another, and only one of the two is visible to `tsc`.
//
// With the flag OFF, a duplicate AST for the same type got a `usedNames`
// suffix (`Delta1`) and was then NOT declared — a DANGLING IDENTIFIER, which
// the `tsconfig.generated.json` gate reds on. With the flag ON, that same
// duplicate AST is declared: the file gets a second, structurally identical
// copy under the invented name, `Delta.kids` is typed `Delta1[]` instead of
// `Delta[]`, and the result compiles perfectly clean. A FORKED TYPE is
// invisible to a type-checker by construction — TypeScript is structural, so
// the two copies are mutually assignable and nothing downstream complains
// until the two halves of the fork drift apart.
//
// So the flag flip needs its own guard, at the layer that can still see the
// difference: the generator itself, which knows which names it is ENTITLED to
// emit. Three invariants over the full emitted set:
//
//   (a) no declaration name appears in more than one output file;
//   (b) no declaration name equals a top-level type name other than the
//       file's own (that name has its own file — a local copy forks it);
//   (c) no declaration name falls outside the file's `declarable` closure
//       (root type + surviving local `$defs`). This is the one that catches
//       the suffix case above: `Delta1` is neither the root nor any `$defs`
//       key, so it can only have been invented by json2ts's uniquifier.
//
// (c) is not redundant with (a)/(b) — it is the only one of the three that
// fires on the shape this guard was written for, because an invented `Delta1`
// lives in exactly one file and is not a top-level type name. (a) and (b) are
// kept because they catch the other direction: a helper duplicated across
// files, and a local redeclaration of a type that already has its own module.
//
// All three currently hold at zero violations. This FAILS rather than warns —
// a warning in a generator nobody watches is the same defect class as a gate
// that cannot go red. It runs before the barrel is written, so a forked tree
// never gets the `index.ts` consumers import.

/** @type {Map<string, string[]>} declaration name → owning type name(s) */
const declarationSites = new Map();
for (const { decl, owner } of declarations) {
  const sites = declarationSites.get(decl);
  if (sites) sites.push(owner);
  else declarationSites.set(decl, [owner]);
}

/** @type {string[]} */
const forkViolations = [];

for (const [decl, owners] of declarationSites) {
  if (owners.length > 1) {
    forkViolations.push(
      `${decl}: declared in ${owners.length} files (${owners
        .map((o) => `${o}.d.ts`)
        .join(', ')}). Each copy is a separate structural type.`,
    );
  }
}

for (const { decl, owner } of declarations) {
  if (typeNames.has(decl) && decl !== owner) {
    forkViolations.push(
      `${decl}: declared in ${owner}.d.ts, but it is a top-level type with its ` +
        `own ${decl}.d.ts. The local copy forks it — it should have been imported.`,
    );
    // (c) would fire on the same declaration for the same underlying reason;
    // one report per offending declaration is enough.
    continue;
  }
  const declarable = processed.get(owner)?.declarable;
  if (declarable && !declarable.has(decl)) {
    forkViolations.push(
      `${decl}: declared in ${owner}.d.ts, but that file's schema names no such ` +
        `type — its declarable closure is {${[...declarable].sort().join(', ')}}. ` +
        `Either json2ts invented the name (a \`usedNames\` suffix on a duplicate ` +
        `AST of a type it already parsed) or a nested schema \`title\` claimed it; ` +
        `either way ${owner}.d.ts carries a second structural copy of that type.`,
    );
  }
}

if (forkViolations.length > 0) {
  console.error(
    `\nERROR: forked type declarations in ${outDir}\n` +
      `${forkViolations.length} violation(s) — the generated tree declares names it must not:\n`,
  );
  for (const v of forkViolations) console.error(`  - ${v}`);
  console.error(
    '\nA forked type compiles clean (TypeScript is structural) but is a real defect:\n' +
      'two declarations that are one type today and drift apart tomorrow. Fix the\n' +
      'schema or the rewrite passes in this script — do not silence this check.\n',
  );
  process.exit(1);
}

// ─── Barrel ─────────────────────────────────────────────────────────────────

const barrelLines = [...processed.keys()]
  .sort()
  .map((n) => `export type { ${n} } from './${n}';`);
const barrel =
  '// Auto-generated by compile_typescript.mjs — do not edit\n' +
  barrelLines.join('\n') +
  '\n';
writeFileSync(join(outDir, 'index.ts'), barrel);

// ─── Cleanup ────────────────────────────────────────────────────────────────

// Older revisions staged a `.tmp-schemas/` directory of sibling schemas inside
// the output dir for `$ref` resolution. `tsType` removed the need for it, but
// `generate_types.sh` only sweeps `*.d.ts` + `index.ts`, so an existing tree
// would otherwise linger in a checkout that was generated before this change.
rmSync(join(outDir, '.tmp-schemas'), { recursive: true, force: true });

console.log(`Emitted ${emitted} .d.ts files to ${outDir}`);
if (failed > 0) {
  console.error(`${failed} type(s) failed to compile`);
  process.exit(1);
}
