//! The host a plain shell provides: what `qontinui-ci run` uses when there is
//! no runner, no host agent and no coord (plan D5 — CI keeps running when
//! coord is down, on a laptop, in a pre-push hook, in any shell).
//!
//! Each implementation is the honest answer for a one-shot process with no
//! enrolment and no long-lived state:
//!
//! - a GitHub token from `GITHUB_TOKEN` / `GH_TOKEN` or `gh auth token`, and no
//!   ETag cache or request-budget meter (nothing outlives the process);
//! - plain process spawning, with no tree reaper and no per-dispatch
//!   containment — `kill_on_drop` still kills each direct child, and a CLI run
//!   ends with its terminal;
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

/// Plain `Command::new`, no tree reaper, no containment.
pub struct PlainSpawn;

struct NoTree;
impl TreeGuard for NoTree {
    fn disarm(self: Box<Self>) {}
}

struct NoContainment;
impl StepContainment for NoContainment {
    fn adopt(&self, _child: &tokio::process::Child) {}
}

impl ProcessSpawn for PlainSpawn {
    fn command(&self, program: &str) -> tokio::process::Command {
        tokio::process::Command::new(program)
    }
    fn std_command(&self, program: &str) -> std::process::Command {
        std::process::Command::new(program)
    }
    fn arm_tree(&self, _cmd: &mut tokio::process::Command) {}
    fn attach_tree(&self, _child: &tokio::process::Child) -> Box<dyn TreeGuard> {
        Box::new(NoTree)
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
}
