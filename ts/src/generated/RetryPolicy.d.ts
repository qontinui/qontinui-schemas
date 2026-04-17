/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Retry policy for a step or stage.
 */
export interface RetryPolicy {
  /**
   * Whether to use exponential backoff.
   */
  backoff: boolean;
  /**
   * Number of retry attempts (`0` = no retries).
   */
  count: number;
  /**
   * Delay between retries in milliseconds.
   */
  delayMs: number;
  [k: string]: unknown;
}
