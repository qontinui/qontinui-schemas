//! The unified **agent text corpus** — one `kind`-discriminated text unit that
//! carries slash commands, agent skills, and (later) agent definitions, with a
//! single append-only version chain shared by all of them.
//!
//! A slash command is a name plus text. A skill is a name plus text *files*.
//! Those are the same concept at two arities, so this module models one:
//!
//! ```text
//! AgentTextUnit             (kind, name, org, files, head pointer, provenance)
//!   └── AgentTextUnitVersion   (version_number, files, checksum)   append-only
//!         └── files: relative path → text        one AgentTextUnitFile each
//! ```
//!
//! A command is the **degenerate single-entry case** of that map
//! (`{"vet-plan.md": "…"}`); a skill carries `SKILL.md` plus siblings
//! (`{"SKILL.md": "…", "coord-revive.sh": "…"}`). Nothing here special-cases
//! either arity — that is the whole point of the unification, and it is why
//! `2026-08-20-fleet-served-agent-skills.md` rejected a parallel
//! `agent_skills` stack (Design decision 1, option B).
//!
//! ## These types describe a LANDED wire, not a proposal
//!
//! qontinui-web's Phase 2 shipped first: `backend/app/models/agent_text_unit.py`
//! (the `project.agent_text_units` tables) and
//! `backend/app/services/agent_text_unit_service.py` (the validators, the
//! canonical checksum, and the `AgentTextUnitResponse` /
//! `AgentTextUnitVersionResponse` shapes). **Every rule in this module mirrors
//! that implementation deliberately, and the Python side is the one that
//! defined them.** Where a rule here looks arbitrary, the Python source is the
//! reason, and the two must be changed together — a divergence does not produce
//! a compile error, it produces a unit the store accepts and the runner then
//! refuses to provision (or the reverse), which is exactly the silent failure
//! this plan exists to close.
//!
//! ## `kind` is an open discriminator, never a boolean in disguise
//!
//! [`AgentTextUnitKind`] is a **string newtype**, not an enum. Two kinds ship
//! with this plan (`command`, `skill`) and two more are already identified —
//! `.claude/agents/*.md` has the identical delivery gap
//! (`agent_runtime.rs::provision_agent_definitions_from_root`), and
//! `.agents/skills/` is a fourth target whose consumers are not `claude`
//! sessions at all. A closed `enum` would emit `enum: ["command","skill"]`
//! into the JSON Schema and therefore into every generated TS union and Python
//! `StrEnum`, so the day a fifth kind is written by a newer producer, an older
//! reader would **reject the whole document** rather than skip one row. A
//! string is forward-readable; the named constants below carry the same
//! ergonomics without the wire cost. The Python column is a plain
//! `String(64)` for the same reason.
//!
//! ## Two layers, expressed by nullability
//!
//! [`AgentTextUnit::organization_id`] is nullable and that nullability is
//! *semantic*, not just an orphan hatch:
//!
//! ```text
//! resolution order (per (kind, name)):
//!     account override (organization_id = <org>)
//!   → fleet default    (organization_id IS NULL)
//!   → embedded default (runner binary, `fleet_commands.rs`)
//! ```
//!
//! [`AgentTextUnit::source`] reports which of the first two rungs a served row
//! came from. Storage backs the pair with a **partial unique index pair** —
//! `UNIQUE (organization_id, kind, name) WHERE organization_id IS NOT NULL`
//! plus `UNIQUE (kind, name) WHERE organization_id IS NULL` — because a plain
//! three-column `UNIQUE` does not collide on NULL in Postgres and would leave
//! the fleet-default layer completely unconstrained.
//!
//! ## The version chain is append-only
//!
//! Every edit **appends** an [`AgentTextUnitVersion`]; the head is
//! [`AgentTextUnit::current_version`]. Reverting to v2 from a head of v5
//! appends v6 whose `files` equal v2's and whose
//! [`restored_from`](AgentTextUnitVersion::restored_from) records the
//! provenance — it never rewinds.
//!
//! ## Wire-format notes
//!
//! Per the crate-level conventions (`lib.rs`): UUIDs and timestamps are
//! `String`s, optional fields use `#[serde(default, skip_serializing_if =
//! "Option::is_none")]`, and field names stay **snake_case** to match the
//! producing qontinui-web FastAPI layer (same reasoning as
//! [`crate::agent_commands`]).
//!
//! ## Relationship to [`crate::agent_commands`]
//!
//! That module is the **superseded** single-body shape. It stays until the
//! runner's `agent_commands` module reads this corpus instead (plan Phase 4),
//! at which point it is deleted outright — qontinui-web has already deleted its
//! half and keeps only a translating route alias at `/api/v1/agent-commands`
//! for runners that have not been rebuilt. There is deliberately **no `pub type
//! AgentCommand = AgentTextUnit` alias**: `schema_for!` resolves an alias to the
//! underlying type, so an alias would emit a schema titled `AgentTextUnit` under
//! the key `AgentCommand` and redden the drift gate, and the compatibility need
//! is served at the wire by that route rather than by a Rust name. The two
//! modules therefore each carry their own `WINDOWS_RESERVED_STEMS` and their own
//! name rule — which is correct rather than duplication, because the rules now
//! genuinely differ (see [`validate_agent_text_unit_name`]'s underscore
//! widening).
//!
//! ## Write-boundary primitives live here, not in the consumers
//!
//! The validators and [`agent_text_unit_files_checksum`] are the canonical Rust
//! statements of "is this writable?" and "what exactly does `checksum`
//! contain?". Three surfaces write these rows — the qontinui-web FastAPI layer,
//! the runner's provisioner, and the config-repo importer — and a rule each
//! re-derives is a rule they will disagree about. Same reasoning as
//! [`crate::apps::validate_app_id`].

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// ---------------------------------------------------------------------------
// Bounds
// ---------------------------------------------------------------------------
//
// Byte-identical to the qontinui-web service module's constants. Grounded in
// the corpus these have to carry (measured 2026-08-22 over
// `qontinui-claude-config/.claude/`): 79 commands / 1.61 MB, largest single
// body 154 KB; 9 skills / 12 files / 193 KB, largest single file 54.5 KB.
// Never treat those counts as expectations — they are a timestamp — but the
// limits below are sized with real headroom over them.

/// Per-file cap. Matches the runner's `agent_commands::MAX_BODY_BYTES`, so a
/// unit the store accepts cannot be one the runner then refuses.
pub const MAX_FILE_BYTES: usize = 1024 * 1024;

/// Cap across the whole map. Per-file caps alone do not bound a bundle.
pub const MAX_UNIT_BYTES: usize = 4 * 1024 * 1024;

/// Maximum number of files in one unit.
pub const MAX_FILES_PER_UNIT: usize = 64;

/// Maximum UTF-8 length of one `files` key.
pub const MAX_PATH_BYTES: usize = 255;

/// Maximum `/`-separated depth of one `files` key.
pub const MAX_PATH_SEGMENTS: usize = 8;

/// Maximum length of a unit `name` or `kind`.
pub const MAX_NAME_LENGTH: usize = 64;

/// A version's content: **relative path → text**.
///
/// `BTreeMap`, not `HashMap`, and that is load-bearing rather than a style
/// preference: Rust orders `str` by its raw UTF-8 bytes, which is exactly the
/// ordering [`agent_text_unit_files_checksum`] is defined over, so iterating
/// this type *is* the canonical order with nothing to re-sort.
pub type AgentTextUnitFiles = BTreeMap<String, String>;

// ---------------------------------------------------------------------------
// AgentTextUnitKind
// ---------------------------------------------------------------------------

/// The unit's kind — an **open** discriminator, serialized as a bare string.
///
/// Known values are exposed as associated constants rather than enum variants
/// so that an unrecognized value round-trips instead of failing to parse (see
/// the module docs for why that matters). Use [`is_known`](Self::is_known) to
/// ask whether *this* build understands a value, and
/// [`provisioning_target`](Self::provisioning_target) to ask where a unit of
/// that kind is written on disk — a kind with no target is storable and
/// editable but is never provisioned into a session.
///
/// | Constant | Value | Provisioning target | Entrypoint | Status |
/// |---|---|---|---|---|
/// | [`COMMAND`](Self::COMMAND) | `command` | `commands/<name>.md` | `<name>.md` | ships now |
/// | [`SKILL`](Self::SKILL) | `skill` | `skills/<name>/…` | `SKILL.md` | ships now |
/// | [`AGENT`](Self::AGENT) | `agent` | `agents/<name>.md` | `<name>.md` | modelled; provisioned today by `provision_agent_definitions_from_root` |
/// | [`AGENTS_SKILL`](Self::AGENTS_SKILL) | `agents_skill` | none | `<name>.md` | modelled only — its consumers (pi, Codex) have no runner spawn hook |
///
/// The `agents_skill` value spells its separator with an underscore, not a
/// hyphen, because [`validate_agent_text_unit_kind`] admits `[a-z][a-z0-9_]*`
/// and nothing else — a hyphenated kind would be refused at the store boundary.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
pub struct AgentTextUnitKind(pub String);

impl AgentTextUnitKind {
    /// A slash command: `<workdir>/.claude/commands/<name>.md`.
    pub const COMMAND: &'static str = "command";
    /// An agent skill: `<workdir>/.claude/skills/<name>/SKILL.md` + siblings.
    pub const SKILL: &'static str = "skill";
    /// An agent definition: `<workdir>/.claude/agents/<name>.md`.
    pub const AGENT: &'static str = "agent";
    /// A `.agents/skills/<name>/` unit for harnesses with no MCP client.
    /// Carried by the corpus, deliberately **not** provisioned.
    pub const AGENTS_SKILL: &'static str = "agents_skill";

