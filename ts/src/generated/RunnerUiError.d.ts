/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Structured error reported by the runner that the UI surfaces verbatim.
 */
export interface RunnerUiError {
  /**
   * Optional long-form detail (stack trace, stderr, etc.).
   */
  detail?: string | null;
  /**
   * Error category (e.g., `"build_failed"`, `"port_conflict"`).
   */
  kind: string;
  /**
   * Short, user-facing error message.
   */
  message: string;
  /**
   * ISO 8601 timestamp when the error was reported.
   */
  reportedAt: string;
}
