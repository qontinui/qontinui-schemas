//! MCP (Model Context Protocol) client configuration DTOs.
//!
//! Wire-format types for the runner's MCP client subsystem: the transport
//! selector enum, the stdio/HTTP transport configs, the server config that
//! ties them together, the tool-info shapes returned by an MCP server, the
//! status / tool-call / DB-record payloads, and the create/update inputs
//! used by the CRUD Tauri commands and the `/mcp-servers` HTTP endpoints.
//!
//! These are ports of the shape-bearing portion of
//! `qontinui-runner/src-tauri/src/mcp_client/types.rs`. Behavior — the
//! `McpClientManager`, its `StdioHandle` / `McpConnection` runtime state,
//! the JSON-RPC 2.0 transport machinery, and the tool-calling methods — stays
//! in the runner. This module is data-only.
//!
//! ## Wire-format notes
//!
//! - `McpTransport` is a plain enum (not a tagged variant on `McpServerConfig`)
//!   because the persisted DB shape already has `transport` as a scalar column
//!   and `stdio_config` / `http_config` as separate nullable columns. Treating
//!   the transport as the tag of a sum over `StdioConfig` / `HttpConfig` would
//!   invert the existing wire contract. Callers check `config.transport` and
//!   then read the matching nested config struct.
//! - `McpTransport` serializes as lowercase (`"stdio"`, `"http"`) because the
//!   DB stores those exact strings; changing to snake_case would be a no-op
//!   here but `rename_all = "lowercase"` documents intent.
//! - `McpToolInfo.input_schema` uses the wire field name `"inputSchema"`
//!   (camelCase) because that's what the MCP spec mandates; every other field
//!   on this module follows the project-wide snake_case convention.
//! - Dates/times are ISO 8601 strings (see crate-level docs).
//!
//! ## Security
//!
//! Several fields on this module are **secret surfaces**:
//!
//! - `StdioConfig::command` and `StdioConfig::args` — arbitrary subprocess
//!   invocation. Any consumer that stores or surfaces these MUST treat them
//!   as code that will be executed on the runner host.
//! - `StdioConfig::env` — environment variables passed to the subprocess.
//!   Commonly holds API tokens, service account keys, etc.
//! - `HttpConfig::headers` — HTTP headers sent on every MCP request.
//!   Typically carries `Authorization: Bearer …` tokens or `X-Api-Key: …`
//!   values that must never leak to logs or UI-facing responses without
//!   redaction.
//!
//! These are persisted in the database (the whole point is reconstructing
//! a connection across runner restarts), so the at-rest shape includes them.
//! MCP / HTTP API responses that surface these configs to end-user UIs MUST
//! redact env values and auth-bearing headers before returning.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Transport selector
// ============================================================================

/// Transport type for an MCP server connection.
///
/// Serialized as lowercase to match the existing DB column values
/// (`"stdio"` / `"http"`). Defaults to `Stdio` to preserve the pre-extraction
/// default from the runner.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum McpTransport {
    /// Stdio subprocess transport — launches the configured command and
    /// speaks newline-delimited JSON-RPC 2.0 over stdin/stdout.
    #[default]
    Stdio,
    /// HTTP transport — POSTs JSON-RPC 2.0 to the configured URL.
    Http,
}

// ============================================================================
// Transport configs
// ============================================================================

/// Stdio-transport configuration.
///
/// **Secret surface**: `command`, `args`, and `env` must be treated as
/// execution-critical. See module-level docs.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct StdioConfig {
    /// Command to execute (e.g. `"npx"`, `"python"`, `"/usr/local/bin/server"`).
    pub command: String,
    /// Command arguments.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    /// Working directory (absolute path). `None` inherits the runner's cwd.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    /// Extra environment variables for the subprocess. **Secret surface** —
    /// frequently holds API tokens.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
}

/// HTTP-transport configuration.
///
/// **Secret surface**: `headers` typically carries an `Authorization` token.
/// See module-level docs.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct HttpConfig {
    /// Server URL (e.g. `"http://localhost:8080/mcp"`). The runner appends
    /// `/tools/list` and `/tools/call` to this base.
    pub url: String,
    /// HTTP headers to include on every request. **Secret surface** —
    /// typically includes `Authorization: Bearer …`.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,
}