    /// Every kind this build understands. Not a closed set on the wire — a
    /// value outside it deserializes fine and simply reports
    /// [`is_known`](Self::is_known) `false`.
    pub const KNOWN: &'static [&'static str] =
        &[Self::COMMAND, Self::SKILL, Self::AGENT, Self::AGENTS_SKILL];

    /// Wrap an arbitrary kind string. Validate it with
    /// [`validate_agent_text_unit_kind`] before persisting.
    pub fn new(kind: impl Into<String>) -> Self {
        Self(kind.into())
    }

    /// `command`.
    pub fn command() -> Self {
        Self(Self::COMMAND.to_string())
    }

    /// `skill`.
    pub fn skill() -> Self {
        Self(Self::SKILL.to_string())
    }

    /// The kind as a `&str`.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Whether this build recognizes the value. A `false` here is a
    /// **forward** document, not a corrupt one: store it, show it, refuse to
    /// provision it.
    pub fn is_known(&self) -> bool {
        Self::KNOWN.contains(&self.0.as_str())
    }

    /// Where a unit of this kind is written inside a session's `.claude/`
    /// directory, or `None` for a kind this build does not provision.
    ///
    /// `None` covers two distinct cases and the caller must treat both the
    /// same way — refuse to write: an unknown kind from a newer producer, and
    /// a known-but-unprovisioned kind ([`AGENTS_SKILL`](Self::AGENTS_SKILL)).
    /// Distinguish them with [`is_known`](Self::is_known) if the *message*
    /// needs to differ.
    pub fn provisioning_target(&self) -> Option<AgentTextUnitTarget> {
        match self.0.as_str() {
            Self::COMMAND => Some(AgentTextUnitTarget::File { subdir: "commands" }),
            Self::SKILL => Some(AgentTextUnitTarget::Directory { subdir: "skills" }),
            Self::AGENT => Some(AgentTextUnitTarget::File { subdir: "agents" }),
            _ => None,
        }
    }
}

impl std::fmt::Display for AgentTextUnitKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for AgentTextUnitKind {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for AgentTextUnitKind {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// The relative path inside `files` that holds a unit's primary text.
///
/// Mirrors `agent_text_unit.entrypoint_path` in qontinui-web exactly, including
/// its fallback: a kind with no fixed entrypoint uses `<name>.md`, which is the
/// `.claude/commands/` and `.claude/agents/` convention (the unit IS one file
/// named for the unit). **An unknown kind falls back rather than failing** —
/// `kind` is widenable, so a hard error here would re-close it.
///
/// [`validate_agent_text_unit_files`] requires this key to be present, so it is
/// the one path every unit is guaranteed to carry.
pub fn agent_text_unit_entrypoint(kind: &AgentTextUnitKind, name: &str) -> String {
    match kind.as_str() {
        AgentTextUnitKind::SKILL => "SKILL.md".to_string(),
        _ => format!("{name}.md"),
    }
}

/// Where a [`AgentTextUnitKind`] lands on disk, relative to a session's
/// `.claude/` directory.
///
/// Rust-only: it is a property of *this* binary's provisioner, not a wire
/// value, so it is deliberately not `Serialize`/`JsonSchema`. A producer that
/// shipped the target over the wire would be letting account-supplied data
/// choose a write location.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentTextUnitTarget {
    /// One file per unit, written to `<subdir>/<entrypoint>`. The unit's
    /// `files` map is the degenerate single-entry case.
    File { subdir: &'static str },
    /// One directory per unit; every entry of `files` lands at
    /// `<subdir>/<name>/<relative path>`.
    Directory { subdir: &'static str },
}

impl AgentTextUnitTarget {
    /// The immediate child of `.claude/` this kind lives under.
    pub fn subdir(&self) -> &'static str {
        match self {
            Self::File { subdir } | Self::Directory { subdir } => subdir,
        }
    }

    /// Whether a unit of this kind occupies a directory of its own. Directory
    /// kinds are the ones whose provisioner must treat every `files` key as a
    /// subpath rather than a single flat filename.
    pub fn is_directory(&self) -> bool {
        matches!(self, Self::Directory { .. })
    }

    /// The `.claude/`-relative path that `file_path` of unit `name` is written
    /// to. `file_path` MUST already have passed
    /// [`validate_agent_text_unit_file_path`]; this joins, it does not
    /// validate.
    pub fn relative_path(&self, name: &str, file_path: &str) -> String {
        match self {
            Self::File { subdir } => format!("{subdir}/{file_path}"),
            Self::Directory { subdir } => format!("{subdir}/{name}/{file_path}"),
        }
    }
}

// ---------------------------------------------------------------------------
// AgentTextUnit
// ---------------------------------------------------------------------------

/// The layer a served [`AgentTextUnit`] was resolved from: an account override.
pub const AGENT_TEXT_UNIT_SOURCE_USER: &str = "user";

/// The layer a served [`AgentTextUnit`] was resolved from: the
/// `organization_id IS NULL` fleet default.
pub const AGENT_TEXT_UNIT_SOURCE_FLEET: &str = "fleet";

fn default_source() -> String {
    AGENT_TEXT_UNIT_SOURCE_USER.to_string()
}

fn default_true() -> bool {
    true
}

/// One unit of the agent text corpus: a `(kind, name)` addressed bundle of
/// text files, owned either by an organization or by the fleet-default layer.
///
/// The content is **untrusted remote content** from the runner's point of
/// view — it is markdown and shell text rather than compiled code, but it is
/// instructions to an agent and it becomes files on a fleet device. Consumers
/// fail soft (a malformed unit falls back to the next rung of the resolution
/// chain and warns) and never fetch cross-org.
///
/// This mirrors qontinui-web's `AgentTextUnitResponse` field for field, plus
/// the two forward provenance fields Phase 5 fills in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AgentTextUnit {
    /// Unit id (UUID v4 string).
    pub id: String,

    /// What sort of unit this is. Open discriminator — see
    /// [`AgentTextUnitKind`].
    pub kind: AgentTextUnitKind,

    /// The unit slug, e.g. `vet-plan` or `coord-revive`. It is the filename
    /// stem (single-file kinds) or the directory name (directory kinds) under
    /// `.claude/<subdir>/`, so it is a write-boundary value — validate it with
    /// [`validate_agent_text_unit_name`] before persisting.
    pub name: String,

    /// Owning organization (`auth.organizations.id`).
    ///
    /// **`None` is meaningful**: it is the *fleet-default* layer, resolved
    /// after an account override and before the runner's embedded default. It
    /// is also what an organization deletion leaves behind (`ondelete SET
    /// NULL`), which is why storage keys the NULL bucket with its own partial
    /// unique index — see the module docs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,

    /// User who last authored this unit. **Attribution only** — it does not
    /// scope visibility; [`organization_id`](Self::organization_id) does.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by_user_id: Option<String>,

    /// The unit's current content: **relative path → text**. A command carries
    /// one entry; a skill carries `SKILL.md` plus siblings.
    ///
    /// Carried inline on the unit, matching the landed
    /// `AgentTextUnitResponse`. Note the cost that buys the convenience: a
    /// whole-corpus list is ~88 units / ~1.8 MB, and the runner fetches it on
    /// the spawn critical path under a 4 s budget, so a caller that only needs
    /// to know whether its cache is current should compare
    /// [`checksum`](Self::checksum) rather than re-reading these bytes.
    pub files: AgentTextUnitFiles,

    /// The key of [`files`](Self::files) holding this unit's primary text,
    /// derived server-side from `(kind, name)` — see
    /// [`agent_text_unit_entrypoint`]. Read-only: it is recomputed on every
    /// response and ignored on write.
    pub entrypoint: String,

    /// Canonical digest over the whole [`files`](Self::files) map, from
    /// [`agent_text_unit_files_checksum`]. `None` on a row written before a
    /// checksum was computed.
    ///
    /// This is **not** [`crate::agent_commands::agent_command_checksum`]: that
    /// one digests a single body and is what the legacy `/agent-commands` wire
    /// still carries. The two deliberately disagree even for a one-entry map.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,

    /// Whether this unit is visible to the whole organization rather than only
    /// its author. Absent means `false`.
    #[serde(default)]
    pub is_shared: bool,

    /// Whether the harness may offer this unit as an invocable slash command.
    ///
    /// `false` means "carried by the corpus, never invocable": the
    /// underscore-prefixed units (`_gate-registration`, `_loop-control`) are
    /// copy-source specs that other units paste from, and `.claude/commands/`
    /// has no include mechanism, so they must be *present on disk* without
    /// appearing as `/_gate-registration`. Absent means `true`.
    ///
    /// Storage enforces the pairing with a CHECK (`left(name,1) <> '_' OR
    /// is_invocable = false`); [`validate_agent_text_unit_invocability`] is the
    /// same rule at this boundary.
    /// Always written on the wire, matching the landed `AgentTextUnitResponse`
    /// (which emits it unconditionally). `#[serde(default)]` covers reading an
    /// older payload that predates the field; the `#[schemars(default)]` twin
    /// is what puts `true` — rather than `null` — into the generated Python and
    /// TypeScript bindings, so a consumer that sees the field absent defaults
    /// it the same way Rust does.
    #[serde(default = "default_true")]
    #[schemars(default = "default_true")]
    pub is_invocable: bool,

    /// Head of the version chain — the `version_number` of the
    /// [`AgentTextUnitVersion`] whose content this unit currently serves.
    /// Starts at 1 and only ever increases (a revert bumps it).
    pub current_version: i32,

    /// Which resolution layer this row was served from —
    /// [`AGENT_TEXT_UNIT_SOURCE_USER`] or [`AGENT_TEXT_UNIT_SOURCE_FLEET`].
    /// Server-derived, defaults to `user`.
    ///
    /// ⚠️ **Unrelated to [`source_path`](Self::source_path) /
    /// [`source_commit`](Self::source_commit) despite the shared prefix.** This
    /// field names the *layer the row came from*; those two name the *config
    /// repo the text was imported from*. Adjacent names, different concepts.
    #[serde(default = "default_source")]
    pub source: String,

