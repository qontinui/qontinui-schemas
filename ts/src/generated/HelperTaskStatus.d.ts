/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Lifecycle status of a helper task.
 *
 * Lifecycle: `OPEN` → (`ANSWERED` | `EXPIRED` | `CANCELLED`). A task becomes
 * `ANSWERED` once [`HelperTask::required_votes`] answers are collected.
 */
export type HelperTaskStatus = "open" | "answered" | "expired" | "cancelled";
