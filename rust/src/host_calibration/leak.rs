//! The orphan CPU-burner leak predicate (plan D3 / D9) and the record it
//! emits.
//!
//! A leak is a FINDING, never a kill: this module decides and describes, it
//! does not act. The record carries `comm`, age, CPU share and owner uid
//! ONLY — argv, cwd and env never leave the host (the capacity plan's
//! Decision 2, applied unchanged).
//!
//! The predicate is pure over parsed facts, every threshold is a parameter,
//! and an UNKNOWN input never fires it (served policy
//! `an-unknown-input-must-not-fire-a-detector`).

use std::collections::BTreeSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::parse::proc::ProcStat;

/// The init process's pid: the kernel reparents orphans to it (or to the
/// nearest subreaper — see [`LeakContext::subreaper_pids`]).
const INIT_PID: u32 = 1;

/// Which leak class a record describes. Wire word = the `computer_events`
/// kind without its `leak_` prefix (see [`LeakKind::event_kind`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LeakKind {
    /// A process orphaned to init / the user manager, with no tty, burning
    /// CPU, outside every known workload class.
    OrphanCpu,
}

impl LeakKind {
    /// The `coord.computer_events` kind this leak is posted as.
    pub fn event_kind(self) -> &'static str {
        match self {
            LeakKind::OrphanCpu => "leak_orphan_cpu",
        }
    }
}

/// One detected leak. These five fields are the whole record by design.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct LeakRecord {
    /// The leak class.
    pub kind: LeakKind,
    /// The kernel `comm` (≤ 15 bytes, no argv).
    pub comm: String,
    /// Seconds since the process started.
    pub age_secs: u64,
    /// Sustained CPU use as a fraction of one core.
    pub cpu_share: f64,
    /// Real uid of the owner.
    pub owner_uid: u32,
}

/// Thresholds — parameters, never host constants.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct LeakThresholds {
    /// CPU share of one core the process must stay ABOVE (plan D3: 0.5).
    pub min_cpu_share: f64,
    /// How long it must have stayed above it, seconds (plan D3: 1 h).
    pub min_sustained_secs: u64,
}

impl Default for LeakThresholds {
    fn default() -> Self {
        Self {
            min_cpu_share: 0.5,
            min_sustained_secs: 3600,
        }
    }
}

/// Host context the predicate needs, supplied by the collector.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LeakContext {
    /// Pids that adopt orphans besides init: each user's systemd manager
    /// (`user@<uid>.service`) is a subreaper.
    pub subreaper_pids: BTreeSet<u32>,
    /// `comm` values of every known workload class (agent sessions, CI
    /// runners, compilers, the runner itself, …) — the capacity plan's
    /// census classes. A known workload is never a leak here.
    pub known_workload_comms: BTreeSet<String>,
}

/// What the collector observed about one process.
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessObservation {
    /// Its parsed `/proc/<pid>/stat`.
    pub stat: ProcStat,
    /// Real uid (`/proc/<pid>/status`); `None` if unreadable.
    pub owner_uid: Option<u32>,
    /// Age in seconds ([`ProcStat::age_secs`]); `None` if not computable.
    pub age_secs: Option<u64>,
    /// The LOWEST per-interval CPU share (fraction of one core) seen over the
    /// observation window — "sustained above X" means this minimum is above
    /// X. `None` before two samples exist.
    pub sustained_cpu_share: Option<f64>,
    /// How long the process has been observed, seconds.
    pub observed_secs: Option<u64>,
}

/// Why a process is not a leak.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NotLeakReason {
    /// It has a controlling terminal.
    HasTty,
    /// Its parent is neither init nor a subreaper.
    NotOrphaned,
    /// Its `comm` is a known workload class.
    KnownWorkload,
    /// Its sustained CPU share is at or below the threshold.
    BelowCpuThreshold,
    /// It has not been observed for the sustain period yet.
    NotYetSustained,
}

/// Which input was UNKNOWN.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UnknownInput {
    /// No sustained CPU share yet.
    CpuShare,
    /// No observation window.
    ObservedWindow,
    /// Owner uid unreadable.
    OwnerUid,
    /// Age not computable.
    Age,
}

