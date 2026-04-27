/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Health/availability state of a runner, computed server-side from the
 * WebSocket-presence and heartbeat-freshness signals.
 */
export type RunnerStatus = "healthy" | "degraded" | "offline" | "starting" | "errored";
