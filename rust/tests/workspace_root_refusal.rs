//! The workspace-root refusal speaks the operator's language — plan
//! `2026-09-20-the-published-product-works-without-knowing-a-development-environment-exists`,
//! Phase B3.
//!
//! `WorkspaceRootUnresolved`'s display must name no fleet noun (no
//! workspace-layout name, no maintainer environment variable) unless the
//! operator themselves set the environment variable that was at fault — the
//! one case where naming it is the actionable answer. Exhaustive over every
//! `(RootSource, RootRejection)` pair.
//!
//! Reads the repo-root `fleet-nouns.toml`, so this is a repo test (excluded
//! from the published crate).

mod common;

use qontinui_types::paths::{
    RejectedRoot, RootRejection, RootSource, SiblingCheckoutAbsent, SiblingFallback,
    WorkspaceRootUnresolved,
};
use qontinui_types::refusal::RefusalSource;

use common::fleet_nouns::FleetNouns;

const SOURCES: [RootSource; 4] = [
    RootSource::Env,
    RootSource::EnvAlias,
    RootSource::Configured,
    RootSource::Probes,
];

const REASONS: [RootRejection; 6] = [
    RootRejection::Blank,
    RootRejection::Relative,
    RootRejection::NotADirectory,
    RootRejection::FilesystemRoot,
    RootRejection::NoAnchor,
    RootRejection::NothingConfigured,
];

#[test]
fn the_sentence_names_no_fleet_noun_unless_the_operator_set_that_variable() {
    let nouns = FleetNouns::load();
    let mut bad = Vec::new();
    for source in SOURCES {
        for reason in REASONS {
            let err = WorkspaceRootUnresolved {
                rejected: RejectedRoot { source, reason },
            };
            // The envelope's own sentence is always clean.
            let rendered = err
                .refusal(RefusalSource::Runner, "2026-09-30T00:00:00Z")
                .render();
            if !nouns.hits(&rendered).is_empty() {
                bad.push(format!("render: {rendered}"));
            }
            // The display adds the input at fault only for an explicit input.
            let shown = err.to_string();
            let operator_set_a_variable = matches!(source, RootSource::Env | RootSource::EnvAlias);
            if !operator_set_a_variable && !nouns.hits(&shown).is_empty() {
                bad.push(format!("display ({source:?}, {reason:?}): {shown}"));
            }
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

/// Vacuity guard: the one case allowed to name a variable does name it, and
/// the matcher sees it — so a clean run above is evidence.
#[test]
fn the_check_sees_the_variable_the_operator_set() {
    let nouns = FleetNouns::load();
    let err = WorkspaceRootUnresolved {
        rejected: RejectedRoot {
            source: RootSource::Env,
            reason: RootRejection::Blank,
        },
    };
    assert!(!nouns.hits(&err.to_string()).is_empty(), "{err}");
}

#[test]
fn sibling_refusal_adds_no_fleet_noun_of_its_own() {
    let nouns = FleetNouns::load();
    for fallback in [SiblingFallback::EmbeddedCopy, SiblingFallback::Unsupported] {
        // A product-neutral name: any hit can only come from the helper's text.
        let s = SiblingCheckoutAbsent::new("tooling", fallback).to_string();
        assert!(nouns.hits(&s).is_empty(), "{s}");
    }
}
