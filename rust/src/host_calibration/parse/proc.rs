//! procfs files: `/proc/meminfo`, `/proc/<pid>/stat`, `/proc/<pid>/status`
//! (uid), `/proc/uptime`, `/proc/loadavg`, `/proc/vmstat` (`oom_kill`),
//! `/proc/sys/kernel/random/boot_id`, and `/proc/<pid>/mountinfo`.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::parse_nonneg_f64;

/// The `/proc/meminfo` fields calibration reads, in BYTES (the file is kB).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
pub struct Meminfo {
    /// `MemTotal`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mem_total_bytes: Option<u64>,
    /// `MemFree`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mem_free_bytes: Option<u64>,
    /// `MemAvailable`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mem_available_bytes: Option<u64>,
    /// `Buffers`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buffers_bytes: Option<u64>,
    /// `Cached`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cached_bytes: Option<u64>,
    /// `Shmem` (tmpfs pages included).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shmem_bytes: Option<u64>,
    /// `SwapTotal`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub swap_total_bytes: Option<u64>,
    /// `SwapFree`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub swap_free_bytes: Option<u64>,
    /// `SwapCached`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub swap_cached_bytes: Option<u64>,
    /// `CommitLimit`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit_limit_bytes: Option<u64>,
    /// `Committed_AS`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub committed_as_bytes: Option<u64>,
}

impl Meminfo {
    /// `SwapTotal − SwapFree`, when both were read.
    pub fn swap_used_bytes(&self) -> Option<u64> {
        Some(self.swap_total_bytes?.saturating_sub(self.swap_free_bytes?))
    }
}

/// One `/proc/meminfo` value in bytes, by key (`"MemAvailable"`). Lines with
/// a `kB` unit are scaled; unitless lines (`HugePages_Total`) are returned
/// as-is.
pub fn meminfo_value(text: &str, key: &str) -> Option<u64> {
    text.lines().find_map(|l| {
        let (k, rest) = l.split_once(':')?;
        if k.trim() != key {
            return None;
        }
        let mut it = rest.split_whitespace();
        let v = it.next()?.parse::<u64>().ok()?;
        match it.next() {
            Some("kB") => v.checked_mul(1024),
            None => Some(v),
            Some(_) => None,
        }
    })
}

/// Parse `/proc/meminfo`. `None` when `MemTotal` is absent (not a meminfo).
pub fn parse_meminfo(text: &str) -> Option<Meminfo> {
    let get = |k| meminfo_value(text, k);
    let m = Meminfo {
        mem_total_bytes: get("MemTotal"),
        mem_free_bytes: get("MemFree"),
        mem_available_bytes: get("MemAvailable"),
        buffers_bytes: get("Buffers"),
        cached_bytes: get("Cached"),
        shmem_bytes: get("Shmem"),
        swap_total_bytes: get("SwapTotal"),
        swap_free_bytes: get("SwapFree"),
        swap_cached_bytes: get("SwapCached"),
        commit_limit_bytes: get("CommitLimit"),
        committed_as_bytes: get("Committed_AS"),
    };
    m.mem_total_bytes.map(|_| m)
}

/// The `/proc/<pid>/stat` fields calibration reads. Field numbers are
/// proc(5)'s, 1-based.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProcStat {
    /// (1) pid.
    pub pid: u32,
    /// (2) comm, without the parentheses. May contain spaces and `)`.
    pub comm: String,
    /// (3) state letter.
    pub state: char,
    /// (4) parent pid.
    pub ppid: u32,
    /// (5) process group.
    pub pgrp: i32,
    /// (6) session id.
    pub session: i32,
    /// (7) controlling terminal device number; 0 = no controlling tty.
    pub tty_nr: i32,
    /// (14) user-mode CPU time, clock ticks.
    pub utime_ticks: u64,
    /// (15) kernel-mode CPU time, clock ticks.
    pub stime_ticks: u64,
    /// (20) thread count.
    pub num_threads: i64,
    /// (22) start time after boot, clock ticks.
    pub starttime_ticks: u64,
    /// (24) resident set size, pages.
    pub rss_pages: i64,
}

