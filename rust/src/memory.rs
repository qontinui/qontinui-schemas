//! Coord-mediated memory wire-format types.
//!
//! Promoted from inline definitions in `qontinui-coord/src/memories.rs`
//! (plan `2026-05-22-memories-on-coord-cross-machine.md`, Phase 6) so
//! all consumers of the memory HTTP surface — coord (handlers), runner
//! (memory bridge), qontinui-web backend (proxy), qontinui-web frontend
//! (dashboard browser) — share a single source of truth.
//!
//! ## Convention
//!
//! Per the crate-level convention (`lib.rs`):
//!  - UUIDs are `String` on the wire (coord-side handlers parse with
//!    `uuid::Uuid::parse_str` when echoing back to PG).
//!  - Timestamps are ISO 8601 `String` (`written_at` is rendered with
//!    `DateTime::<Utc>::to_rfc3339`).
//!  - Field names stay snake_case (no `rename_all = "camelCase"`):
//!    the existing wire shape (`memory_id`, `written_at`,
//!    `written_by_agent`, etc.) is what the dashboard, backend proxy,
//!    and `/coord/memory/*` HTTP routes already speak. Renaming would
//!    fork the wire for no gain.
//!  - Optional fields use `#[serde(default, skip_serializing_if =
//!    "Option::is_none")]` so absence and `null` round-trip faithfully.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Requests
// ---------------------------------------------------------------------------

/// `POST /coord/memory/upsert` body. `name` + `content` are the only
/// required fields; the rest is optional metadata.
///
/// UUID fields (`written_by_agent`, `written_by_device`) are `String` on
/// the wire; coord parses them with `Uuid::parse_str` and returns 400
/// on parse failure.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MemoryUpsertRequest {
    pub name: String,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub written_by_agent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub written_by_device: Option<String>,
}

/// `POST /coord/memory/upsert` response.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MemoryUpsertResponse {
    pub memory_id: String,
    pub name: String,
    pub version: i64,
}

/// `POST /coord/memory/:name/restore` body.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MemoryRestoreRequest {
    pub version: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub written_by_agent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub written_by_device: Option<String>,
}

/// `GET /coord/memory/list` query string.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct MemoryListQuery {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name_prefix: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
}

// ---------------------------------------------------------------------------
// Read shapes
// ---------------------------------------------------------------------------

/// One row in `coord.memories`. Returned by `GET /coord/memory/:name`
/// (as the `latest` field of `MemoryWithHistory` + each `history` entry)
/// and by `GET /coord/memory/:name/version/:version`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MemoryRow {
    pub memory_id: String,
    pub name: String,
    pub version: i64,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub written_by_agent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub written_by_device: Option<String>,
    /// ISO 8601 timestamp (RFC 3339).
    pub written_at: String,
    pub is_tombstone: bool,
}

/// Lighter-weight projection for `/coord/memory/list` — strips the
/// (potentially-large) `content` blob so a list-all-memories scan stays
/// cheap. Fetch full payload via `GET /coord/memory/:name`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MemorySummary {
    pub name: String,
    pub version: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    /// ISO 8601 timestamp (RFC 3339).
    pub written_at: String,
}

/// `GET /coord/memory/:name` response — latest row + the 10
/// most-recent versions (DESC). The first entry of `history` is the
/// same row as `latest`; subsequent entries are prior versions
/// (including tombstones, so the dashboard can render "deleted at v3
/// then restored at v4").
///
/// `latest` is flattened on the wire — the response object's top-level
/// fields are the `MemoryRow` columns plus a `history` array.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MemoryWithHistory {
    #[serde(flatten)]
    pub latest: MemoryRow,
    pub history: Vec<MemoryRow>,
}

