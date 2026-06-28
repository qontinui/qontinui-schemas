/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { AssertionMiss } from "./AssertionMiss";
import type { AssertionOutcome } from "./AssertionOutcome";
import type { AssertionResult } from "./AssertionResult";
import type { AssertionSeverityCounts } from "./AssertionSeverityCounts";
import type { BridgeFingerprint } from "./BridgeFingerprint";
import type { CandidateMiss } from "./CandidateMiss";
import type { CoverageGap } from "./CoverageGap";
import type { FieldDiff } from "./FieldDiff";
import type { GapReason } from "./GapReason";
import type { MatchOutcome } from "./MatchOutcome";
import type { MatchedElement } from "./MatchedElement";
import type { MissReason } from "./MissReason";
import type { ProvenanceMix } from "./ProvenanceMix";
import type { RecommendedState } from "./RecommendedState";
import type { SectionVerdict } from "./SectionVerdict";
import type { SpecCheckConfidence } from "./SpecCheckConfidence";
import type { SpecCheckResult } from "./SpecCheckResult";
import type { SpecCheckSummary } from "./SpecCheckSummary";
import type { SpecProvenance } from "./SpecProvenance";
import type { SpecSection } from "./SpecSection";
import type { StateMatchResult } from "./StateMatchResult";

/**
 * The output of the test-phase verify subtask: how completely a generated
 * app+backend covers the source-observed Functional Spec.
 *
 * Serializes as an A2A `DataPart` in `completion_reports.artifacts`. Durable,
 * fully re-derivable — the reconciler re-reads it each tick.
 *
 * Does not derive `PartialEq`: the embedded [`crate::spec_check::SpecCheckResult`]
 * does not implement it. Compare verdicts by their serialized JSON when needed.
 */
export interface CompletenessVerdict {
  /**
   * Fraction of `Assumed` nodes the generator filled ∈ [0,1] — reported
   * **separately**, never folded into `coverage`.
   */
  assumedFillRate: number;
  /**
   * Overall coverage ∈ [0,1] of `Observed` + `Inferred` nodes confirmed
   * present-and-behaving. The headline number. (Shared `DriftVerdict` key.)
   */
  coverage: number;
  /**
   * Mean credibility of `Inferred` nodes ∈ [0,1]. (Shared `DriftVerdict` key.)
   */
  credibility: number;
  /**
   * ISO-8601 UTC timestamp the verdict was computed.
   */
  evaluatedAt: string;
  /**
   * The gap work-list — uncovered `Observed`/`Inferred` nodes the reconciler
   * re-dispatches generation for.
   */
  gaps?: CoverageGap[];
  provenanceMix: ProvenanceMix;
  /**
   * Per-section breakdown.
   */
  sections?: SectionVerdict[];
  /**
   * The `spec_version` of the [`crate::functional_spec::FunctionalSpec`] this
   * verdict scores.
   */
  specVersion: string;
  /**
   * Freshness of the binding observation when answered from a cached fallback;
   * `None` on a fresh live answer. (Shared `DriftVerdict` key.)
   */
  stalenessSeconds?: number | null;
  /**
   * Embedded Spec-Check result for the `ui_states` / `navigation` dimension —
   * reused, not recomputed. `None` when the UI dimension was not evaluated.
   */
  uiStatesSpecCheck?: SpecCheckResult | null;
  [k: string]: unknown;
}