    /// Import provenance: the repo-relative path this unit's text came from,
    /// e.g. `.claude/skills/coord-revive/`. `None` for a unit authored directly
    /// in the console.
    ///
    /// Recorded per unit at import so the console can show an operator whether
    /// a unit still matches its source. The path is **repo-relative on
    /// purpose** — an absolute one would pin a build machine's layout into
    /// account data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,

    /// Import provenance: the commit of the source repo (full 40-char SHA).
    /// `None` for a console-authored unit, or for an import from a dirty tree
    /// where no commit honestly describes the bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_commit: Option<String>,

    /// ISO 8601 (RFC 3339) creation timestamp.
    pub created_at: String,

    /// ISO 8601 (RFC 3339) last-modification timestamp.
    pub updated_at: String,
}

// ---------------------------------------------------------------------------
// AgentTextUnitVersion
// ---------------------------------------------------------------------------

/// One immutable entry in an [`AgentTextUnit`]'s append-only version chain.
///
/// Rows are **only ever inserted**. An edit appends the next `version_number`;
/// a revert appends a new version whose `files` are copied from an older one
/// and whose [`restored_from`](Self::restored_from) names that older version.
/// Nothing in the chain is updated or deleted, so the chain is the complete
/// edit history and "revert" is itself a recorded edit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AgentTextUnitVersion {
    /// Version-row id (UUID v4 string).
    pub id: String,

    /// The [`AgentTextUnit`] this version belongs to. Deleting the unit
    /// cascades to its versions.
    pub agent_text_unit_id: String,

    /// Monotonic version number within this unit, starting at 1. Unique per
    /// `(agent_text_unit_id, version_number)`; the DB constraint — not
    /// application code — is what rejects a duplicate under concurrent
    /// appends.
    pub version_number: i32,

    /// The unit's whole content as of this version: **relative path → text**.
    ///
    /// Every key must satisfy [`validate_agent_text_unit_file_path`]; the map
    /// must be non-empty and must contain the unit's
    /// [`agent_text_unit_entrypoint`].
    pub files: AgentTextUnitFiles,

    /// Content hash of [`files`](Self::files), from
    /// [`agent_text_unit_files_checksum`]. Equal checksums on two versions mean
    /// the content is identical, which is what makes the Phase 5 re-import
    /// idempotent — it is the field "did the text actually change?" is read
    /// off. `None` on a row written before a checksum was computed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,

    /// User who authored this version. Attribution only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by_user_id: Option<String>,

    /// Free-text note describing what changed, supplied by the editor. A
    /// revert writes a generated one (e.g. `"Restored from version 2"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub change_description: Option<String>,

    /// Provenance for a revert: the `version_number` this version's content
    /// was copied from. `None` for an ordinary edit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restored_from: Option<i32>,

    /// ISO 8601 (RFC 3339) creation timestamp. Versions are immutable, so
    /// there is deliberately no `updated_at`.
    pub created_at: String,
}

impl AgentTextUnitVersion {
    /// This version's content as a path-ordered list of
    /// [`AgentTextUnitFile`]s — the form a provisioner iterates and an editor
    /// renders as tabs.
    pub fn files_sorted(&self) -> Vec<AgentTextUnitFile> {
        AgentTextUnitFile::from_map(&self.files)
    }
}

// ---------------------------------------------------------------------------
// AgentTextUnitFile
// ---------------------------------------------------------------------------

/// One `relative path → text` entry of a `files` map, as a standalone record.
///
/// The map is the storage and wire form (it is one JSONB column, and it makes
/// "no duplicate paths" structural rather than validated). This type is the
/// **entry** form: what a per-file API operation addresses, what a per-file
/// validation failure names, and what a provisioner iterates. Convert with
/// [`from_map`](Self::from_map) / [`into_map`](Self::into_map).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AgentTextUnitFile {
    /// Path relative to the unit's own directory, `/`-separated, e.g.
    /// `SKILL.md` or `coord-revive.sh`. Validated by
    /// [`validate_agent_text_unit_file_path`].
    pub path: String,

    /// The file's full text. Written verbatim (modulo the provisioner's own
    /// line-ending handling); never given an executable bit, because scripts
    /// in this corpus are invoked as `bash <path>` and Windows has no exec bit
    /// anyway. "Account-supplied text written to disk" must not become
    /// "account-supplied program registered with the OS".
    pub text: String,
}

impl AgentTextUnitFile {
    /// Path-ordered entries of a `files` map.
    pub fn from_map(files: &AgentTextUnitFiles) -> Vec<Self> {
        files
            .iter()
            .map(|(path, text)| Self {
                path: path.clone(),
                text: text.clone(),
            })
            .collect()
    }

    /// Collect entries back into a `files` map. A duplicate `path` keeps the
    /// last occurrence — the map is the authority on uniqueness, which is
    /// precisely why the map is the stored form.
    pub fn into_map(entries: impl IntoIterator<Item = Self>) -> AgentTextUnitFiles {
        entries.into_iter().map(|f| (f.path, f.text)).collect()
    }
}

// ---------------------------------------------------------------------------
// The embedded-default layer
// ---------------------------------------------------------------------------

/// One embedded default as the **runner binary** ships it, on its way to the
/// account so there is a baseline to diff an override against.
///
/// ## Why this exists as a third layer
///
/// [`AgentTextUnit`]'s resolution chain is
/// `account override → fleet default → embedded default (runner binary)`, and
/// the store holds rows for the first two only — the binary's copy has never
/// had a row anywhere. That is the gap this type closes. A user who overrode
/// `/implement-plan` can diff their versions against each other but not
/// against what actually ships, and `ResetToDefaultDialog` cannot preview the
/// text it is about to restore, because there is no baseline to put on the
/// left-hand side.
///
/// Plan `2026-08-31-runner-publishes-embedded-command-defaults`; the deferral
/// it discharges is recorded in the code at qontinui-web's
/// `settings/agent-commands/_components/VersionDiff.tsx`.
///
/// ## What it is NOT
///
/// * **Not an [`AgentTextUnit`].** There is no `id`, no version chain, and no
///   `is_shared` — a published default is content plus provenance, not a
///   corpus row with an edit history. It is the *input* to a row.
/// * **Not fleet-scoped.** `organization_id` is deliberately absent from the
///   wire: the runner publishes with the operator's own user bearer, so the
///   server assigns the org from that credential. A client-supplied org — or
///   the fleet layer's `organization_id IS NULL` — would let any signed-in
///   user rewrite another tenant's baseline, or silently clobber an
///   operator's deliberate fleet default.
/// * **Not authoritative for provisioning.** The runner keeps resolving
///   `fresh fetch → disk cache → embedded default`; this is a *display*
///   baseline. Publishing it must never put the network on the
///   out-of-the-box path.
///
/// ## The checksum is the files-map digest, not the single-body one
///
/// [`checksum`](Self::checksum) is [`agent_text_unit_files_checksum`] — the
/// same digest [`AgentTextUnit::checksum`] and [`AgentTextUnitVersion`] carry,
/// so a published default and the override it is diffed against are
/// comparable. It is **not**
/// [`crate::agent_commands::agent_command_checksum`], which digests a single
/// body and is what the legacy `/agent-commands` wire still carries. Those two
/// deliberately disagree even for a one-entry map, so a default digested with
/// the wrong one would never compare equal to its override — an always-drifted
/// baseline, which is strictly worse than the honest "baseline unavailable"
/// state it would replace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AgentTextUnitDefault {
    /// What sort of unit this default is. The corpus is `kind`-discriminated,
    /// so the default layer must be too — otherwise a command and a skill of
    /// the same name collapse into one baseline.
    pub kind: AgentTextUnitKind,

    /// The unit slug, e.g. `vet-plan`. Must satisfy the same
    /// [`validate_agent_text_unit_name`] rules as an override: a default that
    /// could not be named as an override could never be paired with one for a
    /// diff, which is the only thing it is for.
    pub name: String,

    /// The embedded content: **relative path → text**. A command carries one
    /// entry; a skill carries `SKILL.md` plus siblings. Same arity-agnostic
    /// shape as [`AgentTextUnit::files`].
    pub files: AgentTextUnitFiles,

    /// Canonical digest over [`files`](Self::files), from
    /// [`agent_text_unit_files_checksum`].
    ///
    /// Required, unlike [`AgentTextUnit::checksum`] — that one is optional only
    /// to describe rows written before a checksum was computed, whereas a
    /// publish is always freshly computed. The receiving store **recomputes it
    /// and rejects a mismatch**: a client-asserted digest is not evidence.
    /// [`checksum_matches`](Self::checksum_matches) is that check.
    pub checksum: String,

    /// The runner version that published this body, e.g. `"0.4.12"`.
    ///
    /// Carried so the UI can label the baseline **"published by runner
    /// vX.Y.Z"** rather than "the default" — an org whose devices run
    /// different builds has no single default, and the label must not claim
    /// otherwise. It is also the monotonic guard's input: the store rejects a
    /// publish older than the version it already holds.
    ///
    /// That guard is a **mitigation, not a fix** — a genuine downgrade still
    /// wins and equal versions tie-break last-writer — so neither this field
    /// nor the UI built on it may describe the baseline as authoritative.
    pub published_by_version: String,

    /// ISO 8601 (RFC 3339) publish timestamp, matching this module's wire
    /// convention for [`AgentTextUnit::updated_at`] and
    /// [`AgentTextUnitVersion::created_at`].
    pub published_at: String,
}

impl AgentTextUnitDefault {
    /// The key of [`files`](Self::files) holding this default's primary text,
    /// derived from `(kind, name)`.
    ///
    /// Not a stored field: [`AgentTextUnit::entrypoint`] is recomputed on every
    /// response and ignored on write, so carrying one here would be a second
    /// copy of a derived value that could only ever disagree.
    pub fn entrypoint(&self) -> String {
        agent_text_unit_entrypoint(&self.kind, &self.name)
    }

