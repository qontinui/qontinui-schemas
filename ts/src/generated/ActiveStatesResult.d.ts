/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Result of querying currently active states in the state machine.
 */
export interface ActiveStatesResult {
  /**
   * State IDs currently active.
   */
  active_states: string[];
  /**
   * The "primary" current state, when one can be singled out.
   */
  current_state?: string | null;
  /**
   * Error message when the query failed.
   */
  error?: string | null;
  /**
   * Ordered history of states the machine has passed through.
   */
  state_history?: string[] | null;
  /**
   * Whether the query succeeded.
   */
  success: boolean;
}
