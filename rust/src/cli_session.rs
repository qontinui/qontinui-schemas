//! Per-CLI session-handling DTO types: the [`CliProfile`] manifest, the
//! [`SessionFailure`] taxonomy, and the shared [`CapabilityState`] tri-state.
//!
//! Introduced by plan
//! `2026-09-20-ai-session-handling-is-claude-shaped-provider-manifest-and-failure-taxonomy`
//! (Phase 3). Before it, the runner decided "what does this AI CLI do?" by
//! testing the literal program name (`claude`) at dozens of scattered sites,
//! and it had five failure mechanisms that shared no vocabulary. This module
//! is the wire-format source of truth for both halves of the replacement:
//!
//! - [`CliProfile`] — one record per AI CLI (Claude Code, Codex, …) holding
//!   every *fact* the runner needs about that CLI's session behaviour: how its
//!   session id is learned, how it resumes, how its account is isolated, what
//!   it paints on screen when a resume succeeds or fails, and so on. The runner
//!   builds `static` instances of it and serves them to the frontend; the
//!   frontend holds no provider data of its own.
//! - [`SessionFailure`] — one payload for every way an AI session can fail,
//!   whichever lane noticed it (a structured protocol event, a hook, stderr, a
//!   phrase scraped off the terminal grid, an exit status, …). What the runner
//!   *does* about each kind is runner policy and lives there, not here: these
//!   types are pure data.
//!
//! # `Unknown` is a value, never a default-to-yes or default-to-no
//!
//! Every fact enum here carries an explicit `Unknown` arm and `Unknown` is its
//! `Default`. A profile that does not state a fact means "we never verified
//! it", which is a different statement from "the CLI does not do this". That
//! is the fleet's `silent-empty-is-unknown` rule expressed in the type system,
//! and it is why [`CapabilityState`] — moved here from the runner's
//! `model_catalog.rs` — is three-valued. Likewise [`FailureKind::Unknown`] is
//! a real kind a classifier can report; it is never read as "no failure".
//!
//! # Wire-format notes
//!
//! - Struct fields are `camelCase`, matching the sibling [`crate::terminal`]
//!   payloads these travel beside on the Terminal page.
//! - Enum values are `snake_case`. Data-carrying fact enums are internally
//!   tagged with `"kind"` (e.g. `{ "kind": "env_var", "name":
//!   "CLAUDE_CONFIG_DIR" }`); enums with no data are plain strings.
//! - `deny_unknown_fields` is deliberately NOT set: both records are expected
//!   to grow additively as more CLIs and more failure sources are profiled, and
//!   a reader must tolerate a newer writer. Profile fields beyond the identity
//!   triple (`id`, `displayName`, `programs`) are `#[serde(default)]`, so a
//!   field an older writer omits reads as `Unknown`, not as an error.
//! - Timestamps are ISO 8601 / RFC 3339 `String`s, per the crate convention.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ============================================================================
// The shared tri-state
// ============================================================================

/// Three-valued capability fact.
///
/// `Unknown` is the default for anything a profile or catalog does not state.
/// It is a distinct value from `Unsupported` on purpose: a boolean cannot
/// represent "we never asked", and coercing that to "no" silently disables
/// working capabilities.
///
/// Moved here from qontinui-runner `src-tauri/src/model_catalog.rs` so the
/// model/API layer (`ModelFacts`) and the CLI-session layer ([`CliProfile`])
/// speak one tri-state; `model_catalog` re-exports it. Serialized as
/// `"supported"` / `"unsupported"` / `"unknown"`.
///
/// [`CapabilityState::Unknown`] must never be a reason to strip content or
/// refuse a route. Callers that must act on an unknown fact should attempt the
/// capability and let the backend be the authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityState {
    /// The source states the capability is available.
    Supported,
    /// The source states the capability is NOT available.
    Unsupported,
    /// The source says nothing. **Not** a synonym for `Unsupported`.
    ///
    /// This is the `Default` on purpose: a `CapabilityState` that nobody set
    /// must read as "we never asked", never as "no".
    #[default]
    Unknown,
}

impl CapabilityState {
    /// True only when the source positively states support.
    ///
    /// Do NOT use this to decide whether to strip content — `Unknown` returns
    /// `false` here, and treating that as "strip" is the exact coercion this
    /// type exists to prevent. Use [`Self::is_known_unsupported`] for that.
    pub fn is_supported(self) -> bool {
        matches!(self, Self::Supported)
    }

    /// True only when the source positively states the capability is absent.
    ///
    /// This is the correct predicate for "should I downgrade?": an `Unknown`
    /// fact answers `false`, so the capability is attempted and the backend
    /// gets to be the authority.
    pub fn is_known_unsupported(self) -> bool {
        matches!(self, Self::Unsupported)
    }

    /// True when the source has no opinion.
    pub fn is_unknown(self) -> bool {
        matches!(self, Self::Unknown)
    }
}

// ============================================================================
// CliProfile — the per-CLI manifest
// ============================================================================

