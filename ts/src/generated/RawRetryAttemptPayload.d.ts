/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * One retry attempt.
 */
export interface RawRetryAttemptPayload {
  attempt_number: number;
  attempt_timestamp: string;
  delay_ms: number;
  error: string;
  feedback_injected: boolean;
}
