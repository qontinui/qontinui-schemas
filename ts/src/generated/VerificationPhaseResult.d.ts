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
  allPassed: boolean;
  /**
   * Whether a critical step failure short-circuited the phase.
   */
  criticalFailure: boolean;
  /**
   * Number of steps that failed.
   */
  failedSteps: number;
  /**
   * Whether pass/fail is determined by gates rather than overall step counts.
   */
  gateBasedEvaluation: boolean;
  /**
   * Per-gate evaluation results.
   */
  gateResults: GateEvaluationResult[];
  /**
   * 1-based iteration index within the workflow run.
   */
  iteration: number;
  /**
   * Number of steps that passed.
   */
  passedSteps: number;
  /**
   * Number of steps that were skipped.
   */
  skippedSteps: number;
  /**
   * Per-step results.
   */
  stepResults: VerificationStepResult[];
  /**
   * Total duration of the phase in milliseconds.
   */
  totalDurationMs: number;
  /**
   * Total number of steps executed.
   */
  totalSteps: number;
}
