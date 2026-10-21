//! [`class_rollup`]: per owner class count, anon RSS and placed / unplaced split.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::types::*;

/// One build as the rollup sees it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildRow {
    /// The classified owner.
    pub owner: OwnerClass,
    /// Anon RSS summed over the build's tree; `Unknown` when unreadable.
    pub anon_rss_bytes: Reading<u64>,
    /// Whether the build sits in a build or CI slice (bounded) or not.
    pub placed: bool,
}

/// A byte total with its unreadable members counted, so a sum that is a lower
/// bound says so instead of reading as exact.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bytes {
    /// Sum of the measured members.
    pub measured: u64,
    /// How many members had no readable value (each counted as unknown, not 0).
    pub unknown_members: u64,
}

impl Bytes {
    fn add(&mut self, r: Reading<u64>) {
        match r {
            Reading::Measured(v) => self.measured = self.measured.saturating_add(v),
            Reading::Unknown => self.unknown_members += 1,
        }
    }

    /// True when some member was unreadable, so `measured` understates.
    pub fn is_lower_bound(&self) -> bool {
        self.unknown_members > 0
    }
}

/// Totals for one owner class (or for all builds).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassTotals {
    /// Number of builds.
    pub count: u64,
    /// Anon RSS over every build.
    pub anon: Bytes,
    /// Number of placed builds.
    pub placed_count: u64,
    /// Anon RSS of placed builds.
    pub placed_anon: Bytes,
    /// Number of unplaced builds.
    pub unplaced_count: u64,
    /// Anon RSS of unplaced builds.
    pub unplaced_anon: Bytes,
}

impl ClassTotals {
    fn add(&mut self, b: &BuildRow) {
        self.count += 1;
        self.anon.add(b.anon_rss_bytes);
        if b.placed {
            self.placed_count += 1;
            self.placed_anon.add(b.anon_rss_bytes);
        } else {
            self.unplaced_count += 1;
            self.unplaced_anon.add(b.anon_rss_bytes);
        }
    }
}

/// The per-owner-class rollup. Every [`OwnerKind`] is present, zero-filled, so
/// an absent class is a measured zero builds, never a missing key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rollup {
    /// Totals per owner class.
    pub by_class: BTreeMap<OwnerKind, ClassTotals>,
    /// Totals over every build. Includes `Unknown`-owner builds at their
    /// measured RSS; nothing is excluded.
    pub total: ClassTotals,
}

impl Rollup {
    /// The totals of one class.
    pub fn class(&self, kind: OwnerKind) -> ClassTotals {
        self.by_class.get(&kind).copied().unwrap_or_default()
    }
}

/// Roll `builds` up per owner class. No build is excluded: an `Unknown`-owner
/// build counts at its measured RSS in its class and in the total, and an
/// unreadable RSS is counted in `unknown_members`, never added as 0.
pub fn class_rollup(builds: &[BuildRow]) -> Rollup {
    let mut by_class: BTreeMap<OwnerKind, ClassTotals> = OwnerKind::ALL
        .iter()
        .map(|k| (*k, ClassTotals::default()))
        .collect();
    let mut total = ClassTotals::default();
    for b in builds {
        total.add(b);
        by_class.entry(b.owner.kind()).or_default().add(b);
    }
    Rollup { by_class, total }
}
