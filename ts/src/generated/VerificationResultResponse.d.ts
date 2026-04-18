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
import type { VerificationPhaseResult } from './VerificationPhaseResult';
import type { VerificationStepDetails } from './VerificationStepDetails';
import type { VerificationStepResult } from './VerificationStepResult';

/**
 * Response record for a single stored verification result.
 */
export interface VerificationResultResponse {
  /**
   * Whether all steps passed.
   */
  allPassed: boolean;
  /**
   * ISO 8601 timestamp when the record was created.
   */
  createdAt: string;
  /**
   * Whether a critical step failure short-circuited the phase.
   */
  criticalFailure: boolean;
  /**
   * Number of steps that failed.
   */
  failedSteps: number;
  /**
   * Unique identifier (UUID v4 string).
   */
  id: string;
  /**
   * 1-based iteration index within the workflow run.
   */
  iteration: number;
  /**
   * Number of steps that passed.
   */
  passedSteps: number;
  resultJson: VerificationPhaseResult;
  /**
   * Number of steps that were skipped.
   */
  skippedSteps: number;
  /**
   * ID of the owning task run.
   */
  taskRunId: string;
  /**
   * Total duration of the phase in milliseconds.
   */
  totalDurationMs: number;
  /**
   * Total number of steps executed.
   */
  totalSteps: number;
}
