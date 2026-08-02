/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Result of a single `tools/call` invocation.
 *
 * Shape is always the same regardless of success / failure: check `success`
 * first, then read `content` (on success) or `error` (on failure).
 */
export interface McpToolCallResult {
  /**
   * Response content (present on success). Usually a JSON object/array,
   * but can be a primitive if the tool returned text-only content.
   */
  content?: {
    [k: string]: unknown;
  };
  /**
   * Wall-clock duration of the call in milliseconds.
   */
  durationMs: number;
  /**
   * Error message (present on failure).
   */
  error?: string | null;
  /**
   * Response type tag: `"json"`, `"text"`, or `"error"`.
   */
  responseType: string;
  /**
   * Whether the call succeeded.
   */
  success: boolean;
}
