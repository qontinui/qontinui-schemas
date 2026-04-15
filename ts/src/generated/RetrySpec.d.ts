/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Per-step retry configuration carried by [`BaseStepFields::retry`].
 *
 * Distinct from the workflow frame's `RetryPolicy` — that one also carries a
 * `backoff` flag, this per-step form is the older, simpler shape that step
 * DTOs inherited from the TS `BaseStep` interface.
 */
export interface RetrySpec {
  /**
   * Number of retry attempts (`0` = no retries).
   */
  count: number;
  /**
   * Delay between retries in milliseconds.
   */
  delay_ms: number;
  [k: string]: unknown;
}
