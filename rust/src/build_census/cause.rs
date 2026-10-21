//! [`capacity_cause`]: which class a stalled host is attributable to (D6).
//!
//! A DETECTING consumer: an unknown input suppresses the assertion, it never
//! fires it and never defaults to `Other`.

use serde::{Deserialize, Serialize};

use super::rollup::Rollup;
use super::types::*;

/// A class's anon share must be strictly above this fraction of all anon
/// memory counted, to dominate.
pub const DOMINANCE_SHARE: f64 = 0.6;

/// A class's own slice shows stall when its memory PSI `some` avg60 (percent)
/// is strictly above this.
pub const STALL_PSI_SOME_AVG60: f64 = 10.0;

/// A PSI percentage cannot exceed this; a larger reading is a broken probe.
const MAX_PSI_PERCENT: f64 = 100.0;

/// A PSI reading usable as evidence: finite and within 0..=100 percent. A NaN, infinite
/// or negative value is a broken probe, so it is unreadable, never calm.
fn usable_psi(r: Reading<f64>) -> Option<f64> {
    r.value_finite()
        .filter(|p| (0.0..=MAX_PSI_PERCENT).contains(p))
}

/// The memory classes a host's anon memory is split into.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CauseClass {
    /// Non-CI builds: agent, human, orphan and unattributed.
    Builds,
    /// Live agent sessions' own (non-build) memory.
    Sessions,
    /// CI jobs' builds.
    Ci,
    /// Everything else on the host.
    Other,
}

/// The verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "verdict", content = "class", rename_all = "snake_case")]
pub enum Cause {
    /// One class dominates the anon memory AND its own slice shows stall.
    Attributed(CauseClass),
    /// Everything was read and no class both dominates and stalls.
    Mixed,
    /// Something needed was unreadable, or there is nothing to attribute.
    Unknown,
}

/// What the census read beyond the build rows: each class's slice pressure and
/// the anon memory of the classes that are not builds.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SlicePressure {
    /// Memory PSI `some` avg60 of the builds' slices.
    pub builds_psi: Reading<f64>,
    /// Memory PSI `some` avg60 of the CI slices.
    pub ci_psi: Reading<f64>,
    /// Memory PSI `some` avg60 of the sessions' slice.
    pub sessions_psi: Reading<f64>,
    /// Memory PSI `some` avg60 of the rest of the host.
    pub other_psi: Reading<f64>,
    /// Anon bytes of live sessions, builds excluded.
    pub sessions_anon_bytes: Reading<u64>,
    /// Anon bytes of everything else, builds excluded.
    pub other_anon_bytes: Reading<u64>,
}

/// Decide the cause of a stalled host.
///
/// Anon bytes per class: `Builds` is every non-CI owner class, `Ci` the
/// `CiJob` class, `Sessions` and `Other` come from `pressure`. A class is
/// `Attributed` only when its share of the total is above [`DOMINANCE_SHARE`]
/// AND its own slice PSI is above [`STALL_PSI_SOME_AVG60`]. The verdict is
/// `Unknown` when any build RSS was unreadable, when a class that holds memory
/// has no readable slice pressure, when a non-build anon total is unreadable,
/// or when there is no anon memory at all; it is never `Other` by default.
pub fn capacity_cause(rollup: &Rollup, pressure: &SlicePressure) -> Cause {
    if rollup.total.anon.is_lower_bound() {
        return Cause::Unknown;
    }
    let ci = rollup.class(OwnerKind::CiJob).anon.measured;
    // Deliberate: only `CiJob` is charged to Ci, so a CI-slice build whose runner
    // is unnamed (`Unknown`) lands in Builds — UNKNOWN is never free.
    let builds = rollup.total.anon.measured.saturating_sub(ci);
    let (Some(sessions), Some(other)) = (
        pressure.sessions_anon_bytes.value(),
        pressure.other_anon_bytes.value(),
    ) else {
        return Cause::Unknown;
    };

    let classes = [
        (CauseClass::Builds, builds, pressure.builds_psi),
        (CauseClass::Ci, ci, pressure.ci_psi),
        (CauseClass::Sessions, sessions, pressure.sessions_psi),
        (CauseClass::Other, other, pressure.other_psi),
    ];
    // A class holding memory whose slice cannot be read withholds the verdict.
    if classes
        .iter()
        .any(|(_, bytes, psi)| *bytes > 0 && usable_psi(*psi).is_none())
    {
        return Cause::Unknown;
    }
    let total: u128 = classes.iter().map(|(_, b, _)| u128::from(*b)).sum();
    if total == 0 {
        return Cause::Unknown;
    }

    for (class, bytes, psi) in classes {
        let share = bytes as f64 / total as f64;
        let stalled = usable_psi(psi).is_some_and(|p| p > STALL_PSI_SOME_AVG60);
        if share > DOMINANCE_SHARE && stalled {
            return Cause::Attributed(class);
        }
    }
    Cause::Mixed
}
