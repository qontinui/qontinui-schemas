/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * What the reader does next. A closed set: every refusal names exactly one.
 */
export type NextActionKind =
  | "retry_later"
  | "run_command"
  | "open_page"
  | "sign_in"
  | "pair_device"
  | "set_setting"
  | "wait_for_gate"
  | "none_terminal"
  | "report_defect"
  | "fix_request"
  | "resnapshot"
  | "scroll_into_view"
  | "wait_for_enabled"
  | "broaden_selector"
  | "unrecognised";