impl ProcStat {
    /// True when the process has no controlling terminal.
    pub fn has_no_tty(&self) -> bool {
        self.tty_nr == 0
    }

    /// utime + stime, clock ticks.
    pub fn cpu_ticks(&self) -> u64 {
        self.utime_ticks.saturating_add(self.stime_ticks)
    }

    /// Seconds since the process started, given `/proc/uptime`'s first field
    /// and `sysconf(_SC_CLK_TCK)`. `None` for a zero tick rate or a start
    /// time after the given uptime (inconsistent inputs, not age 0).
    pub fn age_secs(&self, uptime_secs: f64, clk_tck: u64) -> Option<u64> {
        if clk_tck == 0 || !uptime_secs.is_finite() {
            return None;
        }
        let started = self.starttime_ticks as f64 / clk_tck as f64;
        let age = uptime_secs - started;
        (age >= 0.0).then_some(age as u64)
    }
}

/// Parse `/proc/<pid>/stat`.
///
/// `comm` is everything between the FIRST `(` and the LAST `)`: the kernel
/// does not escape it, so `(a) b)` is a legal comm of `a) b`. The fields
/// after it are split on whitespace.
pub fn parse_proc_pid_stat(text: &str) -> Option<ProcStat> {
    let text = text.trim_end();
    let open = text.find('(')?;
    let close = text.rfind(')')?;
    if close < open {
        return None;
    }
    let pid = text[..open].trim().parse::<u32>().ok()?;
    let comm = text[open + 1..close].to_string();
    // rest[0] is field 3 (state).
    let rest: Vec<&str> = text[close + 1..].split_whitespace().collect();
    let field = |n: usize| rest.get(n - 3).copied();
    let state = {
        let s = field(3)?;
        let mut cs = s.chars();
        let c = cs.next()?;
        if cs.next().is_some() {
            return None;
        }
        c
    };
    Some(ProcStat {
        pid,
        comm,
        state,
        ppid: field(4)?.parse().ok()?,
        pgrp: field(5)?.parse().ok()?,
        session: field(6)?.parse().ok()?,
        tty_nr: field(7)?.parse().ok()?,
        utime_ticks: field(14)?.parse().ok()?,
        stime_ticks: field(15)?.parse().ok()?,
        num_threads: field(20)?.parse().ok()?,
        starttime_ticks: field(22)?.parse().ok()?,
        rss_pages: field(24)?.parse().ok()?,
    })
}

/// The REAL uid from `/proc/<pid>/status` (`Uid:` line, first of four).
pub fn parse_status_uid(text: &str) -> Option<u32> {
    text.lines()
        .find_map(|l| l.strip_prefix("Uid:"))
        .and_then(|r| r.split_whitespace().next())
        .and_then(|u| u.parse().ok())
}

/// A `kB` field of `/proc/<pid>/status` (`VmRSS`, `VmHWM`, …) in bytes.
pub fn parse_status_bytes(text: &str, key: &str) -> Option<u64> {
    meminfo_value(text, key)
}

/// `/proc/uptime`'s first field: seconds since boot.
pub fn parse_uptime(text: &str) -> Option<f64> {
    text.split_whitespace().next().and_then(parse_nonneg_f64)
}

/// `/proc/loadavg`'s three averages.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct LoadAvg {
    /// 1-minute.
    pub one: f64,
    /// 5-minute.
    pub five: f64,
    /// 15-minute.
    pub fifteen: f64,
}

/// Parse `/proc/loadavg` (`"32.07 28.81 31.69 31/16803 2177659"`).
///
/// Recognised by SHAPE, not position — the host reader's (runner#1982) rule:
/// the first line whose first three tokens are finite non-negative floats and
/// whose fourth is a `running/total` pair, so it is found inside a `cat` of
/// several procfs files. All three or nothing.
pub fn parse_loadavg(text: &str) -> Option<LoadAvg> {
    text.lines().find_map(|line| {
        let mut it = line.split_whitespace();
        let one = parse_nonneg_f64(it.next()?)?;
        let five = parse_nonneg_f64(it.next()?)?;
        let fifteen = parse_nonneg_f64(it.next()?)?;
        let (running, total) = it.next()?.split_once('/')?;
        running.parse::<u64>().ok()?;
        total.parse::<u64>().ok()?;
        Some(LoadAvg { one, five, fifteen })
    })
}

