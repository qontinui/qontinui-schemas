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
  auto_continue: boolean;
  /**
   * ISO 8601 timestamp when the task completed.
   */
  completed_at: string | null;
  /**
   * ISO 8601 timestamp when the task was created.
   */
  created_at: string;
  /**
   * User who created the task run, if known.
   */
  created_by_user_id: string | null;
  /**
   * Total duration in seconds.
   */
  duration_seconds: number | null;
  /**
   * Error message if the task failed.
   */
  error_message: string | null;
  /**
   * Whether the full output log was persisted.
   */
  full_output_stored: boolean;
  /**
   * Unique identifier (UUID v4 string).
   */
  id: string;
  /**
   * Optional cap on AI sessions.
   */
  max_sessions: number | null;
  /**
   * Short summary of the run output, if stored.
   */
  output_summary: string | null;
  /**
   * Owning project ID, if scoped to a project.
   */
  project_id: string | null;
  /**
   * Original prompt text.
   */
  prompt: string;
  /**
   * Runner instance that executed the task, if known.
   */
  runner_id: string | null;
  /**
   * Number of AI sessions that have been run.
   */
  sessions_count: number;
  status: TaskRunStatus;
  /**
   * Display name.
   */
  task_name: string;
  /**
   * ISO 8601 timestamp when the record was last updated.
   */
  updated_at: string;
  [k: string]: unknown;
}