    /// The digest [`checksum`](Self::checksum) *should* hold, recomputed from
    /// [`files`](Self::files).
    pub fn computed_checksum(&self) -> String {
        agent_text_unit_files_checksum(&self.files)
    }

    /// Whether the carried [`checksum`](Self::checksum) matches the content.
    ///
    /// The store calls this after [`validate_agent_text_unit_default`] and
    /// refuses the publish when it is `false`. Kept separate from validation
    /// because a mismatch is a *transport or client* fault, not a malformed
    /// unit, and the two deserve different refusals.
    pub fn checksum_matches(&self) -> bool {
        self.checksum == self.computed_checksum()
    }

    /// This default's content as a path-ordered list of
    /// [`AgentTextUnitFile`]s — the form a diff view iterates.
    pub fn files_sorted(&self) -> Vec<AgentTextUnitFile> {
        AgentTextUnitFile::from_map(&self.files)
    }
}

/// Validate a published default at the write boundary.
///
/// Applies exactly the rules an override must satisfy — name, kind, file
/// paths, size bounds, and the presence of the `(kind, name)` entrypoint — by
/// delegating to [`validate_agent_text_unit_name`],
/// [`validate_agent_text_unit_kind`] and [`validate_agent_text_unit_files`].
/// Nothing is relaxed for defaults: a baseline that could not itself be stored
/// as a unit is not a baseline anything can be diffed against.
///
/// The checksum is deliberately **not** checked here — see
/// [`AgentTextUnitDefault::checksum_matches`] for why it is a separate step.
pub fn validate_agent_text_unit_default(
    unit: &AgentTextUnitDefault,
) -> Result<(), AgentTextUnitError> {
    validate_agent_text_unit_kind(&unit.kind)?;
    validate_agent_text_unit_name(&unit.name)?;
    validate_agent_text_unit_files(&unit.kind, &unit.name, &unit.files)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Write-boundary primitives
// ---------------------------------------------------------------------------

/// Failure modes for agent text units. Mirrors the tagged-enum shape of
/// [`crate::apps::AppError`] and [`crate::agent_commands::AgentCommandError`],
/// so the qontinui-web layer surfaces a rejected unit the same way it surfaces
/// a rejected app registration.
///
/// One variant per raise site in qontinui-web's `AgentTextUnitValidationError`,
/// so a Rust caller can render exactly the refusal the store would give.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, thiserror::Error)]
#[serde(
    tag = "reason",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum AgentTextUnitError {
    #[error("agent text unit name '{name}' is not a valid slug")]
    InvalidName { name: String },

    #[error("agent text unit name '{name}' is a reserved device name on Windows")]
    ReservedName { name: String },

    #[error("agent text unit kind '{kind}' is not a valid slug")]
    InvalidKind { kind: String },

    #[error("file path '{path}' is not a safe relative path")]
    InvalidFilePath { path: String },

    #[error("file path '{path}' contains a reserved device name on Windows")]
    ReservedFilePath { path: String },

    #[error("agent text unit has no files")]
    EmptyFileSet,

    #[error("agent text unit has too many files ({count} > {max})")]
    TooManyFiles { count: usize, max: usize },

    #[error("file '{path}' is blank")]
    BlankFile { path: String },

    #[error("file '{path}' is too large ({bytes} > {max} bytes)")]
    FileTooLarge {
        path: String,
        bytes: usize,
        max: usize,
    },

    #[error("agent text unit is too large ({bytes} > {max} bytes)")]
    UnitTooLarge { bytes: usize, max: usize },

    #[error("a '{kind}' unit named '{name}' must carry its entrypoint '{entrypoint}'")]
    MissingEntrypoint {
        kind: String,
        name: String,
        entrypoint: String,
    },

    #[error("underscore-prefixed unit '{name}' must not be invocable")]
    UnderscoreNotInvocable { name: String },
}

/// Filename stems Windows resolves to a character device no matter what
/// extension follows, so `nul.md` IS the null device, not a file called
/// "nul.md". Matched case-insensitively because the device namespace is.
///
/// Byte-identical to `WINDOWS_RESERVED_STEMS` in
/// `qontinui-web/backend/app/services/agent_text_unit_service.py` and in
/// [`crate::agent_commands`]. The last of those three is scheduled for
/// deletion; until then the duplication is deliberate, because the two Rust
/// modules apply *different* name rules around the same device list.
const WINDOWS_RESERVED_STEMS: &[&str] = &[
    "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
    "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
];

fn is_windows_reserved_stem(stem: &str) -> bool {
    WINDOWS_RESERVED_STEMS
        .iter()
        .any(|r| r.eq_ignore_ascii_case(stem))
}

/// Validate an [`AgentTextUnit::name`] before it is persisted or turned into a
/// path. Returns `Ok(())` for valid slugs.
///
/// Mirrors `validate_unit_name` in qontinui-web: 1–64 chars matching
/// `^_?[a-z0-9][a-z0-9-]*$`, and no Windows reserved device stem (checked after
/// stripping the leading underscore, so `_nul` is refused too).
///
/// ## The underscore widening is deliberate
///
/// The rule is [`crate::agent_commands::validate_agent_command_name`]'s with
/// **one** widening: a single leading underscore. That underscore is the
/// corpus's own marker for a copy-source spec (`_gate-registration`,
/// `_loop-control`) — text the corpus must carry because other units paste from
/// it, but which is not an invocable slash command. Widening the name rule does
/// not widen what can be invoked: see
/// [`validate_agent_text_unit_invocability`].
///
/// ## The two failures this prevents
///
/// The name is not just a database key — the provisioner joins it onto
/// `<workdir>/.claude/<subdir>/` as a filename stem or a directory name.
///
/// - **Escape.** `..`, `/` and `\` are all excluded by the charset, so a unit
///   cannot address anything outside its own `<subdir>`.
/// - **Silent discard (Windows).** The fleet runs on Windows, where a reserved
///   device stem opens a device rather than a file. `std::fs::write` on
///   `nul.md` SUCCEEDS and writes nothing, so a fail-soft provisioner would log
///   a clean success for a unit that does not exist. A no-op that reports
///   success is worse than an error.
pub fn validate_agent_text_unit_name(name: &str) -> Result<(), AgentTextUnitError> {
    let invalid = || AgentTextUnitError::InvalidName { name: name.into() };

    if !(1..=MAX_NAME_LENGTH).contains(&name.len()) {
        return Err(invalid());
    }
    // `^_?[a-z0-9][a-z0-9-]*$` — at most ONE leading underscore.
    let body = name.strip_prefix('_').unwrap_or(name);
    let mut chars = body.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() || c.is_ascii_digit() => {}
        _ => return Err(invalid()),
    }
    if !chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return Err(invalid());
    }
    if is_windows_reserved_stem(body) {
        return Err(AgentTextUnitError::ReservedName { name: name.into() });
    }
    Ok(())
}

/// Validate an [`AgentTextUnit::kind`]: 1–64 chars matching
/// `^[a-z][a-z0-9_]*$`. Mirrors `validate_kind` in qontinui-web.
///
/// This checks **shape, not membership**: a well-formed but unrecognized kind
/// is deliberately accepted, because rejecting it here would turn the open
/// discriminator back into a closed enum enforced one layer down. Ask
/// [`AgentTextUnitKind::is_known`] when you need "does this build understand
/// it?", and [`AgentTextUnitKind::provisioning_target`] when you need "may I
/// write it?".
///
/// Note the charset differs from a unit `name`: `_` is legal here and `-` is
/// not, which is why [`AgentTextUnitKind::AGENTS_SKILL`] is `agents_skill`.
pub fn validate_agent_text_unit_kind(kind: &AgentTextUnitKind) -> Result<(), AgentTextUnitError> {
    let value = kind.as_str();
    let invalid = || AgentTextUnitError::InvalidKind {
        kind: value.to_string(),
    };

    if !(1..=MAX_NAME_LENGTH).contains(&value.len()) {
        return Err(invalid());
    }
    let mut chars = value.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() => {}
        _ => return Err(invalid()),
    }
    if !chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') {
        return Err(invalid());
    }
    Ok(())
}

/// Validate one key of a `files` map: a path relative to the unit's own
/// directory. Mirrors `validate_relative_path` in qontinui-web.
///
/// Rejected, each because it lets a write escape the unit's directory or land
/// somewhere other than where the map says:
///
/// - **Empty, or over [`MAX_PATH_BYTES`] UTF-8 bytes.**
/// - **A backslash anywhere** — Windows treats `\` as a separator, so
///   `..\x` would smuggle a traversal through a forward-slash-only check on
///   the machine this fleet actually runs on.
/// - **A control character** (`< 0x20`, or `0x7F`) — including the CR that
///   would otherwise make [`agent_text_unit_files_checksum`]'s decision not to
///   CR-strip keys unsafe.
/// - **A leading `/`, or a drive letter** (`C:…`) — absolute by either OS's
///   rules.
/// - **More than [`MAX_PATH_SEGMENTS`] segments.**
/// - **An empty segment** (`a//b`, `a/`) — collapses unpredictably.
/// - **A `.` or `..` segment** — the traversal, in *any* position:
///   `x/../../settings.json` escapes just as well as a leading `..`.
/// - **A segment with leading or trailing whitespace, or a trailing `.`** —
///   Windows silently strips all three, so two distinct map keys would race to
///   write one file.
/// - **A segment whose stem is a Windows device name** (`nul.md`, `con.sh`) —
///   `fs::write` succeeds and discards.
///
/// Note what is deliberately **not** restricted: the character set beyond the
/// above. Dotfiles, spaces inside a segment and non-ASCII are all legal, matching
/// the Python validator exactly. A stricter Rust rule would refuse to provision
/// units the store happily accepted, which is the divergence this shared rule
/// exists to prevent.
pub fn validate_agent_text_unit_file_path(path: &str) -> Result<(), AgentTextUnitError> {
    let invalid = || AgentTextUnitError::InvalidFilePath { path: path.into() };

    if path.is_empty() || path.len() > MAX_PATH_BYTES {
        return Err(invalid());
    }
    if path.contains('\\') {
        return Err(invalid());
    }
    if path.chars().any(|c| (c as u32) < 0x20 || c == '\u{7f}') {
        return Err(invalid());
    }
    if path.starts_with('/') {
        return Err(invalid());
    }
    // `^[A-Za-z]:` — a drive-relative or drive-absolute Windows path.
    {
        let mut chars = path.chars();
        if matches!(chars.next(), Some(c) if c.is_ascii_alphabetic()) && chars.next() == Some(':') {
            return Err(invalid());
        }
    }

    let segments: Vec<&str> = path.split('/').collect();
    if segments.len() > MAX_PATH_SEGMENTS {
        return Err(invalid());
    }

    for segment in segments {
        if segment.is_empty() || segment == "." || segment == ".." {
            return Err(invalid());
        }
        if segment.trim() != segment || segment.ends_with('.') {
            return Err(invalid());
        }
        let stem = segment.split('.').next().unwrap_or(segment);
        if is_windows_reserved_stem(stem) {
            return Err(AgentTextUnitError::ReservedFilePath { path: path.into() });
        }
    }

    Ok(())
}

