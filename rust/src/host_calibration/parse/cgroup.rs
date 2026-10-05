//! cgroup v2 interface files: `cpu.stat`, `cpu.weight`, `memory.current`,
//! `memory.peak`, `memory.high`/`memory.max`, `memory.events`, and the
//! `/proc/<pid>/cgroup` membership line. (`*.pressure` files are PSI —
//! [`super::psi::parse_psi`].)

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// `cpu.stat`. `usage_usec` is required (it exists whenever the file does);
/// the rest are present only when the `cpu` controller is enabled on the
/// cgroup, so they are optional.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
pub struct CpuStat {
    /// Total CPU time, microseconds.
    pub usage_usec: u64,
    /// User-mode CPU time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_usec: Option<u64>,
    /// Kernel-mode CPU time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_usec: Option<u64>,
    /// Enforcement periods elapsed (bandwidth control).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nr_periods: Option<u64>,
    /// Periods throttled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nr_throttled: Option<u64>,
    /// Time throttled, microseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub throttled_usec: Option<u64>,
}

/// Parse `cpu.stat`. `None` without a `usage_usec` line.
pub fn parse_cpu_stat(text: &str) -> Option<CpuStat> {
    let mut usage = None;
    let mut s = CpuStat::default();
    for (k, v) in key_u64_lines(text) {
        match k {
            "usage_usec" => usage = Some(v),
            "user_usec" => s.user_usec = Some(v),
            "system_usec" => s.system_usec = Some(v),
            "nr_periods" => s.nr_periods = Some(v),
            "nr_throttled" => s.nr_throttled = Some(v),
            "throttled_usec" => s.throttled_usec = Some(v),
            _ => {}
        }
    }
    s.usage_usec = usage?;
    Some(s)
}

/// Parse `cpu.weight` (kernel range 1..=10000). Out of range is `None`.
pub fn parse_cpu_weight(text: &str) -> Option<u32> {
    text.trim()
        .parse::<u32>()
        .ok()
        .filter(|w| (1..=10_000).contains(w))
}

/// Parse a single-integer byte file: `memory.current`, `memory.peak`,
/// `memory.swap.current`.
pub fn parse_memory_bytes(text: &str) -> Option<u64> {
    text.trim().parse::<u64>().ok()
}

/// `memory.high` / `memory.max`: a byte limit or the literal `max`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", content = "bytes", rename_all = "snake_case")]
pub enum MemoryLimit {
    /// No limit.
    Max,
    /// A limit in bytes.
    Bytes(u64),
}

/// Parse `memory.high` / `memory.max`.
pub fn parse_memory_limit(text: &str) -> Option<MemoryLimit> {
    match text.trim() {
        "max" => Some(MemoryLimit::Max),
        t => t.parse::<u64>().ok().map(MemoryLimit::Bytes),
    }
}

/// `memory.events` counters (hierarchical). Each is optional: `oom_group_kill`
/// is absent before kernel 5.17.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
pub struct MemoryEvents {
    /// Reclaimed under `memory.low` protection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub low: Option<u64>,
    /// Throttled at `memory.high`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub high: Option<u64>,
    /// Hit `memory.max`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<u64>,
    /// OOM events.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oom: Option<u64>,
    /// Processes OOM-killed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oom_kill: Option<u64>,
    /// Group OOM kills.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oom_group_kill: Option<u64>,
}

/// Parse `memory.events`. `None` when no known counter is present.
pub fn parse_memory_events(text: &str) -> Option<MemoryEvents> {
    let mut e = MemoryEvents::default();
    let mut any = false;
    for (k, v) in key_u64_lines(text) {
        let slot = match k {
            "low" => &mut e.low,
            "high" => &mut e.high,
            "max" => &mut e.max,
            "oom" => &mut e.oom,
            "oom_kill" => &mut e.oom_kill,
            "oom_group_kill" => &mut e.oom_group_kill,
            _ => continue,
        };
        *slot = Some(v);
        any = true;
    }
    any.then_some(e)
}

