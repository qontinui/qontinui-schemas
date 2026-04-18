/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { TaskRunStatus } from './TaskRunStatus';

/**
 * A task run as returned by the backend API.
 *
 * Mirrors `TaskRunBackend` in the web app's `task-runs.ts`. Nullable
 * ownership fields (`project_id`, `created_by_user_id`, `runner_id`,
 * `max_sessions`, `output_summary`, `error_message`, `duration_seconds`,
 * `completed_at`) are required on the wire but may be `null`; they are
 * `Option<T>` with `serde(default)` so deserialize tolerates missing, but
 * are always serialized (including as `null`) to preserve the wire shape.
 */
export interface TaskRunBackend {
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
}
