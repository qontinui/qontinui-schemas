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

    /// ISO 8601 timestamp when the runner was first registered.
    pub created_at: String,
}
