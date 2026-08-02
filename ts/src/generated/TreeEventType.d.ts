/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Types of tree events emitted during execution.
 */
export type TreeEventType =
  | "workflow_started"
  | "workflow_completed"
  | "workflow_failed"
  | "action_started"
  | "action_completed"
  | "action_failed"
  | "transition_started"
  | "transition_completed"
  | "transition_failed";
