/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Response envelope returned after creating a batch of action executions.
 */
export interface ActionExecutionBatchResponse {
  /**
   * Assigned action IDs, in the same order as the submitted batch.
   */
  actionIds?: string[];
  /**
   * Number of actions recorded.
   */
  actionsRecorded: number;
  /**
   * Associated run ID.
   */
  runId: string;
  [k: string]: unknown;
}