// ============================================================================
// Serde default helpers (preserved from the runner so wire defaults match)
// ============================================================================

fn default_true() -> bool {
    true
}

fn default_timeout() -> u64 {
    30
}

// ============================================================================
// McpServerConfig
// ============================================================================

/// Full configuration for a registered MCP server.
///
/// Persisted in the `mcp_servers` Postgres table and surfaced to the frontend
/// through the `mcp_*` Tauri commands and the MCP `mcp-servers` HTTP
/// endpoint. The `transport` field selects which of `stdio_config` /
/// `http_config` is meaningful; the other is expected to be `None`.
///
/// **Secret surface**: the nested `stdio_config` / `http_config` carry secrets
/// — see their own docs and the module-level security note.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct McpServerConfig {
    /// Unique identifier (UUID).
    pub id: String,
    /// Display name.
    pub name: String,
    /// Optional human-readable description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Which transport this server uses.
    pub transport: McpTransport,

    /// Stdio-transport settings. Expected to be `Some` iff
    /// `transport == McpTransport::Stdio`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stdio_config: Option<StdioConfig>,

    /// HTTP-transport settings. Expected to be `Some` iff
    /// `transport == McpTransport::Http`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_config: Option<HttpConfig>,

    /// Whether this server is enabled. Disabled servers won't be connected
    /// even if `auto_start` is true. Default: `true`.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Auto-connect when the runner launches. Default: `false`.
    #[serde(default)]
    pub auto_start: bool,

    /// Per-request connection / tool-call timeout in seconds. Default: `30`.
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,

    /// Serialized JSON list of tools cached from the last successful
    /// connection. Stored as a string for DB portability.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cached_tools: Option<String>,

    /// ISO 8601 timestamp of when `cached_tools` was populated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools_cached_at: Option<String>,

    /// ISO 8601 creation timestamp.
    pub created_at: String,

    /// ISO 8601 last-update timestamp.
    pub updated_at: String,
}

// ============================================================================
// Tool schema / info
// ============================================================================

/// Input-parameter schema for an MCP tool.
///
/// Subset of JSON Schema — enough to render a form and validate arguments
/// before dispatching a `tools/call`. The `properties` and `required` fields
/// are passed through verbatim from the server.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct McpToolInputSchema {
    /// JSON Schema `type` (typically `"object"`).
    #[serde(rename = "type")]
    pub schema_type: String,
    /// Optional human-readable description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// JSON-Schema-shaped property descriptors, kept as opaque JSON.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub properties: Option<serde_json::Value>,
    /// Names of required properties.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
}

/// Single tool exposed by an MCP server, as returned by `tools/list`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct McpToolInfo {
    /// Tool name (the argument passed back on `tools/call`).
    pub name: String,
    /// Tool description shown to users.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Input-argument schema. Wire field is `"inputSchema"` per MCP spec.
    #[serde(rename = "inputSchema")]
    pub input_schema: McpToolInputSchema,
}

// ============================================================================
// Server status
// ============================================================================

/// Status of a single MCP server, as reported by the client manager.
///
/// Derived from runtime connection state; never persisted. `tools` is
/// populated only when `connected == true`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct McpServerStatus {
    /// ID of the server this status refers to.
    pub server_id: String,
    /// Whether the client currently holds a live connection.
    pub connected: bool,
    /// Last connection / tool-call error, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Available tools — `Some(…)` when connected, `None` otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<McpToolInfo>>,
    /// ISO 8601 timestamp of the most recent connection attempt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_connect_attempt: Option<String>,
    /// ISO 8601 timestamp of the most recent successful connection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_connected: Option<String>,
}

// ============================================================================
// Create / update inputs
// ============================================================================

