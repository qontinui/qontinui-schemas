//! The host a plain shell provides: what `qontinui-ci run` uses when there is
//! no runner, no host agent and no coord (plan D5 — CI keeps running when
//! coord is down, on a laptop, in a pre-push hook, in any shell).
//!
//! Each implementation is the honest answer for a one-shot process with no
//! enrolment and no long-lived state:
//!
//! - a GitHub token from `GITHUB_TOKEN` / `GH_TOKEN` or `gh auth token`, and no
//!   ETag cache or request-budget meter (nothing outlives the process);
//! - plain process spawning with the prompt-proof git environment, a real
//!   child-tree reaper on Unix (process group + `killpg`), and no per-dispatch
//!   containment — see [`PlainSpawn`];
//! - no declared removable volumes;
//! - no canonical-configuration source, so a manifest that declares
//!   `[canonical]` is REFUSED here rather than run unchecked.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use crate::host::{
    BoxFuture, CanonicalConvergence, CanonicalReading, CiSettings, ConvergeReport, GithubAccess,
    Host, HostIdentity, ProcessSpawn, StepContainment, TreeGuard, VolumeState,
};

/// How long `gh auth token` may take before the run proceeds without a token.
/// `gh` has been observed blocked for hours on a wedged keyring; a sibling probe
/// waiting on it forever is worse than an unauthenticated one.
const GH_TOKEN_TIMEOUT: Duration = Duration::from_secs(10);

/// No consent has been granted: a standalone run never converges a toolchain.
pub struct StandaloneSettings;

impl CiSettings for StandaloneSettings {
    fn canonical_converge(&self) -> bool {
        false
    }
}

/// `GITHUB_TOKEN`, then `GH_TOKEN`, then `gh auth token`; no cache, no meter.
pub struct EnvGithub;

impl GithubAccess for EnvGithub {
    fn token(&self) -> BoxFuture<'_, Option<String>> {
        Box::pin(async {
            for key in ["GITHUB_TOKEN", "GH_TOKEN"] {
                if let Ok(t) = std::env::var(key) {
                    let t = t.trim().to_string();
                    if !t.is_empty() {
                        return Some(t);
                    }
                }
            }
            let mut cmd = tokio::process::Command::new("gh");
            cmd.args(["auth", "token"])
                .stdin(std::process::Stdio::null())
                .kill_on_drop(true);
            let out = tokio::time::timeout(GH_TOKEN_TIMEOUT, cmd.output())
                .await
                .ok()?
                .ok()?;
            if !out.status.success() {
                return None;
            }
            let t = String::from_utf8_lossy(&out.stdout).trim().to_string();
            (!t.is_empty()).then_some(t)
        })
    }
    fn cached_etag(&self, _url: &str) -> Option<String> {
        None
    }
    fn replay(&self, _url: &str) -> Option<Vec<u8>> {
        None
    }
    fn store(&self, _url: &str, _etag: &str, _body: &[u8]) {}
    fn invalidate(&self, _url: &str) {}
    fn record_response(&self, _url: &str, _status: u16, _headers: &reqwest::header::HeaderMap) {}
    fn record_transport_error(&self, _url: &str) {}
}

/// The prompt-proof git environment: every layer that can make a `git`
/// child block on a credential prompt is closed, so a sibling fetch in a
/// non-interactive shell fails fast instead of hanging. The same keys and
/// values the Qontinui runner applies to every `git` it spawns (its
/// `git_posture::prompt_proof_git_env`): Git Credential Manager's UI
/// (`GCM_INTERACTIVE`), git's own terminal prompt (`GIT_TERMINAL_PROMPT`), and
/// the askpass chain (`GIT_ASKPASS`, pointed at a path that does not exist —
/// never empty, which a mingw git may read as unset).
pub const PROMPT_PROOF_GIT_ENV: &[(&str, &str)] = &[
    ("GCM_INTERACTIVE", "never"),
    ("GIT_TERMINAL_PROMPT", "0"),
    ("GIT_ASKPASS", "/qontinui-runner/askpass-disabled"),
];

/// Is `program` git? Matched on the final path component, case-insensitively
/// and with an optional `.exe`, splitting on BOTH separators so a Windows git
/// path is recognised wherever this runs.
fn program_is_git(program: &str) -> bool {
    let base = program.rsplit(['/', '\\']).next().unwrap_or(program);
    let lower = base.to_ascii_lowercase();
    lower == "git" || lower == "git.exe"
}

