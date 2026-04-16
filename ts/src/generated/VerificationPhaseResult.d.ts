/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { CheckIssueDetail } from './CheckIssueDetail';
import type { GateEvaluationResult } from './GateEvaluationResult';
import type { IndividualCheckResult } from './IndividualCheckResult';
import type { StepExecutionConfig } from './StepExecutionConfig';
import type { VerificationStepDetails } from './VerificationStepDetails';
import type { VerificationStepResult } from './VerificationStepResult';

/**
 * Result of a single iteration of the verification phase.
 */
export interface VerificationPhaseResult {
  /**
   * Whether all steps passed.
   */
  all_passed: boolean;
  /**
   * Whether a critical step failure short-circuited the phase.
   */
  critical_failure: boolean;
  /**
   * Number of steps that failed.
   */
  failed_steps: number;
  /**
   * Whether pass/fail is determined by gates rather than overall step counts.
   */
  gate_based_evaluation: boolean;
  /**
   * Per-gate evaluation results.
   */
  gate_results: GateEvaluationResult[];
  /**
   * 1-based iteration index within the workflow run.
   */
  iteration: number;
  /**
   * Number of steps that passed.
   */
  passed_steps: number;
  /**
   * Number of steps that were skipped.
   */
  skipped_steps: number;
  /**
   * Per-step results.
   */
  step_results: VerificationStepResult[];
  /**
   * Total duration of the phase in milliseconds.
   */
  total_duration_ms: number;
  /**
   * Total number of steps executed.
   */
  total_steps: number;
  [k: string]: unknown;
}
