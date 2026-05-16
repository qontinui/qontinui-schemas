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
   * Diagnostic explaining why `recommended_state` is `None`. Set when
   * the matcher *deliberately* withholds a recommendation (e.g. the
   * spec failed distinctness validation per §5.12); absent otherwise
   * (e.g. when every state simply scored below the confidence floor).
   * Free-form for forward-compat; current values:
   * `"spec_validation_failed"`.
   */
  recommendationReason?: string | null;
  /**
   * The single state (if any) the matcher recommends the caller treat
   * as "current". `None` when the matcher has no preference.
   */
  recommendedState?: RecommendedState | null;
  severityCounts: AssertionSeverityCounts;
}
