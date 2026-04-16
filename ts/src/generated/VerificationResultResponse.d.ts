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
  all_passed: boolean;
  /**
   * ISO 8601 timestamp when the record was created.
   */
  created_at: string;
  /**
   * Whether a critical step failure short-circuited the phase.
   */
  critical_failure: boolean;
  /**
   * Number of steps that failed.
   */
  failed_steps: number;
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
  passed_steps: number;
  result_json: VerificationPhaseResult;
  /**
   * Number of steps that were skipped.
   */
  skipped_steps: number;
  /**
   * ID of the owning task run.
   */
  task_run_id: string;
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