/// Plain spawning with the prompt-proof git posture.
///
/// The child-TREE reaper is real on Unix: [`ProcessSpawn::arm_tree`] makes the
/// child the leader of its own process group, and dropping the guard
/// `killpg`s that group, so a timed-out or cancelled `git fetch` takes its
/// `git-remote-https` / `index-pack` helpers down with it. On Windows only the
/// direct child is killed (`kill_on_drop`); the helpers die when they notice
/// their parent is gone. There is no per-dispatch step containment on either
/// OS. An armed child leads its OWN process group, so it does not die with
/// the terminal: `qontinui-ci` turns SIGINT, SIGTERM and SIGHUP into a
/// cancellation so these guards run (a SIGKILL of the CLI still orphans an
/// in-flight git group).
pub struct PlainSpawn;

/// Kills the child's process group on drop unless disarmed (Unix).
struct GroupGuard {
    #[cfg(unix)]
    pgid: Option<i32>,
}

impl TreeGuard for GroupGuard {
    fn disarm(self: Box<Self>) {
        #[cfg(unix)]
        {
            let mut guard = self;
            guard.pgid = None;
        }
    }
}

impl Drop for GroupGuard {
    fn drop(&mut self) {
        #[cfg(unix)]
        if let Some(pgid) = self.pgid.take() {
            // SAFETY: plain syscall on a pid we created as a group leader;
            // ESRCH (already gone) is the outcome we wanted anyway. It assumes
            // the group has not been reaped and its id reused — which is why
            // every caller disarms as soon as its wait returns, and only a
            // future dropped mid-wait (whose child is still unreaped) fires
            // this.
            unsafe { libc::killpg(pgid, libc::SIGKILL) };
        }
    }
}

struct NoContainment;
impl StepContainment for NoContainment {
    fn adopt(&self, _child: &tokio::process::Child) {}
}

impl ProcessSpawn for PlainSpawn {
    fn command(&self, program: &str) -> tokio::process::Command {
        let mut cmd = tokio::process::Command::new(program);
        if program_is_git(program) {
            cmd.envs(PROMPT_PROOF_GIT_ENV.iter().copied());
        }
        cmd
    }
    fn std_command(&self, program: &str) -> std::process::Command {
        let mut cmd = std::process::Command::new(program);
        if program_is_git(program) {
            cmd.envs(PROMPT_PROOF_GIT_ENV.iter().copied());
        }
        cmd
    }
    #[cfg_attr(not(unix), allow(unused_variables))]
    fn arm_tree(&self, cmd: &mut tokio::process::Command) {
        #[cfg(unix)]
        cmd.process_group(0);
    }
    #[cfg_attr(not(unix), allow(unused_variables))]
    fn attach_tree(&self, child: &tokio::process::Child) -> Box<dyn TreeGuard> {
        Box::new(GroupGuard {
            // The armed child IS its group leader, so its pid is the pgid. A
            // child already reaped (`id()` is `None`) attaches nothing, and
            // pid 0/1 are never signalled.
            #[cfg(unix)]
            pgid: child
                .id()
                .and_then(|pid| i32::try_from(pid).ok())
                .filter(|pgid| *pgid > 1),
        })
    }
    fn step_containment(&self) -> Box<dyn StepContainment> {
        Box::new(NoContainment)
    }
}

/// No removable volumes are declared on a standalone host.
pub struct NoVolumes;

impl VolumeState for NoVolumes {
    fn refusal_reason(&self, _root: &Path) -> Option<String> {
        None
    }
}

/// A fixed CI root and label, chosen by the caller.
pub struct FixedIdentity {
    pub root: PathBuf,
    pub label: String,
}

impl HostIdentity for FixedIdentity {
    fn ci_root(&self) -> Result<PathBuf, String> {
        Ok(self.root.clone())
    }
    fn label(&self) -> String {
        self.label.clone()
    }
}

/// No canonical-configuration source. Measuring fails, so the gate refuses.
pub struct NoCanonical;

const NO_CANONICAL: &str = "this host has no canonical-configuration source (canonical \
     measurement and convergence are a Qontinui runner's env agent); run this job on an \
     enrolled runner, or remove [canonical] from the manifest";

impl CanonicalConvergence for NoCanonical {
    fn measure<'a>(
        &'a self,
        _keys: &'a [String],
    ) -> BoxFuture<'a, Result<CanonicalReading, String>> {
        Box::pin(async { Err(NO_CANONICAL.to_string()) })
    }
    fn converge<'a>(
        &'a self,
        _keys: &'a [String],
    ) -> BoxFuture<'a, Result<ConvergeReport, String>> {
        Box::pin(async { Err(NO_CANONICAL.to_string()) })
    }
}