/// Validate a whole `files` map for a unit of `(kind, name)`. Mirrors
/// `validate_files` in qontinui-web, including the order the checks run in, so
/// both surfaces report the same first failure for the same input.
///
/// Non-empty, at most [`MAX_FILES_PER_UNIT`] entries, every key a safe relative
/// path, no blank file, each file within [`MAX_FILE_BYTES`], the whole bundle
/// within [`MAX_UNIT_BYTES`], and the unit's
/// [`agent_text_unit_entrypoint`] actually present.
///
/// The blank-file rejection mirrors the runner's `validate_override`: a blank
/// file in a corpus is indistinguishable from a truncation bug, and a blank
/// override shadowing a working default is the exact failure the fail-soft
/// chain exists to avoid.
pub fn validate_agent_text_unit_files(
    kind: &AgentTextUnitKind,
    name: &str,
    files: &AgentTextUnitFiles,
) -> Result<(), AgentTextUnitError> {
    if files.is_empty() {
        return Err(AgentTextUnitError::EmptyFileSet);
    }
    if files.len() > MAX_FILES_PER_UNIT {
        return Err(AgentTextUnitError::TooManyFiles {
            count: files.len(),
            max: MAX_FILES_PER_UNIT,
        });
    }

    let mut total = 0usize;
    for (path, content) in files {
        validate_agent_text_unit_file_path(path)?;
        if content.trim().is_empty() {
            return Err(AgentTextUnitError::BlankFile { path: path.clone() });
        }
        let size = content.len();
        if size > MAX_FILE_BYTES {
            return Err(AgentTextUnitError::FileTooLarge {
                path: path.clone(),
                bytes: size,
                max: MAX_FILE_BYTES,
            });
        }
        total += size;
    }

    if total > MAX_UNIT_BYTES {
        return Err(AgentTextUnitError::UnitTooLarge {
            bytes: total,
            max: MAX_UNIT_BYTES,
        });
    }

    let entrypoint = agent_text_unit_entrypoint(kind, name);
    if !files.contains_key(&entrypoint) {
        return Err(AgentTextUnitError::MissingEntrypoint {
            kind: kind.as_str().to_string(),
            name: name.to_string(),
            entrypoint,
        });
    }

    Ok(())
}

