// SPDX-License-Identifier: MIT
//! [`PageHealthValue`] — the `value` of a `measured` page-health
//! [`Observation`](crate::Observation): the canonical wire type of what the
//! UI Bridge SDK's `/control/page-health` answers (producer
//! `sdk/page-health`, `@qontinui/ui-bridge` >= 0.30.0) as
//! `Observation<PageHealthValue>`.
//!
//! The TS twin is `PageHealthValue` in ui-bridge
//! `packages/ui-bridge/src/server/page-health.ts`. What holds the two
//! together is not this file but the recorded SDK output under
//! `rust/tests/fixtures/ui_bridge_sdk/page_health/`, which
//! `tests/ui_bridge_sdk_page_health_wire.rs` parses strictly into
//! `Observation<PageHealthValue>` (plan
//! `2026-09-20-ui-bridge-observations-distinguish-cannot-see-from-not-present-and-carry-provenance`,
//! Phase 3).
//!
//! Coverage (how many visible elements carried geometry) is NOT here: it
//! lives in the observation's `provenance.coverage`. A report only ever
//! exists inside a `measured` observation, and "could not look" (no
//! `elements`, an empty registry, visible elements none of which carry
//! geometry) is an `unknown` with a typed code. One gap remains on the SDK
//! side at 0.30.0: elements registered but NONE visible still yields a
//! `measured` CRITICAL report over zero considered elements — the
//! `..._populated.json` fixture records exactly that.
//!
//! Field names are the SDK's wire names verbatim (`element_count`, not
//! camelCase), so there is no `rename_all`. Unrecognised keys are ignored
//! rather than refused, matching the envelope's own forward-compatibility
//! rule (see [`crate::observation`]); the fixture test closes the other
//! direction by asserting that a parse loses nothing the wire carried.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// One finding's severity, and the report's roll-up. Spelled in capitals on
/// the wire (`"OK" | "WARNING" | "CRITICAL"`).
///
/// The variant ORDER is load-bearing: the derived `Ord` ranks `Ok < Warning <
/// Critical`, and the wire gate (`tests/ui_bridge_sdk_page_health_wire.rs`)
/// checks that the producer's `summary` equals the `max` of the findings'
/// severities. Never reorder these variants.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "UPPERCASE")]
pub enum PageHealthSeverity {
    Ok,
    Warning,
    Critical,
}

/// One heuristic family's result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PageHealthFinding {
    /// Check name (e.g. `spatial_coverage`, `layout_regions`).
    pub check: String,
    /// Severity rolled up from this check's heuristics.
    pub severity: PageHealthSeverity,
    /// Human-readable summary of what was observed.
    pub detail: String,
    /// Supporting data; its shape is per check, so it stays a JSON object.
    pub data: Map<String, Value>,
}

/// The page-health report: the `value` of a `measured` page-health
/// observation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PageHealthValue {
    /// Worst severity across all findings.
    pub summary: PageHealthSeverity,
    /// Per-check findings, one per heuristic family.
    pub findings: Vec<PageHealthFinding>,
    /// 20-row x 20-col ASCII heatmap (`#` = covered, `.` = empty).
    pub heatmap: Vec<String>,
    /// Total discovered elements (visible + hidden).
    pub element_count: u64,
    /// Visible elements with a normalized rect.
    pub visible_count: u64,
}
