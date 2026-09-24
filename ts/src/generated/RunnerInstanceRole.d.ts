/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Which role a runner instance plays on its machine.
 *
 * Every runner instance on one box presents the SAME machine `device_id`, so
 * the role is what separates the canonical instance from the supervisor-spawned
 * ones sharing its row. Reported by the runner in `runner_info` as the
 * top-level `instanceRole`.
 */
export type RunnerInstanceRole = "primary" | "secondary";
