/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { CheckIssueDetail } from './CheckIssueDetail';
import type { IndividualCheckResult } from './IndividualCheckResult';
import type { StepExecutionConfig } from './StepExecutionConfig';
import type { VerificationStepDetails } from './VerificationStepDetails';

/**
 * Result of a single step within a verification phase.
 */
export interface VerificationStepResult {
  config: StepExecutionConfig;
  /**
   * Step duration in milliseconds.
   */
  duration_ms: number;
  /**
   * ISO 8601 timestamp when the step ended.
   */
  ended_at: string | null;
  /**
   * Error message if the step failed.
   */
  error: string | null;
  /**
   * Arbitrary structured output produced by the step.
   */
  output_data: {
    [k: string]: unknown;
  } | null;
  /**
   * Path to a screenshot captured for the step.
   */
  screenshot_path: string | null;
  /**
   * ISO 8601 timestamp when the step started.
   */
  started_at: string | null;
  /**
   * ID of the step, if assigned.
   */
  step_id: string | null;
  /**
   * Zero-based index of the step within the phase.
   */
  step_index: number;
  /**
   * Display name of the step.
   */
  step_name: string;
  /**
   * Free-form step type label.
   */
  step_type: string;
  /**
   * Whether the step succeeded.
   */
  success: boolean;
  /**
   * Detailed captured output, if any.
   */
  verification_details: VerificationStepDetails | null;
  [k: string]: unknown;
}
