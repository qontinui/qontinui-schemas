//! Project registry + dashboard snapshot DTOs.
//!
//! Extracted from `qontinui-runner/src-tauri/src/settings.rs` (where
//! `SavedProject` lived as a runner-local 4-field struct) as part of the
//! Runner Projects Dashboard. This module is the wire-format source of truth
//! for:
//!
//! - [`SavedProject`] — the user-curated project registry persisted in
//!   `settings.json` under `saved_projects` and surfaced through the
//!   `list/save/add/remove/discover_saved_project(s)` Tauri commands.
//! - [`ProjectSnapshot`] — the *joined* dashboard view: one server-side struct
//!   that folds together managed processes, live terminal sessions, recent AI
//!   sessions, git state, pending questions, health and spend for a single
//!   project root. Computed by the runner's `projects::snapshot` module so the
//!   Projects page is one round-trip, not eight.
//!
//! Wire-format notes:
//! - Everything here serializes `camelCase`, matching the pre-extraction
//!   `SavedProject` wire (`#[serde(rename_all = "camelCase")]`), so TS
//!   consumers see `frontPageUrl`, `processIds`, `terminalPageId`, …
//! - snake_case `#[serde(alias = …)]`es are kept on every field so a
//!   Rust/Python producer writing snake_case still deserializes — same
//!   convention as [`crate::process_management`].
//! - Timestamps are Unix epoch **milliseconds** (`i64`), not ISO 8601
//!   strings, because every producer here (`recorded_at`, `created_at`, git
//!   `%ct`) is already numeric and the dashboard sorts on them.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::process_management::ProcessState;

// ============================================================================
// SavedProject
// ============================================================================

/// A project the user has told the runner about (typically via the setup
/// wizard's project picker, the new-project flow, or the Projects
/// dashboard's "find on disk" discovery).
///
/// Persisted to `settings.json` under the `saved_projects` key.
///
/// `id` is the stable key: a project's `path` can move on disk (renamed
/// folder, relocated workspace) but its identity must not. Entries persisted
/// before `id` existed are backfilled with a freshly-minted UUID on first
/// load — `#[serde(default)]` exists only so that load succeeds, never as a
/// legitimate steady-state value.
///
/// `project_type` is deliberately a free-form `String` so a new framework
/// never requires a schema change.
/// `Default` exists so a producer that only knows a few fields can write
/// `SavedProject { path, name, ..Default::default() }` — the new-project flow
/// and the setup wizard both do. A defaulted `id` is the empty string, which
/// the registry treats as "mint me one" (see the field docs), never as a key.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct SavedProject {
    /// Stable identifier (UUID). Survives a path move; the registry key.
    #[serde(default, alias = "id")]
    pub id: String,
    /// Absolute path to the project root.
    #[serde(alias = "path")]
    pub path: String,
    /// Human-friendly display name (usually the directory basename).
    #[serde(alias = "name")]
    pub name: String,
    /// Framework/language tag, e.g. "react", "python", "rust", "node".
    #[serde(alias = "project_type")]
    pub project_type: String,
    /// Manifest file that identified the project (e.g. "package.json").
    #[serde(alias = "manifest")]
    pub manifest: String,

    /// One-line plain-English description ("Website for the pizzeria, with
    /// menu + ordering"). Shown on the project card under the name.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "description"
    )]
    pub description: Option<String>,
    /// Emoji used as the card's visual identity (e.g. "🍕").
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "emoji")]
    pub emoji: Option<String>,
    /// Accent colour for the card, as a CSS colour string.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "color")]
    pub color: Option<String>,
    /// The project's front page ("http://localhost:3000"). Seeded from the
    /// setup wizard's dev-server port detection; user-correctable.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "front_page_url"
    )]
    pub front_page_url: Option<String>,
    /// Ids of the `ProcessConfig` entries this project owns.
    #[serde(default, alias = "process_ids")]
    pub process_ids: Vec<String>,
    /// Bound Terminal page id.
    ///
    /// **A hint, not a handle.** Terminal pages persist to the frontend's
    /// `instanceStorage` (localStorage, port-namespaced), which Rust cannot
    /// read, so this id can name a page that does not exist in the current
    /// window. Activation must treat an unknown id as "not created yet" —
    /// create a page named after the project and rewrite the id — and must
    /// never fail on a dangling value.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "terminal_page_id"
    )]
    pub terminal_page_id: Option<String>,
    /// Terminal zone profile to restore when the project is activated.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "zone_profile"
    )]
    pub zone_profile: Option<String>,
    /// `owner/name` GitHub slug, from `repo_detection::detect_repo_slug`.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "repo_slug")]
    pub repo_slug: Option<String>,
    /// The user's own free-form notes about the project.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "notes")]
    pub notes: Option<String>,
    /// Pinned projects sort first and render as sidebar entries.
    #[serde(default, alias = "pinned")]
    pub pinned: bool,
    /// Unix epoch milliseconds of the last time the project was activated.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "last_opened_ms"
    )]
    pub last_opened_ms: Option<i64>,
}

