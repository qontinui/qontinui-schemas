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
  durationMs: number;
  /**
   * ISO 8601 timestamp when the step ended.
   */
  endedAt: string | null;
  /**
   * Error message if the step failed.
   */
  error: string | null;
  /**
   * Arbitrary structured output produced by the step.
   */
  outputData: {
    [k: string]: unknown;
  } | null;
  /**
   * Path to a screenshot captured for the step.
   */
  screenshotPath: string | null;
  /**
   * ISO 8601 timestamp when the step started.
   */
  startedAt: string | null;
  /**
   * ID of the step, if assigned.
   */
  stepId: string | null;
  /**
   * Zero-based index of the step within the phase.
   */
  stepIndex: number;
  /**
   * Display name of the step.
   */
  stepName: string;
  /**
   * Free-form step type label.
   */
  stepType: string;
  /**
   * Whether the step succeeded.
   */
  success: boolean;
  /**
   * Detailed captured output, if any.
   */
  verificationDetails: VerificationStepDetails | null;
  [k: string]: unknown;
}
