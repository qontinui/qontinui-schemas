//! Workload-group pressure facts (plan D3, the `cgroup_pressure` sample
//! column) and the pure assemblers that build them from file contents.

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::cgroup_path::redact_cgroup_path;
use super::measured::{Measured, MeasuredManifest};
use super::parse::cgroup::{
    parse_cpu_stat, parse_cpu_weight, parse_memory_bytes, parse_memory_events, MemoryEvents,
};
use super::parse::proc::ProcStat;
use super::parse::psi::{parse_psi, Psi};
use super::vocab::{CgroupClass, WorkloadGroup};

/// The platform a fact set was measured on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    /// Linux host (cgroup v2, PSI).
    Linux,
    /// A WSL2 guest (Linux facts, guest view).
    Wsl2,
    /// Windows host: no PSI, no cgroups.
    Windows,
    /// macOS: no PSI, no cgroups.
    Macos,
}

impl Platform {
    /// True where cgroup v2 + PSI instruments exist.
    pub fn has_cgroup_psi(self) -> bool {
        matches!(self, Platform::Linux | Platform::Wsl2)
    }
}

/// One workload group's pressure and resource use.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct WorkloadGroupPressure {
    /// The group.
    pub group: WorkloadGroup,
    /// True when the measured cgroup IS the runner process's cgroup or an
    /// ancestor of it ([`crate::host_calibration::vocab::classify_cgroup_with`]).
    /// A mixed group's figures OVERLAP the `runner` record: never sum them.
    #[serde(default)]
    pub mixed: bool,
    /// The REDACTED shapes of the cgroups aggregated into this record
    /// ([`crate::host_calibration::cgroup_path::redact_cgroup_path`]): no uid,
    /// host name or session name ever reaches the wire. Empty for the runner
    /// process and for platforms without cgroups.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cgroup_shapes: Vec<String>,
    /// How many session/container scopes sit beneath the measured cgroup(s) —
    /// the count that replaces their names. `unavailable` unless the collector
    /// supplies it ([`WorkloadGroupPressure::with_scope_count`]);
    /// `not_supported` without cgroups.
    pub scope_count: Measured<u32>,
    /// Share of the host's total CPU capacity used over the sample interval,
    /// 0.0..=1.0 (1.0 = every core busy).
    pub cpu_usage_share: Measured<f64>,
    /// The raw cumulative CPU counter (`cpu.stat usage_usec`, or the runner's
    /// utime+stime in µs), so a reader can take its own deltas.
    pub cpu_usage_usec: Measured<u64>,
    /// `cpu.pressure`.
    pub cpu_pressure: Measured<Psi>,
    /// `cpu.weight`.
    pub cpu_weight: Measured<u32>,
    /// `memory.current` (for the runner: RSS).
    pub memory_current_bytes: Measured<u64>,
    /// `memory.peak` (for the runner: `VmHWM`).
    pub memory_peak_bytes: Measured<u64>,
    /// `memory.events`.
    pub memory_events: Measured<MemoryEvents>,
    /// `io.pressure`.
    pub io_pressure: Measured<Psi>,
}

impl WorkloadGroupPressure {
    /// Axis name → state for this record, keys `<group>.<field>`.
    pub fn manifest_into(&self, prefix: &str, out: &mut MeasuredManifest) {
        let g = self.group.as_str();
        let mut put = |field: &str, s| {
            out.insert(format!("{prefix}{g}.{field}"), s);
        };
        put("scope_count", self.scope_count.state());
        put("cpu_usage_share", self.cpu_usage_share.state());
        put("cpu_usage_usec", self.cpu_usage_usec.state());
        put("cpu_pressure", self.cpu_pressure.state());
        put("cpu_weight", self.cpu_weight.state());
        put("memory_current_bytes", self.memory_current_bytes.state());
        put("memory_peak_bytes", self.memory_peak_bytes.state());
        put("memory_events", self.memory_events.state());
        put("io_pressure", self.io_pressure.state());
    }

    /// Set the scope count the collector measured (`None` → `unavailable`).
    pub fn with_scope_count(mut self, scopes: Option<u32>) -> Self {
        self.scope_count = Measured::from_read(scopes);
        self
    }
}

