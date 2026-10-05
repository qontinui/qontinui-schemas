//! cgroup path helpers that are not classification: the wire-safe redacted
//! SHAPE of a path, and whether a process's cgroup is a managed systemd
//! service.

use super::vocab::{cgroup_components, is_user_manager_unit, RUNNER_SERVICE_UNIT};

/// Reduce a cgroup path to a shape that carries no uid, hostname or session
/// name, for publishing. Rules per component:
///
/// - `user-<digits>.slice` → `user-N.slice`; `user@<digits>.service` →
///   `user@N.service`;
/// - any other `*.slice` is kept (slices are structural: `ci.slice`,
///   `app.slice`, `ci-runners.slice`);
/// - the runner's own unit is kept (a product name);
/// - any other `*.service` → `*.service` (CI runner units embed the host
///   name and repo);
/// - any `*.scope` (session `tmux-spawn-*` / `session-*`, container
///   `docker-*`) → `*.scope`, and the path ENDS there: how many scopes a
///   group holds is published as a count, never as names;
/// - anything else → `*`.
///
/// The root cgroup is `/`.
pub fn redact_cgroup_path(path: &str) -> String {
    let mut out: Vec<&str> = Vec::new();
    for c in cgroup_components(path) {
        if is_user_manager_unit(c) {
            out.push("user@N.service");
        } else if is_user_slice(c) {
            out.push("user-N.slice");
        } else if c.ends_with(".slice") || c == RUNNER_SERVICE_UNIT {
            out.push(c);
        } else if c.ends_with(".service") {
            out.push("*.service");
        } else if c.ends_with(".scope") {
            out.push("*.scope");
            break;
        } else {
            out.push("*");
        }
    }
    format!("/{}", out.join("/"))
}

/// True for a cgroup that is a systemd `*.service` unit (or below one), so a
/// process in it is MANAGED — stopping the unit reaps it — rather than
/// orphaned. Excluded: the user manager itself (`user@<uid>.service`, whose
/// children are session scopes), and the runner's cgroup (`runner_cgroup`, or
/// the runner unit name when unknown), because the runner hosts the sessions
/// it spawns and a leak there is exactly what the detector looks for.
///
/// A `*.scope` below the service (a session scope) is not managed.
pub fn is_managed_service_cgroup(path: &str, runner_cgroup: Option<&str>) -> bool {
    let parts = cgroup_components(path);
    if let Some(rc) = runner_cgroup {
        let rc = cgroup_components(rc);
        if parts.len() >= rc.len() && parts.iter().zip(&rc).all(|(a, b)| a == b) {
            return false;
        }
    }
    let mut managed = false;
    for c in &parts {
        if c.ends_with(".scope") {
            managed = false;
        } else if *c == RUNNER_SERVICE_UNIT {
            return false;
        } else if c.ends_with(".service") && !is_user_manager_unit(c) {
            managed = true;
        }
    }
    managed
}

fn is_user_slice(c: &str) -> bool {
    c.strip_prefix("user-")
        .and_then(|r| r.strip_suffix(".slice"))
        .is_some_and(|uid| !uid.is_empty() && uid.bytes().all(|b| b.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redaction_strips_uids_hosts_and_session_names() {
        assert_eq!(
            redact_cgroup_path(
                "0::/user.slice/user-1000.slice/user@1000.service/tmux-spawn-e0.scope"
            ),
            "/user.slice/user-N.slice/user@N.service/*.scope"
        );
        assert_eq!(
            redact_cgroup_path(
                "/user.slice/user-1000.slice/user@1000.service/app.slice/qontinui-runner.service"
            ),
            "/user.slice/user-N.slice/user@N.service/app.slice/qontinui-runner.service"
        );
        assert_eq!(
            redact_cgroup_path(
                "/ci.slice/ci-runners.slice/actions.runner.org-repo.somehost-7.service"
            ),
            "/ci.slice/ci-runners.slice/*.service"
        );
        assert_eq!(
            redact_cgroup_path("/system.slice/docker-abc.scope/x"),
            "/system.slice/*.scope"
        );
        assert_eq!(redact_cgroup_path("/"), "/");
        assert_eq!(
            redact_cgroup_path("/user.slice/user-.slice"),
            "/user.slice/user-.slice"
        );
    }

    #[test]
    fn managed_service_membership() {
        assert!(is_managed_service_cgroup(
            "/system.slice/cron.service",
            None
        ));
        assert!(is_managed_service_cgroup(
            "/user.slice/user-5.slice/user@5.service/app.slice/foo.service",
            None
        ));
        assert!(is_managed_service_cgroup(
            "/ci.slice/ci-runners.slice/actions.runner.x.service",
            None
        ));
        // Session scopes and the user manager itself are not managed.
        assert!(!is_managed_service_cgroup(
            "/user.slice/user-5.slice/user@5.service/tmux-spawn-a.scope",
            None
        ));
        assert!(!is_managed_service_cgroup(
            "/user.slice/user-5.slice/user@5.service",
            None
        ));
        assert!(!is_managed_service_cgroup(
            "/user.slice/user-5.slice/session-2.scope",
            None
        ));
        // A scope below a service is not managed.
        assert!(!is_managed_service_cgroup(
            "/system.slice/x.service/y.scope",
            None
        ));
        // The runner's cgroup never counts as managed — by name or by pid.
        let rc = "/user.slice/user-5.slice/user@5.service/app.slice/qontinui-runner.service";
        assert!(!is_managed_service_cgroup(rc, None));
        assert!(!is_managed_service_cgroup(
            "/system.slice/renamed.service",
            Some("/system.slice/renamed.service")
        ));
    }
}
