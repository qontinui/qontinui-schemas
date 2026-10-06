//! Whether ONE ticket may start now (D3), and with how many jobs (D2).
//!
//! ```text
//! admit(ticket) ⇔ Σ est(running ∪ paused leases) + est(ticket) ≤ budget      (reservation)
//!               ∧ MemAvailable − est(ticket) ≥ reserve_floor                 (live)
//!               ∧ memory PSI full avg10 < psi_admit_max                      (pressure)
//!               ∧ no running or paused lease holds the ticket's output dir   (target)
//!               ∧ the host's build-admission pause is clear                  (control)
//! ```
//!
//! The target and control checks hold regardless of memory. Every memory
//! check needs its inputs: a `not_supported` input drops its conjunct, an
//! `unknown` one withholds — except on the PROGRESS FLOOR, where no lease is
//! running or paused and the controls are clear, so the ticket is admitted
//! with jobs from live memory. That bounds the box at one build instead of
//! stalling it, and it is the only arm an `unknown` input reaches. The same
//! floor admits an idle host's ticket whose estimate exceeds the whole budget
//! (no amount of waiting would ever make it fit); a live or pressure refusal is
//! never floored, because those clear on their own.

use serde::{Deserialize, Serialize};

use super::budget::{budget, Budget, BudgetTerm};
use super::jobs::jobs;
use super::types::{HostFacts, Lease, Policy, Resolved, Ticket};

/// An input a memory check needed and could not read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnknownInput {
    TicketEstimate,
    /// A running or paused lease admitted with no estimate.
    LeaseEstimate,
    MemAvailable,
    PsiMemFull,
    Budget(BudgetTerm),
}

/// Why a ticket waits. A ticket can wait for several reasons at once; all are
/// reported so the console can name every one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "check", rename_all = "snake_case")]
pub enum Blocking {
    /// The host's build-admission pause is set.
    Control,
    /// A running or paused lease holds the ticket's output directory.
    Target { lease_id: String },
    /// Σ reservations + this ticket exceed the budget.
    Reservation {
        reserved_bytes: u64,
        need_bytes: u64,
        budget_bytes: u64,
    },
    /// MemAvailable − est would fall below the reserve floor.
    Live {
        mem_available_bytes: u64,
        need_bytes: u64,
        reserve_floor_bytes: u64,
    },
    /// Memory PSI full avg10 is at or above `psi_admit_max`.
    Pressure { psi_full_avg10: f64, max: f64 },
    /// A memory input is unknown while a lease runs (gating consumer: withhold).
    Unknown { inputs: Vec<UnknownInput> },
}

/// How the job count was chosen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobsSource {
    /// The caller's explicit `-j` / `CARGO_BUILD_JOBS`, kept.
    Caller,
    /// This lease's share of the budget.
    Share,
    /// The progress floor: live memory only.
    LiveMemory,
}

/// Why a ticket was admitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdmitBasis {
    /// Every check passed on known inputs.
    Fits,
    /// The progress floor (no lease running or paused; an input was unknown).
    ProgressFloor,
}

/// The decision for one ticket.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "decision", rename_all = "snake_case")]
pub enum Admission {
    Admit {
        jobs: u32,
        jobs_source: JobsSource,
        basis: AdmitBasis,
    },
    Wait {
        blocking: Vec<Blocking>,
    },
}

impl Admission {
    pub fn is_admit(&self) -> bool {
        matches!(self, Admission::Admit { .. })
    }
}

/// Σ of the leases' reservations, or `None` when any lease has no estimate.
pub fn reserved_bytes(leases: &[Lease]) -> Option<u64> {
    leases
        .iter()
        .try_fold(0u64, |acc, l| l.est_bytes.map(|b| acc.saturating_add(b)))
}

