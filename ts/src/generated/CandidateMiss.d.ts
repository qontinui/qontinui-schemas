/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { FieldDiff } from "./FieldDiff";

/**
 * A near-match candidate — an element that scored above the cut-off but
 * didn't fully match the assertion criteria.
 */
export interface CandidateMiss {
  /**
   * UI Bridge element identifier (if exposed).
   */
  elementId?: string | null;
  /**
   * Per-field diffs explaining why the candidate didn't fully match.
   */
  fieldDiffs?: FieldDiff[];
  /**
   * CSS selector or DOM path identifying the candidate.
   */
  path: string;
  /**
   * ARIA role of the candidate.
   */
  role?: string | null;
  /**
   * Aggregate match score (0.0..=1.0).
   */
  score: number;
  /**
   * Visible text of the candidate.
   */
  text?: string | null;
}
