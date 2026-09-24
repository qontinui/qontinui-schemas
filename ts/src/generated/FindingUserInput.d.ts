/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
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
  inputType: string;
  /**
   * Options for choice-type input.
   */
  options?: string[] | null;
  /**
   * Question to present to the user.
   */
  question: string;
}
