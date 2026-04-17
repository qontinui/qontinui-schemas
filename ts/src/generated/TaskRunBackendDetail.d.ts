/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { TaskRunFinding } from './TaskRunFinding';
import type { TaskRunFindingActionType } from './TaskRunFindingActionType';
import type { TaskRunFindingCategory } from './TaskRunFindingCategory';
import type { TaskRunFindingSeverity } from './TaskRunFindingSeverity';
import type { TaskRunFindingStatus } from './TaskRunFindingStatus';
import type { TaskRunFindingSummary } from './TaskRunFindingSummary';
import type { TaskRunSession } from './TaskRunSession';
import type { TaskRunStatus } from './TaskRunStatus';

/**
 * Detailed view of a backend task run, including its sessions and findings.
 *
 * The TypeScript `TaskRunBackendDetail extends TaskRunBackend` is modeled in
 * Rust by flattening a [`TaskRunBackend`] base so the wire shape stays flat.
 */
export interface TaskRunBackendDetail {
  /**
   * Whether the task will auto-continue into another session on exit.
   */
  autoContinue: boolean;
  /**
   * ISO 8601 timestamp when the task completed.
   */
  completedAt: string | null;
  /**
   * ISO 8601 timestamp when the task was created.
   */
  createdAt: string;
  /**
   * User who created the task run, if known.
   */
  createdByUserId: string | null;
  /**
   * Total duration in seconds.
   */
  durationSeconds: number | null;
  /**
   * Error message if the task failed.
   */
  errorMessage: string | null;
  findingSummary: TaskRunFindingSummary;
  /**
   * Findings surfaced during this task run.
   */
  findings: TaskRunFinding[];
  /**
   * Whether the full output log was persisted.
   */
  fullOutputStored: boolean;
  /**
   * Unique identifier (UUID v4 string).
   */
  id: string;
  /**
   * Optional cap on AI sessions.
   */
  maxSessions: number | null;
  /**
   * Short summary of the run output, if stored.
   */
  outputSummary: string | null;
  /**
   * Owning project ID, if scoped to a project.
   */
  projectId: string | null;
  /**
   * Original prompt text.
   */
  prompt: string;
  /**
   * Runner instance that executed the task, if known.
   */
  runnerId: string | null;
  /**
   * AI sessions associated with this task run.
   */
  sessions: TaskRunSession[];
  /**
   * Number of AI sessions that have been run.
   */
  sessionsCount: number;
  status: TaskRunStatus;
  /**
   * Display name.
   */
  taskName: string;
  /**
   * ISO 8601 timestamp when the record was last updated.
   */
  updatedAt: string;
  [k: string]: unknown;
}
