//! Account-scoped agent-command overrides + their append-only version chain.
//!
//! An *agent command* is a markdown procedure the runner provisions into a
//! spawned session's `.claude/commands/<name>.md` (today `/vet-plan` and
//! `/implement-plan`; nothing here may assume two). The **defaults ship
//! embedded in the runner binary** (`include_str!`), so an unauthenticated,
//! offline, or first-run session always resolves to a working command and the
//! network is never on the critical path.
//!
//! The types in this module model the **optional account layer on top of that
//! default** — they are *overrides*, never the defaults themselves:
//!
//! ```text
//! resolution order (per command name):
//!     user override (account, this module)  →  embedded default (runner binary)
//! ```
//!
//! Consequences worth stating explicitly, because they are easy to get wrong:
//!
//! - There is **no row for a default**. An [`AgentCommand`] exists only when an
//!   account has customized that command. "Reset to default" *deletes* the
//!   override row; it never deletes or mutates a default.
//! - Override is **by name, replacing** — a stored `vet-plan` REPLACES the
//!   embedded `vet-plan`. It does not coexist with it. (This is the deliberate
//!   divergence from the runner's `SkillRegistry::all()`, which *concatenates*
//!   builtin and user entries. Two entries cannot both become
//!   `.claude/commands/vet-plan.md`.)
//! - Scope is the **organization**, with `created_by_user_id` kept for
//!   attribution only. The storage key is `(organization_id, name)` — one
//!   override per command per account — so a later per-user layer stays purely
//!   additive (`user → org → embedded default`) with no data migration.
//!
//! ## The version chain is append-only
//!
//! Every edit **appends** an [`AgentCommandVersion`] row; the head is
//! [`AgentCommand::current_version`]. History is never mutated and never
//! deleted:
//!
//! - **Revert writes a NEW version** whose `body` equals an older one, with
//!   [`AgentCommandVersion::restored_from`] recording which version it was
//!   restored from. Reverting to v2 from a head of v5 produces v6, not a
//!   rewind to v2. This mirrors the behavior the backend's
//!   `version_history_service.restore_from_version` already implements for
//!   `PromptTemplateVersion`.
//! - `version_number` is monotonic per command and is enforced by a DB unique
//!   constraint on `(agent_command_id, version_number)`, not by application
//!   code alone — that constraint is what makes "append-only" true rather than
//!   aspirational under concurrent writes.
//!
//! ## Wire-format notes
//!
//! Per the crate-level conventions (`lib.rs`):
//!
//! - UUIDs are `String` on the wire, not `uuid::Uuid`.
//! - Timestamps are ISO 8601 (RFC 3339) `String`s — no `chrono` in this layer.
//! - Optional fields use `#[serde(default, skip_serializing_if =
//!   "Option::is_none")]` so absence and `null` round-trip faithfully.
//! - Field names stay **snake_case** (no `rename_all = "camelCase"`), matching
//!   [`crate::memory`]'s reasoning: the producing surface is the qontinui-web
//!   FastAPI layer, whose sibling skills API (`SkillResponse`) already speaks
//!   `created_by_user_id` / `organization_id` / `is_shared` / `created_at`.
//!   Renaming would fork the wire against its closest prior art for no gain.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// AgentCommand
// ---------------------------------------------------------------------------

/// An account-scoped **override** of a runner-embedded default agent command.
///
/// Presence of this record means "this organization has customized the command
/// named `name`"; absence means the embedded default applies. See the module
/// docs for the full resolution order and the delete-is-reset semantics.
///
/// `body` is the whole markdown procedure — the exact bytes written to
/// `.claude/commands/<name>.md`. It is **untrusted remote content** from the
/// runner's point of view: it is markdown rather than code, but it is
/// instructions to an agent, so consumers fail soft (a malformed override falls
/// back to the embedded default and warns) and never fetch cross-org.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AgentCommand {
    /// Override record id (UUID v4 string).
    pub id: String,

    /// Owning organization (`auth.organizations.id`). `None` where the row has
    /// been orphaned by an organization deletion (`ondelete SET NULL`); such a
    /// row is not resolvable by any account and behaves as no override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,

    /// User who last authored this override. **Attribution only** — it does not
    /// scope visibility; [`organization_id`](Self::organization_id) does.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by_user_id: Option<String>,

    /// The command slug, e.g. `vet-plan` or `implement-plan`. This is the
    /// override key: it must equal the name of the embedded default it
    /// replaces, and it is the filename stem under `.claude/commands/`.
    pub name: String,

    /// The full markdown body of the command.
    pub body: String,

    /// Content hash of [`body`](Self::body), for cheap change detection
    /// (cache invalidation, diff-against-default). `None` on rows written
    /// before a checksum was computed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,

    /// Whether this override is visible to the whole organization rather than
    /// only its author. Absent means `false`.
    #[serde(default)]
    pub is_shared: bool,

    /// Head of the version chain — the `version_number` of the
    /// [`AgentCommandVersion`] whose body this record currently carries.
    /// Starts at 1 and only ever increases (a revert bumps it; see the module
    /// docs).
    pub current_version: i32,

    /// ISO 8601 (RFC 3339) creation timestamp.
    pub created_at: String,

    /// ISO 8601 (RFC 3339) last-modification timestamp.
    pub updated_at: String,
}

