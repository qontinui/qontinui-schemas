//! Fleet calibration: the shared fact types and pure parsers both reporters
//! use — the qontinui-runner (every machine that has one) and the CI host
//! agent (machines that do not). Plan
//! `2026-10-05-fleet-calibration-reversible-machine-tuning`, D2/D3, Phase 2a.
//!
//! One implementation, so the two reporters cannot drift. Everything here is
//! pure: parsers take file CONTENTS (`&str`), assemblers take parsed values,
//! and the IO lives in the callers.
//!
//! - [`measured`] — the `measured` / `not_supported` / `unavailable` manifest
//!   and [`measured::Measured<T>`], so an absence never renders as 0.
//! - [`vocab`] — the closed [`vocab::WorkloadGroup`] and [`vocab::ActionId`]
//!   vocabularies (wire-frozen) and [`vocab::classify_cgroup_with`].
//! - [`cgroup_path`] — the redacted wire shape of a cgroup path and the
//!   managed-service test the leak predicate uses.
//! - [`parse`] — PSI (the ONE PSI parser, host and cgroup), cgroup v2 files,
//!   procfs files, and `sccache --show-stats --stats-format json`.
//! - [`pressure`] — workload-group pressure facts and their assemblers.
//! - [`facts`] — compile cache, build volume, disk use, and the per-computer
//!   roll-up [`facts::CalibrationFacts`].
//! - [`outcome`] — the `builds.jsonl` build-outcome line.
//! - [`leak`] — the orphan CPU-burner predicate and its record.
//!
//! Not computer-specific (plan D12): no hostname, uid or host-path literal
//! appears outside tests; every threshold is a parameter.

pub mod cgroup_path;
pub mod facts;
pub mod leak;
pub mod measured;
pub mod outcome;
pub mod parse;
pub mod pressure;
pub mod vocab;

pub use cgroup_path::{is_managed_service_cgroup, redact_cgroup_path, STRUCTURAL_SLICES};
pub use facts::{
    BuildVolumeKind, BuildVolumeState, CalibrationFacts, CompileCacheClass, CompileCacheClassFacts,
    CompileCacheFacts, CompileCacheReading, DiskConsumer, DiskUseEntry, DiskUseFacts,
};
pub use leak::{
    kernel_comm, orphan_cpu_burner, LeakContext, LeakKind, LeakRecord, LeakThresholds, LeakVerdict,
    NotLeakReason, ProcessObservation, UnknownInput,
};
pub use measured::{Measured, MeasuredManifest, MeasuredState};
pub use outcome::{
    parse_build_ledger, parse_build_outcome_line, BuildLedger, BuildOutcome,
    BuildOutcomeParseError, BuildWrapper, BUILD_OUTCOME_SCHEMA_VERSION,
};
pub use pressure::{
    busy_cpu_shares, cpu_share_from_ticks, cpu_share_of_host, group_pressure_from_files,
    group_pressure_without_cgroups, runner_pressure_from_stat, CgroupFile, CgroupFiles,
    CgroupPressureFacts, Platform, WorkloadGroupPressure,
};
pub use vocab::{
    classify_cgroup, classify_cgroup_path, classify_cgroup_with, ActionId, CgroupClass,
    WorkloadGroup,
};
