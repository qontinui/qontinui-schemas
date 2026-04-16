/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Response envelope returned after creating a batch of issues.
 */
export interface ExecutionIssueBatchResponse {
  /**
   * Assigned issue IDs, in the same order as the submitted batch.
   */
  issue_ids?: string[];
  /**
   * Number of issues recorded.
   */
  issues_recorded: number;
  /**
   * Associated run ID.
   */
  run_id: string;
  [k: string]: unknown;
}
