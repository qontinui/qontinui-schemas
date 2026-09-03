//! How much of a snapshot is actually MEASURABLE.
//!
//! An analyzer that returns no findings is saying one of two completely
//! different things: "I checked and the page is fine", or "the input carried
//! nothing I could check". [`SnapshotCoverage`] is the evidence that tells
//! those apart, computed from the snapshot itself rather than from the
//! producer's claims about it.
//!
//! # Why this lives in the crate
//!
//! It did not, until now. The identical four counters have been computed for
//! some time by the projector that feeds this crate —
//! `qontinui-claude-config/scripts/uibridge-to-elementsnapshot.py:297-320`,
//! behind an opt-in `--stats` flag, printed to **stderr**, consumed by
//! nothing, and failing nothing when it reads zero. That is a measurement in
//! the wrong place: out of band, in another repo, downstream of the crate
//! whose analyzers are the party that needs the answer.
//!
//! This module is a PORT of that computation, and the two must not drift.
//! `with_bbox`/`with_z`/`with_text`/`inter` there are
//! [`SnapshotCoverage::with_geometry`]/[`with_stacking`]/[`with_text`]/
//! [`interactable`] here, with the same semantics element for element.
//!
//! [`with_stacking`]: SnapshotCoverage::with_stacking
//! [`with_text`]: SnapshotCoverage::with_text
//! [`interactable`]: SnapshotCoverage::interactable

use serde::{Deserialize, Serialize};

use crate::element_snapshot::ElementSnapshot;

/// Per-dimension counts of what a snapshot's elements actually carry.
///
/// Every field is a count of elements, never a ratio, and the denominator is
/// always [`Self::elements`] — a caller wanting "84 % had geometry" divides,
/// and a caller wanting "did ANY element have geometry" compares to zero.
/// Both readings are needed: the analyzers' preconditions are floors
/// (see [`crate::analyzers::AnalyzerVerdict`]), while a human reading a
/// report wants the proportion.
///
/// # Coverage measures PRESENCE, not CORRECTNESS
///
/// Nothing here inspects whether a populated value is *right*. A snapshot in
/// which every field is present and every value is wrong scores 100 % on all
/// five counters. That distinction is load-bearing for
/// [`Self::with_stacking`] in particular — see its own doc.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotCoverage {
    /// Total elements in the snapshot. The denominator for every other
    /// field, and zero here is its own condition: an empty snapshot is not a
    /// clean page, and [`crate::analyzers::elements`] reports it as
    /// `empty_snapshot` at [`crate::Severity::Critical`].
    pub elements: usize,
    /// Elements carrying a `bbox`.
    ///
    /// **This is the counter the vacuous-pass defect turns on.** Every
    /// geometric check in [`crate::analyzers::layout`] — overlap, occlusion,
    /// zero-area, alignment — filters to bbox-bearing elements, so at zero
    /// the analyzer skips everything and returns an empty finding list that
    /// is byte-identical to the one a healthy page produces.
    pub with_geometry: usize,
    /// Elements carrying a `z_index`, i.e. a populated paint rank.
    ///
    /// **This field counts POPULATED ranks and asserts NOTHING about whether
    /// the producer resolved them correctly.** It is a presence count, and
    /// reading it as a trust signal is a mistake this doc exists to prevent.
    ///
    /// [`crate::Element::z_index`] is specified as a GLOBAL paint rank over
    /// the whole snapshot, and [`crate::analyzers::layout`] compares it
    /// across arbitrary non-ancestor pairs on that basis. A producer that
    /// projects `getComputedStyle(el).zIndex` into it supplies a *per-context
    /// CSS value* instead, which for the canonical failing shape — a `z-50`
    /// dropdown nested inside a `z-10` title bar, against a `z-20` sibling of
    /// that bar — yields the OPPOSITE ordering from the truth. In that
    /// snapshot this counter reads 100 % while every occlusion verdict
    /// derived from it is silently inverted.
    ///
    /// So a high `with_stacking` can accompany confidently wrong occlusion
    /// findings, and a coverage number that is confidently wrong is worse
    /// than an absent one. Use this field to answer "could occlusion be
    /// evaluated at all?" and never "can the occlusion verdicts be trusted?".
    /// The producer-side resolution of that hazard is tracked separately
    /// (coord finding `10ae2835-37cc-449a-8c4f-e0c7aaaf0aa1`) and is
    /// deliberately not something this type can detect.
    ///
    /// Note also that zero here is ORDINARY, not degenerate: the projector
    /// emits `z_index` only when the computed `zIndex` parses as an integer,
    /// and `auto` — the value most elements on a real page carry —
    /// deliberately does not. A healthy snapshot routinely scores zero, which
    /// is why no analyzer treats this counter as a precondition.
    pub with_stacking: usize,
    /// Elements carrying a `text` value.
    ///
    /// Presence of the field, matching the projector's `"text" in p`. The
    /// projector never emits an empty string (it writes `text` only for a
    /// non-blank value), so for projected snapshots this equals "carries
    /// non-empty text". A hand-built snapshot with `Some("")` throughout
    /// would count here while [`crate::analyzers::typography`], which skips
    /// empty strings, still measures nothing — a residual gap this counter
    /// deliberately does not paper over by inventing a stricter rule than
    /// the reference implementation's.
    pub with_text: usize,
    /// Elements flagged `interactable`.
    ///
    /// Easy to overlook, because it is a filter no geometry-shaped mental
    /// model expects: layout's pairwise-overlap pass filters on
    /// `interactable` FIRST and only then on `bbox`, so zero here empties
    /// that pass completely even on a snapshot where every element is
    /// positioned.
    ///
    /// **It empties one pass, not the analyzer.** The directed-occlusion,
    /// zero-area and alignment passes build their working sets from `bbox`
    /// alone, so a snapshot with nothing interactable is still examined by
    /// most of `layout` — which is why zero here is a
    /// [`crate::AnalyzerVerdict::Degraded`] rather than a `Blocked`. The
    /// crate's own `007-widget-occludes-label` corpus fixture is that
    /// snapshot, and it yields a `Severity::Critical` `occlusion` finding.
    pub interactable: usize,
}

