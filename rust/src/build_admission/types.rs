//! The vocabulary of build admission: facts with provenance, tickets, leases
//! and the policy parameters. Wire-shaped (`serde`), so the broker can persist
//! and publish them unchanged.

use serde::{Deserialize, Serialize};

/// One GiB, the unit every human-facing number in this module is quoted in.
pub const GIB: u64 = 1 << 30;

/// A host input together with where it came from (D3).
///
/// `Measured`, `LiveOnly` and `Seed` carry a usable value; `LiveOnly` is a
/// measurement over a stated short window (e.g. a non-build p95 before an hour
/// of samples exists) and `Seed` a labelled starting value — neither is ever
/// presented as a long-window measurement. `NotSupported` means the platform
/// cannot provide the input, which DROPS its conjunct; `Unknown` means it
/// should be readable and is not, which WITHHOLDS (see [`crate::build_admission`]).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "provenance", content = "value", rename_all = "snake_case")]
pub enum Fact<T> {
    Measured(T),
    LiveOnly(T),
    Seed(T),
    NotSupported,
    Unknown,
}

/// What a [`Fact`] resolves to for a decision.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Resolved<T> {
    /// A value the decision may use.
    Value(T),
    /// The platform cannot provide it: the conjunct is dropped.
    Dropped,
    /// Should be readable and is not: the decision withholds.
    Unknown,
}

impl<T: Copy> Fact<T> {
    pub fn resolve(&self) -> Resolved<T> {
        match *self {
            Fact::Measured(v) | Fact::LiveOnly(v) | Fact::Seed(v) => Resolved::Value(v),
            Fact::NotSupported => Resolved::Dropped,
            Fact::Unknown => Resolved::Unknown,
        }
    }

    /// The value when there is one, `None` for both `NotSupported` and `Unknown`.
    pub fn value(&self) -> Option<T> {
        match self.resolve() {
            Resolved::Value(v) => Some(v),
            _ => None,
        }
    }
}

impl Fact<f64> {
    /// [`Fact::resolve`] for a reading that must be a finite number: a NaN or
    /// infinite "measurement" is a broken probe, so it resolves to `Unknown`
    /// rather than comparing false against every threshold (which would read
    /// as calm).
    pub fn resolve_finite(&self) -> Resolved<f64> {
        match self.resolve() {
            Resolved::Value(v) if !v.is_finite() => Resolved::Unknown,
            r => r,
        }
    }
}

/// Priority class, highest first in [`Class::rank`] (D5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Class {
    /// Set only by a coord command; never self-declared; never paused by the governor.
    Operator,
    /// A build for an open PR the merge train is waiting on (verified by the broker).
    Merge,
    /// The default.
    Agent,
    /// Sweeps, pre-warm, mutation runs; opt-in.
    Background,
}

impl Class {
    /// Larger is more important.
    pub fn rank(self) -> u8 {
        match self {
            Class::Operator => 3,
            Class::Merge => 2,
            Class::Agent => 1,
            Class::Background => 0,
        }
    }

    /// The class one promotion step above this one. Promotion is capped at
    /// `Merge`: `Operator` is reachable only through a coord command, never
    /// through age.
    pub fn promoted(self) -> Class {
        match self {
            Class::Background => Class::Agent,
            Class::Agent | Class::Merge => Class::Merge,
            Class::Operator => Class::Operator,
        }
    }
}

/// The cargo subcommand a ticket runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Subcommand {
    Check,
    Clippy,
    Build,
    Test,
    Nextest,
}

/// Whether a build reuses a warm shared target or starts a private cold one —
/// the two differ in peak memory, so their estimates are never pooled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetDirKind {
    SharedWarm,
    PrivateCold,
}

/// How a peak was measured. Estimates are keyed by method and never mixed (D2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeasureMethod {
    /// Max sampled `anon` of the build's cgroup scope.
    CgroupAnon,
    /// Max sampled RSS of the whole leased process tree.
    RssSample,
}

