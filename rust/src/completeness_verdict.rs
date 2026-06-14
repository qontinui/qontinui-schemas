//! Completeness Verdict + Rubric (v0) — **Artifact 2** of the
//! functional-spec-contract keystone (`2026-06-13-functional-spec-contract.md`).
//!
//! Scores a generated app+backend against the source-observed
//! [`crate::functional_spec::FunctionalSpec`], **in spec units**. The output is a
//! [`CompletenessVerdict`]; this module's doc comments + [`CompletenessVerdict`]
//! methods *are* the rubric semantics (Phase 2 of the plan), co-designed with the
//! schema so the two cannot disagree.
//!
//! ## Relationship to `DriftVerdict` (deliberate field-compatibility, NOT reuse)
//!
//! This borrows the **field vocabulary** of `qontinui-coord`'s `DriftVerdict`
//! (`coverage` / `provenance` / `credibility` / `staleness_seconds`) but is **not the
//! same struct**: `DriftVerdict` is `Serialize`-only, coord-internal, not codegen'd,
//! and lives in coord — which depends on `qontinui-schemas`, not the reverse. So
//! [`CompletenessVerdict`] is authored **fresh here** with `#[derive(JsonSchema)]` so
//! it codegens to TS + Python for the web/python verifiers, while staying
//! field-compatible on the shared keys so the peer's test phase can read it as a
//! `DriftVerdict`-family envelope.
//!
//! ## The rubric (Phase 2 semantics — coverage formula + accounting split)
//!
//! - **Denominator** = count of `Observed` + `Inferred` spec nodes (entities, fields,
//!   operations, ui_states, navigation edges, auth facts).
//! - **Numerator** = those a verifier confirmed present-and-behaving in the generated
//!   app — i.e. denominator minus the `Observed`/`Inferred` [`CoverageGap`]s.
//! - `Assumed` nodes are reported **separately** as [`CompletenessVerdict::assumed_fill_rate`]
//!   and are **never** folded into the headline [`CompletenessVerdict::coverage`]. This
//!   is the machine-checked form of "measured, not assumed": coverage is coverage of
//!   what the frontend actually revealed.
//! - **`coverage` = numerator / denominator**, or `1.0` when the denominator is `0`
//!   (a spec with nothing observable is vacuously fully covered).
//!
//! ### Counting `ui_states` / `navigation` nodes
//!
//! [`crate::functional_spec::FunctionalSpec`]'s `ui_states` / `navigation` reuse
//! [`crate::ir::IrState`] / [`crate::ir::IrTransition`], which carry an
//! [`crate::ir::IrProvenance`] (source / file / line) — **not** a
//! [`crate::functional_spec::SpecProvenance`] tag. For v0 the rubric counts **each
//! `IrState` and each `IrTransition` as one `Observed` node**: they are the
//! directly-rendered states / transitions, the most frontend-observable artifacts
//! there are. (A verifier MAY downgrade a node whose `IrProvenance.source` is
//! `"ai-generated"` to `Inferred`; v0 keeps the simple "rendered ⇒ observed" rule so
//! the denominator definition stays invariant.) Every other section's nodes carry an
//! explicit `SpecProvenance` and are tallied by it.
//!
//! ## Per-gap output → a work-list (runner-local first, NOT a coord gate per gap)
//!
//! Each uncovered `Observed`/`Inferred` node becomes a [`CoverageGap`] in
//! [`CompletenessVerdict::gaps`]. The **orchestration reconciler** (the peer
//! conductor) re-derives this work-list from the durable verdict each tick and
//! re-dispatches a generation worker for each gap — that is what makes the loop
//! self-converging. **Coord gates stay reserved for cross-phase durable checkpoints**
//! (e.g. "backend deployed before app integration test"), never one gate per gap.
//!
//! ## UI-state dimension reuses Spec-Check (does not reinvent matching)
//!
//! The `ui_states`/`navigation` half of coverage is exactly what
//! [`crate::spec_check`] (`evaluate(snapshot, spec) -> SpecCheckResult`) already
//! computes. The verdict therefore **embeds** the
//! [`crate::spec_check::SpecCheckResult`] for the UI dimension
//! ([`CompletenessVerdict::ui_states_spec_check`]) rather than recomputing element
//! matching, and adds only the dimensions Spec-Check does not model — entities,
//! operations, auth.
//!
//! Wire format: `camelCase`; ISO-8601 `String` timestamps; `deny_unknown_fields`
//! omitted for additive-only v0 evolution.

use crate::functional_spec::SpecProvenance;
use crate::spec_check::SpecCheckResult;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The five scorable sections of a [`crate::functional_spec::FunctionalSpec`].
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum SpecSection {
    Entities,
    Operations,
    UiStates,
    Navigation,
    Auth,
}

/// Count of nodes by provenance, within a section or overall. The
/// `observed + inferred` sum is the coverage denominator; `assumed` is tracked
/// separately for the assumption-fill rate.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProvenanceMix {
    #[serde(default)]
    pub observed: u32,
    #[serde(default)]
    pub inferred: u32,
    #[serde(default)]
    pub assumed: u32,
}

