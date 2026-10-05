//! The closed vocabularies: workload groups and catalog action ids.
//!
//! Both are wire-frozen. A variant's string never changes meaning; a new
//! group or action is a new variant plus its tests, never a re-spelling.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Which workload a cgroup's (or process's) resource use is charged to.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum WorkloadGroup {
    /// Agent sessions: session scopes under the user manager and the rest of
    /// `user.slice`.
    Agents,
    /// CI: `ci.slice` and everything beneath it.
    Ci,
    /// `system.slice` (system services, containers on the systemd driver) and
    /// `init.scope`.
    System,
    /// The runner PROCESS, measured from `/proc/<pid>/stat`. Never a cgroup:
    /// the runner's service cgroup also holds the sessions it spawns
    /// in-process, so that cgroup is charged to [`WorkloadGroup::Agents`] and
    /// flagged mixed.
    Runner,
    /// Anything no rule matched (the root cgroup, `machine.slice`, …).
    Other,
}

impl WorkloadGroup {
    /// Every group, in wire order.
    pub const ALL: [WorkloadGroup; 5] = [
        WorkloadGroup::Agents,
        WorkloadGroup::Ci,
        WorkloadGroup::System,
        WorkloadGroup::Runner,
        WorkloadGroup::Other,
    ];

    /// The wire word.
    pub fn as_str(self) -> &'static str {
        match self {
            WorkloadGroup::Agents => "agents",
            WorkloadGroup::Ci => "ci",
            WorkloadGroup::System => "system",
            WorkloadGroup::Runner => "runner",
            WorkloadGroup::Other => "other",
        }
    }
}

/// The runner's systemd user unit. Its cgroup holds the runner AND the
/// sessions it spawns in-process (plan vet correction 4), so it classifies as
/// mixed agents. A product unit name, not a host literal.
pub const RUNNER_SERVICE_UNIT: &str = "qontinui-runner.service";

/// The classification of one cgroup path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CgroupClass {
    /// The group the cgroup's use is charged to.
    pub group: WorkloadGroup,
    /// True when the cgroup holds more than one workload: it IS the runner
    /// process's cgroup or an ANCESTOR of it, so its counters include the
    /// runner (which spawns sessions in-process and shares their cgroup).
    ///
    /// A mixed group's figures OVERLAP the `runner` record, which is measured
    /// from the runner process itself: never sum a mixed group and `runner`.
    pub mixed: bool,
}

/// Classify a cgroup v2 path to its [`CgroupClass`], with `mixed` computed
/// from the runner process's ACTUAL cgroup. Pure.
///
/// Paths are accepted as `/proc/<pid>/cgroup` writes them (`0::/user.slice/…`),
/// or cgroupfs-relative with or without the leading `/`. Group rules, first
/// match wins:
///
/// 1. `ci.slice` and its children → `ci`;
/// 2. `system.slice` and `init.scope` → `system`;
/// 3. `user.slice` and everything under it (the `user@<uid>.service/*.scope`
///    session scopes and the runner's own service cgroup included) → `agents`;
/// 4. anything else → `other`.
///
/// `mixed`: with `runner_cgroup = Some(rc)` (from
/// [`super::parse::cgroup::parse_proc_pid_cgroup`] on the runner pid), the
/// path is mixed iff it equals `rc` or is an ancestor of it — so `/user.slice`
/// is mixed on a host whose runner lives under it. With `None` (runner pid
/// unknown), the fallback is the unit-name rule: mixed iff the path is at or
/// below `user@<uid>.service/app.slice/<runner unit>`.
///
/// The `runner` group is never produced here — see [`WorkloadGroup::Runner`].
pub fn classify_cgroup_with(path: &str, runner_cgroup: Option<&str>) -> CgroupClass {
    let parts = cgroup_components(path);
    let group = match parts.first().copied() {
        Some("ci.slice") => WorkloadGroup::Ci,
        Some("system.slice") | Some("init.scope") => WorkloadGroup::System,
        Some("user.slice") => WorkloadGroup::Agents,
        _ => WorkloadGroup::Other,
    };
    let mixed = match runner_cgroup {
        Some(rc) => {
            let rc = cgroup_components(rc);
            parts.len() <= rc.len() && parts.iter().zip(&rc).all(|(a, b)| a == b)
        }
        None => {
            group == WorkloadGroup::Agents
                && parts.windows(3).any(|w| {
                    is_user_manager_unit(w[0]) && w[1] == "app.slice" && w[2] == RUNNER_SERVICE_UNIT
                })
        }
    };
    CgroupClass { group, mixed }
}

