/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Response for `GET /constraints/config`.
 */
export interface ReadConfigResponse {
  /**
   * Resolved file path, if a config file was found.
   */
  path?: string | null;
  /**
   * Raw TOML content of the `constraints.toml` file (empty string if not found).
   */
  toml: string;
}
