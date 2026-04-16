/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Result of executing a single state transition at runtime.
 *
 * Returned by the runner's Tauri commands or the qontinui Python subprocess.
 */
export interface TransitionExecutionResult {
  /**
   * States that are active after the transition.
   */
  active_states: string[];
  /**
   * Error message when execution failed.
   */
  error?: string | null;
  /**
   * Whether the transition executed successfully.
   */
  success: boolean;
  /**
   * Logical transition ID that was executed.
   */
  transition_id: string;
  [k: string]: unknown;
}
