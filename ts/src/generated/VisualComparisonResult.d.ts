/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Result of visual comparison of a screenshot against a baseline.
 */
export interface VisualComparisonResult {
  /**
   * Baseline screenshot ID, if any.
   */
  baselineId?: string | null;
  /**
   * Comparison result ID.
   */
  comparisonId: string;
  /**
   * URL to a diff image, if generated.
   */
  diffImageUrl?: string | null;
  /**
   * Number of diff regions detected.
   */
  diffRegionCount: number;
  /**
   * Whether the comparison passed.
   */
  passed: boolean;
  /**
   * Similarity score in the range `[0.0, 1.0]`.
   */
  similarityScore: number;
  /**
   * Threshold used for the pass/fail decision.
   */
  threshold: number;
  [k: string]: unknown;
}
