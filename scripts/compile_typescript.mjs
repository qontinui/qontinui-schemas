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
 * The fix: rewrite every `$ref: "#/$defs/X"` where `X` is itself a
 * top-level type into a file-relative `$ref: "./X.schema.json"`, strip
 * those externalized entries from the local `$defs`, compile with
 * `declareExternallyReferenced: false` (so json2ts references `X` by name
 * without redeclaring it), then prepend `import type { X } from './X'`.
 * The bundled output then has exactly one declaration per type.
 *
 * Usage:
 *   node compile_typescript.mjs --input SCHEMAS_JSON --output OUT_DIR
 */

import { readFileSync, writeFileSync, mkdirSync, rmSync, existsSync } from 'node:fs';
import { join, resolve, dirname } from 'node:path';
import { compile } from 'json-schema-to-typescript';
import { toSafeString } from 'json-schema-to-typescript/dist/src/utils.js';

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
//   2. Replace `$ref: "#/$defs/X"` with `$ref: "./X.schema.json"` when X is
//      another top-level type (but not T itself — self-references stay local)
//   3. Collect the set of externalized refs → used later for import injection
//   4. Strip externalized entries from T's local $defs

/**
 * @param {unknown} node
 * @param {string} selfName
 * @param {Set<string>} collected
 * @returns {unknown}
 */
function rewriteRefs(node, selfName, collected) {
  if (Array.isArray(node)) {
    return node.map((n) => rewriteRefs(n, selfName, collected));
  }
  if (node === null || typeof node !== 'object') return node;
  if (typeof node.$ref === 'string') {
    const m = node.$ref.match(/^#\/\$defs\/(.+)$/);
    if (m) {
      const refName = m[1];
      if (typeNames.has(refName) && refName !== selfName) {
        collected.add(refName);
        const out = { ...node, $ref: `./${refName}.schema.json` };
        return out;
      }
    }
  }
  const out = {};
  for (const [k, v] of Object.entries(node)) {
    out[k] = rewriteRefs(v, selfName, collected);
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

/** @type {Map<string, { schema: unknown, imports: Set<string> }>} */
const processed = new Map();
/** Temp directory — json2ts's $RefParser resolves ./X.schema.json from here. */
const tmpDir = join(outDir, '.tmp-schemas');
rmSync(tmpDir, { recursive: true, force: true });
mkdirSync(tmpDir, { recursive: true });

for (const [name, schema] of Object.entries(schemas)) {
  const imports = new Set();
  const rewritten = rewriteRefs(schema, name, imports);
  // Convert `$ref` + sibling property constraints into `allOf` so json2ts
  // preserves discriminator literals on tagged-union variants. Must run
  // before the compile call; order relative to `promoteDefaultsToRequired`
  // doesn't matter.
  flattenRefSiblingsToAllOf(rewritten);
  promoteDefaultsToRequired(rewritten);
  // Strip externalized $defs — we don't want stubs emitted for them.
  if (rewritten.$defs) {
    const kept = {};
    for (const [k, v] of Object.entries(rewritten.$defs)) {
      if (!imports.has(k)) kept[k] = v;
    }
    if (Object.keys(kept).length === 0) delete rewritten.$defs;
    else rewritten.$defs = kept;
  }
  // Ensure the top-level type carries its name for json2ts output.
  if (!rewritten.title) rewritten.title = name;
  processed.set(name, { schema: rewritten, imports });
  writeFileSync(join(tmpDir, `${name}.schema.json`), JSON.stringify(rewritten));
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
  cwd: tmpDir,
  // Critical: don't inline sibling schemas; reference them by name and let us
  // inject the matching import statements.
  declareExternallyReferenced: false,
  bannerComment: '',
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

/**
 * Strip `export type X = ...;` and `export interface X { ... }` blocks whose
 * name is in `importedNames`. `declareExternallyReferenced: false` does this
 * for most shapes, but json-schema-to-typescript still re-emits unions
 * referenced across files (verified: a `oneOf` in the external schema
 * produces a duplicate `export type X = A | B;` alongside the import).
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

let emitted = 0;
let failed = 0;
for (const [name, { imports }] of processed) {
  const schemaPath = join(tmpDir, `${name}.schema.json`);
  const schema = JSON.parse(readFileSync(schemaPath, 'utf8'));
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

    const importLines = [...imports]
      .sort()
      .map((dep) => `import type { ${dep} } from './${dep}';`)
      .join('\n');
    const out =
      BANNER +
      '\n\n' +
      (importLines ? importLines + '\n\n' : '') +
      ts.trimStart();
    writeFileSync(join(outDir, `${name}.d.ts`), out);
    emitted++;
  } catch (err) {
    console.error(`  FAILED ${name}: ${err.message}`);
    failed++;
  }
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

rmSync(tmpDir, { recursive: true, force: true });

console.log(`Emitted ${emitted} .d.ts files to ${outDir}`);
if (failed > 0) {
  console.error(`${failed} type(s) failed to compile`);
  process.exit(1);
}