/// Decide whether `ticket` may start now against `leases` (the host's running
/// and paused leases) and `facts`.
pub fn admit(ticket: &Ticket, leases: &[Lease], facts: &HostFacts, policy: &Policy) -> Admission {
    let mut blocking = Vec::new();
    if facts.admissions_paused {
        blocking.push(Blocking::Control);
    }
    if let Some(holder) = leases.iter().find(|l| l.output_dir == ticket.output_dir) {
        blocking.push(Blocking::Target {
            lease_id: holder.id.clone(),
        });
    }

    let reserve = policy.reserve_floor(facts.mem_total_bytes);
    let mut unknown = Vec::new();
    let est = ticket.est.bytes;
    if est.is_none() {
        unknown.push(UnknownInput::TicketEstimate);
    }
    let reserved = reserved_bytes(leases);
    if reserved.is_none() {
        unknown.push(UnknownInput::LeaseEstimate);
    }
    let budget_now = budget(facts, policy);
    if let Budget::Unknown { missing } = &budget_now {
        unknown.extend(missing.iter().map(|t| UnknownInput::Budget(*t)));
    }
    let avail = match facts.mem_available_bytes.resolve() {
        Resolved::Value(v) => Some(v),
        // MemAvailable has no "not supported" platform: commit-available
        // stands in on Windows. A host that says otherwise is unknown.
        Resolved::Dropped | Resolved::Unknown => {
            unknown.push(UnknownInput::MemAvailable);
            None
        }
    };
    let psi = match facts.psi_mem_full_avg10.resolve() {
        Resolved::Value(v) => Some(v),
        Resolved::Dropped => None,
        Resolved::Unknown => {
            unknown.push(UnknownInput::PsiMemFull);
            None
        }
    };

    if let (Some(est), Some(reserved), Some(b)) = (est, reserved, budget_now.bytes()) {
        if reserved.saturating_add(est) > b {
            blocking.push(Blocking::Reservation {
                reserved_bytes: reserved,
                need_bytes: est,
                budget_bytes: b,
            });
        }
    }
    if let (Some(est), Some(avail)) = (est, avail) {
        if avail < est.saturating_add(reserve) {
            blocking.push(Blocking::Live {
                mem_available_bytes: avail,
                need_bytes: est,
                reserve_floor_bytes: reserve,
            });
        }
    }
    if let Some(p) = psi {
        if p >= policy.psi_admit_max {
            blocking.push(Blocking::Pressure {
                psi_full_avg10: p,
                max: policy.psi_admit_max,
            });
        }
    }

    // The progress floor: on an idle host (no lease running or paused) with the
    // controls clear, a ticket held back only by an unknown input or by a
    // reservation larger than the whole budget is admitted alone, sized from
    // live memory. Neither of those can clear by waiting on an idle host, so
    // refusing would stall the queue forever; one build is the bound. Live and
    // pressure refusals still wait: those are conditions that do clear.
    let floor_eligible = leases.is_empty()
        && blocking
            .iter()
            .all(|b| matches!(b, Blocking::Reservation { .. }));
    if floor_eligible && (!unknown.is_empty() || !blocking.is_empty()) {
        let share = avail.map(|a| a.saturating_sub(reserve));
        return Admission::Admit {
            jobs: ticket
                .requested_jobs
                .unwrap_or_else(|| jobs(share, facts.cpus, 1)),
            jobs_source: if ticket.requested_jobs.is_some() {
                JobsSource::Caller
            } else {
                JobsSource::LiveMemory
            },
            basis: AdmitBasis::ProgressFloor,
        };
    }
    if !unknown.is_empty() {
        blocking.push(Blocking::Unknown { inputs: unknown });
    }
    if !blocking.is_empty() {
        return Admission::Wait { blocking };
    }

    // Every input is known here.
    let (est, reserved, b) = (
        est.unwrap_or(0),
        reserved.unwrap_or(0),
        budget_now.bytes().unwrap_or(0),
    );
    let running = leases.iter().filter(|l| !l.state.is_paused()).count() as u32;
    let concurrent = running + 1;
    let unreserved = b.saturating_sub(reserved).saturating_sub(est);
    let share = est.saturating_add(unreserved / concurrent as u64);
    match ticket.requested_jobs {
        Some(j) => Admission::Admit {
            jobs: j,
            jobs_source: JobsSource::Caller,
            basis: AdmitBasis::Fits,
        },
        None => Admission::Admit {
            jobs: jobs(Some(share), facts.cpus, concurrent),
            jobs_source: JobsSource::Share,
            basis: AdmitBasis::Fits,
        },
    }
}
