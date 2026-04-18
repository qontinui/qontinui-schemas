/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Workflow metadata included in a UI Bridge snapshot.
 */
export interface WorkflowInfo {
  /**
   * Optional description.
   */
  description?: string | null;
  /**
   * Workflow ID.
   */
  id: string;
  /**
   * Workflow name.
   */
  name: string;
  /**
   * Number of steps in the workflow.
   */
  stepCount: number;
}
