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
  activeAfter: string[];
  activeBefore: string[];
  changed: boolean;
  deactivated: string[];
}
