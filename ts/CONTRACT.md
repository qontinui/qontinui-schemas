# Wire-format contract for qontinui-schemas consumers

This package publishes the single source of truth for types exchanged between Qontinui services. The contract is simple, but the consequences of breaking it cascade across several repos, so it is pinned here.

## Rules

1. **Wire JSON is camelCase.** All public Rust structs use `#[serde(rename_all = "camelCase")]`. The generated `.d.ts` mirrors that.

2. **Wire JSON is closed.** All public structs use `#[serde(deny_unknown_fields)]`. The generated TypeScript interfaces do not contain `[k: string]: unknown` fallbacks. A value of type `Foo` has exactly the keys declared on `Foo`, and any code that needs to read an arbitrary property must either narrow the type or widen it explicitly (e.g., `as Record<string, unknown>`).

3. **Consumer literals are camelCase.** Any object-literal construction of a schemas type must use camelCase keys. Snake_case keys will fail typecheck: the literal no longer assigns to the declared interface.

4. **Database and external-API serialization is a boundary, not a convention.** If a consumer writes to Postgres (with snake_case columns) or another external API, isolate that translation at a named boundary (a repository function, an adapter module), and type-check at that boundary. Do not scatter snake_case literals through general code to satisfy a single SQL call.

5. **Backward-compat wire aliases do not help at the TS type level.** The Rust side accepts both camelCase and the legacy snake_case on deserialization. This is only relevant for reading old persisted payloads; construction-site TypeScript checks run at compile time, so literals must be camelCase.

## Why this exists

Prior to commits `854d4b7` and `380e4ab`, schemars emitted `additionalProperties: true` on every struct, which json-schema-to-typescript lowered to an `[k: string]: unknown` index signature. That signature silently absorbed snake_case keys and any extra fields, masking drift between Rust and TS. The drift surfaced as runtime bugs: consumers wrote `started_at` while the Rust side expected `startedAt`; nobody noticed until a downstream deserializer rejected the payload.

Closing the structs and normalizing to camelCase removes the silent fallback. The trade-off is that the first time you break the contract, typecheck screams — but that is the correct failure mode.

## Enforcing the contract

- The schemas package's own CI runs `cargo test` + schema regeneration on every commit. A drift between Rust and generated TS cannot land.
- Consumers (`qontinui-runner`, `qontinui-web/frontend`, `qontinui-workflow-ui`) must run `npx tsc --noEmit` as a blocking CI step. Without that gate, schemas changes can silently break consumers and only show up in runtime failures.
- The audit file `D:/qontinui-root/plans/schemas-cascade-audit-2026-04-18.md` documents the one-time cleanup that followed the contract's introduction. Any future cascade of this shape should be short-lived because the gate catches it at PR time.

## Migrating legacy consumer code

If you hit a wall of errors after a schemas bump:

- **TS2741 / TS2322 "is missing property X"** where X is camelCase and your literal has the snake_case equivalent: rename the key.
- **TS2339 "property X does not exist on type `{}`"**: the value came through an `unknown` path. Fix by retyping upstream (preferred) or narrowing with a type guard. A cast is a last resort, and each one should be annotated with the reason.
- **TS2488 "type must have `[Symbol.iterator]()`"**: iterating over a value typed as `unknown`. Same fix — retype upstream.

When fixing, remember Rule 4: if you see snake_case in code that actually writes to Postgres, that literal is intentional. Isolate it with `as unknown as T` at the DB boundary and add a comment explaining why, rather than renaming and breaking the query.
