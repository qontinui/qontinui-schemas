//! Completeness **evaluation** — the rubric walk that turns a
//! [`crate::functional_spec::FunctionalSpec`] + coverage evidence into a
//! [`crate::completeness_verdict::CompletenessVerdict`].
//!
//! [`crate::completeness_verdict`] froze the verdict *shape* + the coverage *formula*
//! ([`CompletenessVerdict::coverage_from`]). This module adds the missing piece both
//! the test-phase verifier and the website→mobile vertical slice need: the
//! deterministic **walk** that
//!
//! 1. enumerates every provenance-bearing spec node ([`enumerate_nodes`]) under a
//!    stable dotted [`SpecNode::ref`] convention (`entities.Invoice.fields.status`,
//!    `auth.roles.admin`, `operations.createInvoice.effect`, …), counting each
//!    `ui_states` / `navigation` node as one `Observed` node per the v0 rubric, and
//! 2. diffs those nodes against a [`CoverageEvidence`] (what a generator produced +
//!    a verifier confirmed) to build the full verdict — `provenance_mix`, per-section
//!    [`SectionVerdict`]s, the [`CoverageGap`] work-list, the assumption-fill rate,
//!    and credibility means ([`evaluate_completeness`]).
//!
//! The evidence is the **seam**: in the real loop it is derived from observing the
//! generated app; in the vertical slice it comes from stub generators. The walk is
//! identical in both worlds — only the evidence source differs. The
//! `ui_states`/`navigation` *element-matching* dimension (`ui_states_spec_check`) is
//! deliberately deferred to the real app-generator (#1), which swaps in
//! [`crate::spec_check::evaluate`]; here those sections are scored from supplied
//! evidence like every other section so the slice stays a pure, deterministic walk
//! with no runner dependency.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::completeness_verdict::{
    CompletenessVerdict, CoverageGap, GapReason, ProvenanceMix, SectionVerdict, SpecSection,
};
use crate::functional_spec::{FunctionalSpec, SpecProvenance};

/// One provenance-bearing node of a [`FunctionalSpec`], with the canonical dotted
/// `ref` a generator targets and a verifier reports gaps against.
#[derive(Debug, Clone, PartialEq)]
pub struct SpecNode {
    /// Dotted ref, e.g. `"entities.Invoice.fields.status"` / `"auth.roles.admin"`.
    pub r#ref: String,
    pub section: SpecSection,
    /// `ui_states` / `navigation` nodes are always `Observed` (v0 rubric).
    pub provenance: SpecProvenance,
    /// Present only on `Inferred` nodes that carry one; drives credibility means.
    pub credibility: Option<f64>,
}

/// Why a generator did not cover an `Observed`/`Inferred` node, as a verifier would
/// report it. `Assumed` nodes never gap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GapEvidence {
    pub reason: GapReason,
    pub detail: Option<String>,
}

/// What a generation+verification pass produced for a spec — the input to
/// [`evaluate_completeness`]. In the real loop this is derived from observing the
/// generated app; in the vertical slice it is built from stub generators.
///
/// Resolution rule per `Observed`/`Inferred` node `ref`:
/// - `ref ∈ covered` → covered (counts toward coverage).
/// - `ref ∈ gaps` → a [`CoverageGap`] with the supplied reason+detail.
/// - in neither → a `NotGenerated` gap with no detail (the default a stub yields by
///   simply not producing the node).
///
/// `Assumed` node `ref ∈ filled_assumed` → filled (counts toward `assumed_fill_rate`).
#[derive(Debug, Clone, Default)]
pub struct CoverageEvidence {
    /// Observed/Inferred refs the verifier confirmed present-and-behaving.
    pub covered: BTreeSet<String>,
    /// Observed/Inferred refs the verifier flagged, with reason+detail.
    pub gaps: BTreeMap<String, GapEvidence>,
    /// Assumed refs the generator filled.
    pub filled_assumed: BTreeSet<String>,
}

