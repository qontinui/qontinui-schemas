/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ScheduledTaskStatus } from './ScheduledTaskStatus';

/**
 * Record of a single task execution.
 */
export interface TaskExecutionRecord {
  /**
   * Session ID of the auto-fix session, if one was triggered.
   */
  autoFixSessionId?: string | null;
  /**
   * Whether this execution record was created by the catch-up reconciler
   * (rather than the normal scheduling tick).
   */
  catchUpRun: boolean;
  /**
   * ISO 8601 timestamp when execution ended.
   */
  endedAt?: string | null;
  /**
   * Error message if the execution failed.
   */
  errorMessage?: string | null;
  /**
   * Unique ID for this execution (UUID v4 string).
   */
  executionId: string;
  /**
   * ISO 8601 timestamp of the *scheduled* slot this execution covers,
   * distinct from `started_at` (which is the actual launch time). Allows
   * detecting "ran X minutes late" and is consumed by the missed-run
   * reconciler. Optional for backward compatibility with execution rows
   * written before this field existed.
   */
  scheduledFor?: string | null;
  /**
   * Session ID if this execution triggered an AI session, used for
   * downstream success tracking.
   */
  sessionId?: string | null;
  /**
   * ISO 8601 timestamp when execution started.
   */
  startedAt: string;
  status: ScheduledTaskStatus;
  /**
   * Whether the task succeeded, read from the session checkpoint.
   */
  success: boolean;
  /**
   * Whether auto-fix was triggered after this execution.
   */
  triggeredAutoFix: boolean;
}
