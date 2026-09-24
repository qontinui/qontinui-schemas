/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Current state of an individual worker.
 */
export type WorkerStatus =
  | "idle"
  | "active"
  | "awaiting_verification"
  | "paused"
  | "completed"
  | "error";
