/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Condition that requires the runner to be idle (not executing workflows or
 * AI tasks) before the task may run.
 */
export interface IdleCondition {
  /**
   * Whether this condition is active.
   */
  enabled: boolean;
  [k: string]: unknown;
}
