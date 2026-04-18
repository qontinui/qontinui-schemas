/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * A single repository to monitor for inactivity.
 */
export interface RepositoryWatch {
  /**
   * Minutes of inactivity required before the watch is considered met.
   */
  inactiveMinutes: number;
  /**
   * Path to the repository directory.
   */
  path: string;
}
