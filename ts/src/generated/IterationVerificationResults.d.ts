/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { Confidence } from './Confidence';
import type { CriterionOverride } from './CriterionOverride';
import type { CriterionType } from './CriterionType';
import type { VerificationResult } from './VerificationResult';

/**
 * Aggregated verification results for a single iteration.
 */
export interface IterationVerificationResults {
  /**
   * Whether all required AI checks passed (true if no AI criteria).
   */
  ai_passed: boolean;
  /**
   * AI verification results (empty if skipped).
   */
  ai_results: VerificationResult[];
  /**
   * Overall pass/fail.
   */
  all_passed: boolean;
  /**
   * Criterion overrides applied in this iteration.
   */
  applied_overrides?: CriterionOverride[];
  /**
   * Whether all required deterministic checks passed.
   */
  deterministic_passed: boolean;
  /**
   * Deterministic verification results.
   */
  deterministic_results: VerificationResult[];
  /**
   * Human-readable summary of failures.
   */
  failure_summary?: string | null;
  /**
   * Iteration number.
   */
  iteration: number;
  /**
   * Criteria that failed but were accepted due to overrides.
   */
  overridden_criteria?: string[];
  [k: string]: unknown;
}
