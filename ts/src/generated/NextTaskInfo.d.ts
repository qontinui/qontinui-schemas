/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Minimal description of the next task due to run.
 */
export interface NextTaskInfo {
  /**
   * Task ID.
   */
  id: string;
  /**
   * Task display name.
   */
  name: string;
  /**
   * ISO 8601 timestamp of the next scheduled run.
   */
  nextRun: string;
  [k: string]: unknown;
}
