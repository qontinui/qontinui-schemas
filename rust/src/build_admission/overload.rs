//! The governor's overload ladder as a pure state machine (D5 rungs 3–4 and
//! resume). Rungs 0–2 (admit, queue, shrink) are [`super::admit`] and
//! [`super::order`]; this module decides, once per governor tick, whether to
//! freeze one build, thaw one, or hold.
//!
//! - **Overloaded** when memory PSI full avg10 has stayed at or above
//!   `psi_pause` for `pause_sustain_s`, or MemAvailable is below the reserve
//!   floor. An `unknown` input can never make the host overloaded (an unknown
//!   input must not fire the ladder); `not_supported` PSI leaves the memory arm.
//! - **Pause** (rung 3): the newest, lowest-class RUNNING lease that is not
//!   `operator`. Rung 4, only when rung 3 has nothing left: the newest RUNNING
//!   unleased build tree. One action per tick, then `settle_s` before the next.
//! - **Resume**: only governor-paused leases and governor-paused unleased trees,
//!   strictly oldest-paused first (a younger one never jumps the queue), when
//!   PSI is known (or not supported) and below `psi_resume`, and
//!   `MemAvailable ≥ reserve_floor + max(0, est − current_anon)` — the
//!   candidate's resident pages are already counted in MemAvailable. An
//!   unleased tree has no estimate, so it needs only the floor.
//! - Between `psi_resume` and `psi_pause` nothing moves (hysteresis).
//! - Nothing here kills or times out a pause: a paused build is resumed only by
//!   the rules above, never by a timer into a thrashing host.

use serde::{Deserialize, Serialize};

use super::types::{Class, Fact, HostFacts, Lease, LeaseState, Policy, Resolved};

/// An attributed build tree with no lease (seen, counted, paused last).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnleasedTree {
    /// The broker's id for the tree (e.g. its root pid and start time).
    pub id: String,
    pub started_at_s: u64,
    /// When the governor paused it; `None` while it runs.
    pub paused_at_s: Option<u64>,
}

/// The governor's memory between ticks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct GovernorState {
    /// Start of the current run of PSI ≥ `psi_pause`.
    pub pressure_since_s: Option<u64>,
    /// When the last pause or resume was issued.
    pub last_action_at_s: Option<u64>,
}

/// What the governor does this tick.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum GovernorAction {
    PauseLease { lease_id: String },
    PauseUnleased { tree_id: String },
    ResumeLease { lease_id: String },
    ResumeUnleased { tree_id: String },
    Hold { reason: HoldReason },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HoldReason {
    /// Within `settle_s` of the last action.
    Settling,
    /// Overloaded, and nothing is left that may be paused.
    NothingPausable,
    /// Not overloaded, and nothing is paused.
    Calm,
    /// Something is paused but the resume condition does not hold yet
    /// (including PSI between `psi_resume` and `psi_pause`).
    NotYetResumable,
    /// An input the resume condition needs is unknown.
    UnknownInput,
}

/// One governor tick: the new state and the action.
pub fn overload_step(
    state: GovernorState,
    leases: &[Lease],
    unleased: &[UnleasedTree],
    facts: &HostFacts,
    now_s: u64,
    policy: &Policy,
) -> (GovernorState, GovernorAction) {
    let psi = facts.psi_mem_full_avg10;
    let mut next = state;
    next.pressure_since_s = match psi.resolve() {
        Resolved::Value(p) if p >= policy.psi_pause => {
            Some(state.pressure_since_s.unwrap_or(now_s))
        }
        _ => None,
    };
    let reserve = policy.reserve_floor(facts.mem_total_bytes);
    let psi_sustained = next
        .pressure_since_s
        .is_some_and(|since| now_s.saturating_sub(since) >= policy.pause_sustain_s);
    let mem_low = facts
        .mem_available_bytes
        .value()
        .is_some_and(|a| a < reserve);
    let overloaded = psi_sustained || mem_low;

    if let Some(last) = state.last_action_at_s {
        if now_s.saturating_sub(last) < policy.settle_s {
            return (next, hold(HoldReason::Settling));
        }
    }

    if overloaded {
        let victim = leases
            .iter()
            .filter(|l| l.state == LeaseState::Running && l.class != Class::Operator)
            .min_by(|a, b| {
                a.class
                    .rank()
                    .cmp(&b.class.rank())
                    .then(b.started_at_s.cmp(&a.started_at_s))
                    .then(a.id.cmp(&b.id))
            });
        if let Some(l) = victim {
            next.last_action_at_s = Some(now_s);
            return (
                next,
                GovernorAction::PauseLease {
                    lease_id: l.id.clone(),
                },
            );
        }
        let tree = unleased
            .iter()
            .filter(|t| t.paused_at_s.is_none())
            .max_by(|a, b| a.started_at_s.cmp(&b.started_at_s).then(b.id.cmp(&a.id)));
        if let Some(t) = tree {
            next.last_action_at_s = Some(now_s);
            return (
                next,
                GovernorAction::PauseUnleased {
                    tree_id: t.id.clone(),
                },
            );
        }
        return (next, hold(HoldReason::NothingPausable));
    }

    // Resume: the single oldest governor-paused build, lease or unleased tree.
    enum Cand<'a> {
        Lease(&'a Lease),
        Tree(&'a UnleasedTree),
    }
    let mut cands: Vec<(u64, &str, Cand)> = leases
        .iter()
        .filter(|l| l.state == LeaseState::PausedByGovernor)
        .map(|l| (l.paused_at_s.unwrap_or(0), l.id.as_str(), Cand::Lease(l)))
        .chain(
            unleased
                .iter()
                .filter_map(|t| t.paused_at_s.map(|p| (p, t.id.as_str(), Cand::Tree(t)))),
        )
        .collect();
    cands.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(b.1)));
    let Some((_, _, oldest)) = cands.into_iter().next() else {
        return (next, hold(HoldReason::Calm));
    };
    let psi_ok = match psi {
        Fact::NotSupported => Some(true),
        _ => psi.value().map(|p| p < policy.psi_resume),
    };
    let Some(avail) = facts.mem_available_bytes.value() else {
        return (next, hold(HoldReason::UnknownInput));
    };
    let need = match &oldest {
        Cand::Lease(l) => match (l.est_bytes, l.current_anon_bytes) {
            (Some(est), Some(cur)) => Some(est.saturating_sub(cur)),
            // Not sampled yet: the whole estimate is still to come.
            (Some(est), None) => Some(est),
            (None, _) => None,
        },
        Cand::Tree(_) => Some(0),
    };
    let (Some(psi_ok), Some(need)) = (psi_ok, need) else {
        return (next, hold(HoldReason::UnknownInput));
    };
    if !psi_ok || avail < reserve.saturating_add(need) {
        return (next, hold(HoldReason::NotYetResumable));
    }
    next.last_action_at_s = Some(now_s);
    let action = match oldest {
        Cand::Lease(l) => GovernorAction::ResumeLease {
            lease_id: l.id.clone(),
        },
        Cand::Tree(t) => GovernorAction::ResumeUnleased {
            tree_id: t.id.clone(),
        },
    };
    (next, action)
}

fn hold(reason: HoldReason) -> GovernorAction {
    GovernorAction::Hold { reason }
}
