/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Response for `POST /constraints/config`.
 */
export interface WriteConfigResponse {
  /**
   * Parse errors or non-fatal warnings.
   */
  errors: string[];
  /**
   * The file path that was written to.
   */
  path: string;
  /**
   * Whether the config is fully valid (parseable with no errors or warnings).
   */
  valid: boolean;
}