/// [`classify_cgroup_with`] with the runner's cgroup unknown (unit-name
/// fallback for `mixed`).
pub fn classify_cgroup(path: &str) -> CgroupClass {
    classify_cgroup_with(path, None)
}

/// The group alone, for callers that do not need `mixed`.
pub fn classify_cgroup_path(path: &str) -> WorkloadGroup {
    classify_cgroup(path).group
}

/// A cgroup path's components, `0::` prefix and empty segments removed.
pub(crate) fn cgroup_components(path: &str) -> Vec<&str> {
    let t = path.trim();
    let t = t.strip_prefix("0::").unwrap_or(t);
    t.split('/').filter(|p| !p.is_empty()).collect()
}

/// `user@<digits>.service` — the per-user systemd manager's unit.
pub(crate) fn is_user_manager_unit(component: &str) -> bool {
    component
        .strip_prefix("user@")
        .and_then(|r| r.strip_suffix(".service"))
        .is_some_and(|uid| !uid.is_empty() && uid.bytes().all(|b| b.is_ascii_digit()))
}

/// The closed calibration action catalog's ids. Wire-frozen snake_case.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ActionId {
    /// Lever 1, agent half: one sccache daemon per user per machine.
    CompileCacheAgent,
    /// Lever 1, CI half: the per-trust-class CI caches (sized, not provisioned).
    CompileCacheCi,
    /// Lever 2: the dedicated fast build volume.
    BuildVolume,
    /// Lever 3, root tier: CPUWeight between the `ci` and `agents` slices.
    CpuWeightRoot,
    /// Lever 3, user tier: the builds slice's weight within the user manager.
    CpuWeightUser,
    /// Lever 4: warm long-lived CI services.
    WarmCiServices,
    /// RESERVED, NOT CATALOGUED. Terminating a leaked orphan enters the
    /// catalog only after the leak detector's precision is proven (plan D9 /
    /// Phase 9). The id is frozen now so the wire word cannot be reused;
    /// [`ActionId::is_catalogued`] is false for it and no rule may recommend it.
    TerminateOrphan,
}

impl ActionId {
    /// Every id, reserved ones included, in wire order.
    pub const ALL: [ActionId; 7] = [
        ActionId::CompileCacheAgent,
        ActionId::CompileCacheCi,
        ActionId::BuildVolume,
        ActionId::CpuWeightRoot,
        ActionId::CpuWeightUser,
        ActionId::WarmCiServices,
        ActionId::TerminateOrphan,
    ];

    /// The ids a rule may recommend today.
    pub const CATALOGUED: [ActionId; 6] = [
        ActionId::CompileCacheAgent,
        ActionId::CompileCacheCi,
        ActionId::BuildVolume,
        ActionId::CpuWeightRoot,
        ActionId::CpuWeightUser,
        ActionId::WarmCiServices,
    ];