/// The monotonic `oom_kill` counter from `/proc/vmstat` (also the line shape
/// of a cgroup `memory.events`). `None` when absent.
pub fn parse_vmstat_oom_kill(text: &str) -> Option<u64> {
    text.lines().find_map(|l| {
        let mut it = l.split_whitespace();
        (it.next()? == "oom_kill")
            .then(|| it.next()?.parse::<u64>().ok())
            .flatten()
    })
}

/// A kernel `boot_id`: a trimmed UUID-shaped token, lowercased.
pub fn parse_boot_id(text: &str) -> Option<String> {
    let t = text.trim();
    let ok = t.len() == 36 && t.chars().all(|c| c.is_ascii_hexdigit() || c == '-');
    ok.then(|| t.to_ascii_lowercase())
}

/// One `/proc/<pid>/mountinfo` entry, reduced to what calibration reads.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MountEntry {
    /// (5) mount point, with the kernel's octal escapes (`\040`) decoded.
    pub mount_point: String,
    /// Filesystem type (first field after ` - `).
    pub fs_type: String,
    /// Mount source.
    pub source: String,
    /// Per-superblock options (`size=…,nr_inodes=…`).
    pub super_options: String,
}

/// Parse every well-formed line of a mountinfo file.
pub fn parse_mountinfo(text: &str) -> Vec<MountEntry> {
    text.lines()
        .filter_map(|l| {
            let (pre, post) = l.split_once(" - ")?;
            let mount_point = pre.split_whitespace().nth(4)?;
            let mut post = post.split_whitespace();
            Some(MountEntry {
                mount_point: decode_mount_escapes(mount_point),
                fs_type: post.next()?.to_string(),
                source: post.next()?.to_string(),
                super_options: post.next().unwrap_or("").to_string(),
            })
        })
        .collect()
}

/// The entry for the mount that CONTAINS `path` (longest mount-point prefix
/// on a component boundary; a later entry for the same mount point wins, as
/// it shadows the earlier one). `path` must be absolute.
pub fn mount_for_path<'a>(entries: &'a [MountEntry], path: &str) -> Option<&'a MountEntry> {
    let mut best: Option<&MountEntry> = None;
    for e in entries {
        let mp = e.mount_point.as_str();
        let contains = path == mp
            || mp == "/"
            || (path.starts_with(mp) && path.as_bytes().get(mp.len()) == Some(&b'/'));
        if contains && best.is_none_or(|b| mp.len() >= b.mount_point.len()) {
            best = Some(e);
        }
    }
    best.filter(|_| path.starts_with('/'))
}