/// A complete standalone host rooted at `root`.
pub fn host(root: PathBuf, label: String) -> Host {
    Host {
        settings: Arc::new(StandaloneSettings),
        github: Arc::new(EnvGithub),
        process: Arc::new(PlainSpawn),
        volume: Arc::new(NoVolumes),
        identity: Arc::new(FixedIdentity { root, label }),
        canonical: Arc::new(NoCanonical),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The standalone host never authorises convergence and has no canonical
    /// source — so a `[canonical]` manifest is refused here, not run unchecked.
    #[tokio::test]
    async fn a_standalone_host_cannot_satisfy_canonical() {
        let h = host(std::env::temp_dir(), "test".to_string());
        assert!(!h.settings.canonical_converge());
        let err = h
            .canonical
            .measure(&["rustc".to_string()])
            .await
            .unwrap_err();
        assert!(err.contains("no canonical-configuration source"), "{err}");
        let declared = crate::manifest::CiCanonical {
            toolchains: vec!["rustc".to_string()],
        };
        let mut log = |_l: String| {};
        let outcome =
            crate::canonical::ensure(Some(&declared), false, h.canonical.as_ref(), &mut log).await;
        assert!(outcome.is_refusal());
    }

    #[test]
    fn a_standalone_host_declares_no_removable_volume_and_its_given_root() {
        let root = std::env::temp_dir().join("ci-root");
        let h = host(root.clone(), "box".to_string());
        assert!(h.volume.refusal_reason(&root).is_none());
        assert_eq!(h.identity.ci_root().unwrap(), root);
        assert_eq!(h.identity.label(), "box");
    }

    #[test]
    fn git_children_get_the_prompt_proof_env_and_others_do_not() {
        for git in ["git", "/usr/bin/git", r"C:\Program Files\Git\cmd\GIT.EXE"] {
            assert!(program_is_git(git), "{git}");
        }
        for other in ["git-lfs", "gitk", "cargo", "/opt/git/bin/node"] {
            assert!(!program_is_git(other), "{other}");
        }
        let cmd = PlainSpawn.std_command("git");
        let envs: Vec<(String, String)> = cmd
            .get_envs()
            .filter_map(|(k, v)| Some((k.to_str()?.to_string(), v?.to_str()?.to_string())))
            .collect();
        for (k, v) in PROMPT_PROOF_GIT_ENV {
            assert!(
                envs.contains(&(k.to_string(), v.to_string())),
                "{k} missing"
            );
        }
        assert_eq!(PlainSpawn.std_command("cargo").get_envs().count(), 0);
    }

    /// Dropping an armed guard kills the child's GRANDCHILDREN too — the
    /// property a cancelled `git fetch` relies on to release its helpers.
    #[cfg(unix)]
    #[tokio::test]
    async fn dropping_the_tree_guard_kills_the_whole_process_group() {
        use tokio::io::AsyncBufReadExt;
        let mut cmd = PlainSpawn.command("sh");
        cmd.args(["-c", "sleep 300 & echo $!; wait"])
            .stdout(std::process::Stdio::piped())
            .kill_on_drop(true);
        PlainSpawn.arm_tree(&mut cmd);
        let mut child = cmd.spawn().expect("sh spawns");
        let guard = PlainSpawn.attach_tree(&child);
        let mut lines = tokio::io::BufReader::new(child.stdout.take().unwrap()).lines();
        let grandchild: i32 = lines
            .next_line()
            .await
            .unwrap()
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        let alive = |pid: i32| unsafe { libc::kill(pid, 0) } == 0;
        assert!(alive(grandchild));
        drop(guard);
        let _ = child.wait().await;
        let mut gone = false;
        for _ in 0..100 {
            // A killed grandchild may linger briefly as a zombie until its new
            // parent reaps it; `ps` state Z counts as gone.
            let state = std::process::Command::new("ps")
                .args(["-o", "stat=", "-p", &grandchild.to_string()])
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_default();
            if state.is_empty() || state.starts_with('Z') {
                gone = true;
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        assert!(gone, "the grandchild {grandchild} survived the group kill");
    }

    /// A disarmed guard leaves the group alone.
    #[cfg(unix)]
    #[tokio::test]
    async fn a_disarmed_tree_guard_kills_nothing() {
        let mut cmd = PlainSpawn.command("sleep");
        cmd.arg("300").kill_on_drop(true);
        PlainSpawn.arm_tree(&mut cmd);
        let mut child = cmd.spawn().expect("sleep spawns");
        PlainSpawn.attach_tree(&child).disarm();
        assert!(child.try_wait().unwrap().is_none(), "disarm must not kill");
        let _ = child.kill().await;
    }
}