/// Enforce the underscore/invocability pairing: a `_`-prefixed unit is a
/// copy-source spec and must never be invocable.
///
/// The Rust statement of qontinui-web's
/// `ck_agent_text_unit_underscore_not_invocable` CHECK. Both exist because the
/// underscore is a *convention* in the corpus and `is_invocable` is the
/// machine-readable form — a check is what stops the two from disagreeing.
pub fn validate_agent_text_unit_invocability(
    name: &str,
    is_invocable: bool,
) -> Result<(), AgentTextUnitError> {
    if name.starts_with('_') && is_invocable {
        return Err(AgentTextUnitError::UnderscoreNotInvocable { name: name.into() });
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// agent-text-unit-files/v1 — the canonical digest
// ---------------------------------------------------------------------------

/// The exact byte stream [`agent_text_unit_files_checksum`] hashes:
/// **`agent-text-unit-files/v1`**.
///
/// The spec was defined by qontinui-web's
/// `agent_text_unit_service.compute_files_checksum`, and this is a port of it,
/// not a second design. Stated precisely, because any writing surface has to
/// reproduce it byte for byte:
///
/// 1. Take the entries as `(path, text)` pairs.
/// 2. `text` is CR-stripped (every `\r` removed) then UTF-8 encoded. `path` is
///    UTF-8 encoded as-is — it can contain no `\r`, because
///    [`validate_agent_text_unit_file_path`] rejects control characters.
/// 3. Sort the pairs by the **raw UTF-8 bytes of `path`**, ascending. Not a
///    locale collation, not a Unicode normalization: byte order, which is total
///    and identical in every language. (Paths are unique within a map, so no
///    tiebreaker is needed.) [`AgentTextUnitFiles`] is a `BTreeMap<String, _>`
///    and Rust orders `str` bytewise, so iteration already *is* this order.
/// 4. Concatenate, for each pair in that order:
///    ```text
///    ascii_decimal(len(path_bytes)) b"\n" path_bytes
///    ascii_decimal(len(text_bytes)) b"\n" text_bytes
///    ```
/// 5. The digest is `"sha256-" + hex(sha256(that stream))`.
///
/// **Length-framing is what makes the encoding injective** — every field is a
/// decimal length, a newline, then exactly that many bytes, so the stream parses
/// back to exactly one sorted pair list and no two distinct maps can collide.
/// A naive `"\n".join(...)` does not have that property: a path or a body
/// containing the separator forges another map's stream. Do not "simplify" this
/// to a joined string.
///
/// **CR-stripping** is why the digest survives the round trip. The same text
/// crosses Postgres, JSON and a Windows filesystem before two of these are
/// compared, and a CRLF-normalizing hop anywhere on that path would otherwise
/// flip the digest and report unchanged content as changed — making the Phase 5
/// re-import append a spurious version on every single run.
///
/// It deliberately does **not** reduce to
/// [`crate::agent_commands::agent_command_checksum`] for a one-entry map:
/// `{"a.md": X}` and `{"b.md": X}` are different units, and a digest that
/// conflated them would defeat the point.
pub fn agent_text_unit_files_canonical_stream(files: &AgentTextUnitFiles) -> Vec<u8> {
    let mut stream = Vec::new();
    for (path, text) in files {
        let path_bytes = path.as_bytes();
        let text_bytes = text.replace('\r', "").into_bytes();
        stream.extend_from_slice(path_bytes.len().to_string().as_bytes());
        stream.push(b'\n');
        stream.extend_from_slice(path_bytes);
        stream.extend_from_slice(text_bytes.len().to_string().as_bytes());
        stream.push(b'\n');
        stream.extend_from_slice(&text_bytes);
    }
    stream
}

/// Canonical checksum over a whole `files` map: `"sha256-<hex>"`.
///
/// See [`agent_text_unit_files_canonical_stream`] for the exact bytes and the
/// rationale. The `"sha256-"` prefix follows this crate's output convention
/// ([`crate::canonical_hash`], [`crate::spec_check`]) and names the algorithm
/// inline, so a future change is distinguishable rather than silently
/// reinterpreted.
pub fn agent_text_unit_files_checksum(files: &AgentTextUnitFiles) -> String {
    let mut hasher = Sha256::new();
    hasher.update(agent_text_unit_files_canonical_stream(files));
    format!("sha256-{}", hex::encode(hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn files(entries: &[(&str, &str)]) -> AgentTextUnitFiles {
        entries
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn sha256_hex(bytes: &[u8]) -> String {
        let mut h = Sha256::new();
        h.update(bytes);
        hex::encode(h.finalize())
    }

    // -- AgentTextUnitKind --------------------------------------------------

    #[test]
    fn kind_is_a_bare_string_on_the_wire() {
        let json = serde_json::to_string(&AgentTextUnitKind::skill()).unwrap();
        assert_eq!(json, r#""skill""#);
        let back: AgentTextUnitKind = serde_json::from_str(r#""command""#).unwrap();
        assert_eq!(back, AgentTextUnitKind::command());
    }

    #[test]
    fn kind_accepts_a_value_this_build_does_not_know() {
        // The whole reason `kind` is not an enum: a newer producer's value must
        // round-trip through an older reader rather than fail the document.
        let forward: AgentTextUnitKind = serde_json::from_str(r#""hook""#).unwrap();
        assert_eq!(forward.as_str(), "hook");
        assert!(!forward.is_known());
        assert_eq!(forward.provisioning_target(), None);
        assert_eq!(serde_json::to_string(&forward).unwrap(), r#""hook""#);
    }

    #[test]
    fn known_kinds_map_to_their_provisioning_targets() {
        assert_eq!(
            AgentTextUnitKind::command().provisioning_target(),
            Some(AgentTextUnitTarget::File { subdir: "commands" })
        );
        assert_eq!(
            AgentTextUnitKind::skill().provisioning_target(),
            Some(AgentTextUnitTarget::Directory { subdir: "skills" })
        );
        assert_eq!(
            AgentTextUnitKind::new(AgentTextUnitKind::AGENT).provisioning_target(),
            Some(AgentTextUnitTarget::File { subdir: "agents" })
        );
        // Known, deliberately unprovisioned — pi/Codex have no spawn hook.
        let agents_skill = AgentTextUnitKind::new(AgentTextUnitKind::AGENTS_SKILL);
        assert!(agents_skill.is_known());
        assert_eq!(agents_skill.provisioning_target(), None);
    }

    #[test]
    fn every_known_kind_is_a_legal_kind_slug() {
        // `agents_skill` spells its separator with `_` because the kind charset
        // admits no `-`. A hyphenated constant would be unstorable.
        for kind in AgentTextUnitKind::KNOWN {
            let k = AgentTextUnitKind::new(*kind);
            assert!(
                validate_agent_text_unit_kind(&k).is_ok(),
                "{kind:?} must satisfy the kind rule"
            );
        }
    }

    #[test]
    fn entrypoint_matches_the_python_table_including_its_fallback() {
        assert_eq!(
            agent_text_unit_entrypoint(&AgentTextUnitKind::command(), "vet-plan"),
            "vet-plan.md"
        );
        assert_eq!(
            agent_text_unit_entrypoint(&AgentTextUnitKind::skill(), "coord-revive"),
            "SKILL.md"
        );
        // An unknown kind falls back to `<name>.md` rather than failing — the
        // discriminator is widenable, so a hard error here would re-close it.
        assert_eq!(
            agent_text_unit_entrypoint(&AgentTextUnitKind::new("agent"), "code-reviewer"),
            "code-reviewer.md"
        );
    }

    #[test]
    fn target_composes_the_claude_relative_path() {
        let cmd = AgentTextUnitKind::command().provisioning_target().unwrap();
        assert_eq!(
            cmd.relative_path("vet-plan", "vet-plan.md"),
            "commands/vet-plan.md"
        );
        assert!(!cmd.is_directory());

        let skill = AgentTextUnitKind::skill().provisioning_target().unwrap();
        assert_eq!(
            skill.relative_path("coord-revive", "coord-revive.sh"),
            "skills/coord-revive/coord-revive.sh"
        );
        assert!(skill.is_directory());
    }

    // -- AgentTextUnit / AgentTextUnitVersion -------------------------------

    fn sample_files() -> AgentTextUnitFiles {
        files(&[
            ("SKILL.md", "# coord-revive\nrun the script\n"),
            ("coord-revive.sh", "#!/usr/bin/env bash\n"),
        ])
    }

    fn sample_unit() -> AgentTextUnit {
        let content = sample_files();
        AgentTextUnit {
            id: "11111111-1111-4111-8111-111111111111".to_string(),
            kind: AgentTextUnitKind::skill(),
            name: "coord-revive".to_string(),
            organization_id: None,
            created_by_user_id: None,
            checksum: Some(agent_text_unit_files_checksum(&content)),
            entrypoint: "SKILL.md".to_string(),
            files: content,
            is_shared: false,
            is_invocable: true,
            current_version: 1,
            source: AGENT_TEXT_UNIT_SOURCE_FLEET.to_string(),
            source_path: Some(".claude/skills/coord-revive/".to_string()),
            source_commit: Some("0".repeat(40)),
            created_at: "2026-08-24T00:00:00Z".to_string(),
            updated_at: "2026-08-24T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn unit_omits_absent_optionals() {
        let mut unit = sample_unit();
        unit.source_path = None;
        unit.source_commit = None;
        let json = serde_json::to_value(&unit).unwrap();
        assert!(json.get("organization_id").is_none());
        assert!(json.get("created_by_user_id").is_none());
        assert!(json.get("source_path").is_none());
        assert!(json.get("source_commit").is_none());
        // `is_invocable` is NOT skipped: the landed response always emits it,
        // and a binding that saw it absent would render `null` rather than the
        // real default.
        assert_eq!(json["is_invocable"], true);
        assert_eq!(json["kind"], "skill");
        assert_eq!(json["source"], "fleet");
    }

    #[test]
    fn unit_deserializes_the_landed_qontinui_web_response_shape() {
        // Field for field what `AgentTextUnitResponse` emits, including a NULL
        // checksum and an absent `is_invocable`.
        let unit: AgentTextUnit = serde_json::from_str(
            r##"{
                "id": "abc",
                "organization_id": null,
                "created_by_user_id": null,
                "kind": "command",
                "name": "vet-plan",
                "files": {"vet-plan.md": "# /vet-plan\n"},
                "entrypoint": "vet-plan.md",
                "checksum": null,
                "is_shared": false,
                "is_invocable": true,
                "current_version": 3,
                "source": "fleet",
                "created_at": "2026-08-24T00:00:00Z",
                "updated_at": "2026-08-24T01:00:00Z"
            }"##,
        )
        .unwrap();
        assert_eq!(unit.kind, AgentTextUnitKind::command());
        assert_eq!(unit.checksum, None);
        assert!(unit.is_invocable);
        assert_eq!(unit.current_version, 3);
        assert_eq!(unit.organization_id, None);
        assert_eq!(unit.source, "fleet");
        assert_eq!(unit.files.len(), 1);
    }

    #[test]
    fn unit_defaults_match_the_python_defaults() {
        // `is_shared` false, `is_invocable` true, `source` "user".
        let unit: AgentTextUnit = serde_json::from_str(
            r##"{
                "id": "abc",
                "kind": "command",
                "name": "vet-plan",
                "files": {"vet-plan.md": "x"},
                "entrypoint": "vet-plan.md",
                "current_version": 1,
                "created_at": "2026-08-24T00:00:00Z",
                "updated_at": "2026-08-24T00:00:00Z"
            }"##,
        )
        .unwrap();
        assert!(!unit.is_shared);
        assert!(unit.is_invocable);
        assert_eq!(unit.source, AGENT_TEXT_UNIT_SOURCE_USER);
    }

    #[test]
    fn unit_roundtrips_provenance() {
        let unit = sample_unit();
        let back: AgentTextUnit =
            serde_json::from_str(&serde_json::to_string(&unit).unwrap()).unwrap();
        assert_eq!(back, unit);
        assert_eq!(
            back.source_path.as_deref(),
            Some(".claude/skills/coord-revive/")
        );
    }

    #[test]
    fn non_invocable_unit_serializes_the_flag() {
        let mut unit = sample_unit();
        unit.name = "_gate-registration".to_string();
        unit.kind = AgentTextUnitKind::command();
        unit.is_invocable = false;
        let json = serde_json::to_value(&unit).unwrap();
        assert_eq!(json["is_invocable"], false);
        assert!(validate_agent_text_unit_name(&unit.name).is_ok());
        assert!(validate_agent_text_unit_invocability(&unit.name, unit.is_invocable).is_ok());
    }

    fn sample_version() -> AgentTextUnitVersion {
        let content = sample_files();
        AgentTextUnitVersion {
            id: "v6".to_string(),
            agent_text_unit_id: "abc".to_string(),
            version_number: 6,
            checksum: Some(agent_text_unit_files_checksum(&content)),
            files: content,
            created_by_user_id: Some("user-1".to_string()),
            change_description: Some("Restored from version 2".to_string()),
            restored_from: Some(2),
            created_at: "2026-08-24T02:00:00Z".to_string(),
        }
    }

    #[test]
    fn version_roundtrips_restore_provenance_and_the_file_map() {
        let v = sample_version();
        let back: AgentTextUnitVersion =
            serde_json::from_str(&serde_json::to_string(&v).unwrap()).unwrap();
        assert_eq!(back, v);
        assert_eq!(back.restored_from, Some(2));
        assert_eq!(
            back.files.get("coord-revive.sh").map(String::as_str),
            Some("#!/usr/bin/env bash\n")
        );
    }

    #[test]
    fn version_files_serialize_as_a_json_object() {
        let json = serde_json::to_value(sample_version()).unwrap();
        assert!(json["files"].is_object());
        assert_eq!(
            json["files"]["SKILL.md"],
            "# coord-revive\nrun the script\n"
        );
    }

    #[test]
    fn single_file_kind_is_the_degenerate_map() {
        // A command is one entry, keyed by the kind's entrypoint — no special
        // case anywhere in the model.
        let kind = AgentTextUnitKind::command();
        let content = files(&[("vet-plan.md", "# /vet-plan\n")]);
        assert_eq!(content.len(), 1);
        assert_eq!(
            content.keys().next().map(String::as_str),
            Some(agent_text_unit_entrypoint(&kind, "vet-plan").as_str())
        );
        assert!(validate_agent_text_unit_files(&kind, "vet-plan", &content).is_ok());
    }

    #[test]
    fn files_sorted_is_path_ordered_and_round_trips() {
        let v = sample_version();
        let entries = v.files_sorted();
        assert_eq!(
            entries.iter().map(|f| f.path.as_str()).collect::<Vec<_>>(),
            vec!["SKILL.md", "coord-revive.sh"]
        );
        assert_eq!(AgentTextUnitFile::into_map(entries), v.files);
    }

    #[test]
    fn file_entry_roundtrips() {
        let f = AgentTextUnitFile {
            path: "SKILL.md".to_string(),
            text: "# skill\n".to_string(),
        };
        let back: AgentTextUnitFile =
            serde_json::from_str(&serde_json::to_string(&f).unwrap()).unwrap();
        assert_eq!(back, f);
    }

    // -- validate_agent_text_unit_name / _kind ------------------------------

    #[test]
    fn name_accepts_the_real_corpus_slugs_including_the_copy_source_specs() {
        for good in [
            "vet-plan",
            "implement-plan",
            "coord-revive",
            "coord-pr-label",
            "pr-status",
            "a",
            "plan2",
            // The underscore widening: copy-source specs must be storable.
            "_gate-registration",
            "_loop-control",
        ] {
            assert!(
                validate_agent_text_unit_name(good).is_ok(),
                "{good:?} must be storable"
            );
        }
        assert!(validate_agent_text_unit_name(&"a".repeat(MAX_NAME_LENGTH)).is_ok());
    }

    #[test]
    fn name_rejects_path_escapes_and_shape_violations() {
        for bad in [
            "..",
            "../vet-plan",
            "sub/vet-plan",
            "sub\\vet-plan",
            "/etc/passwd",
            "C:\\windows\\system32",
            ".hidden",
            "",
            "-leading-hyphen",
            "Vet-Plan",
            "vet plan",
            "vet.plan",
            "vet_plan", // `_` is legal only as the FIRST character
            "__double",
            "_",
        ] {
            assert_eq!(
                validate_agent_text_unit_name(bad),
                Err(AgentTextUnitError::InvalidName { name: bad.into() }),
                "{bad:?} must not be writable as a path component"
            );
        }
        assert!(validate_agent_text_unit_name(&"a".repeat(MAX_NAME_LENGTH + 1)).is_err());
    }

    #[test]
    fn name_rejects_windows_device_stems_underscore_or_not() {
        for reserved in ["nul", "con", "prn", "aux", "com1", "lpt9"] {
            assert_eq!(
                validate_agent_text_unit_name(reserved),
                Err(AgentTextUnitError::ReservedName {
                    name: reserved.into()
                })
            );
        }
        // The stem is checked AFTER stripping the underscore, so the widening
        // cannot be used to smuggle a device name past the rule.
        assert_eq!(
            validate_agent_text_unit_name("_nul"),
            Err(AgentTextUnitError::ReservedName {
                name: "_nul".into()
            })
        );
        // Uppercase is caught by the charset first — the point is only that it
        // never reaches the filesystem, not which arm rejects it.
        assert!(validate_agent_text_unit_name("NUL").is_err());
        // A reserved stem is only reserved bare; extending it is a real name.
        assert!(validate_agent_text_unit_name("console").is_ok());
        assert!(validate_agent_text_unit_name("nul-plan").is_ok());
    }

    #[test]
    fn kind_validation_is_shape_only_not_membership() {
        assert!(validate_agent_text_unit_kind(&AgentTextUnitKind::command()).is_ok());
        assert!(validate_agent_text_unit_kind(&AgentTextUnitKind::skill()).is_ok());
        // Well-formed but unknown: accepted on purpose.
        let forward = AgentTextUnitKind::new("hook");
        assert!(validate_agent_text_unit_kind(&forward).is_ok());
        assert!(!forward.is_known());

        for bad in ["", "Command", "sub/kind", "..", "agents-skill", "2fast"] {
            assert_eq!(
                validate_agent_text_unit_kind(&AgentTextUnitKind::new(bad)),
                Err(AgentTextUnitError::InvalidKind { kind: bad.into() }),
                "{bad:?} is not a well-formed kind"
            );
        }
    }

    #[test]
    fn underscore_units_may_not_be_invocable() {
        assert_eq!(
            validate_agent_text_unit_invocability("_gate-registration", true),
            Err(AgentTextUnitError::UnderscoreNotInvocable {
                name: "_gate-registration".into()
            })
        );
        assert!(validate_agent_text_unit_invocability("_gate-registration", false).is_ok());
        assert!(validate_agent_text_unit_invocability("vet-plan", true).is_ok());
        assert!(validate_agent_text_unit_invocability("vet-plan", false).is_ok());
    }

    // -- validate_agent_text_unit_file_path ---------------------------------

    #[test]
    fn file_path_accepts_the_real_skill_files() {
        for good in [
            "SKILL.md",
            "coord-revive.sh",
            "set-label.sh",
            "coord-read.ps1",
            "references/palette.md",
            "a_b-c.2.md",
            // Deliberately legal, matching the Python validator: a stricter
            // Rust rule would refuse to provision what the store accepted.
            ".gitkeep",
            "notes/my file.md",
            "références/naïve.md",
        ] {
            assert!(
                validate_agent_text_unit_file_path(good).is_ok(),
                "{good:?} must be storable"
            );
        }
    }

    #[test]
    fn file_path_refuses_every_traversal_form() {
        // Falsification target from the plan: if any of these is admitted, the
        // provisioner writes outside the unit's directory.
        for bad in [
            "..",
            "../SKILL.md",
            "a/../../SKILL.md",
            "a/./b.md",
            "/SKILL.md",
            "/",
            "sub\\SKILL.md",
            "..\\..\\settings.json",
            "C:/SKILL.md",
            "C:SKILL.md",
            "a//b.md",
            "a/",
            "",
            "trailing.",
            "trailing ",
            " leading.md",
            "a/b/c/d/e/f/g/h/i.md",
            "carriage\rreturn.md",
            "nul\u{0}byte.md",
        ] {
            assert_eq!(
                validate_agent_text_unit_file_path(bad),
                Err(AgentTextUnitError::InvalidFilePath { path: bad.into() }),
                "{bad:?} must be refused"
            );
        }
        assert!(validate_agent_text_unit_file_path(&"a".repeat(MAX_PATH_BYTES + 1)).is_err());
    }

    #[test]
    fn file_path_refuses_windows_device_segments() {
        for bad in ["nul.md", "CON.sh", "sub/com1.md", "aux"] {
            assert_eq!(
                validate_agent_text_unit_file_path(bad),
                Err(AgentTextUnitError::ReservedFilePath { path: bad.into() }),
                "{bad:?} opens a device rather than a file on Windows"
            );
        }
        // Reserved only bare; extending the stem is a real filename.
        assert!(validate_agent_text_unit_file_path("console.md").is_ok());
        assert!(validate_agent_text_unit_file_path("nul-notes.md").is_ok());
    }

    // -- validate_agent_text_unit_files -------------------------------------

    #[test]
    fn files_must_be_non_empty_and_carry_the_entrypoint() {
        let kind = AgentTextUnitKind::skill();
        assert_eq!(
            validate_agent_text_unit_files(&kind, "coord-revive", &AgentTextUnitFiles::new()),
            Err(AgentTextUnitError::EmptyFileSet)
        );
        assert_eq!(
            validate_agent_text_unit_files(
                &kind,
                "coord-revive",
                &files(&[("coord-revive.sh", "echo hi")])
            ),
            Err(AgentTextUnitError::MissingEntrypoint {
                kind: "skill".into(),
                name: "coord-revive".into(),
                entrypoint: "SKILL.md".into(),
            })
        );
        assert!(validate_agent_text_unit_files(&kind, "coord-revive", &sample_files()).is_ok());
    }

    #[test]
    fn files_reject_blank_content_and_oversize_bundles() {
        let kind = AgentTextUnitKind::command();
        assert_eq!(
            validate_agent_text_unit_files(&kind, "vet-plan", &files(&[("vet-plan.md", "   \n")])),
            Err(AgentTextUnitError::BlankFile {
                path: "vet-plan.md".into()
            })
        );

        let too_big = "x".repeat(MAX_FILE_BYTES + 1);
        assert_eq!(
            validate_agent_text_unit_files(&kind, "vet-plan", &files(&[("vet-plan.md", &too_big)])),
            Err(AgentTextUnitError::FileTooLarge {
                path: "vet-plan.md".into(),
                bytes: MAX_FILE_BYTES + 1,
                max: MAX_FILE_BYTES,
            })
        );

        let mut many = AgentTextUnitFiles::new();
        for i in 0..=MAX_FILES_PER_UNIT {
            many.insert(format!("f{i}.md"), "x".to_string());
        }
        assert_eq!(
            validate_agent_text_unit_files(&kind, "vet-plan", &many),
            Err(AgentTextUnitError::TooManyFiles {
                count: MAX_FILES_PER_UNIT + 1,
                max: MAX_FILES_PER_UNIT,
            })
        );
    }

    #[test]
    fn error_serializes_with_the_apps_tagged_shape() {
        let json = serde_json::to_value(AgentTextUnitError::InvalidFilePath {
            path: "../x".to_string(),
        })
        .unwrap();
        assert_eq!(json["reason"], "invalid-file-path");
        assert_eq!(json["path"], "../x");

        let json = serde_json::to_value(AgentTextUnitError::FileTooLarge {
            path: "a.md".to_string(),
            bytes: 2,
            max: 1,
        })
        .unwrap();
        assert_eq!(json["reason"], "file-too-large");
        assert_eq!(json["max"], 1);
    }

    // -- agent-text-unit-files/v1 -------------------------------------------
    //
    // Ported one-for-one from qontinui-web's
    // `backend/tests/test_agent_text_units_db.py::TestFilesChecksum`, which is
    // the fixture the two implementations are reconciled against. Keep the
    // names recognizable against that class.

    #[test]
    fn key_order_does_not_move_the_digest() {
        let a = files(&[
            ("SKILL.md", "one"),
            ("run.sh", "two"),
            ("z/deep.md", "three"),
        ]);
        let b = files(&[
            ("z/deep.md", "three"),
            ("run.sh", "two"),
            ("SKILL.md", "one"),
        ]);
        assert_eq!(
            agent_text_unit_files_checksum(&a),
            agent_text_unit_files_checksum(&b)
        );
    }

    #[test]
    fn it_is_cr_invariant_like_the_body_digest() {
        assert_eq!(
            agent_text_unit_files_checksum(&files(&[("a.md", "x\r\ny\r\n")])),
            agent_text_unit_files_checksum(&files(&[("a.md", "x\ny\n")]))
        );
    }

    #[test]
    fn length_framing_makes_the_encoding_injective() {
        // The property a naive `"\n".join(...)` does not have: no two distinct
        // maps may collide, however the separators fall inside the content.
        assert_ne!(
            agent_text_unit_files_checksum(&files(&[("ab.md", "x")])),
            agent_text_unit_files_checksum(&files(&[("a.md", ""), ("b.md", "x")]))
        );
        assert_ne!(
            agent_text_unit_files_checksum(&files(&[("a.md", "1\n2")])),
            agent_text_unit_files_checksum(&files(&[("a.md", "1"), ("2", "a.md")]))
        );
    }

    #[test]
    fn the_path_is_part_of_the_digest() {
        assert_ne!(
            agent_text_unit_files_checksum(&files(&[("a.md", "same")])),
            agent_text_unit_files_checksum(&files(&[("b.md", "same")]))
        );
    }

    #[test]
    fn it_does_not_reduce_to_the_body_digest() {
        // Deliberate: a one-entry map still carries its path, so it cannot
        // equal a digest taken over the body alone.
        assert_ne!(
            agent_text_unit_files_checksum(&files(&[("a.md", "body")])),
            crate::agent_commands::agent_command_checksum("body")
        );
    }

    #[test]
    fn pinned_vector() {
        // Hand-computed from the five documented steps — the fixture a second
        // implementation is reconciled against. Keys sort by raw UTF-8 bytes,
        // so `SKILL.md` < `run.sh` because `S` (0x53) < `r` (0x72).
        let stream: &[u8] = b"8\nSKILL.md3\nhi\n6\nrun.sh1\nx";
        let content = files(&[("SKILL.md", "hi\n"), ("run.sh", "x")]);
        assert_eq!(
            agent_text_unit_files_canonical_stream(&content),
            stream.to_vec()
        );
        assert_eq!(
            agent_text_unit_files_checksum(&content),
            format!("sha256-{}", sha256_hex(stream))
        );
    }

    #[test]
    fn sorting_is_by_raw_utf8_bytes_not_by_case_folding() {
        // `S` (0x53) sorts before `r` (0x72); a case-insensitive or locale
        // collation would order these the other way and produce a different
        // digest.
        let stream: &[u8] = b"1\nS1\na1\nr1\nb";
        let content = files(&[("r", "b"), ("S", "a")]);
        assert_eq!(
            agent_text_unit_files_canonical_stream(&content),
            stream.to_vec()
        );
        assert_eq!(
            agent_text_unit_files_checksum(&content),
            format!("sha256-{}", sha256_hex(stream))
        );
    }

    #[test]
    fn checksum_has_the_crate_prefix_and_64_hex_chars() {
        let h = agent_text_unit_files_checksum(&files(&[("SKILL.md", "# skill\n")]));
        assert!(h.starts_with("sha256-"), "got {h}");
        let hex_part = &h["sha256-".len()..];
        assert_eq!(hex_part.len(), 64);
        assert!(hex_part.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn checksum_round_trips_onto_the_records() {
        let v = sample_version();
        assert_eq!(
            v.checksum.as_deref(),
            Some(agent_text_unit_files_checksum(&v.files).as_str())
        );
        let back: AgentTextUnitVersion =
            serde_json::from_str(&serde_json::to_string(&v).unwrap()).unwrap();
        assert_eq!(
            back.checksum.as_deref(),
            Some(agent_text_unit_files_checksum(&back.files).as_str())
        );
    }

    #[test]
    fn digests_match_the_python_implementation_byte_for_byte() {
        // The reconciliation that actually proves the port: these hex strings
        // were produced by RUNNING qontinui-web's
        // `agent_text_unit_service.compute_files_checksum` (2026-08-24) over
        // the same inputs, not by re-deriving them here. If a future edit to
        // either implementation moves a digest, this fails on the Rust side and
        // the Python conformance class fails on the other — which is the point:
        // a spec stated in prose cannot fail, and the whole reason the digest
        // exists is that the two surfaces must agree.
        for (files_in, expected) in [
            (
                files(&[
                    ("SKILL.md", "# coord-revive\nrun the script\n"),
                    ("coord-revive.sh", "#!/usr/bin/env bash\n"),
                ]),
                "sha256-72d07280e4ff0f72f46b9a47e5ade16960c556d73c4c7a89e102b7e61fbc065d",
            ),
            (
                files(&[("SKILL.md", "hi\n"), ("run.sh", "x")]),
                "sha256-e0e50bed79005cfb3e09c488fe374297232bf438beaee54c3a1dd972828b9f45",
            ),
            (
                files(&[("r", "b"), ("S", "a")]),
                "sha256-3746938afa65bd1f60512f87645c3ee1d19ad6e142a78b19b96171d25d1d562d",
            ),
            (
                files(&[("a.md", "x\r\ny\r\n")]),
                "sha256-4aae522c5ab6be12c342c5363dbab82d09c6f645f4075828ea9f87168b504f5c",
            ),
            (
                files(&[("é.md", "λ")]),
                "sha256-3b504b6796f1a0054aa0eac9edf70db67d28398de8e1bab40bf3b9a324952148",
            ),
            (
                files(&[("vet-plan.md", "# /vet-plan\n")]),
                "sha256-9d821fc6a34805612729c37c8b98b7aeb2e52ee2675ca18b4ac4a4bd64c9aac8",
            ),
        ] {
            assert_eq!(
                agent_text_unit_files_checksum(&files_in),
                expected,
                "diverged from the Python implementation for {:?}",
                files_in.keys().collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn canonical_stream_length_prefixes_are_utf8_byte_counts_not_char_counts() {
        // A multi-byte path or body must frame by BYTES, or the stream stops
        // parsing back and the injectivity argument collapses.
        let content = files(&[("é.md", "λ")]);
        assert_eq!(
            agent_text_unit_files_canonical_stream(&content),
            b"5\n\xc3\xa9.md2\n\xce\xbb".to_vec()
        );
    }

    // -- AgentTextUnitDefault (the embedded-default layer) -------------------

    fn a_default(name: &str, entries: &[(&str, &str)]) -> AgentTextUnitDefault {
        let files = files(entries);
        AgentTextUnitDefault {
            kind: AgentTextUnitKind::command(),
            name: name.to_string(),
            checksum: agent_text_unit_files_checksum(&files),
            files,
            published_by_version: "0.4.12".to_string(),
            published_at: "2026-08-31T12:00:00Z".to_string(),
        }
    }

    #[test]
    fn default_round_trips_over_the_wire() {
        let unit = a_default("vet-plan", &[("vet-plan.md", "# Vet Plan\n")]);
        let json = serde_json::to_string(&unit).unwrap();
        let back: AgentTextUnitDefault = serde_json::from_str(&json).unwrap();
        assert_eq!(back, unit);
    }

    #[test]
    fn default_carries_no_organization_on_the_wire() {
        // Org scoping is assigned server-side from the publishing bearer. A
        // client-supplied org would let any signed-in user write another
        // tenant's baseline, so the field must not exist to be supplied.
        let json = serde_json::to_value(a_default("gate", &[("gate.md", "x")])).unwrap();
        assert!(json.get("organization_id").is_none());
        assert!(json.get("id").is_none());
    }

    #[test]
    fn default_checksum_matches_its_own_content() {
        assert!(a_default("policy", &[("policy.md", "body")]).checksum_matches());
    }

    #[test]
    fn default_detects_an_asserted_checksum_that_does_not_match() {
        // A client-asserted digest is not evidence; the store recomputes.
        let mut unit = a_default("policy", &[("policy.md", "body")]);
        unit.files
            .insert("policy.md".to_string(), "edited".to_string());
        assert!(!unit.checksum_matches());
        assert_eq!(
            unit.computed_checksum(),
            agent_text_unit_files_checksum(&unit.files)
        );
    }

    #[test]
    fn default_checksum_is_the_files_digest_not_the_single_body_one() {
        // THE trap this layer exists to avoid. `agent_command_checksum`
        // digests a bare body; `agent_text_unit_files_checksum` digests the
        // path→text map, and the two disagree even at one entry. A default
        // digested with the legacy function would never compare equal to the
        // override it is meant to be diffed against — an always-drifted
        // baseline, worse than the honest "baseline unavailable" state.
        let body = "# Vet Plan\n";
        let unit = a_default("vet-plan", &[("vet-plan.md", body)]);
        assert_ne!(
            unit.checksum,
            crate::agent_commands::agent_command_checksum(body),
            "the two digests must stay distinguishable at arity one"
        );
        assert!(unit.checksum.starts_with("sha256-"));
    }

    #[test]
    fn default_entrypoint_is_derived_not_stored() {
        let unit = a_default("vet-plan", &[("vet-plan.md", "x")]);
        assert_eq!(
            unit.entrypoint(),
            agent_text_unit_entrypoint(&unit.kind, &unit.name)
        );
        let json = serde_json::to_value(&unit).unwrap();
        assert!(json.get("entrypoint").is_none());
    }

    #[test]
    fn default_holds_a_multi_file_skill_the_same_way_as_a_command() {
        // Commands and skills are one shape at two arities — the default layer
        // must not special-case either, or it can never carry the skills and
        // agents bundles.
        let files = files(&[("SKILL.md", "# Skill\n"), ("run.sh", "echo hi\n")]);
        let unit = AgentTextUnitDefault {
            kind: AgentTextUnitKind::skill(),
            name: "coord-revive".to_string(),
            checksum: agent_text_unit_files_checksum(&files),
            files,
            published_by_version: "0.4.12".to_string(),
            published_at: "2026-08-31T12:00:00Z".to_string(),
        };
        assert!(unit.checksum_matches());
        assert!(validate_agent_text_unit_default(&unit).is_ok());
        assert_eq!(unit.files_sorted().len(), 2);
    }

    #[test]
    fn default_validation_applies_the_same_rules_as_an_override() {
        assert!(
            validate_agent_text_unit_default(&a_default("vet-plan", &[("vet-plan.md", "x")]))
                .is_ok()
        );

        // A name an override could not take, a default may not take either —
        // the two must be pairable for a diff.
        assert!(matches!(
            validate_agent_text_unit_default(&a_default("Vet-Plan", &[("Vet-Plan.md", "x")])),
            Err(AgentTextUnitError::InvalidName { .. })
        ));

        // Missing entrypoint for the (kind, name).
        assert!(matches!(
            validate_agent_text_unit_default(&a_default("vet-plan", &[("other.md", "x")])),
            Err(AgentTextUnitError::MissingEntrypoint { .. })
        ));

        // Empty file set.
        assert!(matches!(
            validate_agent_text_unit_default(&a_default("vet-plan", &[])),
            Err(AgentTextUnitError::EmptyFileSet)
        ));
    }
}