// ============================================================================
// ProjectSnapshot members
// ============================================================================

/// A managed process attributed to a project, reduced to what the dashboard
/// renders. Full detail stays in `ProcessStatus` / `ProcessConfig`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct ProcessStatusLite {
    /// `ProcessConfig.id`.
    #[serde(alias = "id")]
    pub id: String,
    /// Human-readable process name ("Website").
    #[serde(alias = "name")]
    pub name: String,
    /// Lifecycle state as reported by the process manager.
    #[serde(alias = "state")]
    pub state: ProcessState,
    /// Health port, when the process declares one.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "health_port"
    )]
    pub health_port: Option<u16>,
    /// Whether the health port is currently responding. `None` when the
    /// process declares no health port.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "port_healthy"
    )]
    pub port_healthy: Option<bool>,
    /// Working directory — the field that attributed this process to the
    /// project in the first place.
    #[serde(alias = "cwd")]
    pub cwd: String,
}

/// Where a [`SessionLite`] was observed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SessionSource {
    /// A live PTY session in the runner's terminal manager, attributed by
    /// its `working_dir`.
    Terminal,
    /// A past or present AI session, attributed by the files it touched
    /// (`coord.session_touched_files`).
    TouchedFiles,
}

/// One session attributed to a project — either a live terminal or an AI
/// session reconstructed from its touched files.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct SessionLite {
    /// `task_runs.id` for AI sessions, the terminal id for live terminals.
    #[serde(alias = "id")]
    pub id: String,
    /// Session name — `task_runs.task_name` or the terminal's title.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "name")]
    pub name: Option<String>,
    /// `task_runs.status` ("running", "completed", …) for AI sessions;
    /// `None` for terminals (liveness is `is_alive` in the terminal list).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "status")]
    pub status: Option<String>,
    /// Where this row came from.
    #[serde(alias = "source")]
    pub source: SessionSource,
    /// Unix epoch milliseconds of the most recent observed activity —
    /// `max(recorded_at)` over the session's touched files, or the
    /// terminal's creation time.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "last_activity_ms"
    )]
    pub last_activity_ms: Option<i64>,
    /// Number of distinct files this session touched **under this project
    /// root**. Zero for terminal-sourced rows.
    #[serde(default, alias = "files_touched")]
    pub files_touched: i64,
    /// Working directory, when known (terminal sessions always carry one).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "working_dir"
    )]
    pub working_dir: Option<String>,
}

/// One commit in [`GitLite::last_commits`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct GitCommitLite {
    /// Abbreviated commit sha.
    #[serde(alias = "sha")]
    pub sha: String,
    /// Commit subject line.
    #[serde(alias = "subject")]
    pub subject: String,
    /// Unix epoch milliseconds of the commit timestamp.
    #[serde(alias = "committed_ms")]
    pub committed_ms: i64,
}

