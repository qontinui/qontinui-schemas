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
  issueIds?: string[];
  /**
   * Number of issues recorded.
   */
  issuesRecorded: number;
  /**
   * Associated run ID.
   */
  runId: string;
  [k: string]: unknown;
}
