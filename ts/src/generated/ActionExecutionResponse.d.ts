/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Response envelope returned after reporting action executions.
 */
export interface ActionExecutionResponse {
  /**
   * Assigned action IDs, in the same order as the submitted batch.
   */
  actionIds?: string[] | null;
  /**
   * Number of actions recorded.
   */
  recorded: number;
  /**
   * Associated run ID.
   */
  runId: string;
  [k: string]: unknown;
}