/// Everything the runner needs to know about one AI CLI's session behaviour.
///
/// One instance per CLI (`claude`, `codex`, …), built by the runner as static
/// data and served to the frontend (`GET /terminals/cli-profiles`, Tauri
/// command `terminal_cli_profiles`) so no consumer holds a second copy.
///
/// Every fact that can be unverified defaults to its `Unknown` arm; see the
/// module docs. Scraped facts ([`Self::handshake`], [`Self::usage_limit_phrases`],
/// [`TrustDialog::Prompt`]) break when the CLI changes its screens, which is
/// why [`Self::verified_version`] records the CLI version they were verified
/// against.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CliProfile {
    /// Stable provider id (`"claude"`, `"codex"`). The same string the runner
    /// stores as a terminal session record's `provider`.
    pub id: String,
    /// Human-readable name for launch menus and zone labels (`"Claude Code"`).
    pub display_name: String,
    /// Program stems that identify this CLI on an argv head or in a process
    /// census, without directory or extension (`["claude"]`). Matching strips
    /// the path and a Windows `.exe`/`.cmd` suffix before comparing.
    pub programs: Vec<String>,
    /// CLI version the scraped facts in this profile were verified against
    /// (`"2.1.285"`). `None` means no version was recorded — treat every
    /// scraped fact as unverified.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verified_version: Option<String>,
    /// How the runner learns the CLI's session id.
    #[serde(default)]
    pub identity: IdentitySource,
    /// How a session is resumed by id.
    #[serde(default)]
    pub resume: ResumeSpec,
    /// Other spellings the CLI accepts for the resume flag of
    /// [`ResumeSpec::ByIdArgv`] (`["-r"]`). Matched only as a whole argv
    /// token followed by the id, and only when the argv's program is this
    /// CLI: a short flag like `-r` means something else to most programs.
    /// Empty means none are known.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub resume_flag_aliases: Vec<String>,
    /// Argv tokens, besides the identity flag, the resume flag and its
    /// aliases, by which the user picks the session themselves
    /// (`["--continue", "-c"]` — continue the most recent session). Any of
    /// them on a launch means the runner must not pin an id of its own. A token
    /// matches a whole argv entry case-insensitively; a `--long` token also
    /// matches `--long=…`. Empty means none are known.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub session_choice_args: Vec<String>,
    /// Arguments the runner appends to every PTY-hosted (interactive TUI)
    /// launch of this CLI, verified against [`Self::verified_version`]
    /// (`["--teammate-mode", "in-process"]`, which keeps Claude Code agent
    /// teams inside the session instead of splitting the caller's tmux window).
    /// Empty means none.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pty_args: Vec<String>,
    /// Arguments that make the structured lane hand each tool approval to the
    /// runner as a typed permission request instead of deciding it itself
    /// (`["--permission-prompt-tool", "stdio"]`), verified against
    /// [`Self::verified_version`]. Rendered only for a launch that asks to be
    /// prompted, and never beside an auto-approve flag. Empty means no such
    /// arguments are known, so a prompted structured launch is not offered.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub permission_prompt_args: Vec<String>,
    /// How one account's CLI state is kept apart from another's.
    #[serde(default)]
    pub account_isolation: AccountIsolation,
    /// How the CLI is launched without per-tool approval prompts, and how an
    /// existing argv is recognised as already doing so.
    #[serde(default)]
    pub auto_approve: AutoApprove,
    /// How the runner asks a live session to exit cleanly.
    #[serde(default)]
    pub graceful_exit: GracefulExit,
    /// On-screen markers that decide whether a resume succeeded or failed.
    #[serde(default)]
    pub handshake: HandshakePatterns,
    /// Case-insensitive phrases the CLI paints when an account's usage limit
    /// is reached. A grid match is a `Hint`-confidence
    /// [`FailureKind::QuotaExhausted`] signal, never a confirmed one. Empty
    /// means no phrases are known.
    #[serde(default)]
    pub usage_limit_phrases: Vec<String>,
    /// Where the CLI writes its conversation transcript.
    #[serde(default)]
    pub transcript: TranscriptSpec,
    /// The machine-readable protocol the CLI offers besides its TUI.
    #[serde(default)]
    pub structured_lane: StructuredLane,
    /// Whether the structured lane delivers a typed permission request (a
    /// tool-use approval the runner answers) rather than bypassing approval.
    #[serde(default)]
    pub typed_permission: CapabilityState,
    /// Whether the structured lane emits an explicit end-of-turn event.
    #[serde(default)]
    pub turn_boundary_event: CapabilityState,
    /// Whether the structured lane emits a typed rate-limit event.
    #[serde(default)]
    pub rate_limit_event: CapabilityState,
    /// Whether the CLI stops on a folder-trust prompt at first launch in a
    /// directory, and how to recognise and answer it.
    #[serde(default)]
    pub trust_dialog: TrustDialog,
    /// The ways the CLI can be authenticated. Empty means none are recorded —
    /// UNKNOWN, not "needs no authentication".
    #[serde(default)]
    pub auth: Vec<AuthMethod>,
    /// What a restore of this CLI's session can honestly bring back.
    #[serde(default)]
    pub restore_tier: RestoreTier,
    /// Per-OS install commands, shown beside a disabled launch entry when the
    /// CLI's binary is absent.
    #[serde(default)]
    pub install: InstallCommands,
    /// Free-form caveats a reader should see beside the profile (e.g. "macOS:
    /// same code path, unverified on hardware").
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

/// How the runner learns a CLI session's id.
///
/// Wire shape (internally tagged with `"kind"`):
/// - `{ "kind": "pinned", "flag": "--session-id" }`
/// - `{ "kind": "read_back", "capture": "codex_session_file" }`
/// - `{ "kind": "unknown" }`
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum IdentitySource {
    /// The runner mints the id and passes it on the launch argv, so the id is
    /// known authoritatively at spawn.
    Pinned {
        /// The flag that takes the id (`"--session-id"`).
        flag: String,
    },
    /// The CLI mints its own id; the runner reads it back after launch. The
    /// session record is provisional until the capture completes.
    ReadBack {
        /// Name of the runner capture mechanism that recovers the id
        /// (`"codex_session_file"`). The mechanism itself is runner behaviour.
        capture: String,
    },
    /// Not verified for this CLI.
    #[default]
    Unknown,
}

/// How a session is resumed by id.
///
/// Wire shape (internally tagged with `"kind"`):
/// - `{ "kind": "by_id_argv", "template": ["claude", "--resume", "{id}"] }`
/// - `{ "kind": "none" }` / `{ "kind": "unknown" }`
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ResumeSpec {
    /// Resume by running an argv. Each element is taken literally except the
    /// placeholder `{id}`, which is replaced by the session id.
    ByIdArgv {
        /// The argv template (`["codex", "resume", "{id}"]`).
        template: Vec<String>,
    },
    /// The CLI positively cannot resume a session by id.
    None,
    /// Not verified for this CLI.
    #[default]
    Unknown,
}

/// How one account's CLI state is kept apart from another's.
///
/// Wire shape (internally tagged with `"kind"`):
/// - `{ "kind": "env_var", "name": "CLAUDE_CONFIG_DIR" }`
/// - `{ "kind": "home_dir" }` / `{ "kind": "none" }` / `{ "kind": "unknown" }`
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AccountIsolation {
    /// An environment variable points the CLI at a per-account config dir.
    EnvVar {
        /// The variable's name (`"CLAUDE_CONFIG_DIR"`).
        name: String,
    },
    /// The CLI keeps its state under `$HOME`; isolation means a per-account
    /// home directory.
    HomeDir,
    /// The CLI positively has no per-account isolation.
    None,
    /// Not verified for this CLI.
    #[default]
    Unknown,
}