/// The key a ticket's estimate is looked up by (D2).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EstimateKey {
    pub repo: String,
    pub subcommand: Subcommand,
    pub profile: String,
    pub target_dir_kind: TargetDirKind,
    pub measure_method: MeasureMethod,
}

/// Where an estimate came from — rendered beside the number everywhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum EstimateSource {
    /// p90 over `n` completed leases of the same key.
    Measured { n: u32 },
    /// A labelled seed, used until enough measurements exist.
    Seed,
    /// No history and no seed.
    Unknown,
}

/// A ticket's memory reservation and, when history has it, its expected run
/// time (used only by EASY backfill).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Estimate {
    pub bytes: Option<u64>,
    pub duration_s: Option<u64>,
    pub source: EstimateSource,
}

impl Estimate {
    pub const UNKNOWN: Estimate = Estimate {
        bytes: None,
        duration_s: None,
        source: EstimateSource::Unknown,
    };
}

/// One wrapper invocation waiting for a lease.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ticket {
    pub id: String,
    pub class: Class,
    /// The resolved output directory cargo will lock (D5). Opaque to the core:
    /// only equality matters.
    pub output_dir: String,
    /// An explicit caller `-j` / `CARGO_BUILD_JOBS`, kept and recorded.
    pub requested_jobs: Option<u32>,
    pub queued_at_s: u64,
    pub est: Estimate,
}

/// What state a lease is in, as far as admission is concerned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeaseState {
    Running,
    /// Frozen by the governor's overload ladder. Blocks new admissions until
    /// resumed, so a paused build cannot starve.
    PausedByGovernor,
    /// Frozen by an explicit command. Holds its directory and its reservation
    /// but does not block other admissions, and the governor never resumes it.
    PausedByCommand,
}

impl LeaseState {
    pub fn is_paused(self) -> bool {
        !matches!(self, LeaseState::Running)
    }
}

/// An admitted build.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Lease {
    pub id: String,
    pub class: Class,
    pub output_dir: String,
    pub state: LeaseState,
    /// The reservation it was admitted with; `None` when admitted on the
    /// progress floor with an unknown estimate.
    pub est_bytes: Option<u64>,
    /// Expected run time from history, for backfill's shadow time.
    pub expected_duration_s: Option<u64>,
    pub started_at_s: u64,
    /// When the governor or a command paused it.
    pub paused_at_s: Option<u64>,
    /// Latest sampled resident anon of its tree, when measured.
    pub current_anon_bytes: Option<u64>,
}

/// The host's measured facts at decision time (D3).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HostFacts {
    pub mem_total_bytes: u64,
    /// Usable parallelism (cgroup- and affinity-aware). At least 1.
    pub cpus: u32,
    /// MemAvailable on Linux; commit-available on Windows.
    pub mem_available_bytes: Fact<u64>,
    /// Memory PSI `full` avg10, percent. `NotSupported` on Windows.
    pub psi_mem_full_avg10: Fact<f64>,
    /// Used memory outside the builds slice, the CI slices and unleased build
    /// trees: p95 over 24 h, or `LiveOnly` before an hour of samples exists.
    pub non_build_p95_bytes: Fact<u64>,
    /// D6: the CI slices' reservation (`Measured(0)` where there is no CI slice).
    pub ci_reservation_bytes: Fact<u64>,
    /// Live RSS of attributed build trees that hold no lease (D4).
    pub unleased_build_rss_bytes: Fact<u64>,
    /// The host's build-admission pause (D7). An unreadable control keeps the
    /// last-known value, so this is always a decided boolean.
    pub admissions_paused: bool,
}

