/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { CandidateMiss } from "./CandidateMiss";
import type { FieldDiff } from "./FieldDiff";
import type { MissReason } from "./MissReason";

/**
 * Top-level miss diagnostic for a failed assertion.
 */
export interface AssertionMiss {
  /**
   * Top-N near-match candidates, capped externally. Ordered by score
   * descending.
   */
  candidates?: CandidateMiss[];
  reason: MissReason;
}
