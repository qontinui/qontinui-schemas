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
import type { VerificationResultResponse } from './VerificationResultResponse';
import type { VerificationStepDetails } from './VerificationStepDetails';
import type { VerificationStepResult } from './VerificationStepResult';

/**
 * Response for listing verification results for a task run.
 */
export interface VerificationResultsListResponse {
  /**
   * Total number of verification results.
   */
  count: number;
  /**
   * Number of iterations that failed.
   */
  failedIterations: number;
  /**
   * Number of iterations that passed.
   */
  passedIterations: number;
  /**
   * All verification results for the task run.
   */
  results: VerificationResultResponse[];
  /**
   * ID of the owning task run.
   */
  taskRunId: string;
}
