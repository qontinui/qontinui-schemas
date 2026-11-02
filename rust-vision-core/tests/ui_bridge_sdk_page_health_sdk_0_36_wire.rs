// SPDX-License-Identifier: MIT
//! The page-health shapes that first appear after SDK 0.30.0, as recorded from
//! `@qontinui/ui-bridge` 0.36.0 (the version qontinui-runner pins).
//!
//! `tests/ui_bridge_sdk_page_health_wire.rs` enumerates the fixture directory,
//! so it already checks that these captures parse, lose nothing and match their
//! sidecars. It cannot say what they contain, and its named shape tests are all
//! on the 0.30.0 captures. This file holds the named shapes for 0.36.0, so a
//! re-capture that stopped exercising one of them reads red instead of passing
//! on the strength of a parse. It is a separate file because the sibling may not
//! be edited without the `wire-gate:override` label.

use qontinui_vision_core::{
    Observation, ObservationStatus, PageHealthSeverity, PageHealthValue, UnknownCode,
};
use std::path::PathBuf;

const ALL_INVISIBLE: &str = "sdk_0_36_0_page_health_all_invisible.json";
const EMPTY_REGISTRY: &str = "sdk_0_36_0_page_health_empty_registry.json";
const LAID_OUT: &str = "sdk_0_36_0_page_health_laid_out.json";
const VISIBILITY_PARTLY_UNSTATED: &str = "sdk_0_36_0_page_health_visibility_partly_unstated.json";
const VISIBILITY_UNSTATED: &str = "sdk_0_36_0_page_health_visibility_unstated.json";
const ZERO_ELEMENTS_WITH_COMPONENTS: &str =
    "sdk_0_36_0_page_health_zero_elements_with_components.json";
/// The 0.30.0 capture over the same recorded snapshot as [`ALL_INVISIBLE`].
const POPULATED_0_30_0: &str = "sdk_0_30_0_page_health_populated.json";

fn load(name: &str) -> Observation<PageHealthValue> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../rust/tests/fixtures/ui_bridge_sdk/page_health")
        .join(name);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_slice(&bytes).unwrap_or_else(|e| panic!("{name} does not parse: {e}"))
}

/// Everything here comes from the SDK's own producer at the pinned version, not
/// from a runner forward or an override.
fn assert_producer_is_the_sdk_at_0_36_0(name: &str, obs: &Observation<PageHealthValue>) {
    let producer = &obs.provenance().producer;
    assert_eq!(producer.id, "sdk/page-health", "{name}");
    assert_eq!(producer.version, "0.36.0", "{name}");
}

/// The same recorded snapshot across the 0.31.1 fix: a CRITICAL report over
/// nothing at 0.30.0, and an `unknown` that names what it saw at 0.36.0.
#[test]
fn all_invisible_snapshot_is_unknown_where_0_30_0_reported_critical() {
    let old = load(POPULATED_0_30_0);
    assert_eq!(old.status(), ObservationStatus::Measured);
    assert_eq!(
        old.value().map(|v| v.summary),
        Some(PageHealthSeverity::Critical)
    );
    assert_eq!(old.provenance().coverage.considered, 0);

    let new = load(ALL_INVISIBLE);
    assert_producer_is_the_sdk_at_0_36_0(ALL_INVISIBLE, &new);
    assert_eq!(new.status(), ObservationStatus::Unknown);
    assert!(new.value().is_none());
    let info = new.unknown_info().expect("an unknown carries its info");
    assert_eq!(info.code, UnknownCode::InputMissing);
    assert!(
        info.detail.contains("visible: false"),
        "the detail says why it could not look: {}",
        info.detail
    );
    let coverage = &new.provenance().coverage;
    assert_eq!((coverage.considered, coverage.measured), (0, 0));
    assert!(coverage.unmeasured.is_empty());

    assert_eq!(
        old.provenance().source,
        new.provenance().source,
        "both captures must be over the same recorded snapshot, or this comparison says nothing"
    );
}