/// How the CLI is launched without per-tool approval prompts.
///
/// Wire shape (internally tagged with `"kind"`):
/// - `{ "kind": "flags", "argv": ["--permission-mode", "bypassPermissions"],
///   "detect": ["--dangerously-skip-permissions", "bypassPermissions"] }`
/// - `{ "kind": "none" }` / `{ "kind": "unknown" }`
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AutoApprove {
    /// Auto-approval is a set of launch flags.
    Flags {
        /// The flags the runner appends to enable auto-approval.
        argv: Vec<String>,
        /// Tokens whose presence on an existing argv means auto-approval is
        /// already on — every spelling the CLI accepts, not only the one the
        /// runner writes.
        detect: Vec<String>,
    },
    /// The CLI positively has no auto-approval mode.
    None,
    /// Not verified for this CLI.
    #[default]
    Unknown,
}

/// How the runner asks a live session to exit cleanly.
///
/// Wire shape (internally tagged with `"kind"`):
/// - `{ "kind": "typed_command", "text": "/exit" }`
/// - `{ "kind": "signal" }` / `{ "kind": "unknown" }`
///
/// `Unknown` means the runner must type nothing and report that it could not
/// exit the session gracefully.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GracefulExit {
    /// Type this text (followed by Enter) into the session.
    TypedCommand {
        /// The command text (`"/exit"`).
        text: String,
    },
    /// Send the platform's interrupt/terminate signal to the CLI process.
    Signal,
    /// Not verified for this CLI.
    #[default]
    Unknown,
}

/// On-screen markers that decide whether a resume succeeded or failed.
///
/// Substring lists match the ANSI-stripped screen case-insensitively; regex
/// lists are compiled by BOTH the Rust `regex` crate and JavaScript `RegExp`,
/// so a pattern must stay in the syntax both engines accept (no look-around,
/// no backreferences, no named-group syntax that differs between them).
///
/// Regex sources are ALWAYS matched case-insensitively, like the substrings:
/// each engine compiles them with its own case-insensitive switch (Rust
/// `RegexBuilder::case_insensitive`, the JavaScript `i` flag). A source must
/// therefore not carry inline flags such as `(?i)`, which JavaScript rejects.
///
/// All body lists empty means no markers are known, and a resume cannot be
/// verified from the screen.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HandshakePatterns {
    /// Substrings whose presence means the resumed session came up.
    #[serde(default)]
    pub success: Vec<String>,
    /// Substrings whose presence means the resume failed.
    #[serde(default)]
    pub failure: Vec<String>,
    /// Regex sources (matched case-insensitively) whose match means the
    /// resumed session came up.
    #[serde(default)]
    pub success_regex: Vec<String>,
    /// Regex sources (matched case-insensitively) whose match means the
    /// resume failed.
    #[serde(default)]
    pub failure_regex: Vec<String>,
    /// Regex sources (matched case-insensitively) matched against the pane's
    /// CURRENT window title only, never its body, whose match means the
    /// resumed session came up. A title the CLI sets at launch is evidence the
    /// body cannot fake, provided the pattern is anchored to the CLI's own
    /// title (the shell titles the window too). Empty means none are known.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub title_regex: Vec<String>,
}

/// Where the CLI writes its conversation transcript.
///
/// Wire shape (internally tagged with `"kind"`):
/// - `{ "kind": "jsonl_under_config_dir", "subdir": "projects" }`
/// - `{ "kind": "session_file_glob", "glob": ".codex/sessions/**/rollout-*.jsonl" }`
/// - `{ "kind": "none" }` / `{ "kind": "unknown" }`
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TranscriptSpec {
    /// One JSONL file per session, under a subdirectory of the account's
    /// config dir (the directory [`AccountIsolation::EnvVar`] points at).
    JsonlUnderConfigDir {
        /// Subdirectory of the config dir holding transcripts (`"projects"`).
        subdir: String,
    },
    /// Session files matched by a glob relative to the account's home
    /// directory (`**` crosses directories).
    SessionFileGlob {
        /// The glob (`".codex/sessions/**/rollout-*.jsonl"`).
        glob: String,
    },
    /// The CLI positively writes no transcript the runner can read.
    None,
    /// Not verified for this CLI.
    #[default]
    Unknown,
}

/// The machine-readable protocol a CLI offers besides its TUI.
///
/// Wire shape (internally tagged with `"kind"` so an arm can gain data, e.g.
/// an adapter command for `acp`, without a breaking change):
/// `{ "kind": "claude_stream_json" }`, `{ "kind": "codex_app_server" }`,
/// `{ "kind": "acp" }`, `{ "kind": "none" }`, `{ "kind": "unknown" }`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StructuredLane {
    /// Claude Code's `--output-format stream-json --input-format stream-json`.
    ClaudeStreamJson,
    /// Codex's `codex app-server` JSON-RPC over stdio.
    CodexAppServer,
    /// The Agent Client Protocol.
    Acp,
    /// The CLI positively offers no structured protocol.
    None,
    /// Not verified for this CLI.
    #[default]
    Unknown,
}

/// Whether the CLI stops on a folder-trust prompt at first launch in a
/// directory.
///
/// Wire shape (internally tagged with `"kind"`):
/// - `{ "kind": "prompt", "markers": ["Do you trust the files in this folder?"], "accept": "1" }`
/// - `{ "kind": "none" }` / `{ "kind": "unknown" }`
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TrustDialog {
    /// The CLI shows a trust prompt that blocks the session until answered.
    Prompt {
        /// Case-insensitive substrings that identify the prompt on the
        /// ANSI-stripped screen.
        markers: Vec<String>,
        /// The keystroke text that accepts it (sent followed by Enter).
        accept: String,
    },
    /// The CLI positively shows no trust prompt.
    None,
    /// Not verified for this CLI.
    #[default]
    Unknown,
}

/// One way a CLI can be authenticated.
///
/// Wire shape (internally tagged with `"kind"`):
/// - `{ "kind": "cli_login", "args": ["login"] }`
/// - `{ "kind": "api_key_env", "vars": ["OPENAI_API_KEY"] }`
///
/// No `unknown` arm: an unknown auth story is an empty
/// [`CliProfile::auth`] list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AuthMethod {
    /// An interactive login subcommand of the CLI itself.
    CliLogin {
        /// Arguments after the program (`["login"]`, or `["/login"]` typed in
        /// the TUI).
        args: Vec<String>,
    },
    /// An API key read from one of these environment variables.
    ApiKeyEnv {
        /// Variable names, in the CLI's precedence order.
        vars: Vec<String>,
    },
}

