/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Request body for `POST /constraints/config`.
 */
export interface WriteConfigRequest {
  /**
   * Project path for the `constraints.toml`. Defaults to workspace root.
   */
  projectPath?: string | null;
  /**
   * Raw TOML content to validate and write.
   */
  toml: string;
  [k: string]: unknown;
}
