//! The dispatch protocol every CI host speaks: the payload coord sends, the
//! identifier gates that run before any of it touches disk or a URL, and the
//! [`Admission`] seam a host puts in front of the executor.
//!
//! Coord publishes two events per device:
//!
//! - `events.ci.build_requested.<device_id>` — a [`DispatchPayload`];
//! - `events.ci.build_cancelled.<device_id>` — a [`CancelPayload`].
//!
//! How a host receives them (the runner's `/ws` subscription, the host agent's
//! slot loop) is the host's business; what they mean is this module's.

use serde::Deserialize;
use tracing::warn;

/// Default manifest path when the dispatch omits it.
fn default_manifest_path() -> String {
    ".qontinui/ci.toml".to_string()
}

/// The job a dispatch runs when it names none. A v1 manifest reads as exactly
/// one job of this name, so every dispatch coord sends today (it does not name
/// a job yet) runs the whole v1 step list, as before.
pub const DEFAULT_JOB: &str = "ci";

fn default_job() -> String {
    DEFAULT_JOB.to_string()
}

/// A `events.ci.build_requested.<device_id>` dispatch payload. Unknown fields
/// are tolerated (coord may grow the shape); `manifest_path`, `job` and
/// `coord_http_url` degrade to sane defaults so a slightly-lean payload is
/// still runnable/reportable.
#[derive(Debug, Clone, Deserialize)]
pub struct DispatchPayload {
    pub dispatch_id: String,
    /// Coord repo slug (`owner/name`) or bare repo name.
    pub repo: String,
    pub head_sha: String,
    /// Resolved fetch URL (for coord-origin repos this is the git door /
    /// mirror — a branch pushed only to GitHub is invisible there).
    pub fetch_url: String,
    /// `refs/heads/merge-candidate/<proposal_id>` or `refs/ci-dispatch/…`.
    pub candidate_ref: String,
    /// Pull request this dispatch is validating, when there is one.
    ///
    /// This is the ONLY key the sibling declaration rule turns on
    /// ([`crate::sibling::resolve_declaration`]), and its absence is a real
    /// answer, not a gap to paper over: coord pushes the SAME
    /// `refs/heads/merge-candidate/<proposal_id>` into every repo of a
    /// multi-repo proposal, so a dispatch with no pull request resolves
    /// siblings to their branch WITHOUT a declaration probe — exactly what the
    /// Actions composite action does off a `pull_request` event, and for the
    /// same reason (the candidate ref is force-pushed and deleted as the
    /// proposal resolves).
    #[serde(default)]
    pub pr_number: Option<u64>,
    #[serde(default = "default_manifest_path")]
    pub manifest_path: String,
    /// The manifest job to run (`[[jobs]].name`). Defaults to [`DEFAULT_JOB`].
    #[serde(default = "default_job")]
    pub job: String,
    /// Check-run context coord will publish the verdict under. The executor
    /// only logs it (the verdict write is coord-side, keyed by dispatch_id).
    #[serde(default)]
    pub check_name: String,
    /// Coord HTTP base for the progress/result POSTs. Empty ⇒ the host falls
    /// back to its own configured coord base.
    #[serde(default)]
    pub coord_http_url: String,
}

/// A `events.ci.build_cancelled.<device_id>` payload.
#[derive(Debug, Clone, Deserialize)]
pub struct CancelPayload {
    pub dispatch_id: String,
}

/// A dispatch_id is used in filesystem paths (`.ci-worktrees/<id>`) and in
/// the progress/result URL path, so it must be a plain token. Coord mints
/// UUIDs; anything else is dropped before it can touch disk or a URL.
pub fn dispatch_id_is_safe(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// The repo slug is joined into local paths via its basename; require plain
/// `owner/name`-style tokens so a hostile slug can't traverse.
pub fn repo_slug_is_safe(repo: &str) -> bool {
    !repo.is_empty()
        && repo.len() <= 200
        && !repo.contains("..")
        && repo
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/'))
}

/// A host's gate between "coord sent a dispatch" and "the executor runs it".
///
/// Host-pressure admission — slot caps, memory headroom, disk floors, queueing
/// — is specific to the machine, so each host implements it; the runner's
/// lives in its `ci_node::admission`. Whatever an implementation admits it runs
/// through [`crate::executor::run_dispatch`], and whatever it rejects it
/// reports as `cancelled` with a reason (a rejection is a non-verdict about the
/// code).
pub trait Admission: Send + Sync {
    /// Accept, defer or reject one dispatch.
    fn submit(&self, payload: DispatchPayload);
    /// Cancel a running or queued dispatch. Unknown ids are a no-op.
    fn cancel(&self, dispatch_id: &str);
}

/// The two coord events this protocol carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchEvent {
    BuildRequested,
    BuildCancelled,
}

