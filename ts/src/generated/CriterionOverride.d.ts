/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * An override for a verification criterion.
 *
 * Workers can emit these to indicate that a failing criterion should be
 * accepted as-is with justification, rather than requiring a fix.
 */
export interface CriterionOverride {
  /**
   * The criterion ID being overridden.
   */
  criterionId: string;
  /**
   * What specifically is being overridden (e.g., class name, file path).
   */
  item: string;
  /**
   * Iteration when this override was recorded.
   */
  iteration: number;
  /**
   * Justification for why this override is acceptable.
   */
  justification: string;
  /**
   * ISO 8601 timestamp when the override was recorded.
   */
  recordedAt: string;
  /**
   * Worker ID that provided the override (if multi-worker).
   */
  workerId?: string | null;
  [k: string]: unknown;
}