/// Enumerate every provenance-bearing node of the spec in canonical order, assigning
/// each its dotted `ref`. Mirrors the independent tally in
/// `tests/functional_spec_fixtures.rs` (entities → operations → ui_states →
/// navigation → auth), so the walk and the regression tally cannot disagree.
pub fn enumerate_nodes(spec: &FunctionalSpec) -> Vec<SpecNode> {
    let mut nodes = Vec::new();
    for e in &spec.entities {
        nodes.push(SpecNode {
            r#ref: format!("entities.{}", e.name),
            section: SpecSection::Entities,
            provenance: e.confidence,
            credibility: e.credibility,
        });
        for f in &e.fields {
            nodes.push(SpecNode {
                r#ref: format!("entities.{}.fields.{}", e.name, f.name),
                section: SpecSection::Entities,
                provenance: f.confidence,
                credibility: f.credibility,
            });
        }
        for r in &e.relationships {
            nodes.push(SpecNode {
                r#ref: format!("entities.{}.relationships.{}", e.name, r.to),
                section: SpecSection::Entities,
                provenance: r.confidence,
                credibility: r.credibility,
            });
        }
    }
    for op in &spec.operations {
        nodes.push(SpecNode {
            r#ref: format!("operations.{}", op.name),
            section: SpecSection::Operations,
            provenance: op.confidence,
            credibility: op.credibility,
        });
        for inp in &op.inputs {
            if let Some(v) = &inp.validation {
                nodes.push(SpecNode {
                    r#ref: format!("operations.{}.inputs.{}.validation", op.name, inp.field),
                    section: SpecSection::Operations,
                    provenance: v.confidence,
                    credibility: v.credibility,
                });
            }
        }
        if let Some(eff) = &op.effect {
            nodes.push(SpecNode {
                r#ref: format!("operations.{}.effect", op.name),
                section: SpecSection::Operations,
                provenance: eff.confidence,
                credibility: eff.credibility,
            });
        }
    }
    for s in &spec.ui_states {
        nodes.push(SpecNode {
            r#ref: format!("uiStates.{}", s.id),
            section: SpecSection::UiStates,
            provenance: SpecProvenance::Observed,
            credibility: None,
        });
    }
    for t in &spec.navigation {
        nodes.push(SpecNode {
            r#ref: format!("navigation.{}", t.id),
            section: SpecSection::Navigation,
            provenance: SpecProvenance::Observed,
            credibility: None,
        });
    }
    if let Some(a) = &spec.auth {
        nodes.push(SpecNode {
            r#ref: "auth".to_string(),
            section: SpecSection::Auth,
            provenance: a.confidence,
            credibility: a.credibility,
        });
        for role in &a.roles {
            nodes.push(SpecNode {
                r#ref: format!("auth.roles.{}", role.name),
                section: SpecSection::Auth,
                provenance: role.confidence,
                credibility: role.credibility,
            });
        }
    }
    nodes
}

/// Canonical section order for the per-section breakdown.
const SECTION_ORDER: [SpecSection; 5] = [
    SpecSection::Entities,
    SpecSection::Operations,
    SpecSection::UiStates,
    SpecSection::Navigation,
    SpecSection::Auth,
];

#[derive(Default)]
struct SectionAcc {
    mix: ProvenanceMix,
    counted_gaps: u32,
    assumed_total: u32,
    assumed_filled: u32,
    cred_sum: f64,
    cred_n: u32,
}

impl SectionAcc {
    fn assumed_fill_rate(&self) -> f64 {
        if self.assumed_total == 0 {
            0.0
        } else {
            f64::from(self.assumed_filled) / f64::from(self.assumed_total)
        }
    }
    fn credibility(&self) -> f64 {
        if self.cred_n == 0 {
            0.0
        } else {
            self.cred_sum / f64::from(self.cred_n)
        }
    }
}

