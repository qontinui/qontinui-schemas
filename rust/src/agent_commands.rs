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
//!
//! ## Two write-boundary primitives live here, not in the consumers
//!
//! [`validate_agent_command_name`] and [`agent_command_checksum`] are the
//! canonical definitions of "is this `name` writable?" and "what exactly does
//! `checksum` contain?". They sit in this crate for the same reason
//! [`crate::apps::validate_app_id`] does: three surfaces write these rows (the
//! qontinui-web FastAPI layer, the runner's override registry, and any future
//! coord-mediated sync), and a rule that each re-derives is a rule they will
//! disagree about. See each function's own docs for the failure it prevents.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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
    ///
    /// Because it becomes a path component, it is a write-boundary value —
    /// validate it with [`validate_agent_command_name`] before persisting.
    pub name: String,

    /// The full markdown body of the command.
    pub body: String,

    /// Content hash of [`body`](Self::body), for cheap change detection
    /// (cache invalidation, diff-against-default). `None` on rows written
    /// before a checksum was computed.
    ///
    /// Computed by [`agent_command_checksum`] — `"sha256-<hex>"` over the
    /// LF-normalized body. Do not hand-roll it; a producer that hashes raw
    /// bytes disagrees with one that hashes a CRLF round-trip.
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

    /// Content hash of [`body`](Self::body) at this version, from
    /// [`agent_command_checksum`]. Equal checksums on two versions mean the
    /// bodies are identical — which is what lets the UI show a revert as
    /// "back to v2's content" without refetching either body.
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

// ---------------------------------------------------------------------------
// Write-boundary primitives
// ---------------------------------------------------------------------------

/// Failure modes for agent-command overrides. Mirrors the tagged-enum shape of
/// [`crate::apps::AppError`], so the qontinui-web layer can surface a rejected
/// override the same way it surfaces a rejected app registration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, thiserror::Error)]
#[serde(
    tag = "reason",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum AgentCommandError {
    #[error("agent command name '{name}' is not a valid slug")]
    InvalidName { name: String },

    #[error("agent command name '{name}' is a reserved device name on Windows")]
    ReservedName { name: String },
}

/// Filename stems Windows resolves to a character device no matter what
/// extension follows, so `nul.md` IS the null device, not a file called
/// "nul.md". Matched case-insensitively because the device namespace is.
const WINDOWS_RESERVED_STEMS: &[&str] = &[
    "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
    "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
];

/// Validate an [`AgentCommand::name`] before it is persisted or turned into a
/// path. Returns `Ok(())` for valid slugs.
///
/// Rules — deliberately the same shape as [`crate::apps::validate_app_id`]:
/// 1–64 chars, lowercase ASCII letters / digits / hyphens, first char
/// `[a-z0-9]`. Plus a rejection of the Windows reserved device stems.
///
/// ## The two failures this prevents
///
/// The name is not just a database key. The runner's
/// `provision_fleet_commands_into` writes each command by joining its
/// `<name>.md` filename onto `<workdir>/.claude/commands/` — a literal today,
/// since only the embedded defaults exist. The moment an override supplies
/// that stem, this stored string becomes a path component on a fleet device.
///
/// - **Escape.** `..`, `/` and `\` are all excluded by the charset, so an
///   override cannot address a file outside `.claude/commands/`. That path
///   never had a hostile author in mind — the module header already flags a
///   body as untrusted remote content, and the name arrives by the same route.
/// - **Silent discard (Windows).** The fleet runs on Windows, where a reserved
///   device stem (`con`, `nul`, `com1`, …) opens a device rather than a file.
///   `std::fs::write` on `nul.md` SUCCEEDS and writes nothing, so the
///   provisioning path — which is fail-soft and only warns on `Err` — would
///   log a clean success for a command that does not exist. A no-op that
///   reports success is worse than an error, which is why this is a rejection
///   at the write boundary rather than a runtime check.
///
/// Shape is all this checks. Whether the name matches an embedded default it
/// may replace is the runner's call — this crate deliberately does not know
/// the default list (the module header notes nothing here may assume two).
pub fn validate_agent_command_name(name: &str) -> Result<(), AgentCommandError> {
    let len_ok = (1..=64).contains(&name.len());
    let first_ok = name
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit());
    let rest_ok = name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if !(len_ok && first_ok && rest_ok) {
        return Err(AgentCommandError::InvalidName { name: name.into() });
    }
    if WINDOWS_RESERVED_STEMS
        .iter()
        .any(|r| r.eq_ignore_ascii_case(name))
    {
        return Err(AgentCommandError::ReservedName { name: name.into() });
    }
    Ok(())
}

