/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * User-input request attached to a finding.
 *
 * Defines the question to pose and the expected input format when a finding
 * requires a user decision.
 */
export interface FindingUserInput {
  /**
   * Type of input expected — typically `"text"` or `"choice"`.
   */
  input_type: string;
  /**
   * Options for choice-type input.
   */
  options?: string[] | null;
  /**
   * Question to present to the user.
   */
  question: string;
  [k: string]: unknown;
}