impl SnapshotCoverage {
    /// Count what `snapshot` carries. Pure; O(elements); reads no field
    /// beyond the five it counts.
    pub fn of(snapshot: &ElementSnapshot) -> Self {
        let mut c = Self {
            elements: snapshot.elements.len(),
            with_geometry: 0,
            with_stacking: 0,
            with_text: 0,
            interactable: 0,
        };
        for el in &snapshot.elements {
            if el.bbox.is_some() {
                c.with_geometry += 1;
            }
            if el.z_index.is_some() {
                c.with_stacking += 1;
            }
            if el.text.is_some() {
                c.with_text += 1;
            }
            if el.interactable {
                c.interactable += 1;
            }
        }
        c
    }

    /// One-line human rendering, in the projector's own wording so the two
    /// surfaces read the same to anyone comparing them:
    /// `"3 elements: 3 with geometry, 0 with stacking order, 3 with text, 2 interactive"`.
    pub fn summary(&self) -> String {
        format!(
            "{} elements: {} with geometry, {} with stacking order, {} with text, \
             {} interactive",
            self.elements,
            self.with_geometry,
            self.with_stacking,
            self.with_text,
            self.interactable
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::element_snapshot::Element;
    use crate::frame::Region;

    fn el(id: &str) -> Element {
        Element {
            id: id.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn counts_each_dimension_independently() {
        let snap = ElementSnapshot {
            elements: vec![
                Element {
                    bbox: Some(Region {
                        x: 0,
                        y: 0,
                        w: 10,
                        h: 10,
                    }),
                    text: Some("Save".into()),
                    interactable: true,
                    z_index: Some(3),
                    ..el("a")
                },
                Element {
                    text: Some("Title".into()),
                    ..el("b")
                },
                el("c"),
            ],
            ..Default::default()
        };
        let cov = SnapshotCoverage::of(&snap);
        assert_eq!(
            cov,
            SnapshotCoverage {
                elements: 3,
                with_geometry: 1,
                with_stacking: 1,
                with_text: 2,
                interactable: 1,
            }
        );
    }

    #[test]
    fn an_empty_snapshot_is_all_zeroes_rather_than_absent() {
        // Zero is a measurement here, not a missing one. The distinction
        // matters because the verdict layer reads these counters as floors:
        // an absent coverage would have to be treated as UNKNOWN.
        let cov = SnapshotCoverage::of(&ElementSnapshot::default());
        assert_eq!(cov.elements, 0);
        assert_eq!(cov.with_geometry, 0);
        assert_eq!(cov.interactable, 0);
    }

    #[test]
    fn serializes_camel_case_for_the_wire() {
        let cov = SnapshotCoverage::of(&ElementSnapshot::default());
        let wire = serde_json::to_value(&cov).unwrap();
        for key in [
            "elements",
            "withGeometry",
            "withStacking",
            "withText",
            "interactable",
        ] {
            assert!(wire.get(key).is_some(), "missing {key} in {wire}");
        }
    }
}