/// What reading one cgroup interface file produced. Absent and failed are
/// different facts: a file that does not EXIST means this cgroup or kernel
/// has no such instrument (`cpu.weight` / `cpu.stat`'s throttling lines where
/// the `cpu` controller is not enabled — `app.slice` enables only
/// `memory pids`; `memory.*` where `memory` is off; `memory.peak` before
/// kernel 5.19; `*.pressure` with `cgroup.pressure` disabled) and maps to
/// `not_supported`; a read that FAILED is UNKNOWN and maps to `unavailable`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CgroupFile<'a> {
    /// The file's contents.
    Contents(&'a str),
    /// The file does not exist (ENOENT).
    Absent,
    /// The read failed for another reason, or was not attempted.
    #[default]
    ReadError,
}

impl<'a> CgroupFile<'a> {
    /// Map a read result: `NotFound` → [`CgroupFile::Absent`], any other
    /// error → [`CgroupFile::ReadError`]. Pure over the result value.
    pub fn from_read_result(result: &'a std::io::Result<String>) -> Self {
        match result {
            Ok(text) => CgroupFile::Contents(text),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => CgroupFile::Absent,
            Err(_) => CgroupFile::ReadError,
        }
    }

    /// Parse the contents: absent → `not_supported`, failed read or
    /// unparseable contents → `unavailable`.
    fn measure<T>(self, parse: impl FnOnce(&str) -> Option<T>) -> Measured<T> {
        match self {
            CgroupFile::Contents(t) => Measured::from_read(parse(t)),
            CgroupFile::Absent => Measured::NotSupported,
            CgroupFile::ReadError => Measured::Unavailable,
        }
    }
}

/// One cgroup's interface files. Fields default to
/// [`CgroupFile::ReadError`] — a file nobody read is UNKNOWN.
#[derive(Debug, Clone, Copy, Default)]
pub struct CgroupFiles<'a> {
    /// `cpu.pressure`.
    pub cpu_pressure: CgroupFile<'a>,
    /// `cpu.stat`.
    pub cpu_stat: CgroupFile<'a>,
    /// `cpu.weight` (absent where the `cpu` controller is not enabled).
    pub cpu_weight: CgroupFile<'a>,
    /// `memory.current` (absent where `memory` is not enabled).
    pub memory_current: CgroupFile<'a>,
    /// `memory.peak` (absent before kernel 5.19, or without `memory`).
    pub memory_peak: CgroupFile<'a>,
    /// `memory.events` (absent without `memory`).
    pub memory_events: CgroupFile<'a>,
    /// `io.pressure`.
    pub io_pressure: CgroupFile<'a>,
}

/// Build a cgroup-backed group record from file contents. PURE.
///
/// `class` comes from
/// [`crate::host_calibration::vocab::classify_cgroup_with`] on the same path.
/// The path itself is published only as its redacted shape.
/// `cpu_usage_share` comes from two samples, so the caller computes it with
/// [`cpu_share_of_host`] (`None` on the first tick → `unavailable`).
pub fn group_pressure_from_files(
    class: CgroupClass,
    cgroup_path: &str,
    files: &CgroupFiles<'_>,
    cpu_usage_share: Option<f64>,
) -> WorkloadGroupPressure {
    WorkloadGroupPressure {
        group: class.group,
        mixed: class.mixed,
        cgroup_shapes: vec![redact_cgroup_path(cgroup_path)],
        scope_count: Measured::Unavailable,
        cpu_usage_share: Measured::from_read(cpu_usage_share),
        cpu_usage_usec: files
            .cpu_stat
            .measure(|t| parse_cpu_stat(t).map(|s| s.usage_usec)),
        cpu_pressure: files.cpu_pressure.measure(parse_psi),
        cpu_weight: files.cpu_weight.measure(parse_cpu_weight),
        memory_current_bytes: files.memory_current.measure(parse_memory_bytes),
        memory_peak_bytes: files.memory_peak.measure(parse_memory_bytes),
        memory_events: files.memory_events.measure(parse_memory_events),
        io_pressure: files.io_pressure.measure(parse_psi),
    }
}

