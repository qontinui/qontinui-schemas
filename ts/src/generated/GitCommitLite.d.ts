/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * One commit in [`GitLite::last_commits`].
 */
export interface GitCommitLite {
  /**
   * Unix epoch milliseconds of the commit timestamp.
   */
  committedMs: number;
  /**
   * Abbreviated commit sha.
   */
  sha: string;
  /**
   * Commit subject line.
   */
  subject: string;
}
