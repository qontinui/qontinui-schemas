/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * An output declared by a stage, available to subsequent stages.
 */
export interface StageOutput {
  /**
   * Human-readable description.
   */
  description: string;
  /**
   * Unique key for this output (e.g. `"api_url"`, `"auth_token"`).
   */
  key: string;
  [k: string]: unknown;
}