#[test]
fn empty_registry_is_still_unknown_producer_not_run() {
    let obs = load(EMPTY_REGISTRY);
    assert_producer_is_the_sdk_at_0_36_0(EMPTY_REGISTRY, &obs);
    assert_eq!(obs.status(), ObservationStatus::Unknown);
    let info = obs.unknown_info().expect("an unknown carries its info");
    assert_eq!(info.code, UnknownCode::ProducerNotRun);
}

#[test]
fn laid_out_page_is_still_measured_with_full_coverage_and_a_warning() {
    let obs = load(LAID_OUT);
    assert_producer_is_the_sdk_at_0_36_0(LAID_OUT, &obs);
    assert_eq!(obs.status(), ObservationStatus::Measured);
    let value = obs.value().expect("a measured observation has a value");
    assert_eq!(value.summary, PageHealthSeverity::Warning);
    assert!(value.heatmap.iter().any(|row| row.contains('#')));
    let coverage = &obs.provenance().coverage;
    assert!(coverage.is_full(), "{coverage:?}");
    assert!(coverage.considered > 0, "{coverage:?}");
    assert_eq!(coverage.measured, coverage.considered);
    assert_eq!(coverage.measured, value.visible_count);
}

/// The one `measured` report over zero considered elements left at this SDK. It
/// is recorded because a consumer has to expect it, not because it is right: read
/// `coverage.considered` before trusting the roll-up.
#[test]
fn zero_elements_with_components_is_measured_over_nothing() {
    let obs = load(ZERO_ELEMENTS_WITH_COMPONENTS);
    assert_producer_is_the_sdk_at_0_36_0(ZERO_ELEMENTS_WITH_COMPONENTS, &obs);
    assert_eq!(obs.status(), ObservationStatus::Measured);
    let value = obs.value().expect("a measured observation has a value");
    assert_eq!(value.summary, PageHealthSeverity::Critical);
    assert_eq!(value.element_count, 0);
    let coverage = &obs.provenance().coverage;
    assert_eq!((coverage.considered, coverage.measured), (0, 0));
}

/// One element never said whether it is visible. The report stands over the
/// rest, and that element is counted, which is what keeps a degraded report from
/// reading as a complete one.
#[test]
fn partly_unstated_visibility_is_measured_and_counts_the_gap() {
    let obs = load(VISIBILITY_PARTLY_UNSTATED);
    assert_producer_is_the_sdk_at_0_36_0(VISIBILITY_PARTLY_UNSTATED, &obs);
    assert_eq!(obs.status(), ObservationStatus::Measured);
    let value = obs.value().expect("a measured observation has a value");
    let coverage = &obs.provenance().coverage;
    assert!(!coverage.is_full(), "{coverage:?}");
    assert_eq!(coverage.considered, coverage.measured + 1);
    assert_eq!(coverage.measured, value.visible_count);
    let [gap] = coverage.unmeasured.as_slice() else {
        panic!("expected exactly one unmeasured dimension: {coverage:?}");
    };
    assert_eq!(gap.dimension, "visibility");
    assert_eq!(gap.count, 1);
    assert_eq!(gap.code, UnknownCode::InputMissing);
}

/// No element said whether it is visible. Nothing can be placed, so there is no
/// report, but the `unknown` still carries the coverage that explains it.
#[test]
fn wholly_unstated_visibility_is_unknown_and_keeps_its_coverage() {
    let obs = load(VISIBILITY_UNSTATED);
    assert_producer_is_the_sdk_at_0_36_0(VISIBILITY_UNSTATED, &obs);
    assert_eq!(obs.status(), ObservationStatus::Unknown);
    assert!(obs.value().is_none());
    let info = obs.unknown_info().expect("an unknown carries its info");
    assert_eq!(info.code, UnknownCode::InputMissing);
    let coverage = &obs.provenance().coverage;
    assert!(coverage.considered > 0, "{coverage:?}");
    assert_eq!(coverage.measured, 0);
    let [gap] = coverage.unmeasured.as_slice() else {
        panic!("expected exactly one unmeasured dimension: {coverage:?}");
    };
    assert_eq!(gap.dimension, "visibility");
    assert_eq!(gap.count, coverage.considered);
    assert_eq!(gap.code, UnknownCode::InputMissing);
}