/// The policy parameters (D9). `Policy::default()` is the compiled default the
/// console shows with source `default`; coord's typed columns override it.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Policy {
    /// Admit only while memory PSI full avg10 is below this (percent).
    pub psi_admit_max: f64,
    /// Overload rung 3: pause while PSI full avg10 is at or above this (percent)...
    pub psi_pause: f64,
    /// ...for at least this long (seconds).
    pub pause_sustain_s: u64,
    /// Resume only while PSI full avg10 is below this (percent; < `psi_pause`).
    pub psi_resume: f64,
    /// `reserve_floor = max(reserve_floor_min_bytes, reserve_floor_fraction × MemTotal)`.
    pub reserve_floor_fraction: f64,
    pub reserve_floor_min_bytes: u64,
    /// A ticket is promoted one class per this many seconds waited (capped at
    /// `merge`), and once the head has waited this long backfill stops.
    pub promote_after_s: u64,
    /// A lease paused this long raises `build_paused_long`.
    pub paused_alert_after_s: u64,
    /// Spawn admission defers when the oldest queued wait exceeds this (Phase 8).
    pub spawn_defer_wait_s: u64,
    /// The governor waits this long after each pause or resume before the next.
    pub settle_s: u64,
    /// Estimates use at most this many most-recent completed leases of a key.
    pub history_window: u32,
    /// A key needs at least this many measurements before its p90 replaces the seed.
    pub min_measurements: u32,
}

impl Default for Policy {
    fn default() -> Self {
        Policy {
            psi_admit_max: 5.0,
            psi_pause: 20.0,
            pause_sustain_s: 10,
            psi_resume: 5.0,
            reserve_floor_fraction: 0.03,
            reserve_floor_min_bytes: 4 * GIB,
            promote_after_s: 30 * 60,
            paused_alert_after_s: 30 * 60,
            spawn_defer_wait_s: 30 * 60,
            settle_s: 20,
            history_window: 20,
            min_measurements: 3,
        }
    }
}

/// Why a [`Policy`] is refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyError {
    /// `psi_resume` must be below `psi_pause`, or the ladder oscillates.
    ResumeNotBelowPause,
    /// A percentage outside `0..=100`.
    PercentOutOfRange,
    /// `reserve_floor_fraction` outside `0..1`.
    FractionOutOfRange,
    /// A window, count or promotion period of zero.
    ZeroCount,
    /// `psi_admit_max` above `psi_pause`: admissions would continue into a
    /// pressure the ladder is already pausing for.
    AdmitAbovePause,
    /// `min_measurements` larger than `history_window`: no key could ever
    /// leave its seed.
    MinMeasurementsAboveWindow,
}

impl Policy {
    /// Refuse a parameter set that would make the ladder unsound.
    pub fn validate(&self) -> Result<(), PolicyError> {
        let pct = |v: f64| (0.0..=100.0).contains(&v);
        if !(pct(self.psi_admit_max) && pct(self.psi_pause) && pct(self.psi_resume)) {
            return Err(PolicyError::PercentOutOfRange);
        }
        if self.psi_resume >= self.psi_pause {
            return Err(PolicyError::ResumeNotBelowPause);
        }
        if !(0.0..1.0).contains(&self.reserve_floor_fraction) {
            return Err(PolicyError::FractionOutOfRange);
        }
        if self.psi_admit_max > self.psi_pause {
            return Err(PolicyError::AdmitAbovePause);
        }
        if self.history_window == 0 || self.min_measurements == 0 || self.promote_after_s == 0 {
            return Err(PolicyError::ZeroCount);
        }
        if self.min_measurements > self.history_window {
            return Err(PolicyError::MinMeasurementsAboveWindow);
        }
        Ok(())
    }

    /// `max(reserve_floor_min_bytes, reserve_floor_fraction × MemTotal)` (D3).
    pub fn reserve_floor(&self, mem_total_bytes: u64) -> u64 {
        let frac = (mem_total_bytes as f64 * self.reserve_floor_fraction) as u64;
        frac.max(self.reserve_floor_min_bytes)
    }
}