/// Build the `runner` group record from the runner PROCESS. PURE.
///
/// This record OVERLAPS every mixed group (the runner's cgroup and its
/// ancestors also count the runner): it is the runner's own share, never an
/// addend to them.
///
/// Per-process PSI, `cpu.weight` and `memory.events` do not exist, so those
/// axes are `not_supported` (never `unavailable`, never 0). `stat = None`
/// (the runner pid was not readable) makes the measurable axes `unavailable`.
pub fn runner_pressure_from_stat(
    stat: Option<&ProcStat>,
    clk_tck: u64,
    page_size_bytes: u64,
    vm_hwm_bytes: Option<u64>,
    cpu_usage_share: Option<f64>,
) -> WorkloadGroupPressure {
    let usec =
        stat.and_then(|s| (clk_tck > 0).then(|| s.cpu_ticks().saturating_mul(1_000_000) / clk_tck));
    let rss = stat.and_then(|s| {
        u64::try_from(s.rss_pages)
            .ok()?
            .checked_mul(page_size_bytes)
    });
    WorkloadGroupPressure {
        group: WorkloadGroup::Runner,
        mixed: false,
        cgroup_shapes: Vec::new(),
        scope_count: Measured::NotSupported,
        cpu_usage_share: Measured::from_read(cpu_usage_share),
        cpu_usage_usec: Measured::from_read(usec),
        cpu_pressure: Measured::NotSupported,
        cpu_weight: Measured::NotSupported,
        memory_current_bytes: Measured::from_read(rss),
        memory_peak_bytes: Measured::from_read(vm_hwm_bytes),
        memory_events: Measured::NotSupported,
        io_pressure: Measured::NotSupported,
    }
}

/// A group record on a platform with no cgroups/PSI (Windows, macOS). The
/// per-class CPU share and RSS come from the process census when the caller
/// has them; every cgroup instrument is `not_supported`.
pub fn group_pressure_without_cgroups(
    group: WorkloadGroup,
    cpu_usage_share: Option<f64>,
    rss_bytes: Option<u64>,
) -> WorkloadGroupPressure {
    WorkloadGroupPressure {
        group,
        mixed: false,
        cgroup_shapes: Vec::new(),
        scope_count: Measured::NotSupported,
        cpu_usage_share: Measured::from_read(cpu_usage_share),
        cpu_usage_usec: Measured::NotSupported,
        cpu_pressure: Measured::NotSupported,
        cpu_weight: Measured::NotSupported,
        memory_current_bytes: Measured::from_read(rss_bytes),
        memory_peak_bytes: Measured::NotSupported,
        memory_events: Measured::NotSupported,
        io_pressure: Measured::NotSupported,
    }
}

/// The `cgroup_pressure` fact: every group's record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CgroupPressureFacts {
    /// Where it was measured.
    pub platform: Platform,
    /// One record per group measured.
    pub groups: Vec<WorkloadGroupPressure>,
}

impl CgroupPressureFacts {
    /// The record for `group`, if measured.
    pub fn group(&self, group: WorkloadGroup) -> Option<&WorkloadGroupPressure> {
        self.groups.iter().find(|g| g.group == group)
    }

    /// The Windows/macOS arm before any census data: every group present,
    /// every pressure axis `not_supported`, CPU share and RSS `unavailable`.
    pub fn without_cgroups(platform: Platform) -> Self {
        Self {
            platform,
            groups: WorkloadGroup::ALL
                .iter()
                .map(|g| group_pressure_without_cgroups(*g, None, None))
                .collect(),
        }
    }

    /// Manifest for every group, keys `cgroup_pressure.<group>.<field>`.
    pub fn manifest_into(&self, out: &mut MeasuredManifest) {
        for g in &self.groups {
            g.manifest_into("cgroup_pressure.", out);
        }
    }
}

/// Share of host CPU capacity: `Δusage_usec / (Δwall_usec × ncpu)`, clamped
/// to 0..=1. `None` on a zero interval, zero cpus, or a counter that went
/// backwards (a reset is UNKNOWN, not idle).
pub fn cpu_share_of_host(
    prev_usage_usec: u64,
    cur_usage_usec: u64,
    interval_usec: u64,
    ncpu: u32,
) -> Option<f64> {
    if interval_usec == 0 || ncpu == 0 || cur_usage_usec < prev_usage_usec {
        return None;
    }
    let share =
        (cur_usage_usec - prev_usage_usec) as f64 / (interval_usec as f64 * f64::from(ncpu));
    Some(share.clamp(0.0, 1.0))
}

