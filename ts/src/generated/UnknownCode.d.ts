/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * WHY a producer could not answer. A closed vocabulary: an agent branches on
 * the code, and the human-readable [`UnknownInfo::detail`] is never the only
 * carrier of the reason.
 *
 * Adding a member is a wire change for every consumer (runner, ui-bridge,
 * ui-bridge-mcp); the diagnostic-code registry name of each is
 * [`Self::diagnostic_code`].
 */
export type UnknownCode =
  | "app_unreachable"
  | "capability_unavailable_in_build"
  | "producer_failed"
  | "producer_not_run"
  | "input_missing"
  | "below_confidence_floor"
  | "model_reply_unparseable"
  | "needs_multi_frame_input"
  | "stale_input";
