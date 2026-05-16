/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { AssertionSeverityCounts } from './AssertionSeverityCounts';
import type { MatchOutcome } from './MatchOutcome';
import type { RecommendedState } from './RecommendedState';
import type { SpecCheckConfidence } from './SpecCheckConfidence';

/**
 * Aggregate summary across all evaluated states.
 */
export interface SpecCheckSummary {
  matchOutcome: MatchOutcome;
  /**
   * Mean of per-state match rates, weighted equally.
   */
  overallMatchRate: number;
  /**
   * The single state (if any) the matcher recommends the caller treat
   * as "current". `None` when the matcher has no preference.
   */
  recommendedState?: RecommendedState | null;
  severityCounts: AssertionSeverityCounts;
}
