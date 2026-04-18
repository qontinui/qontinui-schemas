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
  completedAt?: string | null;
  /**
   * Total duration in seconds.
   */
  durationSeconds?: number | null;
  /**
   * Error message to attach.
   */
  errorMessage?: string | null;
  /**
   * Full output log to persist.
   */
  fullOutput?: string | null;
  /**
   * Whether the full output log has been stored.
   */
  fullOutputStored?: boolean | null;
  /**
   * Updated output summary.
   */
  outputSummary?: string | null;
  /**
   * Updated session count.
   */
  sessionsCount?: number | null;
  /**
   * New lifecycle status.
   */
  status?: TaskRunStatus | null;
}
