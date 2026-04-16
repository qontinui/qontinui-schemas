/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Information about the workflow being executed in the run.
 */
export interface WorkflowMetadata {
  /**
   * Workflow description.
   */
  description?: string | null;
  /**
   * IDs of the states that are active when the workflow starts.
   */
  initial_state_ids?: string[] | null;
  /**
   * Free-form tags attached to the workflow.
   */
  tags?: string[] | null;
  /**
   * Number of states declared by the workflow.
   */
  total_states?: number | null;
  /**
   * Number of transitions declared by the workflow.
   */
  total_transitions?: number | null;
  /**
   * Workflow identifier.
   */
  workflow_id: string;
  /**
   * Human-readable workflow name.
   */
  workflow_name: string;
  /**
   * Workflow version, if tracked.
   */
  workflow_version?: string | null;
  [k: string]: unknown;
}
