//! Runner DTO types.
//!
//! Canonical wire-format types for a registered runner. One row per registered
//! runner; the same shape is served to every consumer (mobile, web, runner UI).
//!
//! Replaces the prior split between `runners` (fleet/heartbeat),
//! `runner_connections` (transient WS sessions), and per-consumer projection
//! types like `RunnerConnection` / `ServerRunner` / `RegisteredRunner` /
//! `WebIntegrationStatus`.
//!
//! ## Conventions
//!
//! Per the crate-level docs (`lib.rs`), this module follows the wire-format
//! conventions used by every other type in this crate:
//!
//! - Timestamps are ISO 8601 `String`s, not `chrono::DateTime<Utc>`. The DTO
//!   layer is intentionally decoupled from any particular `chrono` version,
//!   and JSON Schema output stays clean.
//! - UUIDs are `String`s, not `uuid::Uuid`. Same rationale.
//! - Optional fields use `#[serde(default, skip_serializing_if = "Option::is_none")]`
//!   so absence and `null` round-trip faithfully.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ============================================================================
// Enums
// ============================================================================

/// Health/availability state of a runner, computed server-side from the
/// WebSocket-presence and heartbeat-freshness signals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum RunnerStatus {
    /// WS connected OR heartbeat fresh — fully reachable.
    Healthy,
    /// Heartbeat is stale but not yet cold; runner may still be reachable.
    Degraded,
    /// No WS connection and heartbeat is cold.
    Offline,
    /// Just registered; waiting for first heartbeat.
    Starting,
    /// Runner reported a fatal error (see `ui_error` / `recent_crash`).
    Errored,
}

/// Which role a runner instance plays on its machine.
///
/// Every runner instance on one box presents the SAME machine `device_id`, so
/// the role is what separates the canonical instance from the supervisor-spawned
/// ones sharing its row. Reported by the runner in `runner_info` as the
/// top-level `instanceRole`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum RunnerInstanceRole {
    /// The canonical instance — the one that owns machine-wide shared state
    /// (the default API port, `:9876`). The runner's predicate is "owns the
    /// shared machine root state", not "is the only runner", so a lone runner
    /// on a non-default port reads as `secondary`.
    Primary,
    /// A supervisor-spawned sibling (temp or named runner, `:9877-9899`).
    Secondary,
}

// ============================================================================
// Error / Crash Reports
// ============================================================================

/// Structured error reported by the runner that the UI surfaces verbatim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct RunnerUiError {
    /// Error category (e.g., `"build_failed"`, `"port_conflict"`).
    pub kind: String,
    /// Short, user-facing error message.
    pub message: String,
    /// Optional long-form detail (stack trace, stderr, etc.).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// ISO 8601 timestamp when the error was reported.
    pub reported_at: String,
}

/// Most recent crash dump metadata, if any.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct RunnerCrash {
    /// Path to the crash dump file on disk.
    pub file_path: String,
    /// Panic message captured from the runner process.
    pub panic_message: String,
    /// Source location (file/line) where the panic originated.
    pub panic_location: String,
    /// Name of the thread that panicked.
    pub thread: String,
    /// ISO 8601 timestamp when the crash was reported.
    pub reported_at: String,
}

// ============================================================================
// Runner instances
// ============================================================================

/// One live runner instance under a [`Runner`] (machine) row.
///
/// A machine can host several runner instances at once — the primary on
/// `:9876` and supervisor-spawned secondaries on `:9877-9899` — all presenting
/// the one machine `device_id`, so they share a single [`Runner`] row. One
/// entry per currently connected instance lets a consumer address `:9876` and
/// `:9877` on the same box separately rather than seeing only whichever
/// instance wrote the row's `port` last (plan
/// `2026-09-20-runner-selector-drives-a-transport-not-a-target`, reading A0.2
/// and Phase 6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct RunnerInstance {
    /// Per-instance key, reported by the runner as `instanceKey`. Built from
    /// namespaced sources so no source can spell another's key: `"primary"`
    /// for the primary; `"runner:<id>"` for a supervisor-spawned secondary
    /// (supervisor-assigned ids are unique, and a named runner's id is reused
    /// across its restarts); `"name:<name>"` for a secondary carrying only an
    /// instance name; `"port:<port>"` for a nameless one (stable only while it
    /// keeps that port). NOT guaranteed unique: two processes that each believe
    /// themselves the primary both report `"primary"`, so a consumer must treat
    /// two live instances with the same key as a conflict, never as one
    /// overwriting the other.
    pub instance_key: String,
    /// Whether this instance is the machine's primary or a secondary.
    pub instance_role: RunnerInstanceRole,
    /// Port this instance's HTTP API is listening on, if reported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    /// ISO 8601 timestamp when this instance's current WebSocket connection
    /// was established.
    pub connected_at: String,
}