/// What a restore of a CLI's session can honestly bring back.
///
/// Serialized in the coord wire spelling the runner's `RestoreTier::wire_str`
/// already uses: `"full"` / `"terminal_only"`.
///
/// Defaults to `TerminalOnly` — the conservative claim: a profile that has not
/// established resume must not promise the conversation back.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RestoreTier {
    /// The CLI can deterministically resume the full conversation by id.
    Full,
    /// Only the terminal, its cwd and its launch command are restored; the
    /// conversation starts fresh, and the UI says so.
    #[default]
    TerminalOnly,
}

/// Per-OS install commands for a CLI. `None` for an OS means no command is
/// recorded for it.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct InstallCommands {
    /// Install command on Linux (`"npm i -g @openai/codex"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub linux: Option<String>,
    /// Install command on macOS.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub macos: Option<String>,
    /// Install command on Windows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub windows: Option<String>,
}

// ============================================================================
// SessionFailure — one taxonomy for every failure source
// ============================================================================

/// One failure of an AI session, whichever lane noticed it.
///
/// Emitted on the runner's `session-failure` event and readable from its
/// failures route, so a surface can render *why* a session stopped and which
/// actions are available, instead of an exit code or a bare banner. The
/// runner's classifier fills it from a signal and its recovery table chooses
/// [`Self::recovery_policy`]; neither lives in this crate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionFailure {
    /// Unique id of this failure occurrence (UUID minted by the runner).
    pub id: String,
    /// What went wrong. [`FailureKind::Unknown`] is a real answer, not "fine".
    pub kind: FailureKind,
    /// Coarse grouping of [`Self::kind`] for filtering and styling.
    pub category: FailureCategory,
    /// How serious the failure is for the session.
    pub severity: FailureSeverity,
    /// One-line human-readable summary.
    pub title: String,
    /// Longer human-readable explanation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    /// The raw reason the source gave, verbatim (a provider error type, a
    /// stderr line, the matched screen phrase), for diagnosis.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// The [`CliProfile::id`] of the CLI whose session failed.
    pub provider: String,
    /// The account the session was running under, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    /// The turn that failed, when the source identifies one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_id: Option<String>,
    /// When a limit resets (RFC 3339), when the source states it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reset_at: Option<String>,
    /// Where the failure was observed and how sure the runner is of it.
    pub evidence: FailureEvidence,
    /// Actions a surface may offer the user, in display order.
    #[serde(default)]
    pub actions: Vec<FailureAction>,
    /// What the runner does about it automatically.
    pub recovery_policy: RecoveryPolicy,
}

/// What went wrong. Serialized `snake_case` (`"rate_limited"`, …).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FailureKind {
    /// Short-window request-rate limit; waiting fixes it, switching accounts
    /// is not required.
    RateLimited,
    /// The account's usage quota for its window is used up.
    QuotaExhausted,
    /// A spend or billing budget is used up.
    BudgetExhausted,
    /// The conversation no longer fits the model's context (or output) limit.
    ContextExhausted,
    /// The CLI needs (re-)authentication.
    AuthRequired,
    /// Authenticated, but the account/org is not allowed to do this.
    AccessDenied,
    /// The provider is overloaded. No account switch fixes it.
    Overloaded,
    /// The connection to the CLI or the provider was lost.
    TransportLost,
    /// A resume by id did not bring the session back.
    ResumeFailed,
    /// The CLI process could not be started.
    SpawnFailed,
    /// The CLI process exited without a more specific cause.
    ProcessExited,
    /// The request was rejected as malformed (bad model id, invalid input).
    BadRequest,
    /// The provider reported an internal error.
    InternalError,
    /// A failure was observed but not classified. Never read as success.
    #[default]
    Unknown,
}

/// Coarse grouping of [`FailureKind`]s. Serialized `snake_case`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FailureCategory {
    /// Rate, quota or budget limits.
    Limit,
    /// Context-window exhaustion.
    Context,
    /// Authentication or authorization.
    Auth,
    /// Provider-side faults (overload, internal error).
    Provider,
    /// The CLI process or its transport (spawn, exit, lost connection).
    Process,
    /// Session continuity (resume).
    Session,
    /// The request itself was invalid.
    Request,
    /// Not categorised.
    #[default]
    Unknown,
}

/// How serious a failure is for the session. Serialized `snake_case`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FailureSeverity {
    /// Informational; the session continues unaffected.
    Info,
    /// The session is degraded or paused but may recover by itself.
    Warning,
    /// The session cannot continue without recovery or user action.
    Error,
}

/// Where a failure was observed and how sure the runner is of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FailureEvidence {
    /// The signal source.
    pub source: FailureEvidenceSource,
    /// Whether the source states the failure or merely suggests it.
    pub confidence: FailureConfidence,
}

/// The signal source a failure was observed through. Serialized `snake_case`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FailureEvidenceSource {
    /// A typed event on a structured protocol lane (stream-json, app-server).
    StructuredEvent,
    /// A CLI hook payload (e.g. Claude's `StopFailure` with its `error_type`).
    Hook,
    /// A line on the CLI's stderr.
    Stderr,
    /// A phrase scraped off the terminal grid.
    GridScrape,
    /// The CLI process's exit status.
    ExitStatus,
    /// A record in the CLI's transcript.
    Transcript,
    /// An expected handshake never appeared within its bound.
    HandshakeTimeout,
}

/// Whether a failure's source states it or merely suggests it. Serialized
/// `snake_case`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FailureConfidence {
    /// The source states the failure authoritatively.
    Confirmed,
    /// The failure is inferred (a scraped phrase, a timeout) and should be
    /// worded as "may have …" until something confirms it.
    Hint,
}

/// An action a surface may offer for a failure. Serialized `snake_case`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FailureAction {
    /// Retry the failed turn.
    Retry,
    /// Authenticate the CLI.
    Login,
    /// Move the session to another account.
    SwitchAccount,
    /// Start a fresh session.
    NewSession,
    /// Resume the session by id.
    Resume,
    /// No action is available.
    None,
}

