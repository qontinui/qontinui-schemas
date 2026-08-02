/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Rules for automatically including a context in AI tasks.
 *
 * When an AI task is created, the runner evaluates these rules to decide
 * which contexts should be auto-included. Multiple rules are OR'd together
 * (any match triggers inclusion).
 */
export interface ContextAutoInclude {
  /**
   * Action types in the loaded config that trigger inclusion
   * (e.g., `CLICK`, `FIND`).
   */
  actionTypes?: string[] | null;
  /**
   * Regex patterns in recent logs that trigger inclusion.
   */
  errorPatterns?: string[] | null;
  /**
   * Glob patterns for files being worked on (e.g., `*.rs`, `src/api/**`).
   */
  filePatterns?: string[] | null;
  /**
   * Keywords in the task prompt that trigger inclusion
   * (case-insensitive).
   */
  taskMentions?: string[] | null;
}
