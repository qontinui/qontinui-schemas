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
  failed_step_ids: string[];
  /**
   * Name of the gate.
   */
  gate_name: string;
  /**
   * IDs of required steps that were missing.
   */
  missing_step_ids: string[];
  /**
   * Whether the gate passed overall.
   */
  passed: boolean;
  /**
   * IDs of required steps that passed.
   */
  passed_step_ids: string[];
  /**
   * IDs of steps the gate required.
   */
  required_step_ids: string[];
  [k: string]: unknown;
}