    /// The wire word.
    pub fn as_str(self) -> &'static str {
        match self {
            ActionId::CompileCacheAgent => "compile_cache_agent",
            ActionId::CompileCacheCi => "compile_cache_ci",
            ActionId::BuildVolume => "build_volume",
            ActionId::CpuWeightRoot => "cpu_weight_root",
            ActionId::CpuWeightUser => "cpu_weight_user",
            ActionId::WarmCiServices => "warm_ci_services",
            ActionId::TerminateOrphan => "terminate_orphan",
        }
    }

    /// False for a reserved id.
    pub fn is_catalogued(self) -> bool {
        !matches!(self, ActionId::TerminateOrphan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_wire_words_are_frozen() {
        let words: Vec<String> = WorkloadGroup::ALL
            .iter()
            .map(|g| {
                serde_json::to_value(g)
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string()
            })
            .collect();
        assert_eq!(words, ["agents", "ci", "system", "runner", "other"]);
        for g in WorkloadGroup::ALL {
            assert_eq!(serde_json::to_value(g).unwrap(), g.as_str());
        }
    }

    #[test]
    fn action_wire_words_are_frozen() {
        let words: Vec<&str> = ActionId::ALL.iter().map(|a| a.as_str()).collect();
        assert_eq!(
            words,
            [
                "compile_cache_agent",
                "compile_cache_ci",
                "build_volume",
                "cpu_weight_root",
                "cpu_weight_user",
                "warm_ci_services",
                "terminate_orphan",
            ]
        );
        for a in ActionId::ALL {
            assert_eq!(serde_json::to_value(a).unwrap(), a.as_str());
        }
    }

    #[test]
    fn terminate_orphan_is_reserved_not_catalogued() {
        assert!(!ActionId::TerminateOrphan.is_catalogued());
        assert!(!ActionId::CATALOGUED.contains(&ActionId::TerminateOrphan));
        assert!(ActionId::CATALOGUED.iter().all(|a| a.is_catalogued()));
    }

    #[test]
    fn ci_slice_and_children_are_ci() {
        assert_eq!(classify_cgroup_path("/ci.slice"), WorkloadGroup::Ci);
        assert_eq!(
            classify_cgroup_path("ci.slice/ci-runners.slice/actions.runner.x-7.service"),
            WorkloadGroup::Ci
        );
    }

    #[test]
    fn user_slice_session_scopes_are_agents() {
        let c = classify_cgroup(
            "0::/user.slice/user-1000.slice/user@1000.service/tmux-spawn-0a1b.scope",
        );
        assert_eq!(
            c,
            CgroupClass {
                group: WorkloadGroup::Agents,
                mixed: false
            }
        );
        assert_eq!(classify_cgroup_path("/user.slice"), WorkloadGroup::Agents);
        assert_eq!(
            classify_cgroup_path("/user.slice/user-1000.slice/session-3.scope"),
            WorkloadGroup::Agents
        );
    }

    #[test]
    fn the_runner_service_cgroup_is_mixed_agents() {
        let c = classify_cgroup(
            "0::/user.slice/user-1000.slice/user@1000.service/app.slice/qontinui-runner.service",
        );
        assert_eq!(
            c,
            CgroupClass {
                group: WorkloadGroup::Agents,
                mixed: true
            }
        );
        // Below it too.
        assert!(
            classify_cgroup(
                "/user.slice/user-42.slice/user@42.service/app.slice/qontinui-runner.service/x"
            )
            .mixed
        );
        // Another app.slice unit is plain agents.
        assert!(
            !classify_cgroup("/user.slice/user-42.slice/user@42.service/app.slice/other.service")
                .mixed
        );
        // `user@.service` without a uid is not a user manager.
        assert!(
            !classify_cgroup("/user.slice/user@.service/app.slice/qontinui-runner.service").mixed
        );
    }

    #[test]
    fn system_and_other() {
        assert_eq!(
            classify_cgroup_path("/system.slice/docker-ab.scope"),
            WorkloadGroup::System
        );
        assert_eq!(classify_cgroup_path("/init.scope"), WorkloadGroup::System);
        assert_eq!(classify_cgroup_path("/"), WorkloadGroup::Other);
        assert_eq!(classify_cgroup_path(""), WorkloadGroup::Other);
        assert_eq!(
            classify_cgroup_path("/machine.slice/vm.scope"),
            WorkloadGroup::Other
        );
        // A prefix match on the NAME is not a match on the component.
        assert_eq!(classify_cgroup_path("/ci.slicex"), WorkloadGroup::Other);
    }

    #[test]
    fn mixed_follows_the_runner_pids_actual_cgroup() {
        let rc = "0::/user.slice/user-7.slice/user@7.service/app.slice/qontinui-runner.service";
        // The runner's cgroup and every ancestor are mixed.
        for p in [
            "/user.slice",
            "/user.slice/user-7.slice",
            "/user.slice/user-7.slice/user@7.service",
            "/user.slice/user-7.slice/user@7.service/app.slice/qontinui-runner.service",
            "/",
        ] {
            assert!(classify_cgroup_with(p, Some(rc)).mixed, "{p}");
        }
        // Siblings and descendants are not.
        for p in [
            "/ci.slice",
            "/system.slice",
            "/user.slice/user-7.slice/user@7.service/tmux-spawn-a.scope",
            "/user.slice/user-7.slice/user@7.service/app.slice/qontinui-runner.service/x",
            "/user.slice/user-7.slice/user@7.servicex",
        ] {
            assert!(!classify_cgroup_with(p, Some(rc)).mixed, "{p}");
        }
        // A runner run as a system service makes system.slice mixed instead.
        let sys = Some("/system.slice/qontinui-runner.service");
        assert!(classify_cgroup_with("/system.slice", sys).mixed);
        assert!(!classify_cgroup_with("/user.slice", sys).mixed);
        // Without the runner cgroup, /user.slice falls back to the unit rule.
        assert!(!classify_cgroup("/user.slice").mixed);
    }

    #[test]
    fn runner_is_never_a_cgroup_classification() {
        for p in [
            "/user.slice/user-1000.slice/user@1000.service/app.slice/qontinui-runner.service",
            "/system.slice/qontinui-runner.service",
        ] {
            assert_ne!(classify_cgroup_path(p), WorkloadGroup::Runner);
        }
    }
}