/// A process's CPU use as a fraction of ONE core (1.0 = one core saturated;
/// a multi-threaded process can exceed 1.0) over an interval. `None` on a
/// zero tick rate or interval, or a counter that went backwards.
pub fn cpu_share_from_ticks(
    prev_ticks: u64,
    cur_ticks: u64,
    interval_secs: f64,
    clk_tck: u64,
) -> Option<f64> {
    if clk_tck == 0 || !interval_secs.is_finite() || interval_secs <= 0.0 || cur_ticks < prev_ticks
    {
        return None;
    }
    Some((cur_ticks - prev_ticks) as f64 / clk_tck as f64 / interval_secs)
}

/// Each group's share of BUSY CPU (the groups' own sum = 1.0), from CPU-time
/// deltas over one interval. Empty when nothing was busy — a share of zero
/// work is UNKNOWN, not an even split.
pub fn busy_cpu_shares(deltas_usec: &[(WorkloadGroup, u64)]) -> BTreeMap<WorkloadGroup, f64> {
    let total: u64 = deltas_usec.iter().map(|(_, d)| *d).sum();
    if total == 0 {
        return BTreeMap::new();
    }
    let mut out = BTreeMap::new();
    for (g, d) in deltas_usec {
        *out.entry(*g).or_insert(0.0) += *d as f64 / total as f64;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host_calibration::measured::MeasuredState;
    use crate::host_calibration::parse::proc::parse_proc_pid_stat;
    use crate::host_calibration::vocab::{classify_cgroup, classify_cgroup_with};

    #[test]
    fn group_from_files_marks_unreadable_as_unavailable() {
        let files = CgroupFiles {
            cpu_pressure: CgroupFile::Contents("some avg10=1 avg60=2 avg300=3 total=4\n"),
            cpu_weight: CgroupFile::Contents("100\n"),
            memory_current: CgroupFile::Contents("garbage"),
            ..CgroupFiles::default()
        };
        let g = group_pressure_from_files(classify_cgroup("/ci.slice"), "/ci.slice", &files, None);
        assert_eq!(g.group, WorkloadGroup::Ci);
        assert_eq!(g.cpu_weight, Measured::Measured(100));
        assert_eq!(
            g.cpu_pressure.value().unwrap().some.unwrap().avg300,
            Some(3.0)
        );
        assert_eq!(g.memory_current_bytes, Measured::Unavailable);
        assert_eq!(g.cpu_usage_share, Measured::Unavailable);
        assert_eq!(g.io_pressure, Measured::Unavailable);
        assert_eq!(g.cgroup_shapes, ["/ci.slice"]);
        assert_eq!(g.scope_count, Measured::Unavailable);
        assert_eq!(
            g.with_scope_count(Some(3)).scope_count,
            Measured::Measured(3)
        );
    }

    #[test]
    fn absent_files_are_not_supported_failed_reads_unavailable() {
        // The runner's app.slice cgroup: the cpu controller is not enabled
        // there (app.slice enables only memory+pids), so cpu.weight does not
        // exist; this kernel predates memory.peak; io.pressure read failed.
        let path = "/user.slice/user-9.slice/user@9.service/app.slice/qontinui-runner.service";
        let files = CgroupFiles {
            cpu_pressure: CgroupFile::Contents("some avg10=1 avg60=1 avg300=1 total=1\n"),
            cpu_stat: CgroupFile::Contents("usage_usec 5\nuser_usec 3\nsystem_usec 2\n"),
            cpu_weight: CgroupFile::Absent,
            memory_current: CgroupFile::Contents("4096\n"),
            memory_peak: CgroupFile::Absent,
            memory_events: CgroupFile::Contents("low 0\nhigh 0\nmax 0\noom 0\noom_kill 0\n"),
            io_pressure: CgroupFile::ReadError,
        };
        let g =
            group_pressure_from_files(classify_cgroup_with(path, Some(path)), path, &files, None);
        assert_eq!(g.group, WorkloadGroup::Agents);
        assert!(g.mixed);
        assert_eq!(g.cpu_weight, Measured::NotSupported);
        assert_eq!(g.memory_peak_bytes, Measured::NotSupported);
        assert_eq!(g.io_pressure, Measured::Unavailable);
        assert_eq!(g.cpu_usage_usec, Measured::Measured(5));
        assert_eq!(g.memory_current_bytes, Measured::Measured(4096));
        // No uid on the wire.
        assert_eq!(
            g.cgroup_shapes,
            ["/user.slice/user-N.slice/user@N.service/app.slice/qontinui-runner.service"]
        );
        assert!(!serde_json::to_string(&g).unwrap().contains("user-9"));
    }

    #[test]
    fn from_read_result_splits_enoent_from_other_errors() {
        use std::io::{Error, ErrorKind};
        let ok: std::io::Result<String> = Ok("100".into());
        let missing: std::io::Result<String> = Err(Error::from(ErrorKind::NotFound));
        let denied: std::io::Result<String> = Err(Error::from(ErrorKind::PermissionDenied));
        assert_eq!(
            CgroupFile::from_read_result(&ok),
            CgroupFile::Contents("100")
        );
        assert_eq!(CgroupFile::from_read_result(&missing), CgroupFile::Absent);
        assert_eq!(CgroupFile::from_read_result(&denied), CgroupFile::ReadError);
    }

    #[test]
    fn runner_axes_without_a_per_process_instrument_are_not_supported() {
        let st = parse_proc_pid_stat(
            "77 (qontinui-runner) S 1 77 77 0 -1 0 0 0 0 0 150 50 0 0 20 0 40 0 10 1 1000 0",
        )
        .unwrap();
        let r = runner_pressure_from_stat(Some(&st), 100, 4096, Some(8192), Some(0.02));
        assert_eq!(r.cpu_usage_usec, Measured::Measured(2_000_000));
        assert_eq!(r.memory_current_bytes, Measured::Measured(4_096_000));
        assert_eq!(r.memory_peak_bytes, Measured::Measured(8192));
        assert_eq!(r.cpu_pressure, Measured::NotSupported);
        assert_eq!(r.cpu_weight, Measured::NotSupported);
        let gone = runner_pressure_from_stat(None, 100, 4096, None, None);
        assert_eq!(gone.cpu_usage_usec, Measured::Unavailable);
        assert_eq!(gone.io_pressure, Measured::NotSupported);
    }

    #[test]
    fn windows_arm_is_not_supported_never_zero() {
        let f = CgroupPressureFacts::without_cgroups(Platform::Windows);
        assert_eq!(f.groups.len(), 5);
        let mut m = MeasuredManifest::new();
        f.manifest_into(&mut m);
        assert_eq!(m.len(), 45);
        assert_eq!(
            m["cgroup_pressure.ci.cpu_pressure"],
            MeasuredState::NotSupported
        );
        assert_eq!(
            m["cgroup_pressure.agents.cpu_usage_share"],
            MeasuredState::Unavailable
        );
        let json = serde_json::to_string(&f).unwrap();
        assert!(
            !json.contains("\"value\""),
            "no value may be rendered: {json}"
        );
        assert!(!Platform::Windows.has_cgroup_psi());
        assert!(Platform::Wsl2.has_cgroup_psi());
    }

    #[test]
    fn share_arithmetic() {
        assert_eq!(cpu_share_of_host(0, 24_000_000, 1_000_000, 48), Some(0.5));
        assert_eq!(cpu_share_of_host(10, 5, 1, 1), None);
        assert_eq!(cpu_share_of_host(0, 5, 0, 1), None);
        assert_eq!(cpu_share_from_ticks(0, 98, 1.0, 100), Some(0.98));
        assert_eq!(cpu_share_from_ticks(0, 98, 0.0, 100), None);
        let b = busy_cpu_shares(&[
            (WorkloadGroup::Agents, 156),
            (WorkloadGroup::Ci, 88),
            (WorkloadGroup::System, 0),
        ]);
        assert!((b[&WorkloadGroup::Agents] - 156.0 / 244.0).abs() < 1e-12);
        assert!(busy_cpu_shares(&[(WorkloadGroup::Ci, 0)]).is_empty());
    }
}
