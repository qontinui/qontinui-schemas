/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * A single AI session within a task run.
 */
export interface TaskRunSession {
  /**
   * Duration of the session in seconds.
   */
  duration_seconds: number | null;
  /**
   * ISO 8601 timestamp when the session ended.
   */
  ended_at: string | null;
  /**
   * Unique identifier (UUID v4 string).
   */
  id: string;
  /**
   * Short summary of the session output, if stored.
   */
  output_summary: string | null;
  /**
   * 1-based session index within the parent task run.
   */
  session_number: number;
  /**
   * ISO 8601 timestamp when the session started.
   */
  started_at: string;
  /**
   * Parent task run ID.
   */
  task_run_id: string;
  [k: string]: unknown;
}