/// Request body for creating a new MCP server configuration.
///
/// **Secret surface**: as with [`McpServerConfig`], the nested transport
/// configs carry secrets.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateMcpServerInput {
    /// Display name.
    pub name: String,
    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Transport selector.
    pub transport: McpTransport,
    /// Stdio config (required when `transport == Stdio`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stdio_config: Option<StdioConfig>,
    /// HTTP config (required when `transport == Http`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_config: Option<HttpConfig>,
    /// Override for the default `enabled = true`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// Override for the default `auto_start = false`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_start: Option<bool>,
    /// Override for the default `timeout_seconds = 30`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_seconds: Option<u64>,
}

/// Request body for updating an MCP server configuration. Every field is
/// optional — fields left as `None` are preserved from the existing row.
///
/// **Secret surface**: as with [`McpServerConfig`], the nested transport
/// configs carry secrets.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct UpdateMcpServerInput {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transport: Option<McpTransport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stdio_config: Option<StdioConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_config: Option<HttpConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_start: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_seconds: Option<u64>,
}

// ============================================================================
// Tool-call result
// ============================================================================

/// Result of a single `tools/call` invocation.
///
/// Shape is always the same regardless of success / failure: check `success`
/// first, then read `content` (on success) or `error` (on failure).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct McpToolCallResult {
    /// Whether the call succeeded.
    pub success: bool,
    /// Response content (present on success). Usually a JSON object/array,
    /// but can be a primitive if the tool returned text-only content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<serde_json::Value>,
    /// Error message (present on failure).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Response type tag: `"json"`, `"text"`, or `"error"`.
    pub response_type: String,
    /// Wall-clock duration of the call in milliseconds.
    pub duration_ms: u64,
}

// ============================================================================
// Call-history DB types
// ============================================================================

/// Input shape for recording an MCP call to the `mcp_calls` table.
///
/// `arguments` / `resolved_arguments` / `response` / `extractions` /
/// `assertions` are serialized-JSON strings rather than `serde_json::Value`
/// because the DB layer stores them as `TEXT` / `JSONB` strings and does not
/// round-trip through a `Value`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateMcpCallInput {
    pub task_run_id: String,
    pub step_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_name: Option<String>,
    pub server_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_name: Option<String>,
    pub tool_name: String,
    /// JSON-serialized arguments as originally submitted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
    /// JSON-serialized arguments after variable resolution.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_arguments: Option<String>,
    /// JSON-serialized response body.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response: Option<String>,
    /// Response type tag (see [`McpToolCallResult::response_type`]).
    pub response_type: String,
    /// Wall-clock duration in milliseconds.
    pub duration_ms: i64,
    /// JSON-serialized extractions (variables pulled from the response).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extractions: Option<String>,
    /// JSON-serialized assertion results.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assertions: Option<String>,
    /// Whether the call succeeded.
    pub success: bool,
    /// Error message if the call failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

/// Persisted MCP call record as read back from the `mcp_calls` table.
///
/// Same shape as [`CreateMcpCallInput`] plus the row id and creation
/// timestamp.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct McpCallRecord {
    pub id: String,
    pub task_run_id: String,
    pub step_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_name: Option<String>,
    pub server_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_name: Option<String>,
    pub tool_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_arguments: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response: Option<String>,
    pub response_type: String,
    pub duration_ms: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extractions: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assertions: Option<String>,
    pub success: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    /// ISO 8601 creation timestamp.
    pub created_at: String,
}

/// Result envelope for an `mcp_calls` query scoped to a single task run.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct McpCallsResult {
    /// Task run the query was scoped to.
    pub task_run_id: String,
    /// Matched call records (paginated; see `total_count` for the full size).
    pub calls: Vec<McpCallRecord>,
    /// Number of calls returned in `calls`.
    pub count: usize,
    /// Total matching rows across all pages. Defaults to `0` on older rows
    /// that predate the field; consumers should prefer `count` when this is
    /// absent.
    #[serde(default)]
    pub total_count: usize,
    /// Number of successful calls in the current page.
    pub success_count: usize,
    /// Number of failed calls in the current page.
    pub failed_count: usize,
    /// Whether there are more rows after the current page.
    #[serde(default)]
    pub has_more: bool,
}
