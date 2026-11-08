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
//!
//! The module also declares the [`TerminalLayout`] family: the snapshot the
//! runner serves on `GET /terminal-layout` (which tab each session sits in,
//! with its account, name and finished state). That family follows two wire
//! rules of its own, set out on [`TerminalLayout`]: null means unknown or not
//! applicable, as each field says, and is never a stand-in such as `false`,
//! `""` or `-1`; and no type in it carries `deny_unknown_fields`. Its
//! timestamps are RFC 3339 strings, per the crate convention, not epoch millis
//! like [`TerminalInfo::created_at`].

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

// ============================================================================
// GET /terminal-layout — the per-tab session snapshot behind saved tab lists
// ============================================================================

/// One runner's terminal tabs and the sessions open in each, read at one
/// instant: the body of the runner's `GET /terminal-layout`.
///
/// qontinui-runner serializes it. qontinui-web relays one GET to the runner,
/// validates the body with the generated Python model and keeps it as a saved
/// tab list on the `/sessions` page for 14 days (plan
/// `2026-11-08-save-per-tab-runner-session-lists-on-sessions-page-kept-14-days`).
/// One definition, here, serves both sides.
///
/// Two wire rules hold for this type and the three it contains
/// ([`TerminalLayoutRunner`], [`TerminalLayoutPage`],
/// [`TerminalLayoutSession`]):
///
/// - **null means unknown or not applicable, as each field says, and is never
///   a stand-in such as `false`, `""` or `-1`.** An unknown finished state is
///   `null`, not `false`; an unknown account is `null`, not `""`; an unzoned
///   tile is `null`, not `-1`. The optional fields are always serialized,
///   `null` included, so the wire states the null instead of leaving the key
///   out, and a missing key also reads as `null`.
/// - **No `deny_unknown_fields`**, neither serde's nor schemars', on purpose.
///   The generated models then ignore keys they do not know, so a newer runner
///   that adds a field never makes an older web refuse a capture. Do not add
///   it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TerminalLayout {
    /// The runner instance that answered, as it reports itself.
    pub runner: TerminalLayoutRunner,
    /// Whether the runner's webview has registered its tab list (labels and
    /// left-to-right order) with the runner backend since this runner process
    /// started.
    ///
    /// `false` means every page's `label` and `order` is `null` because nothing
    /// was registered yet, not because the tabs have no names.
    pub labels_registered: bool,
    /// When the runner built this snapshot: an RFC 3339 timestamp on the
    /// runner's own clock.
    pub captured_at: String,
    /// The runner's terminal pages (tabs): registered pages first, in their
    /// registered order, then any page the webview never registered.
    pub pages: Vec<TerminalLayoutPage>,
}

/// Which runner instance answered a [`TerminalLayout`] read, as the runner
/// reports itself.
///
/// Each field is `null` when the runner does not know or report it. See
/// [`TerminalLayout`] for the null and forward-compatibility rules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TerminalLayoutRunner {
    /// The runner instance's name.
    #[serde(default)]
    pub instance_name: Option<String>,
    /// The port the runner instance's HTTP API listens on.
    #[serde(default)]
    pub port: Option<u16>,
    /// The runner build that answered (the same value as `buildId` on the
    /// runner's `GET /health`).
    #[serde(default)]
    pub build_id: Option<String>,
    /// The device id the runner reports for itself.
    #[serde(default)]
    pub device_id: Option<String>,
}

/// One terminal page (a tab of the runner's terminal view) and the sessions
/// in it.
///
/// See [`TerminalLayout`] for the null and forward-compatibility rules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TerminalLayoutPage {
    /// The page id, the same value as [`TerminalInfo::page_id`].
    pub page_id: String,
    /// The tab label the webview registered for this page; `null` when no
    /// label was registered for it.
    #[serde(default)]
    pub label: Option<String>,
    /// The page's left-to-right position as registered by the webview; `null`
    /// when the page was never registered.
    #[serde(default)]
    pub order: Option<u32>,
    /// The terminals on this page.
    pub sessions: Vec<TerminalLayoutSession>,
}

/// One terminal on a [`TerminalLayoutPage`], joined to the session running in
/// it.
///
/// The session fields (`claudeSessionId`, `accountLabel`, `sessionName`,
/// `zoneIndex`, `finished`, `finishedAt`, `finishReason`) come from the
/// terminal's CONFIRMED open session record only. A terminal with none, such
/// as a plain shell, is still listed, with all of them `null`. `finished` is
/// the runner's own finished marker; coord's view of the session is not part
/// of this type.
///
/// See [`TerminalLayout`] for the null and forward-compatibility rules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TerminalLayoutSession {
    /// The runner terminal id, the same value as [`TerminalInfo::id`].
    pub terminal_id: TerminalId,
    /// The terminal's title. It is set by the program in the terminal (OSC 0/2
    /// escapes in its output), so it is session content. `null` when the
    /// terminal has no title, or when it was withheld (`titleWithheld`).
    #[serde(default)]
    pub title: Option<String>,
    /// `true` when the runner's egress gate withheld `title` because the
    /// session's tenant does not let terminal content leave the machine (its
    /// terminal-stream switch is off); `title` is then `null`. `false` when
    /// nothing was withheld.
    pub title_withheld: bool,
    /// Whether the terminal's process is still running.
    pub is_alive: bool,
    /// The name the runner spawned the terminal under, where it has one.
    #[serde(default)]
    pub spawn_name: Option<String>,
    /// The id of the terminal's confirmed session (the runner's
    /// `claude_session_id`; a UUID for a Claude Code session).
    #[serde(default)]
    pub claude_session_id: Option<String>,
    /// The Claude account the session runs under; `null` when unknown (the
    /// runner's own `"unknown"` label is served as `null`).
    #[serde(default)]
    pub account_label: Option<String>,
    /// The session's display name.
    #[serde(default)]
    pub session_name: Option<String>,
    /// The grid zone the session's tile occupies; `null` when the tile is
    /// unzoned or its zone is unknown. Unsigned, so the runner's `-1` sentinel
    /// cannot reach the wire: it is served as `null`.
    #[serde(default)]
    pub zone_index: Option<u32>,
    /// The runner's finished marker for the session: `true` when it was
    /// marked finished, `false` when its confirmed record carries no finished
    /// mark, `null` when unknown.
    #[serde(default)]
    pub finished: Option<bool>,
    /// When the session was marked finished, as an RFC 3339 timestamp; `null`
    /// when it was not marked finished or the time is unknown.
    #[serde(default)]
    pub finished_at: Option<String>,
    /// Why the session was marked finished, as recorded with the mark.
    #[serde(default)]
    pub finish_reason: Option<String>,
}
