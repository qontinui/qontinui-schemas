/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { RootCauseCategory } from './RootCauseCategory';

/**
 * Result of a single diagnostic evaluation.
 */
export interface DiagnosticResult {
  /**
   * Assertion results (each with pass/fail and details).
   */
  assertionResults: unknown[];
  /**
   * AI-generated explanation of the failure.
   */
  diagnosis?: string | null;
  /**
   * Page-health status blob from UI Bridge.
   */
  pageHealth: {
    [k: string]: unknown;
  };
  /**
   * Whether all assertions passed and the page is healthy.
   */
  passed: boolean;
  /**
   * AI-generated recommendation for the next iteration's prompt.
   */
  promptRewriteSuggestion?: string | null;
  /**
   * Classified root cause (only meaningful when `passed == false`).
   */
  rootCause?: RootCauseCategory | null;
  [k: string]: unknown;
}