fn decode_mount_escapes(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'\\' && i + 4 <= b.len() {
            let oct = &b[i + 1..i + 4];
            if oct.iter().all(|c| (b'0'..=b'7').contains(c)) {
                let v = oct
                    .iter()
                    .fold(0u32, |acc, c| acc * 8 + u32::from(c - b'0'));
                if let Ok(v) = u8::try_from(v) {
                    out.push(v);
                    i += 4;
                    continue;
                }
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meminfo_scales_kb_to_bytes() {
        let t = "MemTotal:       386753544 kB\nMemAvailable:   222320708 kB\n\
                 SwapTotal:      105985020 kB\nSwapFree:        44248244 kB\nHugePages_Total:       0\n";
        let m = parse_meminfo(t).unwrap();
        assert_eq!(m.mem_total_bytes, Some(386_753_544 * 1024));
        assert_eq!(m.swap_used_bytes(), Some((105_985_020 - 44_248_244) * 1024));
        assert_eq!(m.cached_bytes, None);
        assert_eq!(meminfo_value(t, "HugePages_Total"), Some(0));
        assert_eq!(parse_meminfo("SwapFree: 1 kB\n"), None);
    }

    #[test]
    fn stat_comm_with_spaces_and_parens() {
        let t = "4242 (tmux: server (x)) S 1 4242 4242 0 -1 4194560 1 0 0 0 7 3 0 0 20 0 2 0 900 1 33 18446744073709551615";
        let s = parse_proc_pid_stat(t).unwrap();
        assert_eq!(s.pid, 4242);
        assert_eq!(s.comm, "tmux: server (x)");
        assert_eq!(s.state, 'S');
        assert_eq!(s.ppid, 1);
        assert_eq!(s.tty_nr, 0);
        assert!(s.has_no_tty());
        assert_eq!(s.cpu_ticks(), 10);
        assert_eq!(s.num_threads, 2);
        assert_eq!(s.starttime_ticks, 900);
        assert_eq!(s.rss_pages, 33);
        assert_eq!(s.age_secs(19.0, 100), Some(10));
        assert_eq!(s.age_secs(1.0, 100), None);
        assert_eq!(s.age_secs(19.0, 0), None);
    }

    #[test]
    fn stat_rejects_truncated_or_malformed() {
        assert_eq!(parse_proc_pid_stat(""), None);
        assert_eq!(parse_proc_pid_stat("1 (init) S 0 1 1"), None);
        assert_eq!(parse_proc_pid_stat("x (init) S 0"), None);
        assert_eq!(parse_proc_pid_stat("1 )init( S"), None);
    }

    #[test]
    fn status_uid_and_uptime() {
        assert_eq!(
            parse_status_uid("Name:\tcat\nUid:\t1000\t1000\t1000\t1000\n"),
            Some(1000)
        );
        assert_eq!(parse_status_uid("Name:\tcat\n"), None);
        assert_eq!(
            parse_status_bytes("VmHWM:\t    2036 kB\n", "VmHWM"),
            Some(2036 * 1024)
        );
        assert_eq!(
            parse_uptime("3262291.72 119515615.81\n"),
            Some(3_262_291.72)
        );
        assert_eq!(parse_uptime(""), None);
    }

    #[test]
    fn loadavg_vmstat_boot_id() {
        let l = parse_loadavg("MemTotal: 1 kB\n32.07 28.81 31.69 31/16803 2177659\n").unwrap();
        assert_eq!((l.one, l.five, l.fifteen), (32.07, 28.81, 31.69));
        assert_eq!(parse_loadavg("32.07 28.81\n"), None);
        assert_eq!(parse_vmstat_oom_kill("pgfault 9\noom_kill 6\n"), Some(6));
        assert_eq!(parse_vmstat_oom_kill("oom_kill_foo 3\n"), None);
        assert_eq!(
            parse_boot_id("0F0E0D0C-0B0A-4908-8706-050403020100\n").as_deref(),
            Some("0f0e0d0c-0b0a-4908-8706-050403020100")
        );
        assert_eq!(parse_boot_id("nope"), None);
    }

    #[test]
    fn mountinfo_and_containing_mount() {
        let t = "30 1 8:1 / / rw,relatime shared:1 - ext4 /dev/sda1 rw,errors=remount-ro\n\
                 44 30 0:35 / /tmp rw,nosuid,nodev shared:47 - tmpfs tmpfs rw,nr_inodes=1048576,inode64\n\
                 50 30 0:40 / /mnt/a\\040b rw - tmpfs tmpfs rw,size=10k\n\
                 garbage line\n";
        let m = parse_mountinfo(t);
        assert_eq!(m.len(), 3);
        assert_eq!(m[2].mount_point, "/mnt/a b");
        assert_eq!(mount_for_path(&m, "/tmp/x/y").unwrap().fs_type, "tmpfs");
        assert_eq!(mount_for_path(&m, "/tmpfoo").unwrap().fs_type, "ext4");
        assert_eq!(
            mount_for_path(&m, "/mnt/a b/t").unwrap().super_options,
            "rw,size=10k"
        );
        assert_eq!(mount_for_path(&m, "relative"), None);
    }
}
