/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Type of action executed.
 *
 * Covers vision, input, state-machine, control-flow, utility, AI, and custom
 * action kinds. Variants are serialized as snake_case strings.
 */
export type ExecutionActionType =
  | "find"
  | "find_all"
  | "wait_for"
  | "wait_until_gone"
  | "click"
  | "double_click"
  | "right_click"
  | "type"
  | "press_key"
  | "hotkey"
  | "scroll"
  | "drag"
  | "go_to_state"
  | "transition"
  | "verify_state"
  | "conditional"
  | "loop"
  | "parallel"
  | "sequence"
  | "wait"
  | "screenshot"
  | "log"
  | "assert"
  | "ai_prompt"
  | "run_prompt_sequence"
  | "custom";
