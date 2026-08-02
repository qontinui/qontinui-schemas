/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ProvenanceMix } from "./ProvenanceMix";
import type { SpecSection } from "./SpecSection";

/**
 * Per-section coverage breakdown. Mirrors the overall-verdict shape so a consumer
 * can render the same widget at either granularity.
 */
export interface SectionVerdict {
  /**
   * Fraction of this section's `assumed` nodes the generator filled.
   */
  assumedFillRate: number;
  /**
   * `numerator / denominator` for this section (`1.0` when denominator is 0).
   */
  coverage: number;
  /**
   * Mean credibility of this section's `inferred` nodes (∈ [0,1]).
   */
  credibility: number;
  provenanceMix: ProvenanceMix;
  section: SpecSection;
  /**
   * Freshness of the binding observation, when answered from a cached fallback.
   */
  stalenessSeconds?: number | null;
  [k: string]: unknown;
}
