//! Process-management DTO types.
//!
//! Extracted from
//! `qontinui-runner/src-tauri/src/process_capture/types.rs` (see commit that
//! introduced this module). This module is the wire-format source of truth for
//! managed-process configuration and status as surfaced through settings.json,
//! the Tauri commands in `process_capture::commands`, and the MCP
//! `processes` endpoints.
//!
//! Runtime-only state (tokio JoinHandles, mpsc channels, `std::time::Instant`
//! stamps) stays in the runner. Behavior that was previously implemented via
//! inherent `impl` blocks on these types is exposed on the runner side through
//! extension traits — see `process_capture::types` for the shim module.
//!
//! Wire-format notes:
//! - `ParserType` is included here because `ProcessConfig.parser` is a field of
//!   `ProcessConfig`. If/when the error-monitor subsystem is extracted into its
//!   own `qontinui-types` module, `ParserType` should move with it and this
//!   module should re-export from there.
//! - Dates/times are ISO 8601 strings (see crate-level docs).
//! - Enum string values use `snake_case` to match the pre-extraction wire.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Parser type (used by ProcessConfig.parser and error_monitor log sources)
// ============================================================================

/// Parser type for extracting errors from a managed process's log stream.
///
/// Mirrors the pre-extraction `crate::error_monitor::types::ParserType` wire
/// format. Note the JavaScript rename: the canonical wire value is
/// `"javascript"` but the serde `alias = "java_script"` is preserved for
/// backwards compatibility with any settings that were written before the
/// rename.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ParserType {
    /// Python tracebacks and exceptions.
    Python,
    /// JavaScript/TypeScript errors and stack traces.
    #[serde(alias = "java_script", rename = "javascript")]
    JavaScript,
    /// Rust panics and errors.
    Rust,
    /// Generic regex-based pattern matching.
    #[default]
    Generic,
}

// ============================================================================
// Process lifecycle state
// ============================================================================

/// State of a managed process.
///
/// Ordering mirrors the lifecycle progression:
/// `Stopped → Starting → (Building) → Running → Healthy → Stopping → Stopped`,
/// or to `Failed` on any abnormal exit.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ProcessState {
    Stopped,
    Starting,
    Building,
    Running,
    Healthy,
    Stopping,
    Failed,
}

// ============================================================================
// Output streams
// ============================================================================

/// Which output stream a captured line came from.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum OutputStream {
    Stdout,
    Stderr,
}

/// A single line of output from a managed process.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OutputLine {
    /// ISO 8601 timestamp
    pub timestamp: String,
    /// Which stream this came from
    pub stream: OutputStream,
    /// The line content
    pub line: String,
}

// ============================================================================
// Serde default helpers (preserved from the runner so wire defaults match)
// ============================================================================

fn default_category() -> String {
    "general".to_string()
}

fn default_buffer_size() -> usize {
    2000
}

fn default_true() -> bool {
    true
}

// ============================================================================
// ProcessConfig
// ============================================================================

/// Configuration for a managed process.
///
/// Persisted in `settings.json` under `managed_processes`. Surfaced to the
/// frontend through the `get_process_configs` command and to MCP consumers
/// through the `processes` endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProcessConfig {
    /// Unique identifier (UUID)
    pub id: String,
    /// Human-readable name (e.g., "FastAPI Backend")
    pub name: String,
    /// Command to execute (e.g., "python", "npm", "cargo")
    pub command: String,
    /// Command arguments (e.g., ["run", "dev"])
    #[serde(default)]
    pub args: Vec<String>,
    /// Working directory (absolute path)
    pub cwd: String,
    /// Extra environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Port to check for health (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub health_port: Option<u16>,
    /// Parser type for error detection
    #[serde(default)]
    pub parser: ParserType,
    /// Start when runner launches
    #[serde(default)]
    pub auto_start: bool,
    /// Category (e.g., "backend", "frontend")
    #[serde(default = "default_category")]
    pub category: String,
    /// Ring buffer max lines (default 2000)
    #[serde(default = "default_buffer_size")]
    pub buffer_size: usize,
    /// Whether this config is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Regex patterns for errors to ignore (matched against error message and
    /// raw entry).
    #[serde(default)]
    pub ignore_patterns: Vec<String>,
    /// Startup group for ordered startup (lower groups start first, default 0).
    /// Processes in the same group start together. The runner waits for health
    /// ports in each group to be ready before starting the next group.
    #[serde(default)]
    pub start_group: u32,
    /// Whether this is a dev-mode-only service (not started in production
    /// builds).
    #[serde(default)]
    pub dev_only: bool,
    /// Whether rebuild and AI fix features are enabled for this process.
    #[serde(default = "default_true")]
    pub rebuild_enabled: bool,
    /// Build command to run before restarting (e.g., "cargo", "npm").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_command: Option<String>,
    /// Build command arguments (e.g., ["build"], ["run", "build"]).
    #[serde(default)]
    pub build_args: Vec<String>,
}

// ============================================================================
// ProcessStatus
// ============================================================================

/// Status summary of a managed process.
///
/// Derived from runtime state each time it is requested; never persisted.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProcessStatus {
    pub id: String,
    pub name: String,
    pub state: ProcessState,
    pub pid: Option<u32>,
    /// Uptime in seconds (None if not running)
    pub uptime_secs: Option<u64>,
    /// Whether the health port is responding
    pub port_healthy: Option<bool>,
    /// Number of times this process has been restarted
    pub restart_count: u32,
    /// Number of errors detected from this process
    pub error_count: u32,
    pub category: String,
    /// Whether this process has a build command configured
    pub has_build_command: bool,
}
