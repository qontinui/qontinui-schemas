//! Fixture tests for `qontinui_types::host_calibration` against real readings.
//!
//! `fixtures/host_calibration/merytshost_1020z/` is the motivating reading of
//! plan `2026-10-05-fleet-calibration-reversible-machine-tuning` (merytshost,
//! about 10:20Z on 2026-10-05). The plan records each PSI file's avg300
//! (host cpu `some` 6.19; `user.slice` 4.52, `ci.slice` 1.73, `system.slice`
//! 0.16), `cpu.weight` 100 on all three slices, the `ci-runners.slice` and
//! `user.slice` memory figures from `systemctl show`, and the `cat /dev/zero`
//! orphan (pid 118544, ppid 1, ~98 % of one core, started 2026-10-01 07:46).
//! Those values are exact; the avg10/avg60/total fields beside them are filled
//! in, in the kernel's line shape, because the plan did not record them. The
//! orphan's `stat` line is the real one (same process), and `proc_uptime` is
//! set so its age is the ~4.1 days it had at 10:20Z. `runner_proc_pid_cgroup`
//! is the runner process's `/proc/<pid>/cgroup` line on that host.
//!
//! `fixtures/host_calibration/live_2026-10-05/` holds unedited copies of the
//! same files read from the same host later that day (the mountinfo and
//! status files are line excerpts).

use qontinui_types::host_calibration::parse::cgroup::{
    parse_cpu_stat, parse_cpu_weight, parse_memory_bytes, parse_memory_events, parse_memory_limit,
    parse_proc_pid_cgroup, MemoryLimit,
};
use qontinui_types::host_calibration::parse::proc::{
    mount_for_path, parse_loadavg, parse_meminfo, parse_mountinfo, parse_proc_pid_stat,
    parse_status_uid, parse_uptime,
};
use qontinui_types::host_calibration::parse::psi::parse_psi;
use qontinui_types::host_calibration::{
    classify_cgroup, classify_cgroup_with, group_pressure_from_files, is_managed_service_cgroup,
    orphan_cpu_burner, CalibrationFacts, CgroupFile, CgroupFiles, CgroupPressureFacts, LeakContext,
    LeakThresholds, LeakVerdict, Measured, MeasuredState, Platform, ProcessObservation,
    WorkloadGroup,
};

macro_rules! m1020 {
    ($f:literal) => {
        include_str!(concat!("fixtures/host_calibration/merytshost_1020z/", $f))
    };
}
macro_rules! live {
    ($f:literal) => {
        include_str!(concat!("fixtures/host_calibration/live_2026-10-05/", $f))
    };
}

/// The plan's sysconf(_SC_CLK_TCK) on x86-64 Linux; a fixture input.
const CLK_TCK: u64 = 100;

fn avg300_some(text: &str) -> f64 {
    parse_psi(text).unwrap().some.unwrap().avg300.unwrap()
}

#[test]
fn motivating_reading_host_and_slice_psi() {
    assert_eq!(avg300_some(m1020!("proc_pressure_cpu")), 6.19);
    assert_eq!(avg300_some(m1020!("user.slice_cpu.pressure")), 4.52);
    assert_eq!(avg300_some(m1020!("ci.slice_cpu.pressure")), 1.73);
    assert_eq!(avg300_some(m1020!("system.slice_cpu.pressure")), 0.16);
    let mem = parse_psi(m1020!("proc_pressure_memory")).unwrap();
    assert_eq!(mem.some.unwrap().avg300, Some(0.0));
    assert_eq!(mem.full.unwrap().avg300, Some(0.0));
    assert_eq!(avg300_some(m1020!("proc_pressure_io")), 0.02);
}

#[test]
fn motivating_reading_weights_and_memory() {
    for w in [
        m1020!("user.slice_cpu.weight"),
        m1020!("ci.slice_cpu.weight"),
        m1020!("system.slice_cpu.weight"),
    ] {
        assert_eq!(parse_cpu_weight(w), Some(100), "nothing is weighted");
    }
    // ci-runners.slice peaked AT its MemoryHigh.
    let peak = parse_memory_bytes(m1020!("ci-runners.slice_memory.peak")).unwrap();
    assert_eq!(
        parse_memory_limit(m1020!("ci-runners.slice_memory.high")),
        Some(MemoryLimit::Bytes(peak))
    );
    assert_eq!(
        parse_memory_limit(m1020!("ci-runners.slice_memory.max")),
        Some(MemoryLimit::Bytes(198_000_000_000))
    );
    assert_eq!(
        parse_memory_bytes(m1020!("ci-runners.slice_memory.current")),
        Some(61_400_000_000)
    );
    assert_eq!(
        parse_memory_bytes(m1020!("user.slice_memory.peak")),
        Some(388_000_000_000)
    );
    assert_eq!(
        parse_memory_bytes(m1020!("user.slice_memory.current")),
        Some(235_000_000_000)
    );
}

