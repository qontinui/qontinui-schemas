/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Result of evaluating whether the loop should exit.
 */
export interface ExitCheckResult {
  /**
   * Human-readable reason for the decision.
   */
  reason: string;
  /**
   * Whether the loop should stop after this iteration.
   */
  shouldExit: boolean;
}