/// Git state of the project root. `None` on the snapshot when the root is
/// not a git working tree (or git is unavailable).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct GitLite {
    /// Current branch. `None` on a detached HEAD.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "branch")]
    pub branch: Option<String>,
    /// Number of `git status --porcelain` entries (uncommitted changes).
    #[serde(default, alias = "dirty_count")]
    pub dirty_count: u32,
    /// Newest commits first.
    #[serde(default, alias = "last_commits")]
    pub last_commits: Vec<GitCommitLite>,
}

/// A question an agent is blocked on, awaiting the user's answer.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct PendingQuestion {
    /// `deferred_questions.id`.
    #[serde(alias = "id")]
    pub id: String,
    /// The session that asked.
    #[serde(alias = "task_run_id")]
    pub task_run_id: String,
    /// The question text, as shown to the user.
    #[serde(alias = "question")]
    pub question: String,
    /// Risk level the asking agent assigned ("low" | "medium" | "high").
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "risk_level")]
    pub risk_level: Option<String>,
    /// Unix epoch milliseconds the question was raised.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "created_at_ms"
    )]
    pub created_at_ms: Option<i64>,
}

/// Traffic-light level for [`HealthLite`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HealthLevel {
    /// Everything the project declares is running and responding.
    Green,
    /// Something is degraded but not broken — a process is starting, or a
    /// health port is not answering yet.
    Amber,
    /// A declared process has failed.
    Red,
    /// Nothing observable — the project declares no processes, so there is
    /// nothing to be healthy *about*.
    #[default]
    Unknown,
}

/// One green/amber/red answer to "is it broken?", plus the plain-English
/// reason behind it.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct HealthLite {
    /// The traffic light.
    #[serde(default, alias = "level")]
    pub level: HealthLevel,
    /// Why — a sentence a non-developer can read
    /// ("The website still starts up fine").
    #[serde(default, alias = "reason")]
    pub reason: String,
    /// Names of the processes driving an amber/red verdict.
    #[serde(default, alias = "failing_processes")]
    pub failing_processes: Vec<String>,
}

// ============================================================================
// ProjectSnapshot
// ============================================================================

/// The joined dashboard view of one project.
///
/// Every field on both the grid card and the detail page is a projection of
/// this single struct — computed server-side in one call so the page never
/// fans out.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct ProjectSnapshot {
    /// The registry entry this snapshot describes.
    #[serde(alias = "project")]
    pub project: SavedProject,
    /// Managed processes whose `cwd` is under the project root.
    #[serde(default, alias = "processes")]
    pub processes: Vec<ProcessStatusLite>,
    /// Live terminal sessions whose `working_dir` is under the project root.
    #[serde(default, alias = "live_sessions")]
    pub live_sessions: Vec<SessionLite>,
    /// Recent AI sessions, newest first, attributed via
    /// `coord.session_touched_files`.
    #[serde(default, alias = "recent_sessions")]
    pub recent_sessions: Vec<SessionLite>,
    /// Git state of the root, when it is a git working tree.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "git")]
    pub git: Option<GitLite>,
    /// Questions waiting on the user, from `deferred_questions`.
    #[serde(default, alias = "questions")]
    pub questions: Vec<PendingQuestion>,
    /// The project's traffic light.
    #[serde(default, alias = "health")]
    pub health: HealthLite,
    /// Rolling 7-day spend attributed to this project, in USD. `None` when
    /// cost data is unavailable — distinct from `Some(0.0)`, which means
    /// "measured, and it was free".
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "spend_7d_usd"
    )]
    pub spend_7d_usd: Option<f64>,
    /// Unix epoch milliseconds of the newest activity of any kind.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "last_activity_ms"
    )]
    pub last_activity_ms: Option<i64>,
}