/// The predicate's answer.
#[derive(Debug, Clone, PartialEq)]
pub enum LeakVerdict {
    /// A leak, described.
    Leak(LeakRecord),
    /// Definitely not a leak.
    NotLeak(NotLeakReason),
    /// Cannot decide — never treated as a leak.
    Unknown(UnknownInput),
}

/// Decide whether `obs` is an orphan CPU burner. PURE.
///
/// A leak needs ALL of: parent is init or a subreaper; no controlling tty;
/// `comm` not a known workload; sustained CPU share above
/// `min_cpu_share` for at least `min_sustained_secs`. A definite negative on
/// any measured input wins over an unknown on another; otherwise any unknown
/// input yields [`LeakVerdict::Unknown`].
pub fn orphan_cpu_burner(
    obs: &ProcessObservation,
    ctx: &LeakContext,
    thresholds: &LeakThresholds,
) -> LeakVerdict {
    let st = &obs.stat;
    if !st.has_no_tty() {
        return LeakVerdict::NotLeak(NotLeakReason::HasTty);
    }
    if st.ppid != INIT_PID && !ctx.subreaper_pids.contains(&st.ppid) {
        return LeakVerdict::NotLeak(NotLeakReason::NotOrphaned);
    }
    if ctx.known_workload_comms.contains(&st.comm) {
        return LeakVerdict::NotLeak(NotLeakReason::KnownWorkload);
    }
    // A non-finite share is a broken measurement: UNKNOWN, not a value.
    let share = obs.sustained_cpu_share.filter(|s| s.is_finite());
    if let Some(share) = share {
        if share <= thresholds.min_cpu_share {
            return LeakVerdict::NotLeak(NotLeakReason::BelowCpuThreshold);
        }
    }
    if let Some(secs) = obs.observed_secs {
        if secs < thresholds.min_sustained_secs {
            return LeakVerdict::NotLeak(NotLeakReason::NotYetSustained);
        }
    }
    let Some(cpu_share) = share else {
        return LeakVerdict::Unknown(UnknownInput::CpuShare);
    };
    if obs.observed_secs.is_none() {
        return LeakVerdict::Unknown(UnknownInput::ObservedWindow);
    }
    let Some(owner_uid) = obs.owner_uid else {
        return LeakVerdict::Unknown(UnknownInput::OwnerUid);
    };
    let Some(age_secs) = obs.age_secs else {
        return LeakVerdict::Unknown(UnknownInput::Age);
    };
    LeakVerdict::Leak(LeakRecord {
        kind: LeakKind::OrphanCpu,
        comm: st.comm.clone(),
        age_secs,
        cpu_share,
        owner_uid,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host_calibration::parse::proc::parse_proc_pid_stat;

    fn stat(ppid: u32, tty: i32, comm: &str) -> ProcStat {
        parse_proc_pid_stat(&format!(
            "500 ({comm}) R {ppid} 500 500 {tty} -1 0 0 0 0 0 9000 10 0 0 20 0 1 0 100 1 10 0"
        ))
        .unwrap()
    }

    fn obs(st: ProcStat) -> ProcessObservation {
        ProcessObservation {
            stat: st,
            owner_uid: Some(4321),
            age_secs: Some(90_000),
            sustained_cpu_share: Some(0.98),
            observed_secs: Some(7200),
        }
    }

    #[test]
    fn fires_on_an_orphan_burner_and_records_only_five_fields() {
        let v = orphan_cpu_burner(
            &obs(stat(1, 0, "cat")),
            &LeakContext::default(),
            &LeakThresholds::default(),
        );
        let LeakVerdict::Leak(r) = v else {
            panic!("{v:?}")
        };
        assert_eq!(r.comm, "cat");
        let json = serde_json::to_value(&r).unwrap();
        let keys: Vec<&String> = json.as_object().unwrap().keys().collect();
        assert_eq!(keys, ["age_secs", "comm", "cpu_share", "kind", "owner_uid"]);
        assert_eq!(json["kind"], "orphan_cpu");
        assert_eq!(LeakKind::OrphanCpu.event_kind(), "leak_orphan_cpu");
    }

    #[test]
    fn subreaper_parent_counts_as_orphaned() {
        let ctx = LeakContext {
            subreaper_pids: [777].into_iter().collect(),
            ..LeakContext::default()
        };
        assert!(matches!(
            orphan_cpu_burner(&obs(stat(777, 0, "yes")), &ctx, &LeakThresholds::default()),
            LeakVerdict::Leak(_)
        ));
        assert_eq!(
            orphan_cpu_burner(&obs(stat(778, 0, "yes")), &ctx, &LeakThresholds::default()),
            LeakVerdict::NotLeak(NotLeakReason::NotOrphaned)
        );
    }

    #[test]
    fn definite_negatives() {
        let t = LeakThresholds::default();
        let c = LeakContext {
            known_workload_comms: ["rustc".to_string()].into_iter().collect(),
            ..LeakContext::default()
        };
        assert_eq!(
            orphan_cpu_burner(&obs(stat(1, 34816, "cat")), &c, &t),
            LeakVerdict::NotLeak(NotLeakReason::HasTty)
        );
        assert_eq!(
            orphan_cpu_burner(&obs(stat(1, 0, "rustc")), &c, &t),
            LeakVerdict::NotLeak(NotLeakReason::KnownWorkload)
        );
        let mut low = obs(stat(1, 0, "cat"));
        low.sustained_cpu_share = Some(0.5);
        assert_eq!(
            orphan_cpu_burner(&low, &c, &t),
            LeakVerdict::NotLeak(NotLeakReason::BelowCpuThreshold)
        );
        let mut young = obs(stat(1, 0, "cat"));
        young.observed_secs = Some(3599);
        assert_eq!(
            orphan_cpu_burner(&young, &c, &t),
            LeakVerdict::NotLeak(NotLeakReason::NotYetSustained)
        );
        // A definite negative beats an unknown elsewhere.
        low.owner_uid = None;
        assert_eq!(
            orphan_cpu_burner(&low, &c, &t),
            LeakVerdict::NotLeak(NotLeakReason::BelowCpuThreshold)
        );
    }

    #[test]
    fn unknown_inputs_never_fire() {
        let t = LeakThresholds::default();
        let c = LeakContext::default();
        let mut o = obs(stat(1, 0, "cat"));
        o.sustained_cpu_share = None;
        assert_eq!(
            orphan_cpu_burner(&o, &c, &t),
            LeakVerdict::Unknown(UnknownInput::CpuShare)
        );
        let mut o = obs(stat(1, 0, "cat"));
        o.observed_secs = None;
        assert_eq!(
            orphan_cpu_burner(&o, &c, &t),
            LeakVerdict::Unknown(UnknownInput::ObservedWindow)
        );
        let mut o = obs(stat(1, 0, "cat"));
        o.owner_uid = None;
        assert_eq!(
            orphan_cpu_burner(&o, &c, &t),
            LeakVerdict::Unknown(UnknownInput::OwnerUid)
        );
        let mut o = obs(stat(1, 0, "cat"));
        o.age_secs = None;
        assert_eq!(
            orphan_cpu_burner(&o, &c, &t),
            LeakVerdict::Unknown(UnknownInput::Age)
        );
        let mut o = obs(stat(1, 0, "cat"));
        o.sustained_cpu_share = Some(f64::NAN);
        assert_eq!(
            orphan_cpu_burner(&o, &c, &t),
            LeakVerdict::Unknown(UnknownInput::CpuShare)
        );
    }

    #[test]
    fn thresholds_are_parameters() {
        let strict = LeakThresholds {
            min_cpu_share: 0.99,
            min_sustained_secs: 60,
        };
        assert_eq!(
            orphan_cpu_burner(&obs(stat(1, 0, "cat")), &LeakContext::default(), &strict),
            LeakVerdict::NotLeak(NotLeakReason::BelowCpuThreshold)
        );
    }
}
