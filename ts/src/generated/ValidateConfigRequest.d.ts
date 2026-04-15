/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Request body for `POST /constraints/validate`.
 */
export interface ValidateConfigRequest {
  /**
   * Raw TOML content to validate.
   */
  toml: string;
  [k: string]: unknown;
}
