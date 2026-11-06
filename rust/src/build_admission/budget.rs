//! The bytes builds may reserve on a host (D3).
//!
//! ```text
//! budget_bytes = MemTotal − reserve_floor − non_build_p95_24h − ci_reservation − unleased_build_rss
//! ```

use serde::{Deserialize, Serialize};

use super::types::{HostFacts, Policy, Resolved};

/// A budget term that could not be resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BudgetTerm {
    NonBuild,
    CiReservation,
    UnleasedBuildRss,
}

/// The budget, or the terms that kept it from being known.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Budget {
    Known {
        bytes: u64,
        reserve_floor_bytes: u64,
    },
    Unknown {
        missing: Vec<BudgetTerm>,
    },
}

impl Budget {
    pub fn bytes(&self) -> Option<u64> {
        match self {
            Budget::Known { bytes, .. } => Some(*bytes),
            Budget::Unknown { .. } => None,
        }
    }
}

/// D3's budget. A `not_supported` term subtracts nothing (the platform has no
/// such consumer to measure); an `unknown` term makes the whole budget unknown.
/// Saturates at 0 rather than going negative.
pub fn budget(facts: &HostFacts, policy: &Policy) -> Budget {
    let reserve = policy.reserve_floor(facts.mem_total_bytes);
    let mut missing = Vec::new();
    let mut sub = |term: BudgetTerm, r: Resolved<u64>| -> u64 {
        match r {
            Resolved::Value(v) => v,
            Resolved::Dropped => 0,
            Resolved::Unknown => {
                missing.push(term);
                0
            }
        }
    };
    let non_build = sub(BudgetTerm::NonBuild, facts.non_build_p95_bytes.resolve());
    let ci = sub(
        BudgetTerm::CiReservation,
        facts.ci_reservation_bytes.resolve(),
    );
    let unleased = sub(
        BudgetTerm::UnleasedBuildRss,
        facts.unleased_build_rss_bytes.resolve(),
    );
    if !missing.is_empty() {
        return Budget::Unknown { missing };
    }
    Budget::Known {
        bytes: facts
            .mem_total_bytes
            .saturating_sub(reserve)
            .saturating_sub(non_build)
            .saturating_sub(ci)
            .saturating_sub(unleased),
        reserve_floor_bytes: reserve,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_admission::types::{Fact, GIB};

    fn facts() -> HostFacts {
        HostFacts {
            mem_total_bytes: 100 * GIB,
            cpus: 16,
            mem_available_bytes: Fact::Measured(50 * GIB),
            psi_mem_full_avg10: Fact::Measured(0.0),
            non_build_p95_bytes: Fact::Measured(20 * GIB),
            ci_reservation_bytes: Fact::Measured(30 * GIB),
            unleased_build_rss_bytes: Fact::Measured(10 * GIB),
            admissions_paused: false,
        }
    }

    #[test]
    fn subtracts_every_term() {
        // reserve = max(4 GiB, 3 GiB) = 4 GiB; 100 - 4 - 20 - 30 - 10 = 36.
        assert_eq!(
            budget(&facts(), &Policy::default()),
            Budget::Known {
                bytes: 36 * GIB,
                reserve_floor_bytes: 4 * GIB
            }
        );
    }

    #[test]
    fn not_supported_drops_and_unknown_withholds() {
        let mut f = facts();
        f.ci_reservation_bytes = Fact::NotSupported;
        assert_eq!(budget(&f, &Policy::default()).bytes(), Some(66 * GIB));
        f.non_build_p95_bytes = Fact::Unknown;
        f.unleased_build_rss_bytes = Fact::Unknown;
        assert_eq!(
            budget(&f, &Policy::default()),
            Budget::Unknown {
                missing: vec![BudgetTerm::NonBuild, BudgetTerm::UnleasedBuildRss]
            }
        );
    }

    #[test]
    fn live_only_and_seed_are_usable_values() {
        let mut f = facts();
        f.non_build_p95_bytes = Fact::LiveOnly(20 * GIB);
        assert_eq!(budget(&f, &Policy::default()).bytes(), Some(36 * GIB));
    }

    #[test]
    fn saturates_at_zero() {
        let mut f = facts();
        f.unleased_build_rss_bytes = Fact::Measured(500 * GIB);
        assert_eq!(budget(&f, &Policy::default()).bytes(), Some(0));
    }
}