impl ProvenanceMix {
    /// The coverage **denominator**: `observed + inferred`. `assumed` is excluded
    /// by construction.
    pub fn denominator(&self) -> u32 {
        self.observed + self.inferred
    }

    /// Tally a single node's provenance into the mix.
    pub fn add(&mut self, p: SpecProvenance) {
        match p {
            SpecProvenance::Observed => self.observed += 1,
            SpecProvenance::Inferred => self.inferred += 1,
            SpecProvenance::Assumed => self.assumed += 1,
        }
    }
}

/// Why an `Observed`/`Inferred` node is not covered in the generated app.
/// (`Assumed` nodes never produce a gap — they are reported via the fill rate.)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum GapReason {
    /// The generator never produced anything for this node.
    NotGenerated,
    /// Something was generated but the verifier observed it behaving wrong.
    BehaviorMismatch,
    /// The node could not be verified in the generated app (a coverage gap in
    /// the verifier itself, not necessarily a missing feature).
    Unverifiable,
}

/// One uncovered `Observed`/`Inferred` node — a work-list item the orchestration
/// reconciler re-dispatches generation for.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CoverageGap {
    /// Dotted ref into the spec, e.g. `"entities.Invoice.fields.amount"` or
    /// `"operations.createInvoice"`.
    #[serde(rename = "ref")]
    pub r#ref: String,
    /// Which section the gapped node belongs to.
    pub section: SpecSection,
    /// Provenance of the gapped node — always `Observed` or `Inferred`.
    pub node_provenance: SpecProvenance,
    /// Why it is uncovered.
    pub reason: GapReason,
    /// Free-form diagnostic detail (e.g. the verifier's near-miss explanation).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Per-section coverage breakdown. Mirrors the overall-verdict shape so a consumer
/// can render the same widget at either granularity.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SectionVerdict {
    pub section: SpecSection,
    /// `numerator / denominator` for this section (`1.0` when denominator is 0).
    pub coverage: f64,
    /// Fraction of this section's `assumed` nodes the generator filled.
    #[serde(default)]
    pub assumed_fill_rate: f64,
    pub provenance_mix: ProvenanceMix,
    /// Mean credibility of this section's `inferred` nodes (∈ [0,1]).
    #[serde(default)]
    pub credibility: f64,
    /// Freshness of the binding observation, when answered from a cached fallback.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub staleness_seconds: Option<i64>,
}

/// The output of the test-phase verify subtask: how completely a generated
/// app+backend covers the source-observed Functional Spec.
///
/// Serializes as an A2A `DataPart` in `completion_reports.artifacts`. Durable,
/// fully re-derivable — the reconciler re-reads it each tick.
///
/// Does not derive `PartialEq`: the embedded [`crate::spec_check::SpecCheckResult`]
/// does not implement it. Compare verdicts by their serialized JSON when needed.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CompletenessVerdict {
    /// The `spec_version` of the [`crate::functional_spec::FunctionalSpec`] this
    /// verdict scores.
    pub spec_version: String,

    /// Overall coverage ∈ [0,1] of `Observed` + `Inferred` nodes confirmed
    /// present-and-behaving. The headline number. (Shared `DriftVerdict` key.)
    pub coverage: f64,

    /// Fraction of `Assumed` nodes the generator filled ∈ [0,1] — reported
    /// **separately**, never folded into `coverage`.
    #[serde(default)]
    pub assumed_fill_rate: f64,

    /// Overall node counts by provenance.
    pub provenance_mix: ProvenanceMix,

    /// Mean credibility of `Inferred` nodes ∈ [0,1]. (Shared `DriftVerdict` key.)
    #[serde(default)]
    pub credibility: f64,

    /// Freshness of the binding observation when answered from a cached fallback;
    /// `None` on a fresh live answer. (Shared `DriftVerdict` key.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub staleness_seconds: Option<i64>,

    /// Per-section breakdown.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sections: Vec<SectionVerdict>,

    /// The gap work-list — uncovered `Observed`/`Inferred` nodes the reconciler
    /// re-dispatches generation for.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gaps: Vec<CoverageGap>,

    /// Embedded Spec-Check result for the `ui_states` / `navigation` dimension —
    /// reused, not recomputed. `None` when the UI dimension was not evaluated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ui_states_spec_check: Option<SpecCheckResult>,

    /// ISO-8601 UTC timestamp the verdict was computed.
    pub evaluated_at: String,
}

impl CompletenessVerdict {
    /// The canonical coverage formula (Phase 2): `numerator / denominator`, where
    /// `denominator = observed + inferred` and `numerator = denominator - (# of
    /// Observed/Inferred gaps)`. A `0` denominator yields `1.0` (a spec with
    /// nothing observable is vacuously fully covered).
    ///
    /// This is the single source of truth for the formula; a verifier builds the
    /// [`ProvenanceMix`] and gap list, then calls this to fill
    /// [`CompletenessVerdict::coverage`] — guaranteeing the stored number always
    /// matches the documented rubric.
    pub fn coverage_from(mix: &ProvenanceMix, observed_inferred_gaps: u32) -> f64 {
        let denom = mix.denominator();
        if denom == 0 {
            return 1.0;
        }
        let numerator = denom.saturating_sub(observed_inferred_gaps);
        f64::from(numerator) / f64::from(denom)
    }

