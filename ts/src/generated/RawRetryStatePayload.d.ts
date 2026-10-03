/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { RawRetryAttemptPayload } from "./RawRetryAttemptPayload";

/**
 * Accumulated retry state.
 */
export interface RawRetryStatePayload {
  attempt: number;
  error_history: RawRetryAttemptPayload[];
  last_attempt_at: string | null;
  last_error: string | null;
  total_delay_ms: number;
}
