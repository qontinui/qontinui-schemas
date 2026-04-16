/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Response envelope returned after reporting issues.
 */
export interface ExecutionIssueResponse {
  /**
   * Assigned issue IDs, in the same order as the submitted batch.
   */
  issue_ids?: string[] | null;
  /**
   * Number of issues recorded.
   */
  recorded: number;
  /**
   * Associated run ID.
   */
  run_id: string;
  [k: string]: unknown;
}
