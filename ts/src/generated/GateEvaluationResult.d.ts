/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Result of evaluating a named gate across a set of steps.
 */
export interface GateEvaluationResult {
  /**
   * IDs of required steps that failed.
   */
  failedStepIds: string[];
  /**
   * Name of the gate.
   */
  gateName: string;
  /**
   * IDs of required steps that were missing.
   */
  missingStepIds: string[];
  /**
   * Whether the gate passed overall.
   */
  passed: boolean;
  /**
   * IDs of required steps that passed.
   */
  passedStepIds: string[];
  /**
   * IDs of steps the gate required.
   */
  requiredStepIds: string[];
}