#[test]
fn motivating_reading_group_pressure_assembles() {
    let runner_cgroup = parse_proc_pid_cgroup(m1020!("runner_proc_pid_cgroup")).unwrap();
    let groups = [
        (
            "/user.slice",
            m1020!("user.slice_cpu.pressure"),
            m1020!("user.slice_cpu.weight"),
        ),
        (
            "/ci.slice",
            m1020!("ci.slice_cpu.pressure"),
            m1020!("ci.slice_cpu.weight"),
        ),
        (
            "/system.slice",
            m1020!("system.slice_cpu.pressure"),
            m1020!("system.slice_cpu.weight"),
        ),
    ];
    let facts = CgroupPressureFacts {
        platform: Platform::Linux,
        groups: groups
            .iter()
            .map(|(path, psi, w)| {
                group_pressure_from_files(
                    Some(&runner_cgroup),
                    path,
                    &CgroupFiles {
                        cpu_pressure: CgroupFile::Contents(psi),
                        cpu_weight: CgroupFile::Contents(w),
                        ..CgroupFiles::default()
                    },
                    None,
                )
            })
            .collect(),
    };
    let agents = facts.group(WorkloadGroup::Agents).unwrap();
    assert_eq!(
        agents.cpu_pressure.value().unwrap().some.unwrap().avg300,
        Some(4.52)
    );
    assert_eq!(
        facts.group(WorkloadGroup::Ci).unwrap().cpu_weight,
        Measured::Measured(100)
    );
    assert!(facts.group(WorkloadGroup::System).is_some());
    // The runner lives under /user.slice, so agents is mixed (it overlaps the
    // runner record); ci and system are not.
    assert_eq!(agents.mixed, Measured::Measured(true));
    assert_eq!(
        facts.group(WorkloadGroup::Ci).unwrap().mixed,
        Measured::Measured(false)
    );
    assert_eq!(
        facts.group(WorkloadGroup::System).unwrap().mixed,
        Measured::Measured(false)
    );
    // Files not read are UNKNOWN, never 0.
    assert_eq!(agents.memory_current_bytes, Measured::Unavailable);
}

#[test]
fn motivating_reading_orphan_is_a_leak() {
    let st = parse_proc_pid_stat(m1020!("proc_118544_stat")).unwrap();
    assert_eq!((st.pid, st.ppid, st.comm.as_str()), (118_544, 1, "cat"));
    assert!(st.has_no_tty());
    let uptime = parse_uptime(m1020!("proc_uptime")).unwrap();
    let age = st.age_secs(uptime, CLK_TCK).unwrap();
    assert_eq!(age, 355_440); // ~4.1 days at 10:20Z
    let obs = ProcessObservation {
        stat: st,
        owner_uid: parse_status_uid(live!("proc_118544_status_excerpt")),
        age_secs: Some(age),
        sustained_cpu_share: Some(0.98),
        observed_secs: Some(3600),
        managed_unit: Measured::from_read(
            parse_proc_pid_cgroup(live!("proc_118544_cgroup"))
                .map(|cg| is_managed_service_cgroup(&cg, None)),
        ),
    };
    // It sits in a session scope, not a managed service.
    assert_eq!(obs.managed_unit, Measured::Measured(false));
    let ctx = LeakContext::default().with_known_workloads([
        "rustc",
        "cargo",
        "claude",
        "node",
        "Runner.Worker",
        "qontinui-runner",
    ]);
    let LeakVerdict::Leak(r) = orphan_cpu_burner(&obs, &ctx, &LeakThresholds::default()) else {
        panic!("the cat /dev/zero orphan must be detected");
    };
    assert_eq!(r.comm, "cat");
    assert_eq!(r.owner_uid, 1000);
    assert_eq!(r.age_secs, 355_440);
}

