/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * What went wrong, as a stable machine-readable code.
 *
 * Extend by adding a variant (and its row in [`RefusalCode::headline`]); the
 * render table is an exhaustive `match`, so a new code without a sentence is
 * a compile error.
 */
export type RefusalCode =
  | "workspace_root_unresolved"
  | "sibling_checkout_absent"
  | "endpoint_unresolved"
  | "glossary_term_unknown"
  | "unknown";
