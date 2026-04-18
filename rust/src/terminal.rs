//! Terminal session DTO types.
//!
//! Extracted from `qontinui-runner/src-tauri/src/terminal/types.rs`. This
//! module is the wire-format source of truth for the embedded PTY terminal
//! system: session info returned from the `terminal_*` Tauri commands and
//! event payloads emitted on the `terminal-output` / `terminal-exit` Tauri
//! channels (mirrored to the WebSocket relay for remote/mobile consumers).
//!
//! Runtime-only state — PTY master/writer handles, tokio broadcast channels,
//! OS thread join handles, scrollback ring buffers, atomic flow-control
//! counters — stays in the runner on `TerminalSession`. This module only
//! describes the bytes that cross the wire.
//!
//! Wire-format notes:
//! - `TerminalId` is a type alias for `String`; the runner mints these as
//!   UUIDs but the wire treats them as opaque identifiers.
//! - `created_at` is a Unix epoch millisecond timestamp, not an ISO 8601
//!   string, because the runner needs a cheap monotonic sort key for the
//!   `list` command and JS/TS `Date` constructors accept epoch millis.
//! - Output payloads carry base64-encoded raw bytes in `data` because PTY
//!   output can contain partial UTF-8 sequences that would corrupt a
//!   `String`-typed field.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Unique identifier for a terminal session.
///
/// Minted as a UUID v4 by `TerminalManager::create`, but treated as an
/// opaque `String` on the wire so that backends minting IDs in other formats
/// (tests, replayed fixtures) round-trip unchanged.
pub type TerminalId = String;

/// Default `page_id` value when absent from older wire forms.
///
/// Pre-multi-page clients didn't send `page_id`; they are bucketed into the
/// `"default"` page so the UI still lays them out on a single tab group.
fn default_page_id() -> String {
    "default".to_string()
}

/// Information about a terminal session, returned to the frontend.
///
/// Returned from the `terminal_create` and `terminal_list` Tauri commands
/// and emitted as the payload of the `terminal-created` event. Derived
/// fresh from the live `TerminalSession` each time — never persisted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct TerminalInfo {
    /// Unique session identifier (UUID v4 minted by the runner).
    #[serde(alias = "id")]
    pub id: TerminalId,
    /// Human-readable title shown in the UI tab (e.g., "Terminal 1").
    #[serde(alias = "title")]
    pub title: String,
    /// OS process ID of the spawned shell, if still known.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "pid")]
    pub pid: Option<u32>,
    /// Current terminal width in columns.
    #[serde(alias = "cols")]
    pub cols: u16,
    /// Current terminal height in rows.
    #[serde(alias = "rows")]
    pub rows: u16,
    /// Absolute working directory the shell was started in.
    #[serde(alias = "working_dir")]
    pub working_dir: String,
    /// Whether the shell process is still running.
    #[serde(alias = "is_alive")]
    pub is_alive: bool,
    /// Process exit code, once the shell has exited.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "exit_code")]
    pub exit_code: Option<i32>,
    /// Unix timestamp in milliseconds when the session was created.
    ///
    /// Used as the sort key for `TerminalManager::list` so the UI shows
    /// terminals in creation order.
    #[serde(alias = "created_at")]
    pub created_at: u64,
    /// Monotonic counter of all bytes ever produced by this PTY.
    ///
    /// Read by the frontend to detect missed output after a reconnect; the
    /// scrollback buffer's `start_offset` is derived from this counter.
    #[serde(alias = "total_bytes_produced")]
    pub total_bytes_produced: u64,
    /// Which terminal page this session belongs to (for multi-page support).
    ///
    /// Older wire forms without this field hydrate to `"default"` via
    /// [`default_page_id`].
    #[serde(default = "default_page_id", alias = "page_id")]
    pub page_id: String,
}

/// Event payload emitted when a terminal produces output.
///
/// Delivered over the Tauri `terminal-output` channel and re-broadcast on
/// the WebSocket relay for remote/mobile consumers. The `data` payload is
/// base64-encoded raw bytes because xterm.js needs raw bytes (PTY output
/// can contain partial UTF-8 sequences that would corrupt a `String`
/// field).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct TerminalOutputEvent {
    /// ID of the terminal session producing this output.
    #[serde(alias = "terminal_id")]
    pub terminal_id: TerminalId,
    /// Base64-encoded bytes produced by the PTY.
    ///
    /// Raw bytes are required (not UTF-8 text) because PTY output can
    /// contain partial UTF-8 sequences that span read boundaries.
    #[serde(alias = "data")]
    pub data: String,
}

/// Event payload emitted when a terminal process exits.
///
/// Delivered over the Tauri `terminal-exit` channel and re-broadcast on the
/// WebSocket relay. After this event fires the corresponding
/// [`TerminalInfo::is_alive`] will be `false` and [`TerminalInfo::exit_code`]
/// will carry the same value surfaced here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct TerminalExitEvent {
    /// ID of the terminal session that exited.
    #[serde(alias = "terminal_id")]
    pub terminal_id: TerminalId,
    /// Exit code reported by the OS, or `None` if the status could not be
    /// captured (e.g., the wait call itself errored).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "exit_code")]
    pub exit_code: Option<i32>,
}
