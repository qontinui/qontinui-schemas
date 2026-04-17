/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Request to execute an action on a UI Bridge component.
 */
export interface ComponentActionRequest {
  /**
   * Action name.
   */
  action: string;
  /**
   * Optional action-specific parameters.
   */
  params?: {
    [k: string]: unknown;
  };
  [k: string]: unknown;
}
