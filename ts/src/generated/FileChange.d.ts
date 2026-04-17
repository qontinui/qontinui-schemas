/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * A file change made during the agentic phase.
 */
export interface FileChange {
  /**
   * Action performed on the file (e.g., "modified", "created", "deleted").
   */
  action: string;
  /**
   * Path of the changed file.
   */
  path: string;
  [k: string]: unknown;
}
