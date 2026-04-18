/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { Confidence } from './Confidence';
import type { CriterionType } from './CriterionType';

/**
 * Result of a single verification check.
 */
export interface VerificationResult {
  /**
   * Confidence level (for AI verification).
   */
  confidence?: Confidence | null;
  /**
   * The criterion that was checked.
   */
  criterionId: string;
  criterionType: CriterionType;
  /**
   * Issues found (if failed).
   */
  issues?: string[];
  /**
   * What was observed.
   */
  observations?: string[];
  /**
   * Whether the check passed.
   */
  passed: boolean;
  /**
   * Raw output / details, e.g., captured command output.
   */
  rawOutput?: string | null;
  /**
   * Suggestions for fixing (if failed).
   */
  suggestions?: string[];
}