/// `GET /coord/memory/list` response envelope. Wrapped in a struct
/// (rather than a bare `Vec<MemorySummary>`) so the generated TS / Python
/// schemas get a named exported interface rather than an anonymous array
/// alias.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MemoryListResponse {
    pub items: Vec<MemorySummary>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upsert_request_minimal_deserializes() {
        let raw = serde_json::json!({
            "name": "feedback_demo",
            "content": "body",
        });
        let req: MemoryUpsertRequest = serde_json::from_value(raw).unwrap();
        assert_eq!(req.name, "feedback_demo");
        assert_eq!(req.content, "body");
        assert!(req.description.is_none());
        assert!(req.r#type.is_none());
        assert!(req.written_by_agent.is_none());
        assert!(req.written_by_device.is_none());
    }

    #[test]
    fn upsert_request_full_deserializes() {
        let raw = serde_json::json!({
            "name": "feedback_demo",
            "content": "body",
            "description": "what it does",
            "type": "feedback",
            "written_by_agent": "11111111-1111-1111-1111-111111111111",
            "written_by_device": "22222222-2222-2222-2222-222222222222",
        });
        let req: MemoryUpsertRequest = serde_json::from_value(raw).unwrap();
        assert_eq!(req.description.as_deref(), Some("what it does"));
        assert_eq!(req.r#type.as_deref(), Some("feedback"));
        assert_eq!(
            req.written_by_agent.as_deref(),
            Some("11111111-1111-1111-1111-111111111111")
        );
        assert_eq!(
            req.written_by_device.as_deref(),
            Some("22222222-2222-2222-2222-222222222222")
        );
    }

    #[test]
    fn upsert_request_skips_none_on_serialize() {
        let req = MemoryUpsertRequest {
            name: "n".into(),
            content: "c".into(),
            description: None,
            r#type: None,
            written_by_agent: None,
            written_by_device: None,
        };
        let s = serde_json::to_string(&req).unwrap();
        assert!(!s.contains("description"));
        assert!(!s.contains("type"));
        assert!(!s.contains("written_by_agent"));
        assert!(!s.contains("written_by_device"));
    }

    #[test]
    fn list_query_defaults() {
        let q: MemoryListQuery = serde_json::from_value(serde_json::json!({})).unwrap();
        assert!(q.r#type.is_none());
        assert!(q.name_prefix.is_none());
        assert!(q.limit.is_none());
    }

    #[test]
    fn list_query_full_deserializes() {
        let q: MemoryListQuery = serde_json::from_value(serde_json::json!({
            "type": "feedback",
            "name_prefix": "feedback_",
            "limit": 50,
        }))
        .unwrap();
        assert_eq!(q.r#type.as_deref(), Some("feedback"));
        assert_eq!(q.name_prefix.as_deref(), Some("feedback_"));
        assert_eq!(q.limit, Some(50));
    }

    #[test]
    fn restore_request_deserializes() {
        let raw = serde_json::json!({"version": 3});
        let req: MemoryRestoreRequest = serde_json::from_value(raw).unwrap();
        assert_eq!(req.version, 3);
        assert!(req.written_by_agent.is_none());
    }

    #[test]
    fn memory_with_history_flattens_latest() {
        let latest = MemoryRow {
            memory_id: "11111111-1111-1111-1111-111111111111".into(),
            name: "n".into(),
            version: 2,
            content: "c".into(),
            description: None,
            r#type: None,
            written_by_agent: None,
            written_by_device: None,
            written_at: "2026-05-22T00:00:00+00:00".into(),
            is_tombstone: false,
        };
        let payload = MemoryWithHistory {
            latest: latest.clone(),
            history: vec![latest],
        };
        let s = serde_json::to_value(&payload).unwrap();
        // `latest`'s fields should be at the top level alongside `history`.
        assert_eq!(s["name"], "n");
        assert_eq!(s["version"], 2);
        assert!(s["history"].is_array());
    }

    #[test]
    fn list_response_serialized_as_named_envelope() {
        let resp = MemoryListResponse {
            items: vec![MemorySummary {
                name: "n".into(),
                version: 1,
                description: None,
                r#type: None,
                written_at: "2026-05-22T00:00:00+00:00".into(),
            }],
        };
        let s = serde_json::to_string(&resp).unwrap();
        assert!(s.contains("\"items\""));
        assert!(s.contains("\"name\":\"n\""));
    }
}
