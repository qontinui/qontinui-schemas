/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * The same item in an external system (a Jira issue, say).
 */
export interface ExternalRef {
  /**
   * The item's key in that system, e.g. `PAY-123`.
   */
  key: string;
  /**
   * The external system, e.g. `jira`.
   */
  system: string;
  url?: string | null;
  [k: string]: unknown;
}
