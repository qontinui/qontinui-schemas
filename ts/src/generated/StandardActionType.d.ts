/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * All UI Bridge SDK standard actions plus workflow-level actions
 * (`wait`, `navigate`).
 *
 * Parameterized actions: `click`, `doubleClick`, `rightClick`, `type`,
 * `select`, `scroll`, `drag`.
 * No-param actions: `clear`, `focus`, `blur`, `hover`, `check`, `uncheck`,
 * `toggle`, `setValue`, `submit`, `reset`.
 * Workflow-level actions: `wait`, `navigate`.
 *
 * Serialized as a bare camelCase string matching the TypeScript literal
 * union (e.g., `"doubleClick"`, `"setValue"`).
 */
export type StandardActionType =
  | "click"
  | "doubleClick"
  | "rightClick"
  | "type"
  | "clear"
  | "select"
  | "focus"
  | "blur"
  | "hover"
  | "scroll"
  | "check"
  | "uncheck"
  | "toggle"
  | "setValue"
  | "drag"
  | "submit"
  | "reset"
  | "wait"
  | "navigate";
