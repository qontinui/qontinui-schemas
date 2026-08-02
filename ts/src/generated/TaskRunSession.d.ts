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
  durationSeconds: number | null;
  /**
   * ISO 8601 timestamp when the session ended.
   */
  endedAt: string | null;
  /**
   * Unique identifier (UUID v4 string).
   */
  id: string;
  /**
   * Short summary of the session output, if stored.
   */
  outputSummary: string | null;
  /**
   * 1-based session index within the parent task run.
   */
  sessionNumber: number;
  /**
   * ISO 8601 timestamp when the session started.
   */
  startedAt: string;
  /**
   * Parent task run ID.
   */
  taskRunId: string;
}
