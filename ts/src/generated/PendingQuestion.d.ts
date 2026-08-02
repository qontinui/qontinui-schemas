/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * A question an agent is blocked on, awaiting the user's answer.
 */
export interface PendingQuestion {
  /**
   * Unix epoch milliseconds the question was raised.
   */
  createdAtMs?: number | null;
  /**
   * `deferred_questions.id`.
   */
  id: string;
  /**
   * The question text, as shown to the user.
   */
  question: string;
  /**
   * Risk level the asking agent assigned ("low" | "medium" | "high").
   */
  riskLevel?: string | null;
  /**
   * The session that asked.
   */
  taskRunId: string;
}
