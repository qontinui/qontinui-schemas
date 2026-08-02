/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { GitCommitLite } from "./GitCommitLite";

/**
 * Git state of the project root. `None` on the snapshot when the root is
 * not a git working tree (or git is unavailable).
 */
export interface GitLite {
  /**
   * Current branch. `None` on a detached HEAD.
   */
  branch?: string | null;
  /**
   * Number of `git status --porcelain` entries (uncommitted changes).
   */
  dirtyCount: number;
  /**
   * Newest commits first.
   */
  lastCommits: GitCommitLite[];
}
