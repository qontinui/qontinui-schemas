/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Strategy for selecting among multiple matches in a FIND action.
 *
 * Variants serialize uppercase (`"FIRST"`, `"ALL"`, `"BEST"`, `"EACH"`) to
 * match the existing Python enum and all stored action configs.
 */
export type SearchStrategy = "FIRST" | "ALL" | "BEST" | "EACH";
