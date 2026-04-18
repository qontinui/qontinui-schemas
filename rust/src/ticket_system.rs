//! Ticket system DTO types.
//!
//! Wire-format types for the ticket-system integration: the provider-source
//! enum, the fetched ticket + comment shapes, and the provider configuration
//! that is both persisted in the database and round-tripped over MCP/HTTP for
//! the trigger-watcher control plane.
//!
//! These are ports of the shape-bearing portion of
//! `qontinui-runner/src-tauri/src/ticket_system/types.rs`. Behavior — the
//! `TicketProvider` trait and its GitHub/Linear/Jira implementations, the
//! sync service, and the `to_json_with_token` / `from_json_with_token`
//! serialization helpers — stays in the runner. This module is data-only.
//!
//! ## Security note on `TicketProviderConfig`
//!
//! `TicketProviderConfig::api_token` is a secret. It is persisted in the
//! database (inside `workflow_triggers.trigger_config` and
//! `ticket_provider_configs.config_json`) so that watcher processes can
//! reconstruct a provider across restarts. Consumers that surface this type
//! to user-facing UIs MUST redact the `api_token` field before serialization
//! out over the MCP/HTTP API boundary; the wire type itself still serializes
//! it because the same shape is used for at-rest persistence. Treat it as
//! secret-in-transit and secret-at-rest at every boundary.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ============================================================================
// Provider source
// ============================================================================

/// Which ticket provider a config / ticket refers to.
///
/// Serialized as the lowercase provider tag (`"github"`, `"linear"`, `"jira"`)
/// to match the on-the-wire tags used by the watcher config, the MCP API
/// layer, and the runner's DB storage keys. Variants are explicitly renamed
/// because `rename_all = "snake_case"` would emit `"git_hub"` for the
/// `GitHub` variant, which would drift from the existing persisted rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum TicketSource {
    /// GitHub Issues (REST v3).
    #[serde(rename = "github")]
    GitHub,
    /// Linear Issues (GraphQL).
    #[serde(rename = "linear")]
    Linear,
    /// Atlassian Jira (REST). Provider implementation is not yet wired; the
    /// variant exists for forward compatibility of the config schema.
    #[serde(rename = "jira")]
    Jira,
}

// ============================================================================
// Ticket state
// ============================================================================

/// Abstract, provider-neutral ticket lifecycle state.
///
/// Maps onto provider-specific states (`open` / `closed` on GitHub, workflow
/// state types on Linear, etc.) inside the runner's provider implementations.
/// Serialized as `snake_case`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TicketState {
    /// Ticket is open and ready for work.
    Open,
    /// A task is actively working on this ticket.
    InProgress,
    /// Work is done and the ticket is awaiting review/close.
    Done,
    /// Ticket is closed.
    Closed,
}

// ============================================================================
// Ticket
// ============================================================================

/// A ticket fetched from an external provider.
///
/// Identity is carried by `(source, external_id)` — `external_id` is the
/// provider-specific identifier (GitHub issue number, Linear identifier, Jira
/// key) rather than a runner-assigned UUID.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct Ticket {
    /// Provider-assigned external ID (issue number, ticket key, etc.).
    #[serde(alias = "external_id")]
    pub external_id: String,
    /// Which provider this ticket came from.
    #[serde(alias = "source")]
    pub source: TicketSource,
    /// Ticket title / summary.
    #[serde(alias = "title")]
    pub title: String,
    /// Ticket body / description (Markdown for GitHub/Linear, wiki-markup on Jira).
    #[serde(alias = "body")]
    pub body: String,
    /// Labels applied to the ticket by the provider.
    #[serde(default, skip_serializing_if = "Vec::is_empty", alias = "labels")]
    pub labels: Vec<String>,
    /// Assignee username / handle, if any.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "assignee")]
    pub assignee: Option<String>,
    /// Canonical URL to the ticket in the provider's UI.
    #[serde(alias = "url")]
    pub url: String,
    /// Abstract lifecycle state.
    #[serde(alias = "state")]
    pub state: TicketState,
    /// ISO 8601 timestamp when the ticket was created.
    #[serde(alias = "created_at")]
    pub created_at: String,
    /// ISO 8601 timestamp when the ticket was last updated.
    #[serde(alias = "updated_at")]
    pub updated_at: String,
}

// ============================================================================
// Ticket comment
// ============================================================================

/// A comment on a ticket.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct TicketComment {
    /// Provider-assigned comment ID.
    #[serde(alias = "id")]
    pub id: String,
    /// Comment author's username / handle.
    #[serde(alias = "author")]
    pub author: String,
    /// Comment body (Markdown for GitHub/Linear, wiki-markup on Jira).
    #[serde(alias = "body")]
    pub body: String,
    /// ISO 8601 timestamp when the comment was created.
    #[serde(alias = "created_at")]
    pub created_at: String,
}

// ============================================================================
// Provider configuration
// ============================================================================

/// Configuration for a ticket provider watcher.
///
/// Persisted in `workflow_triggers.trigger_config` and
/// `ticket_provider_configs.config_json` so that watcher processes can be
/// reconstructed across runner restarts.
///
/// **Security**: `api_token` is a secret. This struct serializes it so that
/// the same shape works for at-rest persistence, but MCP / HTTP API
/// responses exposing this config to end-user UIs MUST redact the token
/// before returning. See the module-level doc for the full policy.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct TicketProviderConfig {
    /// Which provider this config targets.
    #[serde(alias = "source")]
    pub source: TicketSource,
    /// API token for the provider. **Secret** — see security note above.
    /// Persisted in the DB so the watcher can reconstruct a provider across
    /// restarts; redact before exposing over UI-facing APIs.
    #[serde(alias = "api_token")]
    pub api_token: String,
    /// Provider-specific target:
    /// - GitHub: `"owner/repo"`
    /// - Linear: team key (e.g. `"ENG"`)
    /// - Jira: project key
    #[serde(alias = "target")]
    pub target: String,
    /// Labels / filters that mark a ticket as "actionable" (i.e. eligible to
    /// spawn a workflow task). All listed labels must be present for a match.
    #[serde(default, skip_serializing_if = "Vec::is_empty", alias = "actionable_labels")]
    pub actionable_labels: Vec<String>,
    /// ID of the workflow to spawn for matched tickets.
    #[serde(alias = "workflow_id")]
    pub workflow_id: String,
    /// Poll interval in seconds. Default: 60.
    #[serde(default = "default_poll_interval", alias = "poll_interval_seconds")]
    pub poll_interval_seconds: u64,
    /// Whether to update the remote ticket's state when the spawned task
    /// completes. Default: true.
    #[serde(default = "default_true", alias = "update_on_completion")]
    pub update_on_completion: bool,
}

fn default_poll_interval() -> u64 {
    60
}

fn default_true() -> bool {
    true
}
