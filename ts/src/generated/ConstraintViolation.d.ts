/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * A specific violation found during constraint evaluation.
 */
export interface ConstraintViolation {
  /**
   * What was found / what went wrong.
   */
  detail: string;
  /**
   * File where the violation was found (if applicable).
   */
  file?: string | null;
  /**
   * Line number (if applicable).
   */
  line?: number | null;
  [k: string]: unknown;
}
