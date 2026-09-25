/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * UI Bridge action kind.
 *
 * `wait_for_element`, `click`, `element_action`, `wait` and
 * `component_action` are the actions the runner's `UiBridgeHandler` runs
 * beyond the original seven; the generator and the runner's Builder emit
 * them.
 *
 * The default is `snapshot`: both step editors display an action-less step
 * as `snapshot` and the runner's handler runs it as one, so the typed view
 * agrees with what the user was shown.
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
  | "wait"
  | "component_action";
