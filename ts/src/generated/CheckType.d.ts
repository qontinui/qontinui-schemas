/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Kinds of deterministic check surfaced by `CommandStep` when
 * `mode = "check"` or `mode = "check_group"`.
 */
export type CheckType =
  | "lint"
  | "format"
  | "typecheck"
  | "analyze"
  | "security"
  | "custom_command"
  | "http_status"
  | "ai_review"
  | "ci_cd";
