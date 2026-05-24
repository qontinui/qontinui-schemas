//! Coord-mediated GitOp federation wire-format types.
//!
//! Promoted from inline definitions in `qontinui-coord/src/git_ops.rs`
//! (plan `2026-05-24-federation-verify-and-gitop.md`, Phase 5) so all
//! consumers of the git-ops HTTP surface — coord (handlers), runner
//! (`GitOpBridge`), qontinui-web backend (proxy), qontinui-web frontend
//! (dashboard browser) — share a single source of truth.
//!
//! GitOp federation is the **post-action observational feed** of real
//! git operations (commits, checkouts, branch creates, pushes, …) that a
//! runner-spawned session performs on its local working tree. It is the
//! exact mirror of the shipped `coord.memories` substrate (see
//! `memory.rs`) — same tenant-scoping, same wire conventions.
//!
//! Do not confuse this with the "git-ops orchestration" surface
//! (coord's `session_view`/`agent_worktrees`), which is the *pre-action*
//! claims-derived allocation/touch-set view. This feed records what
//! actually happened after the fact; that surface records intent + locks.
//!
//! ## Convention
//!
//! Per the crate-level convention (`lib.rs`):
//!  - UUIDs are `String` on the wire (coord-side handlers parse with
//!    `uuid::Uuid::parse_str` when echoing back to PG).
//!  - Timestamps are ISO 8601 `String` (`recorded_at` is rendered with
//!    `DateTime::<Utc>::to_rfc3339`).
//!  - Field names stay snake_case — the wire shape (`op_id`, `op_kind`,
//!    `recorded_at`, …) is what the `/coord/git-ops/*` routes speak.
//!  - Optional fields use `#[serde(default, skip_serializing_if =
//!    "Option::is_none")]` so absence and `null` round-trip faithfully.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Requests
// ---------------------------------------------------------------------------

/// `POST /coord/git-ops/record` body. `repo` + `op_kind` are the only
/// required fields; the rest is optional metadata.
///
/// `device_id` / `session_id` are NOT in this body — coord resolves the
/// tenant from the `X-Qontinui-Tenant-Id` header and reads device/session
/// from the request wrapper (mirroring the federation-report surface).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RecordGitOpRequest {
    /// Repo basename (origin remote URL basename, falling back to the
    /// working-dir basename for remote-less clones).
    pub repo: String,
    /// Affected branch (null for stash/tag ops).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    /// One of: `commit | push | checkout | branch_create | merge |
    /// rebase | reset | stash | tag` (free-form on the wire).
    pub op_kind: String,
    /// Resulting commit SHA (null for branch_create/stash).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha: Option<String>,
    /// Commit message or op description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Extensible metadata (files_changed, remote, ahead_count, …).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Read shapes
// ---------------------------------------------------------------------------

/// One row in `coord.git_ops`. Returned by `GET /coord/git-ops/:op_id`,
/// `GET /coord/git-ops/list`, and `GET /coord/git-ops/by-session/:id`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GitOpRecord {
    pub op_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    pub device_id: String,
    pub session_id: String,
    pub repo: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    pub op_kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// ISO 8601 timestamp (RFC 3339).
    pub recorded_at: String,
    pub metadata: serde_json::Value,
}

/// `GET /coord/git-ops/list` + `GET /coord/git-ops/by-session/:id`
/// response envelope. Wrapped in a struct (rather than a bare
/// `Vec<GitOpRecord>`) so the generated TS / Python schemas get a named
/// exported interface rather than an anonymous array alias.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GitOpListResponse {
    pub items: Vec<GitOpRecord>,
}

/// One entry in the `GET /coord/git-ops/branches` response — the latest
/// branch a given device is on for a given repo, derived from the most
/// recent `checkout`/`branch_create` op per `(device_id, repo)`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DeviceBranchSummary {
    pub device_id: String,
    pub repo: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha: Option<String>,
    /// ISO 8601 timestamp (RFC 3339) of the op this summary was derived from.
    pub recorded_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_request_minimal_deserializes() {
        let raw = serde_json::json!({
            "repo": "qontinui-runner",
            "op_kind": "commit",
        });
        let req: RecordGitOpRequest = serde_json::from_value(raw).unwrap();
        assert_eq!(req.repo, "qontinui-runner");
        assert_eq!(req.op_kind, "commit");
        assert!(req.branch.is_none());
        assert!(req.sha.is_none());
        assert!(req.message.is_none());
        assert!(req.metadata.is_none());
    }

    #[test]
    fn record_request_full_deserializes() {
        let raw = serde_json::json!({
            "repo": "qontinui-runner",
            "branch": "main",
            "op_kind": "commit",
            "sha": "deadbeef",
            "message": "feat: x",
            "metadata": {"files_changed": 3},
        });
        let req: RecordGitOpRequest = serde_json::from_value(raw).unwrap();
        assert_eq!(req.branch.as_deref(), Some("main"));
        assert_eq!(req.sha.as_deref(), Some("deadbeef"));
        assert_eq!(req.message.as_deref(), Some("feat: x"));
        assert_eq!(req.metadata.unwrap()["files_changed"], 3);
    }

    #[test]
    fn record_request_skips_none_on_serialize() {
        let req = RecordGitOpRequest {
            repo: "r".into(),
            branch: None,
            op_kind: "commit".into(),
            sha: None,
            message: None,
            metadata: None,
        };
        let s = serde_json::to_string(&req).unwrap();
        assert!(!s.contains("branch"));
        assert!(!s.contains("sha"));
        assert!(!s.contains("message"));
        assert!(!s.contains("metadata"));
    }

    #[test]
    fn record_serializes_with_named_fields() {
        let rec = GitOpRecord {
            op_id: "11111111-1111-1111-1111-111111111111".into(),
            tenant_id: None,
            device_id: "22222222-2222-2222-2222-222222222222".into(),
            session_id: "33333333-3333-3333-3333-333333333333".into(),
            repo: "qontinui-runner".into(),
            branch: Some("main".into()),
            op_kind: "commit".into(),
            sha: Some("deadbeef".into()),
            message: Some("feat: x".into()),
            recorded_at: "2026-05-24T00:00:00+00:00".into(),
            metadata: serde_json::json!({}),
        };
        let s = serde_json::to_value(&rec).unwrap();
        assert_eq!(s["repo"], "qontinui-runner");
        assert_eq!(s["op_kind"], "commit");
        // tenant_id is None → skipped on serialize.
        assert!(s.get("tenant_id").is_none());
    }

    #[test]
    fn list_response_serialized_as_named_envelope() {
        let resp = GitOpListResponse { items: vec![] };
        let s = serde_json::to_string(&resp).unwrap();
        assert!(s.contains("\"items\""));
    }

    #[test]
    fn device_branch_summary_round_trips() {
        let summary = DeviceBranchSummary {
            device_id: "22222222-2222-2222-2222-222222222222".into(),
            repo: "qontinui-runner".into(),
            branch: Some("main".into()),
            sha: Some("deadbeef".into()),
            recorded_at: "2026-05-24T00:00:00+00:00".into(),
        };
        let s = serde_json::to_value(&summary).unwrap();
        let back: DeviceBranchSummary = serde_json::from_value(s).unwrap();
        assert_eq!(back.branch.as_deref(), Some("main"));
    }
}
