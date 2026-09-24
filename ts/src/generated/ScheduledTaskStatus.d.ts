/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Status of a scheduled task execution.
 */
export type ScheduledTaskStatus =
  | "pending"
  | "running"
  | "completed"
  | "failed"
  | "launch_failed"
  | "skipped"
  | "cancelled"
  | "missed_runner_down";