    /// Number of gaps whose node is `Observed` or `Inferred` (i.e. gaps that count
    /// against coverage). `Assumed` nodes never appear in `gaps`, but this filters
    /// defensively so the formula stays correct even if a caller misclassifies.
    pub fn counted_gaps(&self) -> u32 {
        self.gaps
            .iter()
            .filter(|g| g.node_provenance != SpecProvenance::Assumed)
            .count() as u32
    }

    /// Whether [`CompletenessVerdict::coverage`] is internally consistent with the
    /// provenance mix and the counted gaps (within f64 epsilon). Used by tests and
    /// by a verifier as a post-construction self-check.
    pub fn coverage_is_consistent(&self) -> bool {
        let expected = Self::coverage_from(&self.provenance_mix, self.counted_gaps());
        (self.coverage - expected).abs() < 1e-9
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn verdict(mix: ProvenanceMix, gaps: Vec<CoverageGap>) -> CompletenessVerdict {
        let counted = gaps
            .iter()
            .filter(|g| g.node_provenance != SpecProvenance::Assumed)
            .count() as u32;
        CompletenessVerdict {
            spec_version: "0".into(),
            coverage: CompletenessVerdict::coverage_from(&mix, counted),
            assumed_fill_rate: 0.0,
            provenance_mix: mix,
            credibility: 0.0,
            staleness_seconds: None,
            sections: vec![],
            gaps,
            ui_states_spec_check: None,
            evaluated_at: "2026-06-13T00:00:00Z".into(),
        }
    }

    fn gap(p: SpecProvenance) -> CoverageGap {
        CoverageGap {
            r#ref: "entities.Invoice".into(),
            section: SpecSection::Entities,
            node_provenance: p,
            reason: GapReason::NotGenerated,
            detail: None,
        }
    }

    #[test]
    fn coverage_denominator_excludes_assumed() {
        // 6 observed + 2 inferred + 4 assumed; 2 observed/inferred gaps.
        let mix = ProvenanceMix {
            observed: 6,
            inferred: 2,
            assumed: 4,
        };
        assert_eq!(mix.denominator(), 8);
        // numerator = 8 - 2 = 6; coverage = 6/8 = 0.75.
        assert!((CompletenessVerdict::coverage_from(&mix, 2) - 0.75).abs() < 1e-12);
    }

    #[test]
    fn assumed_nodes_never_count_against_coverage() {
        let mix = ProvenanceMix {
            observed: 4,
            inferred: 0,
            assumed: 10,
        };
        // Even with an (incorrectly supplied) assumed gap, counted_gaps drops it.
        let v = verdict(mix, vec![gap(SpecProvenance::Assumed)]);
        assert_eq!(v.counted_gaps(), 0);
        assert!(
            (v.coverage - 1.0).abs() < 1e-12,
            "all observed covered → 1.0"
        );
        assert!(v.coverage_is_consistent());
    }

    #[test]
    fn empty_observable_spec_is_vacuously_complete() {
        let mix = ProvenanceMix {
            observed: 0,
            inferred: 0,
            assumed: 3,
        };
        assert_eq!(CompletenessVerdict::coverage_from(&mix, 0), 1.0);
    }

    #[test]
    fn consistency_check_catches_wrong_coverage() {
        let mut v = verdict(
            ProvenanceMix {
                observed: 4,
                inferred: 0,
                assumed: 0,
            },
            vec![gap(SpecProvenance::Observed)],
        );
        assert!(v.coverage_is_consistent());
        v.coverage = 0.99; // tamper
        assert!(!v.coverage_is_consistent());
    }

    #[test]
    fn verdict_round_trips_with_embedded_spec_check_absent() {
        let v = verdict(
            ProvenanceMix {
                observed: 1,
                inferred: 1,
                assumed: 0,
            },
            vec![],
        );
        let json = serde_json::to_string(&v).unwrap();
        assert!(
            !json.contains("uiStatesSpecCheck"),
            "absent embed must skip-serialize"
        );
        let round: CompletenessVerdict = serde_json::from_str(&json).unwrap();
        // CompletenessVerdict has no PartialEq (embedded SpecCheckResult lacks it);
        // compare by re-serialized JSON.
        assert_eq!(
            serde_json::to_value(&round).unwrap(),
            serde_json::to_value(&v).unwrap()
        );
    }

    #[test]
    fn section_enum_camel_case() {
        assert_eq!(
            serde_json::to_string(&SpecSection::UiStates).unwrap(),
            "\"uiStates\""
        );
        assert_eq!(
            serde_json::to_string(&SpecSection::Entities).unwrap(),
            "\"entities\""
        );
    }
}
