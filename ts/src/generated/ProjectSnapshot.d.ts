/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { GitCommitLite } from "./GitCommitLite";
import type { GitLite } from "./GitLite";
import type { HealthLevel } from "./HealthLevel";
import type { HealthLite } from "./HealthLite";
import type { PendingQuestion } from "./PendingQuestion";
import type { ProcessState } from "./ProcessState";
import type { ProcessStatusLite } from "./ProcessStatusLite";
import type { SavedProject } from "./SavedProject";
import type { SessionLite } from "./SessionLite";
import type { SessionSource } from "./SessionSource";

/**
 * The joined dashboard view of one project.
 *
 * Every field on both the grid card and the detail page is a projection of
 * this single struct — computed server-side in one call so the page never
 * fans out.
 */
export interface ProjectSnapshot {
  /**
   * Git state of the root, when it is a git working tree.
   */
  git?: GitLite | null;
  /**
   * The project's traffic light.
   */
  health?: HealthLite & {};
  /**
   * Unix epoch milliseconds of the newest activity of any kind.
   */
  lastActivityMs?: number | null;
  /**
   * Live terminal sessions whose `working_dir` is under the project root.
   */
  liveSessions: SessionLite[];
  /**
   * Managed processes whose `cwd` is under the project root.
   */
  processes: ProcessStatusLite[];
  project: SavedProject;
  /**
   * Questions waiting on the user, from `deferred_questions`.
   */
  questions: PendingQuestion[];
  /**
   * Recent AI sessions, newest first, attributed via
   * `coord.session_touched_files`.
   */
  recentSessions: SessionLite[];
  /**
   * Rolling 7-day spend attributed to this project, in USD. `None` when
   * cost data is unavailable — distinct from `Some(0.0)`, which means
   * "measured, and it was free".
   */
  spend7dUsd?: number | null;
}
