/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * State-machine context captured before/after an action.
 */
export interface StateContext {
  activated: string[];
  active_after: string[];
  active_before: string[];
  changed: boolean;
  deactivated: string[];
  [k: string]: unknown;
}
