/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * UI Bridge action kind.
 *
 * `wait_for_element`, `click`, `element_action` and `wait` are the actions
 * the runner's `UiBridgeHandler` runs beyond the original seven; the
 * generator and the runner's Builder emit them.
 */
export type UiBridgeAction =
  | "navigate"
  | "execute"
  | "assert"
  | "snapshot"
  | "compare"
  | "snapshot_assert"
  | "action_plan"
  | "wait_for_element"
  | "click"
  | "element_action"
  | "wait";
