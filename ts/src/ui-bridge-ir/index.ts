/**
 * @qontinui/shared-types/ui-bridge-ir
 *
 * Authoring-time intermediate representation for the UI Bridge redesign.
 * Build plugins (Vite, Next.js, Metro) emit `IRDocument` instances; the
 * adapter (`adaptIRToWorkflowConfig`) folds them into ui-bridge-auto's
 * runtime shape.
 *
 * Boundary with `@qontinui/shared-types/state-machine` (per ADR-001 /
 * decision #6): `state-machine` houses snake_case Rust-generated wire types
 * for the runtime/db. `ui-bridge-ir` is camelCase, authoring-time, and
 * carries IR-only fields (provenance, metadata, effect, crossRefs,
 * visualRefs). The adapter is the single point of translation between the
 * two namespaces.
 */

export * from "./primitives";
export * from "./element-criteria";
export * from "./state";
export * from "./transition";
export * from "./document";
export * from "./adapter";
export * from "./projection";