#[test]
fn live_files_parse() {
    assert!(parse_psi(live!("proc_pressure_cpu"))
        .unwrap()
        .full
        .is_some());
    assert!(parse_psi(live!("proc_pressure_memory")).is_some());
    assert!(parse_psi(live!("proc_pressure_io")).is_some());
    for f in [
        live!("ci.slice_cpu.pressure"),
        live!("ci.slice_io.pressure"),
        live!("user.slice_cpu.pressure"),
        live!("system.slice_cpu.pressure"),
    ] {
        let p = parse_psi(f).unwrap();
        assert!(p.some.unwrap().total_usec.is_some());
        assert!(p.full.is_some());
    }
    let files = CgroupFiles {
        cpu_pressure: CgroupFile::Contents(live!("ci.slice_cpu.pressure")),
        cpu_stat: CgroupFile::Contents(live!("ci.slice_cpu.stat")),
        cpu_weight: CgroupFile::Contents(live!("ci.slice_cpu.weight")),
        memory_current: CgroupFile::Contents(live!("ci.slice_memory.current")),
        memory_peak: CgroupFile::Contents(live!("ci.slice_memory.peak")),
        memory_events: CgroupFile::Contents(live!("ci.slice_memory.events")),
        io_pressure: CgroupFile::Contents(live!("ci.slice_io.pressure")),
    };
    let ci = group_pressure_from_files(None, "/ci.slice", &files, Some(0.2));
    for (axis, state) in [
        ("cpu_usage_usec", ci.cpu_usage_usec.state()),
        ("cpu_pressure", ci.cpu_pressure.state()),
        ("cpu_weight", ci.cpu_weight.state()),
        ("memory_current", ci.memory_current_bytes.state()),
        ("memory_peak", ci.memory_peak_bytes.state()),
        ("memory_events", ci.memory_events.state()),
        ("io_pressure", ci.io_pressure.state()),
    ] {
        assert_eq!(state, MeasuredState::Measured, "{axis}");
    }
    assert_eq!(ci.memory_peak_bytes, Measured::Measured(148_518_645_760));
    assert!(parse_cpu_stat(live!("ci.slice_cpu.stat"))
        .unwrap()
        .user_usec
        .is_some());
    assert_eq!(
        parse_memory_events(live!("ci.slice_memory.events"))
            .unwrap()
            .high,
        Some(22_808)
    );

    let mi = parse_meminfo(live!("proc_meminfo")).unwrap();
    assert!(mi.mem_available_bytes.unwrap() < mi.mem_total_bytes.unwrap());
    assert!(mi.swap_used_bytes().unwrap() > 0);
    assert!(parse_loadavg(live!("proc_loadavg")).is_some());

    // The runner's own cgroup is mixed agents (vet correction 4), both by the
    // runner pid's cgroup and by the unit-name fallback.
    let cg = parse_proc_pid_cgroup(live!("proc_self_cgroup")).unwrap();
    let class = classify_cgroup_with(&cg, Some(&cg));
    assert_eq!(class.group, WorkloadGroup::Agents);
    assert_eq!(class.mixed, Measured::Measured(true));
    assert_eq!(classify_cgroup(&cg).mixed, Measured::Measured(true));
    // The runner's cgroup is never "managed" for the leak predicate.
    assert!(!is_managed_service_cgroup(&cg, Some(&cg)));

    let mounts = parse_mountinfo(live!("proc_self_mountinfo_excerpt"));
    assert_eq!(
        mount_for_path(&mounts, "/dev/shm/x").unwrap().fs_type,
        "tmpfs"
    );
    assert_eq!(
        mount_for_path(&mounts, "/home/x/target").unwrap().fs_type,
        "ext4"
    );
}

#[test]
fn windows_arm_yields_not_supported_pressure() {
    let facts = CalibrationFacts {
        platform: Platform::Windows,
        cgroup_pressure: Measured::Measured(CgroupPressureFacts::without_cgroups(
            Platform::Windows,
        )),
        compile_cache: Measured::Unavailable,
        build_volume: Measured::Unavailable,
        leaks: Measured::Unavailable,
        disk_use: Measured::Unavailable,
    };
    let m = facts.measured_manifest();
    for g in ["agents", "ci", "system", "runner", "other"] {
        for axis in ["cpu_pressure", "io_pressure", "cpu_weight"] {
            assert_eq!(
                m[&format!("cgroup_pressure.{g}.{axis}")],
                MeasuredState::NotSupported,
                "{g}.{axis}"
            );
        }
    }
    let json = serde_json::to_value(&facts).unwrap();
    let ci = &json["cgroup_pressure"]["value"]["groups"][1];
    assert_eq!(ci["group"], "ci");
    assert_eq!(
        ci["cpu_pressure"],
        serde_json::json!({"state": "not_supported"})
    );
}
