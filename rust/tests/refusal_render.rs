//! `Refusal::render()` never returns an empty sentence and never contains a
//! fleet noun — plan
//! `2026-09-20-the-published-product-works-without-knowing-a-development-environment-exists`,
//! Phase D1 acceptance.
//!
//! Exhaustive rather than sampled: every `RefusalCode` × every
//! `NextActionKind` × a set of target, delay and discriminator shapes. The
//! targets are product-neutral, so a hit can only come from the render TABLE
//! itself — which is the property: `render()` adds no fleet noun of its own.
//! (What a caller puts in `target` or `discriminator` is the caller's text,
//! checked by that caller's own lint.)
//!
//! Reads the repo-root `fleet-nouns.toml`, so this is a repo test (excluded
//! from the published crate).

mod common;

use qontinui_types::glossary::GlossaryTerm;
use qontinui_types::refusal::{NextAction, NextActionKind, Refusal, RefusalCode, RefusalSource};

use common::fleet_nouns::FleetNouns;

const AT: &str = "2026-09-29T00:00:00Z";

fn every_shape() -> Vec<Refusal> {
    let targets = [
        None,
        Some(""),
        Some("workspace_root"),
        Some("https://example.com/settings"),
        Some("a1b2c3"),
    ];
    let delays = [None, Some(0), Some(1), Some(45), Some(u32::MAX)];
    let discriminators = [None, Some(""), Some("backend")];
    let mut out = Vec::new();
    for &code in RefusalCode::ALL {
        for &kind in NextActionKind::ALL {
            for target in targets {
                for retry_after_s in delays {
                    for disc in discriminators {
                        let mut na = NextAction::new(kind);
                        na.target = target.map(str::to_string);
                        na.retry_after_s = retry_after_s;
                        let mut r = Refusal::new(code, na, RefusalSource::Runner, AT)
                            .with_glossary_terms([GlossaryTerm::Gate]);
                        r.discriminator = disc.map(str::to_string);
                        out.push(r);
                    }
                }
            }
        }
    }
    out
}

#[test]
fn render_is_never_empty_and_never_names_a_fleet_noun() {
    let nouns = FleetNouns::load();
    let shapes = every_shape();
    assert_eq!(
        shapes.len(),
        RefusalCode::ALL.len() * NextActionKind::ALL.len() * 5 * 5 * 3
    );
    let mut bad = Vec::new();
    for r in &shapes {
        let s = r.render();
        if s.trim().is_empty() {
            bad.push(format!("empty render for {r:?}"));
            continue;
        }
        let hits = nouns.hits(&s);
        if !hits.is_empty() {
            bad.push(format!("{hits:?} in {s:?}"));
        }
    }
    assert!(
        bad.is_empty(),
        "{} bad renders:\n  {}",
        bad.len(),
        bad.join("\n  ")
    );
}

/// The table strings themselves, independent of any composition.
#[test]
fn no_table_string_names_a_fleet_noun() {
    let nouns = FleetNouns::load();
    for &code in RefusalCode::ALL {
        assert!(!code.headline().trim().is_empty(), "{code:?}");
        assert!(nouns.hits(code.headline()).is_empty(), "{code:?}");
    }
    for &kind in NextActionKind::ALL {
        let s = NextAction::new(kind).render();
        assert!(!s.trim().is_empty(), "{kind:?}");
        assert!(nouns.hits(&s).is_empty(), "{kind:?}: {s}");
    }
}

/// Vacuity guard: the same check DOES flag a fleet noun when one reaches the
/// sentence (here through a caller-supplied target), so a green run above is
/// evidence rather than a matcher that sees nothing.
#[test]
fn the_check_sees_a_fleet_noun_that_reaches_the_sentence() {
    let nouns = FleetNouns::load();
    let r = Refusal::new(
        RefusalCode::SiblingCheckoutAbsent,
        NextAction::new(NextActionKind::RunCommand).with_target("cd ../qontinui-schemas"),
        RefusalSource::Runner,
        AT,
    );
    assert!(
        nouns.hits(&r.render()).contains(&"repo_layout"),
        "{}",
        r.render()
    );
}
