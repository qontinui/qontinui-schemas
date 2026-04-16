/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Information about a single available transition from the current state.
 */
export interface TransitionInfo {
  /**
   * Source state ID.
   */
  from_state: string;
  /**
   * Transition ID.
   */
  id: string;
  /**
   * Destination state ID (may be `null`/`None` if the transition stays
   * within the same state set).
   */
  to_state?: string | null;
  /**
   * Names of workflows attached to this transition.
   */
  workflows: string[];
  [k: string]: unknown;
}