/// The `oom_kill` counter of a `memory.events` file — the host reader's
/// (runner#1982) entry point, kept so it can switch here unchanged.
pub fn parse_memory_events_oom_kill(text: &str) -> Option<u64> {
    super::proc::parse_vmstat_oom_kill(text)
}

/// The cgroup v2 path from a `/proc/<pid>/cgroup` file: the `0::<path>`
/// line. `None` on a v1-only (no unified line) file. A trailing ` (deleted)`
/// (the kernel's mark for a removed cgroup) is stripped.
pub fn parse_proc_pid_cgroup(text: &str) -> Option<String> {
    text.lines()
        .find_map(|l| l.strip_prefix("0::"))
        .map(|p| {
            let p = p.trim();
            p.strip_suffix(" (deleted)").unwrap_or(p).to_string()
        })
        .filter(|p| !p.is_empty())
}

/// `<key> <u64>` lines, skipping anything that does not fit.
fn key_u64_lines(text: &str) -> impl Iterator<Item = (&str, u64)> {
    text.lines().filter_map(|l| {
        let mut it = l.split_whitespace();
        let k = it.next()?;
        let v = it.next()?.parse::<u64>().ok()?;
        Some((k, v))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_stat_full_and_minimal() {
        let s = parse_cpu_stat(
            "usage_usec 1214496025050\nuser_usec 810168459018\nsystem_usec 404327566032\n\
             nice_usec 0\nnr_periods 0\nnr_throttled 0\nthrottled_usec 0\n",
        )
        .unwrap();
        assert_eq!(s.usage_usec, 1_214_496_025_050);
        assert_eq!(s.user_usec, Some(810_168_459_018));
        assert_eq!(s.nr_throttled, Some(0));
        // Controller not enabled: three lines only.
        let m = parse_cpu_stat("usage_usec 5\nuser_usec 3\nsystem_usec 2\n").unwrap();
        assert_eq!(m.nr_periods, None);
        assert_eq!(parse_cpu_stat("user_usec 3\n"), None);
    }

    #[test]
    fn cpu_weight_range() {
        assert_eq!(parse_cpu_weight("100\n"), Some(100));
        assert_eq!(parse_cpu_weight("0"), None);
        assert_eq!(parse_cpu_weight("10001"), None);
        assert_eq!(parse_cpu_weight("max"), None);
    }

    #[test]
    fn memory_files() {
        assert_eq!(parse_memory_bytes("41140686848\n"), Some(41_140_686_848));
        assert_eq!(parse_memory_bytes(""), None);
        assert_eq!(parse_memory_limit("max\n"), Some(MemoryLimit::Max));
        assert_eq!(parse_memory_limit("1024"), Some(MemoryLimit::Bytes(1024)));
        assert_eq!(parse_memory_limit("x"), None);
    }

    #[test]
    fn memory_events() {
        let e =
            parse_memory_events("low 0\nhigh 22808\nmax 0\noom 0\noom_kill 0\noom_group_kill 0\n")
                .unwrap();
        assert_eq!(e.high, Some(22808));
        assert_eq!(e.oom_kill, Some(0));
        let old = parse_memory_events("low 0\nhigh 1\nmax 0\noom 0\noom_kill 2\n").unwrap();
        assert_eq!(old.oom_group_kill, None);
        assert_eq!(parse_memory_events("junk\n"), None);
        assert_eq!(parse_memory_events_oom_kill("oom 1\noom_kill 2\n"), Some(2));
    }

    #[test]
    fn proc_pid_cgroup() {
        assert_eq!(
            parse_proc_pid_cgroup("0::/user.slice/a.scope\n").as_deref(),
            Some("/user.slice/a.scope")
        );
        assert_eq!(
            parse_proc_pid_cgroup("12:cpu:/x\n0::/ci.slice\n").as_deref(),
            Some("/ci.slice")
        );
        assert_eq!(parse_proc_pid_cgroup("12:cpu:/x\n"), None);
        assert_eq!(
            parse_proc_pid_cgroup("0::/user.slice/a.scope (deleted)\n").as_deref(),
            Some("/user.slice/a.scope")
        );
    }
}
