/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Context passed to the verification agent (intentionally limited — does not
 * include work history, to avoid biasing AI evaluation).
 */
export interface VerificationAgentContext {
  /**
   * The evaluation prompt from the criterion.
   */
  evaluationPrompt: string;
  /**
   * Brief goal context (NOT work history).
   */
  goalContext: string;
  /**
   * The screenshot to evaluate, base64-encoded.
   */
  screenshotBase64: string;
}
