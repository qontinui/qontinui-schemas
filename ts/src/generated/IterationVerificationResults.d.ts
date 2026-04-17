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
  aiPassed: boolean;
  /**
   * AI verification results (empty if skipped).
   */
  aiResults: VerificationResult[];
  /**
   * Overall pass/fail.
   */
  allPassed: boolean;
  /**
   * Criterion overrides applied in this iteration.
   */
  appliedOverrides?: CriterionOverride[];
  /**
   * Whether all required deterministic checks passed.
   */
  deterministicPassed: boolean;
  /**
   * Deterministic verification results.
   */
  deterministicResults: VerificationResult[];
  /**
   * Human-readable summary of failures.
   */
  failureSummary?: string | null;
  /**
   * Iteration number.
   */
  iteration: number;
  /**
   * Criteria that failed but were accepted due to overrides.
   */
  overriddenCriteria?: string[];
  [k: string]: unknown;
}
