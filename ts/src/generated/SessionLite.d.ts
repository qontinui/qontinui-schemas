/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { SessionSource } from "./SessionSource";

/**
 * One session attributed to a project — either a live terminal or an AI
 * session reconstructed from its touched files.
 */
export interface SessionLite {
  /**
   * Number of distinct files this session touched **under this project
   * root**. Zero for terminal-sourced rows.
   */
  filesTouched: number;
  /**
   * `task_runs.id` for AI sessions, the terminal id for live terminals.
   */
  id: string;
  /**
   * Unix epoch milliseconds of the most recent observed activity —
   * `max(recorded_at)` over the session's touched files, or the
   * terminal's creation time.
   */
  lastActivityMs?: number | null;
  /**
   * Session name — `task_runs.task_name` or the terminal's title.
   */
  name?: string | null;
  source: SessionSource;
  /**
   * `task_runs.status` ("running", "completed", …) for AI sessions;
   * `None` for terminals (liveness is `is_alive` in the terminal list).
   */
  status?: string | null;
  /**
   * Working directory, when known (terminal sessions always carry one).
   */
  workingDir?: string | null;
}