/// What the runner does about a failure automatically. Serialized
/// `snake_case`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryPolicy {
    /// Take no automatic action.
    Never,
    /// Retry after a bounded backoff, without changing account.
    BackoffThenRetry,
    /// Wait until the limit's reset time, then continue.
    WaitUntilReset,
    /// Move the session to another account.
    MigrateAccount,
    /// Wait for the user to authenticate, then resume.
    LoginThenResume,
    /// Resume the same session id.
    ResumeSameId,
    /// Hand off to a new session carrying the context forward.
    HandoffNewSession,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::DeserializeOwned;
    use serde_json::{json, Value};
    use std::fmt::Debug;

    /// Serialize, assert the wire shape, deserialize, and re-serialize.
    fn round_trip<T>(value: &T, expected: Value)
    where
        T: Serialize + DeserializeOwned + PartialEq + Debug,
    {
        let wire = serde_json::to_value(value).unwrap();
        assert_eq!(wire, expected, "wire shape of {value:?}");
        let back: T = serde_json::from_value(wire.clone()).unwrap();
        assert_eq!(&back, value);
        assert_eq!(serde_json::to_value(&back).unwrap(), wire);
    }

    fn claude_profile() -> CliProfile {
        CliProfile {
            id: "claude".into(),
            display_name: "Claude Code".into(),
            programs: vec!["claude".into()],
            verified_version: Some("2.1.285".into()),
            identity: IdentitySource::Pinned {
                flag: "--session-id".into(),
            },
            resume: ResumeSpec::ByIdArgv {
                template: vec!["claude".into(), "--resume".into(), "{id}".into()],
            },
            resume_flag_aliases: vec!["-r".into()],
            session_choice_args: vec!["--continue".into(), "-c".into()],
            pty_args: vec!["--teammate-mode".into(), "in-process".into()],
            permission_prompt_args: vec!["--permission-prompt-tool".into(), "stdio".into()],
            account_isolation: AccountIsolation::EnvVar {
                name: "CLAUDE_CONFIG_DIR".into(),
            },
            auto_approve: AutoApprove::Flags {
                argv: vec!["--permission-mode".into(), "bypassPermissions".into()],
                detect: vec![
                    "--dangerously-skip-permissions".into(),
                    "bypassPermissions".into(),
                ],
            },
            graceful_exit: GracefulExit::TypedCommand {
                text: "/exit".into(),
            },
            handshake: HandshakePatterns {
                success: vec![],
                failure: vec![],
                success_regex: vec!["Claude Code v\\d".into()],
                failure_regex: vec!["No conversation found".into()],
                title_regex: Vec::new(),
            },
            usage_limit_phrases: vec!["usage limit reached".into()],
            transcript: TranscriptSpec::JsonlUnderConfigDir {
                subdir: "projects".into(),
            },
            structured_lane: StructuredLane::ClaudeStreamJson,
            typed_permission: CapabilityState::Unknown,
            turn_boundary_event: CapabilityState::Supported,
            rate_limit_event: CapabilityState::Unsupported,
            trust_dialog: TrustDialog::Prompt {
                markers: vec!["Do you trust the files in this folder?".into()],
                accept: "1".into(),
            },
            auth: vec![
                AuthMethod::CliLogin {
                    args: vec!["/login".into()],
                },
                AuthMethod::ApiKeyEnv {
                    vars: vec!["ANTHROPIC_API_KEY".into()],
                },
            ],
            restore_tier: RestoreTier::Full,
            install: InstallCommands {
                linux: Some("npm i -g @anthropic-ai/claude-code".into()),
                macos: None,
                windows: Some("npm i -g @anthropic-ai/claude-code".into()),
            },
            notes: vec!["macOS: same code path, unverified on hardware".into()],
        }
    }

    // ── defaults: Unknown everywhere a fact can be unverified ────────────

    #[test]
    fn capability_state_defaults_to_unknown() {
        assert_eq!(CapabilityState::default(), CapabilityState::Unknown);
    }

    #[test]
    fn failure_kind_defaults_to_unknown() {
        assert_eq!(FailureKind::default(), FailureKind::Unknown);
        assert_eq!(FailureCategory::default(), FailureCategory::Unknown);
    }

    #[test]
    fn every_unknown_bearing_fact_enum_defaults_to_unknown() {
        assert_eq!(IdentitySource::default(), IdentitySource::Unknown);
        assert_eq!(ResumeSpec::default(), ResumeSpec::Unknown);
        assert_eq!(AccountIsolation::default(), AccountIsolation::Unknown);
        assert_eq!(AutoApprove::default(), AutoApprove::Unknown);
        assert_eq!(GracefulExit::default(), GracefulExit::Unknown);
        assert_eq!(TranscriptSpec::default(), TranscriptSpec::Unknown);
        assert_eq!(StructuredLane::default(), StructuredLane::Unknown);
        assert_eq!(TrustDialog::default(), TrustDialog::Unknown);
    }

    #[test]
    fn restore_tier_defaults_to_the_conservative_claim() {
        assert_eq!(RestoreTier::default(), RestoreTier::TerminalOnly);
    }

    #[test]
    fn a_profile_stating_only_its_identity_reads_every_fact_as_unknown() {
        let p: CliProfile = serde_json::from_value(json!({
            "id": "future",
            "displayName": "Future CLI",
            "programs": ["future"],
        }))
        .unwrap();
        assert_eq!(p.verified_version, None);
        assert_eq!(p.identity, IdentitySource::Unknown);
        assert_eq!(p.resume, ResumeSpec::Unknown);
        assert!(p.resume_flag_aliases.is_empty());
        assert!(p.session_choice_args.is_empty());
        assert!(p.pty_args.is_empty());
        assert!(p.permission_prompt_args.is_empty());
        assert_eq!(p.account_isolation, AccountIsolation::Unknown);
        assert_eq!(p.auto_approve, AutoApprove::Unknown);
        assert_eq!(p.graceful_exit, GracefulExit::Unknown);
        assert_eq!(p.handshake, HandshakePatterns::default());
        assert!(p.usage_limit_phrases.is_empty());
        assert_eq!(p.transcript, TranscriptSpec::Unknown);
        assert_eq!(p.structured_lane, StructuredLane::Unknown);
        assert_eq!(p.typed_permission, CapabilityState::Unknown);
        assert_eq!(p.turn_boundary_event, CapabilityState::Unknown);
        assert_eq!(p.rate_limit_event, CapabilityState::Unknown);
        assert_eq!(p.trust_dialog, TrustDialog::Unknown);
        assert!(p.auth.is_empty());
        assert_eq!(p.restore_tier, RestoreTier::TerminalOnly);
        assert_eq!(p.install, InstallCommands::default());
        assert!(p.notes.is_empty());
    }

    #[test]
    fn a_profile_missing_its_identity_triple_is_rejected() {
        let err = serde_json::from_value::<CliProfile>(json!({ "id": "x" }));
        assert!(err.is_err());
    }

    // ── CapabilityState predicates (moved verbatim from model_catalog) ───

    #[test]
    fn capability_state_predicates() {
        use CapabilityState::*;
        assert!(Supported.is_supported());
        assert!(!Unsupported.is_supported());
        assert!(!Unknown.is_supported());

        assert!(!Supported.is_known_unsupported());
        assert!(Unsupported.is_known_unsupported());
        // The load-bearing case: Unknown is NOT a reason to downgrade.
        assert!(!Unknown.is_known_unsupported());

        assert!(!Supported.is_unknown());
        assert!(!Unsupported.is_unknown());
        assert!(Unknown.is_unknown());
    }

    // ── plain-string enums ───────────────────────────────────────────────

    #[test]
    fn capability_state_round_trips() {
        round_trip(&CapabilityState::Supported, json!("supported"));
        round_trip(&CapabilityState::Unsupported, json!("unsupported"));
        round_trip(&CapabilityState::Unknown, json!("unknown"));
    }

    #[test]
    fn restore_tier_uses_the_coord_wire_spelling() {
        round_trip(&RestoreTier::Full, json!("full"));
        round_trip(&RestoreTier::TerminalOnly, json!("terminal_only"));
        // The frontend's hyphenated spelling is a different vocabulary.
        assert!(serde_json::from_value::<RestoreTier>(json!("terminal-only")).is_err());
    }

    #[test]
    fn failure_kind_round_trips() {
        use FailureKind::*;
        let cases = [
            (RateLimited, "rate_limited"),
            (QuotaExhausted, "quota_exhausted"),
            (BudgetExhausted, "budget_exhausted"),
            (ContextExhausted, "context_exhausted"),
            (AuthRequired, "auth_required"),
            (AccessDenied, "access_denied"),
            (Overloaded, "overloaded"),
            (TransportLost, "transport_lost"),
            (ResumeFailed, "resume_failed"),
            (SpawnFailed, "spawn_failed"),
            (ProcessExited, "process_exited"),
            (BadRequest, "bad_request"),
            (InternalError, "internal_error"),
            (Unknown, "unknown"),
        ];
        for (kind, wire) in cases {
            round_trip(&kind, json!(wire));
        }
    }

    #[test]
    fn failure_category_round_trips() {
        use FailureCategory::*;
        let cases = [
            (Limit, "limit"),
            (Context, "context"),
            (Auth, "auth"),
            (Provider, "provider"),
            (Process, "process"),
            (Session, "session"),
            (Request, "request"),
            (Unknown, "unknown"),
        ];
        for (v, wire) in cases {
            round_trip(&v, json!(wire));
        }
    }

    #[test]
    fn failure_severity_round_trips() {
        round_trip(&FailureSeverity::Info, json!("info"));
        round_trip(&FailureSeverity::Warning, json!("warning"));
        round_trip(&FailureSeverity::Error, json!("error"));
    }

    #[test]
    fn failure_evidence_source_round_trips() {
        use FailureEvidenceSource::*;
        let cases = [
            (StructuredEvent, "structured_event"),
            (Hook, "hook"),
            (Stderr, "stderr"),
            (GridScrape, "grid_scrape"),
            (ExitStatus, "exit_status"),
            (Transcript, "transcript"),
            (HandshakeTimeout, "handshake_timeout"),
        ];
        for (v, wire) in cases {
            round_trip(&v, json!(wire));
        }
    }

    #[test]
    fn failure_confidence_round_trips() {
        round_trip(&FailureConfidence::Confirmed, json!("confirmed"));
        round_trip(&FailureConfidence::Hint, json!("hint"));
    }

    #[test]
    fn failure_action_round_trips() {
        use FailureAction::*;
        let cases = [
            (Retry, "retry"),
            (Login, "login"),
            (SwitchAccount, "switch_account"),
            (NewSession, "new_session"),
            (Resume, "resume"),
            (None, "none"),
        ];
        for (v, wire) in cases {
            round_trip(&v, json!(wire));
        }
    }

    #[test]
    fn recovery_policy_round_trips() {
        use RecoveryPolicy::*;
        let cases = [
            (Never, "never"),
            (BackoffThenRetry, "backoff_then_retry"),
            (WaitUntilReset, "wait_until_reset"),
            (MigrateAccount, "migrate_account"),
            (LoginThenResume, "login_then_resume"),
            (ResumeSameId, "resume_same_id"),
            (HandoffNewSession, "handoff_new_session"),
        ];
        for (v, wire) in cases {
            round_trip(&v, json!(wire));
        }
    }

    // ── kind-tagged fact enums ───────────────────────────────────────────

    #[test]
    fn identity_source_round_trips() {
        round_trip(
            &IdentitySource::Pinned {
                flag: "--session-id".into(),
            },
            json!({ "kind": "pinned", "flag": "--session-id" }),
        );
        round_trip(
            &IdentitySource::ReadBack {
                capture: "codex_session_file".into(),
            },
            json!({ "kind": "read_back", "capture": "codex_session_file" }),
        );
        round_trip(&IdentitySource::Unknown, json!({ "kind": "unknown" }));
    }

    #[test]
    fn resume_spec_round_trips() {
        round_trip(
            &ResumeSpec::ByIdArgv {
                template: vec!["codex".into(), "resume".into(), "{id}".into()],
            },
            json!({ "kind": "by_id_argv", "template": ["codex", "resume", "{id}"] }),
        );
        round_trip(&ResumeSpec::None, json!({ "kind": "none" }));
        round_trip(&ResumeSpec::Unknown, json!({ "kind": "unknown" }));
    }

    #[test]
    fn account_isolation_round_trips() {
        round_trip(
            &AccountIsolation::EnvVar {
                name: "CLAUDE_CONFIG_DIR".into(),
            },
            json!({ "kind": "env_var", "name": "CLAUDE_CONFIG_DIR" }),
        );
        round_trip(&AccountIsolation::HomeDir, json!({ "kind": "home_dir" }));
        round_trip(&AccountIsolation::None, json!({ "kind": "none" }));
        round_trip(&AccountIsolation::Unknown, json!({ "kind": "unknown" }));
    }

    #[test]
    fn auto_approve_round_trips() {
        round_trip(
            &AutoApprove::Flags {
                argv: vec!["--yolo".into()],
                detect: vec!["--yolo".into(), "--full-auto".into()],
            },
            json!({ "kind": "flags", "argv": ["--yolo"], "detect": ["--yolo", "--full-auto"] }),
        );
        round_trip(&AutoApprove::None, json!({ "kind": "none" }));
        round_trip(&AutoApprove::Unknown, json!({ "kind": "unknown" }));
    }

    #[test]
    fn graceful_exit_round_trips() {
        round_trip(
            &GracefulExit::TypedCommand {
                text: "/exit".into(),
            },
            json!({ "kind": "typed_command", "text": "/exit" }),
        );
        round_trip(&GracefulExit::Signal, json!({ "kind": "signal" }));
        round_trip(&GracefulExit::Unknown, json!({ "kind": "unknown" }));
    }

    #[test]
    fn transcript_spec_round_trips() {
        round_trip(
            &TranscriptSpec::JsonlUnderConfigDir {
                subdir: "projects".into(),
            },
            json!({ "kind": "jsonl_under_config_dir", "subdir": "projects" }),
        );
        round_trip(
            &TranscriptSpec::SessionFileGlob {
                glob: ".codex/sessions/**/rollout-*.jsonl".into(),
            },
            json!({ "kind": "session_file_glob", "glob": ".codex/sessions/**/rollout-*.jsonl" }),
        );
        round_trip(&TranscriptSpec::None, json!({ "kind": "none" }));
        round_trip(&TranscriptSpec::Unknown, json!({ "kind": "unknown" }));
    }

    #[test]
    fn structured_lane_round_trips() {
        round_trip(
            &StructuredLane::ClaudeStreamJson,
            json!({ "kind": "claude_stream_json" }),
        );
        round_trip(
            &StructuredLane::CodexAppServer,
            json!({ "kind": "codex_app_server" }),
        );
        round_trip(&StructuredLane::Acp, json!({ "kind": "acp" }));
        round_trip(&StructuredLane::None, json!({ "kind": "none" }));
        round_trip(&StructuredLane::Unknown, json!({ "kind": "unknown" }));
    }

    #[test]
    fn trust_dialog_round_trips() {
        round_trip(
            &TrustDialog::Prompt {
                markers: vec!["trust the files".into()],
                accept: "1".into(),
            },
            json!({ "kind": "prompt", "markers": ["trust the files"], "accept": "1" }),
        );
        round_trip(&TrustDialog::None, json!({ "kind": "none" }));
        round_trip(&TrustDialog::Unknown, json!({ "kind": "unknown" }));
    }

    #[test]
    fn auth_method_round_trips() {
        round_trip(
            &AuthMethod::CliLogin {
                args: vec!["login".into()],
            },
            json!({ "kind": "cli_login", "args": ["login"] }),
        );
        round_trip(
            &AuthMethod::ApiKeyEnv {
                vars: vec!["OPENAI_API_KEY".into()],
            },
            json!({ "kind": "api_key_env", "vars": ["OPENAI_API_KEY"] }),
        );
    }

    // ── structs ──────────────────────────────────────────────────────────

    #[test]
    fn handshake_patterns_round_trip_camel_case() {
        round_trip(
            &HandshakePatterns {
                success: vec!["ok".into()],
                failure: vec!["bad".into()],
                success_regex: vec!["^ok$".into()],
                failure_regex: vec!["^bad$".into()],
                title_regex: vec!["^ok title$".into()],
            },
            json!({
                "success": ["ok"],
                "failure": ["bad"],
                "successRegex": ["^ok$"],
                "failureRegex": ["^bad$"],
                "titleRegex": ["^ok title$"],
            }),
        );
    }

    #[test]
    fn handshake_patterns_omit_an_empty_title_regex() {
        let json = serde_json::to_value(HandshakePatterns::default()).unwrap();
        assert!(json.get("titleRegex").is_none(), "{json}");
        let back: HandshakePatterns = serde_json::from_value(json!({})).unwrap();
        assert!(back.title_regex.is_empty());
    }

    #[test]
    fn install_commands_omit_unrecorded_platforms() {
        round_trip(
            &InstallCommands {
                linux: Some("npm i -g x".into()),
                macos: None,
                windows: None,
            },
            json!({ "linux": "npm i -g x" }),
        );
    }

    #[test]
    fn cli_profile_round_trips() {
        let p = claude_profile();
        round_trip(
            &p,
            json!({
                "id": "claude",
                "displayName": "Claude Code",
                "programs": ["claude"],
                "verifiedVersion": "2.1.285",
                "identity": { "kind": "pinned", "flag": "--session-id" },
                "resume": { "kind": "by_id_argv", "template": ["claude", "--resume", "{id}"] },
                "resumeFlagAliases": ["-r"],
                "sessionChoiceArgs": ["--continue", "-c"],
                "ptyArgs": ["--teammate-mode", "in-process"],
                "permissionPromptArgs": ["--permission-prompt-tool", "stdio"],
                "accountIsolation": { "kind": "env_var", "name": "CLAUDE_CONFIG_DIR" },
                "autoApprove": {
                    "kind": "flags",
                    "argv": ["--permission-mode", "bypassPermissions"],
                    "detect": ["--dangerously-skip-permissions", "bypassPermissions"],
                },
                "gracefulExit": { "kind": "typed_command", "text": "/exit" },
                "handshake": {
                    "success": [],
                    "failure": [],
                    "successRegex": ["Claude Code v\\d"],
                    "failureRegex": ["No conversation found"],
                },
                "usageLimitPhrases": ["usage limit reached"],
                "transcript": { "kind": "jsonl_under_config_dir", "subdir": "projects" },
                "structuredLane": { "kind": "claude_stream_json" },
                "typedPermission": "unknown",
                "turnBoundaryEvent": "supported",
                "rateLimitEvent": "unsupported",
                "trustDialog": {
                    "kind": "prompt",
                    "markers": ["Do you trust the files in this folder?"],
                    "accept": "1",
                },
                "auth": [
                    { "kind": "cli_login", "args": ["/login"] },
                    { "kind": "api_key_env", "vars": ["ANTHROPIC_API_KEY"] },
                ],
                "restoreTier": "full",
                "install": {
                    "linux": "npm i -g @anthropic-ai/claude-code",
                    "windows": "npm i -g @anthropic-ai/claude-code",
                },
                "notes": ["macOS: same code path, unverified on hardware"],
            }),
        );
    }

    #[test]
    fn session_failure_round_trips() {
        let f = SessionFailure {
            id: "7f1c".into(),
            kind: FailureKind::QuotaExhausted,
            category: FailureCategory::Limit,
            severity: FailureSeverity::Error,
            title: "Usage limit reached".into(),
            details: Some("The account's 5-hour window is used up.".into()),
            reason: Some("usage limit reached".into()),
            provider: "claude".into(),
            account: Some("acct-2".into()),
            turn_id: Some("turn-9".into()),
            reset_at: Some("2026-09-30T18:00:00Z".into()),
            evidence: FailureEvidence {
                source: FailureEvidenceSource::GridScrape,
                confidence: FailureConfidence::Hint,
            },
            actions: vec![FailureAction::SwitchAccount, FailureAction::Retry],
            recovery_policy: RecoveryPolicy::MigrateAccount,
        };
        round_trip(
            &f,
            json!({
                "id": "7f1c",
                "kind": "quota_exhausted",
                "category": "limit",
                "severity": "error",
                "title": "Usage limit reached",
                "details": "The account's 5-hour window is used up.",
                "reason": "usage limit reached",
                "provider": "claude",
                "account": "acct-2",
                "turnId": "turn-9",
                "resetAt": "2026-09-30T18:00:00Z",
                "evidence": { "source": "grid_scrape", "confidence": "hint" },
                "actions": ["switch_account", "retry"],
                "recoveryPolicy": "migrate_account",
            }),
        );
    }

    #[test]
    fn session_failure_minimal_omits_absent_optionals() {
        let f = SessionFailure {
            id: "a".into(),
            kind: FailureKind::Unknown,
            category: FailureCategory::Unknown,
            severity: FailureSeverity::Warning,
            title: "Session stopped".into(),
            details: None,
            reason: None,
            provider: "codex".into(),
            account: None,
            turn_id: None,
            reset_at: None,
            evidence: FailureEvidence {
                source: FailureEvidenceSource::ExitStatus,
                confidence: FailureConfidence::Confirmed,
            },
            actions: vec![],
            recovery_policy: RecoveryPolicy::Never,
        };
        round_trip(
            &f,
            json!({
                "id": "a",
                "kind": "unknown",
                "category": "unknown",
                "severity": "warning",
                "title": "Session stopped",
                "provider": "codex",
                "evidence": { "source": "exit_status", "confidence": "confirmed" },
                "actions": [],
                "recoveryPolicy": "never",
            }),
        );
        // `actions` may be absent on the wire.
        let back: SessionFailure = serde_json::from_value(json!({
            "id": "a", "kind": "unknown", "category": "unknown", "severity": "warning",
            "title": "Session stopped", "provider": "codex",
            "evidence": { "source": "exit_status", "confidence": "confirmed" },
            "recoveryPolicy": "never",
        }))
        .unwrap();
        assert_eq!(back, f);
    }

    // ── JSON Schema presence ─────────────────────────────────────────────

    /// The string values a unit-variant enum's schema admits. schemars emits
    /// `{"enum": [...]}` for undocumented variants and `{"oneOf": [{"const":
    /// ...}, ...]}` when variants carry doc comments; accept either.
    fn string_enum_values(def: &Value) -> Vec<String> {
        if let Some(values) = def["enum"].as_array() {
            return values
                .iter()
                .map(|v| v.as_str().unwrap().to_owned())
                .collect();
        }
        def["oneOf"]
            .as_array()
            .unwrap_or_else(|| panic!("not a string enum schema: {def}"))
            .iter()
            .map(|arm| {
                arm["const"]
                    .as_str()
                    .unwrap_or_else(|| panic!("arm has no string const: {arm}"))
                    .to_owned()
            })
            .collect()
    }

    /// The schema a consumer generates bindings from must describe the wire
    /// shape the serde tests above pin: camelCase properties, the required
    /// identity triple, and a `kind` discriminator on every tagged fact.
    #[test]
    fn cli_profile_schema_describes_the_wire_shape() {
        let schema = serde_json::to_value(schemars::schema_for!(CliProfile)).unwrap();
        assert_eq!(schema["title"], "CliProfile");
        let props = schema["properties"].as_object().unwrap();
        for key in [
            "id",
            "displayName",
            "programs",
            "verifiedVersion",
            "identity",
            "resume",
            "resumeFlagAliases",
            "sessionChoiceArgs",
            "ptyArgs",
            "permissionPromptArgs",
            "accountIsolation",
            "autoApprove",
            "gracefulExit",
            "handshake",
            "usageLimitPhrases",
            "transcript",
            "structuredLane",
            "typedPermission",
            "turnBoundaryEvent",
            "rateLimitEvent",
            "trustDialog",
            "auth",
            "restoreTier",
            "install",
            "notes",
        ] {
            assert!(props.contains_key(key), "CliProfile schema lacks `{key}`");
        }
        let mut required: Vec<&str> = schema["required"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        required.sort_unstable();
        assert_eq!(required, ["displayName", "id", "programs"]);

        let defs = schema["$defs"].as_object().unwrap();
        for tagged in [
            "IdentitySource",
            "ResumeSpec",
            "AccountIsolation",
            "AutoApprove",
            "GracefulExit",
            "TranscriptSpec",
            "StructuredLane",
            "TrustDialog",
            "AuthMethod",
        ] {
            let arms = defs[tagged]["oneOf"]
                .as_array()
                .unwrap_or_else(|| panic!("{tagged} is not a oneOf union"));
            for arm in arms {
                assert!(
                    arm["properties"]["kind"].is_object(),
                    "{tagged} arm lacks a `kind` discriminator: {arm}"
                );
            }
        }
        assert_eq!(
            string_enum_values(&defs["CapabilityState"]),
            ["supported", "unsupported", "unknown"]
        );
    }

    #[test]
    fn session_failure_schema_describes_the_wire_shape() {
        let schema = serde_json::to_value(schemars::schema_for!(SessionFailure)).unwrap();
        assert_eq!(schema["title"], "SessionFailure");
        let mut required: Vec<&str> = schema["required"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        required.sort_unstable();
        assert_eq!(
            required,
            [
                "category",
                "evidence",
                "id",
                "kind",
                "provider",
                "recoveryPolicy",
                "severity",
                "title",
            ]
        );
        let defs = schema["$defs"].as_object().unwrap();
        assert_eq!(string_enum_values(&defs["FailureKind"]).len(), 14);
        assert_eq!(string_enum_values(&defs["FailureCategory"]).len(), 8);
        assert_eq!(string_enum_values(&defs["FailureSeverity"]).len(), 3);
        assert_eq!(string_enum_values(&defs["FailureEvidenceSource"]).len(), 7);
        assert_eq!(string_enum_values(&defs["FailureConfidence"]).len(), 2);
        assert_eq!(string_enum_values(&defs["FailureAction"]).len(), 6);
        assert_eq!(string_enum_values(&defs["RecoveryPolicy"]).len(), 7);
    }
}
