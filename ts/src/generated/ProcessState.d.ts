/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * State of a managed process.
 *
 * Ordering mirrors the lifecycle progression:
 * `Stopped → Starting → (Building) → Running → Healthy → Stopping → Stopped`,
 * or to `Failed` on any abnormal exit.
 */
export type ProcessState = "stopped" | "starting" | "building" | "running" | "healthy" | "stopping" | "failed";
