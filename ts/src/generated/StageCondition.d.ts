/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Condition for conditional stage execution.
 *
 * When attached to a [`WorkflowStage`], the stage is skipped if the condition
 * evaluates to "should skip". All condition fields are optional and combine
 * with AND semantics — all specified conditions must be met for the stage to
 * run.
 */
export interface StageCondition {
  /**
   * Run this stage only if the previous stage had this outcome.
   *
   * - `"passed"`: run only if previous stage verification passed
   * - `"failed"`: run only if previous stage verification failed
   * - `"any"`: always run regardless of previous outcome (default behavior)
   */
  ifPrevious?: string | null;
  /**
   * Skip this stage if the total number of failed stages so far is below
   * this threshold. Useful for "recovery" stages that only run when
   * multiple prior stages have failed.
   */
  minFailures?: number | null;
  /**
   * Run this stage only after this many loop iterations have occurred
   * (across all stages). Useful for "escalation" stages that only kick in
   * after initial attempts have failed.
   */
  minIteration?: number | null;
  [k: string]: unknown;
}
