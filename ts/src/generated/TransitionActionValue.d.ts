/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * A value for `select` / `setValue` actions: either a single string or a list
 * of strings (for multi-select).
 *
 * Serialized as an untagged union matching the TS `string | string[]`.
 */
export type TransitionActionValue = string | string[];