// ---------------------------------------------------------------------------
// AgentCommandVersion
// ---------------------------------------------------------------------------

/// One immutable entry in an [`AgentCommand`]'s append-only version chain.
///
/// Rows are **only ever inserted**. An edit appends the next `version_number`;
/// a revert appends a new version whose `body` is copied from an older one and
/// whose [`restored_from`](Self::restored_from) names that older version.
/// Nothing in the chain is updated or deleted, so the chain is the complete
/// edit history and "revert" is itself a recorded edit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AgentCommandVersion {
    /// Version-row id (UUID v4 string).
    pub id: String,

    /// The [`AgentCommand`] this version belongs to. Deleting the override
    /// cascades to its versions — resetting to the default discards the
    /// account's edit history along with the override.
    pub agent_command_id: String,

    /// Monotonic version number within this command, starting at 1. Unique per
    /// `(agent_command_id, version_number)`; the DB constraint — not
    /// application code — is what rejects a duplicate under concurrent appends.
    pub version_number: i32,

    /// The full markdown body as of this version.
    pub body: String,

    /// Content hash of [`body`](Self::body) at this version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,

    /// User who authored this version. Attribution only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by_user_id: Option<String>,

    /// Free-text note describing what changed, supplied by the editor. Revert
    /// writes a generated one (e.g. `"Restored from version 2"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub change_description: Option<String>,

    /// Provenance for a revert: the `version_number` this version's body was
    /// copied from. `None` for an ordinary edit. Present only on versions
    /// produced by a restore, which is how the UI distinguishes "edited" from
    /// "reverted to v2" in the history list.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restored_from: Option<i32>,

    /// ISO 8601 (RFC 3339) creation timestamp. Versions are immutable, so
    /// there is deliberately no `updated_at`.
    pub created_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_command_omits_absent_optionals() {
        let cmd = AgentCommand {
            id: "11111111-1111-4111-8111-111111111111".to_string(),
            organization_id: None,
            created_by_user_id: None,
            name: "vet-plan".to_string(),
            body: "# /vet-plan\n".to_string(),
            checksum: None,
            is_shared: false,
            current_version: 1,
            created_at: "2026-08-04T00:00:00Z".to_string(),
            updated_at: "2026-08-04T00:00:00Z".to_string(),
        };
        let json = serde_json::to_value(&cmd).unwrap();
        assert!(json.get("organization_id").is_none());
        assert!(json.get("created_by_user_id").is_none());
        assert!(json.get("checksum").is_none());
        assert_eq!(json["name"], "vet-plan");
        assert_eq!(json["current_version"], 1);
    }

    #[test]
    fn agent_command_minimal_deserializes() {
        // `is_shared` absent → false; absent optionals → None.
        // `r##"…"##`, not `r#"…"#`: the JSON body contains `"#`, which would
        // terminate a single-hash raw string mid-literal.
        let cmd: AgentCommand = serde_json::from_str(
            r##"{
                "id": "abc",
                "name": "implement-plan",
                "body": "# body",
                "current_version": 3,
                "created_at": "2026-08-04T00:00:00Z",
                "updated_at": "2026-08-04T01:00:00Z"
            }"##,
        )
        .unwrap();
        assert!(!cmd.is_shared);
        assert_eq!(cmd.current_version, 3);
        assert_eq!(cmd.organization_id, None);
    }

    #[test]
    fn version_roundtrips_restore_provenance() {
        let v = AgentCommandVersion {
            id: "v6".to_string(),
            agent_command_id: "abc".to_string(),
            version_number: 6,
            body: "# body as of v2".to_string(),
            checksum: Some("deadbeef".to_string()),
            created_by_user_id: Some("user-1".to_string()),
            change_description: Some("Restored from version 2".to_string()),
            restored_from: Some(2),
            created_at: "2026-08-04T02:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&v).unwrap();
        let back: AgentCommandVersion = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
        assert_eq!(back.restored_from, Some(2));
    }

    #[test]
    fn version_omits_absent_optionals() {
        let v = AgentCommandVersion {
            id: "v1".to_string(),
            agent_command_id: "abc".to_string(),
            version_number: 1,
            body: "# body".to_string(),
            checksum: None,
            created_by_user_id: None,
            change_description: None,
            restored_from: None,
            created_at: "2026-08-04T00:00:00Z".to_string(),
        };
        let json = serde_json::to_value(&v).unwrap();
        assert!(json.get("checksum").is_none());
        assert!(json.get("change_description").is_none());
        assert!(json.get("restored_from").is_none());
        assert_eq!(json["version_number"], 1);
    }
}
