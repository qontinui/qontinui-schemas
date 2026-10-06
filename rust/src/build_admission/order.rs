//! Which queued ticket starts next (D5): class, then age, with promotion,
//! EASY backfill and the paused-holder skip.
//!
//! - **Order.** Effective class (highest first), then queue time (oldest
//!   first), then id. A ticket waiting at least `promote_after_s` is promoted
//!   one class (capped at `merge`), so an `agent` ticket cannot wait forever
//!   behind a stream of `merge` tickets.
//! - **Head eligibility.** A ticket whose output directory is held by a lease
//!   (running or paused) cannot start until that lease ends, so it is neither
//!   the head nor a backfill candidate — it waits behind exactly that one build
//!   and blocks nobody else.
//! - **Paused leases first.** While any lease is paused BY THE GOVERNOR,
//!   nothing new is admitted: resuming it comes first, so a paused build
//!   cannot starve. (A lease paused by a command does not hold the queue.)
//! - **EASY backfill.** When the head does not fit, a later ticket may start
//!   only if it fits now on known inputs AND cannot delay the head: either it
//!   is expected to finish before the head's SHADOW time (when enough running
//!   leases are expected to have ended for the head to fit), or it fits in the
//!   EXTRA bytes left over at that time once the head is placed. Unknown
//!   durations never qualify. Once the head has waited `promote_after_s`,
//!   backfill stops entirely.

use serde::{Deserialize, Serialize};

use super::admit::{admit, reserved_bytes, Admission, AdmitBasis};
use super::budget::budget;
use super::types::{Class, HostFacts, Lease, LeaseState, Policy, Ticket};

/// A ticket's class after promotion.
pub fn effective_class(t: &Ticket, now_s: u64, policy: &Policy) -> Class {
    if now_s.saturating_sub(t.queued_at_s) >= policy.promote_after_s {
        t.class.promoted()
    } else {
        t.class
    }
}

/// The queue in admission order (indices into `queue`).
pub fn order(queue: &[Ticket], now_s: u64, policy: &Policy) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..queue.len()).collect();
    idx.sort_by(|&a, &b| {
        let (ta, tb) = (&queue[a], &queue[b]);
        effective_class(tb, now_s, policy)
            .rank()
            .cmp(&effective_class(ta, now_s, policy).rank())
            .then(ta.queued_at_s.cmp(&tb.queued_at_s))
            .then(ta.id.cmp(&tb.id))
    });
    idx
}

/// What the scheduler does this tick.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Schedule {
    /// Start this ticket.
    Start {
        ticket_id: String,
        jobs: u32,
        basis: AdmitBasis,
        /// True when it started ahead of a head that did not fit.
        backfilled: bool,
    },
    /// Nothing starts. `head` is the ticket everything waits for, if any.
    Hold {
        head: Option<String>,
        reason: HoldReason,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum HoldReason {
    EmptyQueue,
    /// Every queued ticket's output directory is held by a lease.
    AllHeldByLeases,
    /// A governor-paused lease must resume first.
    PausedLeaseFirst,
    /// The head waits for this and no backfill candidate qualifies.
    HeadWaits {
        head: Admission,
    },
}

/// The next scheduling action for `queue` against `leases`.
pub fn schedule(
    queue: &[Ticket],
    leases: &[Lease],
    facts: &HostFacts,
    now_s: u64,
    policy: &Policy,
) -> Schedule {
    if queue.is_empty() {
        return Schedule::Hold {
            head: None,
            reason: HoldReason::EmptyQueue,
        };
    }
    if leases
        .iter()
        .any(|l| l.state == LeaseState::PausedByGovernor)
    {
        return Schedule::Hold {
            head: None,
            reason: HoldReason::PausedLeaseFirst,
        };
    }
    let held = |t: &Ticket| leases.iter().any(|l| l.output_dir == t.output_dir);
    let eligible: Vec<&Ticket> = order(queue, now_s, policy)
        .into_iter()
        .map(|i| &queue[i])
        .filter(|t| !held(t))
        .collect();
    let Some(head) = eligible.first() else {
        return Schedule::Hold {
            head: None,
            reason: HoldReason::AllHeldByLeases,
        };
    };
    let head_decision = admit(head, leases, facts, policy);
    if let Admission::Admit { jobs, basis, .. } = head_decision {
        return Schedule::Start {
            ticket_id: head.id.clone(),
            jobs,
            basis,
            backfilled: false,
        };
    }
    let head_waited = now_s.saturating_sub(head.queued_at_s);
    if head_waited < policy.promote_after_s {
        if let Some(window) = shadow(head, leases, facts, now_s, policy) {
            for cand in eligible.iter().skip(1) {
                if cand.output_dir == head.output_dir {
                    continue;
                }
                let Admission::Admit {
                    jobs,
                    basis: AdmitBasis::Fits,
                    ..
                } = admit(cand, leases, facts, policy)
                else {
                    continue;
                };
                let ends_before_shadow = cand
                    .est
                    .duration_s
                    .is_some_and(|d| now_s.saturating_add(d) <= window.at_s);
                let fits_in_extra = cand.est.bytes.is_some_and(|b| b <= window.extra_bytes);
                if ends_before_shadow || fits_in_extra {
                    return Schedule::Start {
                        ticket_id: cand.id.clone(),
                        jobs,
                        basis: AdmitBasis::Fits,
                        backfilled: true,
                    };
                }
            }
        }
    }
    Schedule::Hold {
        head: Some(head.id.clone()),
        reason: HoldReason::HeadWaits {
            head: head_decision,
        },
    }
}

/// When the head is expected to fit, and the bytes spare at that moment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShadowWindow {
    pub at_s: u64,
    pub extra_bytes: u64,
}

/// EASY's shadow time for `head`, from the reservation check alone: walk the
/// RUNNING leases in expected-end order, releasing each one's reservation,
/// until the head fits. `None` (no backfill) when any input is unknown — the
/// head's estimate, the budget, a lease's estimate or a running lease's
/// expected duration — or when the head cannot fit even after every running
/// lease ends (paused leases are not expected to end).
pub fn shadow(
    head: &Ticket,
    leases: &[Lease],
    facts: &HostFacts,
    now_s: u64,
    policy: &Policy,
) -> Option<ShadowWindow> {
    let need = head.est.bytes?;
    let b = budget(facts, policy).bytes()?;
    let reserved = reserved_bytes(leases)?;
    let mut free = b.saturating_sub(reserved);
    if free >= need {
        // The head fits by reservation and waits on something else (live
        // memory, pressure): its start time is not predictable from leases.
        return None;
    }
    let mut ends: Vec<(u64, u64)> = Vec::new();
    for l in leases.iter().filter(|l| l.state == LeaseState::Running) {
        let end = l.started_at_s.saturating_add(l.expected_duration_s?);
        ends.push((end.max(now_s), l.est_bytes?));
    }
    ends.sort_unstable();
    for (end, bytes) in ends {
        free = free.saturating_add(bytes);
        if free >= need {
            return Some(ShadowWindow {
                at_s: end,
                extra_bytes: free - need,
            });
        }
    }
    None
}