/// Canonical checksum for an agent-command body. Returns `"sha256-<hex>"`.
///
/// **CR bytes are stripped before hashing.** That is the load-bearing part.
/// The same body is round-tripped through Postgres, JSON, and a Windows
/// filesystem before anyone compares two of these, and a CRLF-normalizing hop
/// anywhere on that path would otherwise flip the checksum and report an
/// unchanged command as changed — defeating the only thing the field is for.
/// The runner already learned this for the bundled defaults, whose
/// `STAGED_COMMAND_HASHES` pin is likewise taken over CR-stripped content.
///
/// The `"sha256-"` prefix follows this crate's existing output convention
/// ([`crate::canonical_hash`], [`crate::spec_check`]) — note that the runner's
/// `STAGED_COMMAND_HASHES` constant stores BARE hex over those same
/// LF-normalized bytes, so comparing an override against an embedded default
/// means comparing the hex tails, not the whole strings.
///
/// This is not [`crate::canonical_hash::canonical_hash`]: that one runs its
/// input through RFC 8785 JCS, which for a bare string adds JSON quoting and
/// escaping that every non-Rust producer would then have to reproduce
/// byte-exactly. There is no key order to canonicalize in a markdown
/// document, so the JCS pass would buy nothing and cost interoperability.
/// A Python producer matches this with
/// `"sha256-" + sha256(body.replace("\r", "").encode("utf-8")).hexdigest()`.
pub fn agent_command_checksum(body: &str) -> String {
    let normalized = body.replace('\r', "");
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    format!("sha256-{}", hex::encode(hasher.finalize()))
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

    // -- validate_agent_command_name ---------------------------------------

    #[test]
    fn name_accepts_the_embedded_default_slugs() {
        // The two commands the runner bundles today must both be overridable;
        // a validator that rejected either would make the feature unusable.
        assert!(validate_agent_command_name("vet-plan").is_ok());
        assert!(validate_agent_command_name("implement-plan").is_ok());
        assert!(validate_agent_command_name("a").is_ok());
        assert!(validate_agent_command_name("plan2").is_ok());
        assert!(validate_agent_command_name(&"a".repeat(64)).is_ok());
    }

    #[test]
    fn name_rejects_path_escapes() {
        for bad in [
            "..",
            "../vet-plan",
            "../../.claude/settings",
            "sub/vet-plan",
            "sub\\vet-plan",
            "/etc/passwd",
            "C:\\windows\\system32",
            ".hidden",
        ] {
            assert_eq!(
                validate_agent_command_name(bad),
                Err(AgentCommandError::InvalidName { name: bad.into() }),
                "{bad:?} must not be writable as a path component"
            );
        }
    }

    #[test]
    fn name_rejects_shape_violations() {
        for bad in [
            "",                // empty
            &"a".repeat(65),   // over the 64-char ceiling
            "-leading-hyphen", // first char must be [a-z0-9]
            "Vet-Plan",        // uppercase
            "vet plan",        // whitespace
            "vet.plan",        // dot would split the stem
            "vet_plan",        // underscore is not in the charset
        ] {
            assert!(
                validate_agent_command_name(bad).is_err(),
                "{bad:?} should be rejected"
            );
        }
    }

    #[test]
    fn name_rejects_windows_device_stems_case_insensitively() {
        // `nul.md` is the null device on Windows: fs::write succeeds and
        // discards, so this must fail here or it fails silently there.
        assert_eq!(
            validate_agent_command_name("nul"),
            Err(AgentCommandError::ReservedName { name: "nul".into() })
        );
        for reserved in ["con", "prn", "aux", "com1", "lpt9"] {
            assert_eq!(
                validate_agent_command_name(reserved),
                Err(AgentCommandError::ReservedName {
                    name: reserved.into()
                }),
                "{reserved:?} is a Windows device stem"
            );
        }
        // Uppercase is caught by the charset rule first — the point is only
        // that it never reaches the filesystem, not which arm rejects it.
        assert!(validate_agent_command_name("NUL").is_err());
        // A reserved stem is only reserved bare; extending it is a real name.
        assert!(validate_agent_command_name("console").is_ok());
        assert!(validate_agent_command_name("nul-plan").is_ok());
    }

    #[test]
    fn error_serializes_with_the_apps_tagged_shape() {
        let json = serde_json::to_value(AgentCommandError::InvalidName {
            name: "../x".to_string(),
        })
        .unwrap();
        assert_eq!(json["reason"], "invalid-name");
        assert_eq!(json["name"], "../x");
    }

    // -- agent_command_checksum --------------------------------------------

    #[test]
    fn checksum_has_the_crate_prefix_and_64_hex_chars() {
        let h = agent_command_checksum("# /vet-plan\n");
        assert!(h.starts_with("sha256-"), "got {h}");
        let hex_part = &h["sha256-".len()..];
        assert_eq!(hex_part.len(), 64);
        assert!(hex_part.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn checksum_is_line_ending_invariant() {
        // The whole point: a CRLF round-trip through Postgres/JSON/Windows
        // must not read as a content change.
        assert_eq!(
            agent_command_checksum("# /vet-plan\nstep one\n"),
            agent_command_checksum("# /vet-plan\r\nstep one\r\n")
        );
    }

    #[test]
    fn checksum_distinguishes_real_edits() {
        assert_ne!(
            agent_command_checksum("# /vet-plan\n"),
            agent_command_checksum("# /vet-plan\nstep one\n")
        );
    }

    #[test]
    fn checksum_matches_bare_lf_normalized_sha256_of_the_body() {
        // Pins the algorithm against an independently computed digest rather
        // than against itself, so a future refactor cannot quietly redefine
        // the wire value. The hex tail is exactly what the runner's
        // STAGED_COMMAND_HASHES stores for the embedded defaults.
        let expected = {
            let mut h = Sha256::new();
            h.update(b"# /vet-plan\nstep one\n");
            hex::encode(h.finalize())
        };
        assert_eq!(
            agent_command_checksum("# /vet-plan\r\nstep one\r\n"),
            format!("sha256-{expected}")
        );
    }

    #[test]
    fn checksum_round_trips_onto_the_record() {
        let body = "# /implement-plan\n";
        let cmd = AgentCommand {
            id: "11111111-1111-4111-8111-111111111111".to_string(),
            organization_id: None,
            created_by_user_id: None,
            name: "implement-plan".to_string(),
            body: body.to_string(),
            checksum: Some(agent_command_checksum(body)),
            is_shared: false,
            current_version: 1,
            created_at: "2026-08-04T00:00:00Z".to_string(),
            updated_at: "2026-08-04T00:00:00Z".to_string(),
        };
        let back: AgentCommand =
            serde_json::from_str(&serde_json::to_string(&cmd).unwrap()).unwrap();
        assert_eq!(back.checksum, Some(agent_command_checksum(&back.body)));
    }
}
