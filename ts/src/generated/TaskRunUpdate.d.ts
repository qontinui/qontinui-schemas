/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { TaskRunStatus } from './TaskRunStatus';

/**
 * Request payload for updating an existing task run. All fields are optional;
 * only those supplied are applied.
 */
export interface TaskRunUpdate {
  /**
   * ISO 8601 timestamp when the task completed.
   */
  completed_at?: string | null;
  /**
   * Total duration in seconds.
   */
  duration_seconds?: number | null;
  /**
   * Error message to attach.
   */
  error_message?: string | null;
  /**
   * Full output log to persist.
   */
  full_output?: string | null;
  /**
   * Whether the full output log has been stored.
   */
  full_output_stored?: boolean | null;
  /**
   * Updated output summary.
   */
  output_summary?: string | null;
  /**
   * Updated session count.
   */
  sessions_count?: number | null;
  /**
   * New lifecycle status.
   */
  status?: TaskRunStatus | null;
  [k: string]: unknown;
}