/// Walk the spec, diff against the evidence, and assemble the full
/// [`CompletenessVerdict`]. `evaluated_at` is passed in (not read from the clock) so
/// the result is deterministic and testable.
pub fn evaluate_completeness(
    spec: &FunctionalSpec,
    evidence: &CoverageEvidence,
    evaluated_at: &str,
) -> CompletenessVerdict {
    let mut by_section: HashMap<SpecSection, SectionAcc> = HashMap::new();
    let mut overall = SectionAcc::default();
    let mut gaps: Vec<CoverageGap> = Vec::new();

    for node in enumerate_nodes(spec) {
        let acc = by_section.entry(node.section).or_default();
        acc.mix.add(node.provenance);
        overall.mix.add(node.provenance);

        match node.provenance {
            SpecProvenance::Assumed => {
                acc.assumed_total += 1;
                overall.assumed_total += 1;
                if evidence.filled_assumed.contains(&node.r#ref) {
                    acc.assumed_filled += 1;
                    overall.assumed_filled += 1;
                }
            }
            SpecProvenance::Observed | SpecProvenance::Inferred => {
                // Credibility means span every Inferred node (covered or gapped).
                if node.provenance == SpecProvenance::Inferred {
                    if let Some(c) = node.credibility {
                        acc.cred_sum += c;
                        acc.cred_n += 1;
                        overall.cred_sum += c;
                        overall.cred_n += 1;
                    }
                }
                if !evidence.covered.contains(&node.r#ref) {
                    let (reason, detail) = match evidence.gaps.get(&node.r#ref) {
                        Some(g) => (g.reason, g.detail.clone()),
                        None => (GapReason::NotGenerated, None),
                    };
                    gaps.push(CoverageGap {
                        r#ref: node.r#ref.clone(),
                        section: node.section,
                        node_provenance: node.provenance,
                        reason,
                        detail,
                    });
                    acc.counted_gaps += 1;
                    overall.counted_gaps += 1;
                }
            }
        }
    }

    let sections: Vec<SectionVerdict> = SECTION_ORDER
        .iter()
        .filter_map(|sec| {
            by_section.get(sec).map(|acc| SectionVerdict {
                section: *sec,
                coverage: CompletenessVerdict::coverage_from(&acc.mix, acc.counted_gaps),
                assumed_fill_rate: acc.assumed_fill_rate(),
                provenance_mix: acc.mix,
                credibility: acc.credibility(),
                staleness_seconds: None,
            })
        })
        .collect();

    CompletenessVerdict {
        spec_version: spec.spec_version.clone(),
        coverage: CompletenessVerdict::coverage_from(&overall.mix, overall.counted_gaps),
        assumed_fill_rate: overall.assumed_fill_rate(),
        provenance_mix: overall.mix,
        credibility: overall.credibility(),
        staleness_seconds: None,
        sections,
        gaps,
        ui_states_spec_check: None,
        evaluated_at: evaluated_at.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::completeness_verdict::SpecSection;

    fn ev(covered: &[&str], filled: &[&str]) -> CoverageEvidence {
        CoverageEvidence {
            covered: covered.iter().map(|s| s.to_string()).collect(),
            gaps: BTreeMap::new(),
            filled_assumed: filled.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn empty_spec_is_vacuously_complete() {
        let spec = FunctionalSpec {
            spec_version: "0".into(),
            target: crate::functional_spec::SpecTarget {
                source_url: "https://x.test".into(),
                observed_at: None,
            },
            entities: vec![],
            operations: vec![],
            ui_states: vec![],
            navigation: vec![],
            auth: None,
            assumptions: vec![],
        };
        let v = evaluate_completeness(&spec, &CoverageEvidence::default(), "2026-06-14T00:00:00Z");
        assert!((v.coverage - 1.0).abs() < 1e-12);
        assert!(v.gaps.is_empty());
        assert!(v.sections.is_empty());
        assert!(v.coverage_is_consistent());
    }

    #[test]
    fn uncovered_observed_node_becomes_not_generated_gap() {
        let spec = FunctionalSpec {
            spec_version: "0".into(),
            target: crate::functional_spec::SpecTarget {
                source_url: "https://x.test".into(),
                observed_at: None,
            },
            entities: vec![crate::functional_spec::Entity {
                name: "Widget".into(),
                fields: vec![],
                relationships: vec![],
                confidence: SpecProvenance::Observed,
                provenance: None,
                credibility: None,
            }],
            operations: vec![],
            ui_states: vec![],
            navigation: vec![],
            auth: None,
            assumptions: vec![],
        };
        // No coverage at all → the one Observed node is a NotGenerated gap.
        let v = evaluate_completeness(&spec, &CoverageEvidence::default(), "2026-06-14T00:00:00Z");
        assert_eq!(v.gaps.len(), 1);
        assert_eq!(v.gaps[0].r#ref, "entities.Widget");
        assert_eq!(v.gaps[0].section, SpecSection::Entities);
        assert_eq!(v.gaps[0].reason, GapReason::NotGenerated);
        assert!((v.coverage - 0.0).abs() < 1e-12);
        assert!(v.coverage_is_consistent());
        // Covering it flips coverage to 1.0 with no gaps.
        let v2 = evaluate_completeness(&spec, &ev(&["entities.Widget"], &[]), "2026-06-14T00:00:00Z");
        assert!(v2.gaps.is_empty());
        assert!((v2.coverage - 1.0).abs() < 1e-12);
    }
}
