/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Configuration for the implement-fixes phase (Claude CLI).
 */
export interface ImplementFixesConfig {
  /**
   * Additional context to include in the fix prompt.
   */
  additionalContext?: string | null;
  /**
   * Model to use (e.g., `"claude-opus-4-6"`). Defaults to `claude-opus-4-6`
   * when unset.
   */
  model?: string | null;
  /**
   * Timeout in seconds for the fix agent. Defaults to 600 when unset.
   */
  timeoutSecs?: number | null;
}
