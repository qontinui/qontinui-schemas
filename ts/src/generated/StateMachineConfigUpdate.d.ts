/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Payload for updating an existing state machine configuration.
 */
export interface StateMachineConfigUpdate {
  /**
   * New description.
   */
  description?: string | null;
  /**
   * New display name.
   */
  name?: string | null;
  [k: string]: unknown;
}