// ============================================================================
// Runner
// ============================================================================

/// Canonical runner entity. One row per registered runner; the same shape is
/// served to every consumer (mobile, web, runner UI).
///
/// Replaces the prior split between `runners` (fleet/heartbeat),
/// `runner_connections` (transient WS sessions), and per-consumer projection
/// types like `RunnerConnection` / `ServerRunner` / `RegisteredRunner` /
/// `WebIntegrationStatus`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct Runner {
    /// Runner identifier (UUID as a string).
    pub id: String,
    /// Owning user identifier (UUID as a string).
    pub user_id: String,
    /// Human-readable runner name.
    pub name: String,

    /// Reported hostname of the machine the runner is running on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    /// Reported network address the runner is reachable on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
    /// Reported port the runner's HTTP API is listening on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    /// Operating system identifier (e.g., `"windows"`, `"macos"`, `"linux"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub os: Option<String>,
    /// Operating system version string.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub os_version: Option<String>,
    /// Free-form capability tags advertised by the runner
    /// (e.g., `["python", "playwright", "vision"]`).
    #[serde(default)]
    pub capabilities: Vec<String>,

    /// Computed health state derived from `ws_connected` and
    /// `last_heartbeat`. Authoritative for UI display.
    pub derived_status: RunnerStatus,
    /// Whether the server currently holds an open WebSocket from the runner.
    pub ws_connected: bool,
    /// ISO 8601 timestamp of the most recent heartbeat received from the
    /// runner, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_heartbeat: Option<String>,

    /// Most recently reported structured UI error, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ui_error: Option<RunnerUiError>,
    /// Most recently captured crash dump metadata, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recent_crash: Option<RunnerCrash>,

    /// Currently connected runner instances on this machine, one entry per
    /// instance (see [`RunnerInstance`]). Empty when none is connected — and
    /// when the serving backend predates per-instance rows and omits the
    /// field, so absence deserializes to an empty list rather than failing.
    #[serde(default)]
    pub instances: Vec<RunnerInstance>,

    /// ISO 8601 timestamp when the runner was first registered.
    pub created_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn runner_json_without_instances() -> serde_json::Value {
        serde_json::json!({
            "id": "00000000-0000-0000-0000-000000000001",
            "userId": "00000000-0000-0000-0000-000000000002",
            "name": "box-1",
            "port": 9876,
            "capabilities": [],
            "derivedStatus": "healthy",
            "wsConnected": true,
            "createdAt": "2026-09-23T00:00:00Z"
        })
    }

    /// A backend that predates per-instance rows omits `instances`; that must
    /// read as "no instances", not fail the whole `Runner`.
    #[test]
    fn runner_without_instances_deserializes_to_an_empty_list() {
        let runner: Runner = serde_json::from_value(runner_json_without_instances()).unwrap();
        assert!(runner.instances.is_empty());
    }

    /// Pins the camelCase wire keys and the lowercase role spelling the
    /// backend emits and the runner reports (`instanceKey` / `instanceRole`).
    #[test]
    fn runner_instances_round_trip_with_camel_case_keys() {
        let mut json = runner_json_without_instances();
        json["instances"] = serde_json::json!([
            {
                "instanceKey": "primary",
                "instanceRole": "primary",
                "port": 9876,
                "connectedAt": "2026-09-23T00:00:01Z"
            },
            {
                "instanceKey": "port:9878",
                "instanceRole": "secondary",
                "connectedAt": "2026-09-23T00:00:02Z"
            }
        ]);

        let runner: Runner = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(
            runner.instances,
            vec![
                RunnerInstance {
                    instance_key: "primary".to_string(),
                    instance_role: RunnerInstanceRole::Primary,
                    port: Some(9876),
                    connected_at: "2026-09-23T00:00:01Z".to_string(),
                },
                RunnerInstance {
                    instance_key: "port:9878".to_string(),
                    instance_role: RunnerInstanceRole::Secondary,
                    port: None,
                    connected_at: "2026-09-23T00:00:02Z".to_string(),
                },
            ]
        );

        let back = serde_json::to_value(&runner).unwrap();
        assert_eq!(back["instances"], json["instances"]);
    }
}