/// Parse one event body and hand it to `admission`. A body that does not parse
/// is logged and dropped — one malformed frame must never stop a host's event
/// loop.
pub fn route(admission: &dyn Admission, event: DispatchEvent, body: serde_json::Value) {
    match event {
        DispatchEvent::BuildRequested => match serde_json::from_value::<DispatchPayload>(body) {
            Ok(payload) => admission.submit(payload),
            Err(e) => warn!("ci: build_requested payload did not parse: {e}"),
        },
        DispatchEvent::BuildCancelled => match serde_json::from_value::<CancelPayload>(body) {
            Ok(cancel) => admission.cancel(&cancel.dispatch_id),
            Err(e) => warn!("ci: build_cancelled payload did not parse: {e}"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[test]
    fn dispatch_id_safety_gate() {
        assert!(dispatch_id_is_safe("0198f2b4-1111-7aaa-bbbb-cccccccccccc"));
        assert!(dispatch_id_is_safe("abc_DEF-123"));
        assert!(!dispatch_id_is_safe(""));
        assert!(!dispatch_id_is_safe("../../etc"));
        assert!(!dispatch_id_is_safe("a/b"));
        assert!(!dispatch_id_is_safe("a b"));
        assert!(!dispatch_id_is_safe(&"x".repeat(65)));
    }

    #[test]
    fn repo_slug_safety_gate() {
        assert!(repo_slug_is_safe("qontinui/qontinui-runner"));
        assert!(repo_slug_is_safe("qontinui-runner"));
        assert!(repo_slug_is_safe("owner/repo.name"));
        assert!(!repo_slug_is_safe(""));
        assert!(!repo_slug_is_safe("owner/../secret"));
        assert!(!repo_slug_is_safe("repo name"));
        assert!(!repo_slug_is_safe("repo\\name"));
    }

    /// The dispatch payload parses from the pinned wire contract, and the
    /// optional fields default when omitted — `job` to the one job a v1
    /// manifest has, so a coord that does not send it keeps working.
    #[test]
    fn dispatch_payload_parses_pinned_contract() {
        let full: DispatchPayload = serde_json::from_value(serde_json::json!({
            "dispatch_id": "0198f2b4-1111-7aaa-bbbb-cccccccccccc",
            "repo": "qontinui/qontinui-runner",
            "head_sha": "deadbeef",
            "fetch_url": "https://github.com/qontinui/qontinui-runner.git",
            "candidate_ref": "refs/heads/merge-candidate/42",
            "manifest_path": ".qontinui/ci.toml",
            "check_name": "qontinui-ci-node",
            "coord_http_url": "https://coord.qontinui.io",
        }))
        .expect("pinned contract must parse");
        assert_eq!(full.manifest_path, ".qontinui/ci.toml");
        assert_eq!(full.check_name, "qontinui-ci-node");
        assert_eq!(full.pr_number, None);
        assert_eq!(full.job, DEFAULT_JOB);

        // The sibling declaration key and a named job, when coord sends them.
        let with_pr: DispatchPayload = serde_json::from_value(serde_json::json!({
            "dispatch_id": "d1",
            "repo": "r",
            "head_sha": "s",
            "fetch_url": "u",
            "candidate_ref": "refs/heads/merge-candidate/1",
            "pr_number": 1008,
            "job": "holder-crates",
        }))
        .expect("pr_number and job must parse");
        assert_eq!(with_pr.pr_number, Some(1008));
        assert_eq!(with_pr.job, "holder-crates");

        let lean: DispatchPayload = serde_json::from_value(serde_json::json!({
            "dispatch_id": "d1",
            "repo": "r",
            "head_sha": "s",
            "fetch_url": "u",
            "candidate_ref": "refs/heads/merge-candidate/1",
        }))
        .expect("lean payload must parse with defaults");
        assert_eq!(lean.manifest_path, ".qontinui/ci.toml");
        assert!(lean.check_name.is_empty());
        assert!(lean.coord_http_url.is_empty());
    }

    #[derive(Default)]
    struct Seen {
        submitted: Mutex<Vec<String>>,
        cancelled: Mutex<Vec<String>>,
    }

    impl Admission for Seen {
        fn submit(&self, payload: DispatchPayload) {
            self.submitted.lock().unwrap().push(payload.dispatch_id);
        }
        fn cancel(&self, dispatch_id: &str) {
            self.cancelled.lock().unwrap().push(dispatch_id.to_string());
        }
    }

    /// Each event reaches its admission method, and a body that does not parse
    /// reaches neither.
    #[test]
    fn route_hands_each_event_to_its_admission_method() {
        let seen = Seen::default();
        route(
            &seen,
            DispatchEvent::BuildRequested,
            serde_json::json!({
                "dispatch_id": "d1", "repo": "r", "head_sha": "s",
                "fetch_url": "u", "candidate_ref": "refs/ci-dispatch/x",
            }),
        );
        route(
            &seen,
            DispatchEvent::BuildCancelled,
            serde_json::json!({ "dispatch_id": "d2" }),
        );
        route(
            &seen,
            DispatchEvent::BuildRequested,
            serde_json::json!({ "nope": 1 }),
        );
        route(
            &seen,
            DispatchEvent::BuildCancelled,
            serde_json::json!("garbage"),
        );
        assert_eq!(seen.submitted.lock().unwrap().as_slice(), ["d1"]);
        assert_eq!(seen.cancelled.lock().unwrap().as_slice(), ["d2"]);
    }
}
