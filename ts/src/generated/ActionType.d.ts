/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Types of actions in the automation system.
 *
 * Corresponds to the action types defined in the qontinui-schemas config
 * models. Variants use SCREAMING_SNAKE_CASE on the wire to match the Python
 * enum values.
 */
export type ActionType =
  | "FIND"
  | "FIND_STATE_IMAGE"
  | "EXISTS"
  | "VANISH"
  | "CLICK"
  | "DOUBLE_CLICK"
  | "RIGHT_CLICK"
  | "DRAG"
  | "SCROLL"
  | "MOUSE_MOVE"
  | "TYPE"
  | "KEY_PRESS"
  | "HOTKEY"
  | "IF"
  | "LOOP"
  | "SWITCH"
  | "TRY_CATCH"
  | "BREAK"
  | "CONTINUE"
  | "GO_TO_STATE"
  | "WAIT"
  | "RUN_WORKFLOW"
  | "SCREENSHOT"
  | "CODE_BLOCK"
  | "SHELL"
  | "CUSTOM";
