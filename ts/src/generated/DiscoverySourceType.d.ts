/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Source of a state-discovery result.
 *
 * Identifies which discovery pathway produced the state machine. Mirrors
 * Python `DiscoverySourceType(str, Enum)`.
 */
export type DiscoverySourceType =
  "playwright" | "ui_bridge" | "recording" | "vision" | "manual";
