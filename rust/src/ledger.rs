//! The development-substrate ledger **envelope**, frozen as a versioned
//! contract.
//!
//! Plan `2026-11-22-phase1-envelope-frozen-and-measurement-dispatch-proven`
//! Phase 1 (D1, D2), which freezes the envelope of the programme plan
//! `2026-11-19-in-house-development-substrate-for-the-digital-twin` (D8 "one
//! ledger", D12 "causal claims come from interventions") **before the first
//! writer**: an assignment block, a causation id or a graded principal cannot
//! be back-filled onto events already written.
//!
//! Not to be confused with qontinui-schemas `ci-exec/src/model/ledger.rs`,
//! which is the CI executor's own per-run step ledger in another crate and
//! shares nothing with this one but the word.
//!
//! # What is frozen
//!
//! [`LedgerEnvelope`] at [`ENVELOPE_SCHEMA_VERSION`]. The JSON Schema emitted
//! for it is committed as `rust/tests/fixtures/ledger_envelope.v1.schema.json`
//! and `tests/ledger_envelope_golden.rs` fails on any **structural** change to
//! the emitted schema unless the version constant moved and a new
//! `ledger_envelope.v<N>.schema.json` was added beside the old one (a frozen
//! version's file is never edited, and its structural digest is pinned).
//!
//! # One rule set, two spellings
//!
//! Non-Rust writers (shell, Python) validate against the committed schema;
//! Rust writers call [`validate`]. The schema therefore carries every rule of
//! [`validate`] that JSON Schema can express — the UUIDv7 id pattern, the
//! timestamp pattern plus `format: date-time`, the activity grammar, the
//! probability bounds, the hash shape, the label rule and `schema_version`'s
//! `const` — through the `*_PATTERN` constants below, which are the ONE
//! spelling of each pattern. The golden test validates a shared corpus with a
//! JSON Schema validator and with [`validate`] and fails if they disagree.
//! What only `format: date-time` carries (a day that exists in its month) is
//! enforced by a validator that asserts formats.
//!
//! # Every field is required — absence is spelled `null`, never omitted
//!
//! Every field of the envelope must be **present** on the wire. A field that
//! may have no value (`assignment`, `causation_id`, `prev_hash`, a principal
//! part's `value`, …) is spelled `null`, and a missing key is refused at
//! deserialization: a reader must never have to guess whether an absent
//! `assignment` meant "not an experiment" or "a writer that predates the
//! field". [`required_nullable`] is how a Rust `Option` gets that behaviour
//! (serde would otherwise default a missing `Option` to `None`), and a
//! schema-only `Nullable<T>` makes the emitted schema list every such field
//! in `required` with a nullable type.
//!
//! # Two grades, deliberately separate (D2)
//!
//! - [`EvidenceGrade`] — *how strongly* a fact is established, an ORDERED
//!   closed enum `declared < observed < corroborated < gate_grade`. Its
//!   `Ord` is the order D9's thresholds read ("corroborated or better").
//!   [`PrincipalPart::attribution_grade`] uses the same enum.
//! - [`SourceClass`] — *who* observed it, the process log's four
//!   `evidence_tier` tokens (`coord_observed | derived | agent_reported |
//!   operator`, qontinui-web `coord_proclog_01_process_log.py`), so the OCEL
//!   projection keeps its tier. Conflating the two would let a coord-observed
//!   fact read as corroborated.
//!
//! # Wire conventions
//!
//! Field names and enum tokens are `snake_case`; ids and timestamps are
//! `String`s (the crate convention, as in [`crate::cloud_twin`]); timestamps
//! are RFC 3339 with an upper-case `T` and a `Z` or `±hh:mm` offset. No
//! TypeScript or Python export is registered — bindings are added where a
//! consumer exists (the `cloud_twin` precedent).
//!
//! PURE: no I/O, no clock.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::marker::PhantomData;

use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::canonical_hash::{canonical_hash, CanonicalHashError};
use crate::cloud_twin::WorldKind;

/// The envelope contract version this crate emits and accepts. Bumping it
/// requires a new `tests/fixtures/ledger_envelope.v<N>.schema.json`; the old
/// file is kept.
pub const ENVELOPE_SCHEMA_VERSION: u32 = 1;

/// The activity namespaces D8 names for the substrate's own writers. The
/// process log's coord-owned namespaces are valid too, so [`validate`] checks
/// the `<namespace>.<verb>` grammar, not membership in this list.
pub const SUBSTRATE_ACTIVITY_NAMESPACES: &[&str] = &[
    "worktree",
    "fleet",
    "git",
    "ci",
    "artifact",
    "deploy",
    "secret",
    "experiment",
];

// ============================================================================
// Patterns — the schema spelling of validate()'s rules
// ============================================================================

/// A lower-case, hyphenated UUIDv7 with the RFC 4122 variant. One id, one
/// spelling: upper-case hex is refused.
pub const UUID_V7_PATTERN: &str =
    r"^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$";

/// RFC 3339 with an upper-case `T`, optional fraction, and `Z` or `±hh:mm`.
/// No leap second (`:60`). Whether the day exists in its month is the
/// `format: date-time` half.
pub const TIMESTAMP_PATTERN: &str = r"^[0-9]{4}-(0[1-9]|1[0-2])-(0[1-9]|[12][0-9]|3[01])T([01][0-9]|2[0-3]):[0-5][0-9]:[0-5][0-9](\.[0-9]+)?(Z|[+-]([01][0-9]|2[0-3]):[0-5][0-9])$";

/// `<namespace>.<verb>`, each `[a-z][a-z0-9_]*`.
pub const ACTIVITY_PATTERN: &str = r"^[a-z][a-z0-9_]*\.[a-z][a-z0-9_]*$";

/// A [`canonical_hash`]-shaped digest.
pub const HASH_PATTERN: &str = r"^sha256-[0-9a-f]{64}$";

/// A label: non-empty, no control character (Unicode `Cc`), and no
/// whitespace (Unicode `White_Space`, plus U+FEFF) at either end.
pub const LABEL_PATTERN: &str = r"^[^\u0000-\u001F\u007F-\u009F \u00A0\u1680\u2000-\u200A\u2028\u2029\u202F\u205F\u3000\uFEFF]([^\u0000-\u001F\u007F-\u009F]*[^\u0000-\u001F\u007F-\u009F \u00A0\u1680\u2000-\u200A\u2028\u2029\u202F\u205F\u3000\uFEFF])?$";

// ============================================================================
// Vocabulary
// ============================================================================

/// How strongly a fact is established. ORDERED: `declared < observed <
/// corroborated < gate_grade` — the variant order below is the `Ord` order,
/// and D9's thresholds compare with it. Never reorder the variants.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceGrade {
    /// Merely claimed by the writer; nothing checked it.
    Declared,
    /// Seen by one observer.
    Observed,
    /// Agreed by an independent second source.
    Corroborated,
    /// Established to the standard a coord gate attests to.
    GateGrade,
}

/// Who observed a fact — the process log's `evidence_tier` vocabulary, kept
/// token-for-token so the OCEL projection keeps its tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SourceClass {
    CoordObserved,
    Derived,
    AgentReported,
    Operator,
}

/// What `occurred_at` measures — the process log's `time_basis` vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TimeBasis {
    /// The source's own event time.
    Event,
    /// The time the source row was inserted.
    Insert,
    /// The time an observer first saw the state.
    Observed,
    /// A time the writer declared and nothing checked.
    Declared,
}

// ============================================================================
// Parts
// ============================================================================

/// One graded identity of the principal. `value` is `null` when the identity
/// is unknown — spelled, never omitted — and `attribution_grade` says how
/// well a non-null value is established.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PrincipalPart {
    #[serde(deserialize_with = "required_nullable::deserialize")]
    #[schemars(with = "Nullable<String>", extend("minLength" = 1, "pattern" = LABEL_PATTERN))]
    pub value: Option<String>,
    pub attribution_grade: EvidenceGrade,
}

/// The graded principal of an event (D8): session, agent, device, tenant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Principal {
    pub session: PrincipalPart,
    pub agent: PrincipalPart,
    pub device: PrincipalPart,
    pub tenant: PrincipalPart,
}

/// An OCEL 2.0 event-to-object reference with its qualifier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ObjectRef {
    #[schemars(extend("minLength" = 1, "pattern" = LABEL_PATTERN))]
    pub object_type: String,
    #[schemars(extend("minLength" = 1, "pattern" = LABEL_PATTERN))]
    pub object_id: String,
    /// The OCEL qualifier: the role this object plays in the event.
    #[schemars(extend("minLength" = 1, "pattern" = LABEL_PATTERN))]
    pub qualifier: String,
}

/// The world an event happened in — production is a world. References a
/// [`crate::cloud_twin::World`] by id and kind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WorldRef {
    #[schemars(extend("minLength" = 1, "pattern" = LABEL_PATTERN))]
    pub id: String,
    pub kind: WorldKind,
}

/// The D12 assignment block: present on every event a randomised policy
/// decided, because assignment cannot be back-filled.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Assignment {
    /// The randomisation unit (a session, a PR, a switchback window, …).
    #[schemars(extend("minLength" = 1, "pattern" = LABEL_PATTERN))]
    pub unit: String,
    /// The arm the unit was assigned to.
    #[schemars(extend("minLength" = 1, "pattern" = LABEL_PATTERN))]
    pub arm: String,
    /// The logged probability of this arm for this unit, in `(0, 1]`.
    #[schemars(extend("exclusiveMinimum" = 0, "maximum" = 1))]
    pub probability: f64,
    /// The covariates recorded at assignment time.
    pub covariates: BTreeMap<String, Value>,
    /// The version of the policy that made the assignment.
    #[schemars(extend("minLength" = 1, "pattern" = LABEL_PATTERN))]
    pub policy_version: String,
    /// The id of the seed the assignment was drawn from.
    #[schemars(extend("minLength" = 1, "pattern" = LABEL_PATTERN))]
    pub assignment_seed_id: String,
}

/// The event's payload: inline JSON, or a reference to a payload stored
/// beside the chain so it can be erased without breaking a sealed record
/// (D8 "Erasure").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Payload {
    Inline {
        value: Value,
    },
    Erasable {
        /// `sha256-<64 hex>` of the salted payload.
        #[schemars(extend("pattern" = HASH_PATTERN))]
        salted_hash: String,
        /// The side store holding the payload.
        #[schemars(extend("minLength" = 1, "pattern" = LABEL_PATTERN))]
        store: String,
    },
}

// ============================================================================
// The envelope
// ============================================================================

/// One ledger event, version [`ENVELOPE_SCHEMA_VERSION`]. Every field is
/// required on the wire; nullable fields are spelled `null`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LedgerEnvelope {
    /// A lower-case hyphenated UUIDv7.
    #[schemars(extend("format" = "uuid", "pattern" = UUID_V7_PATTERN))]
    pub id: String,
    /// The writer that produced the event.
    #[schemars(extend("minLength" = 1, "pattern" = LABEL_PATTERN))]
    pub source: String,
    /// The writer's own monotone sequence number (idempotent replay key with
    /// `source`).
    pub source_seq: u64,
    /// RFC 3339; what it measures is `time_basis`.
    #[schemars(extend("format" = "date-time", "pattern" = TIMESTAMP_PATTERN))]
    pub occurred_at: String,
    /// RFC 3339; when the ledger recorded the event.
    #[schemars(extend("format" = "date-time", "pattern" = TIMESTAMP_PATTERN))]
    pub recorded_at: String,
    pub time_basis: TimeBasis,
    /// `<namespace>.<verb>`, both `[a-z][a-z0-9_]*`.
    #[schemars(extend("pattern" = ACTIVITY_PATTERN))]
    pub activity: String,
    pub objects: Vec<ObjectRef>,
    pub principal: Principal,
    pub evidence_grade: EvidenceGrade,
    pub source_class: SourceClass,
    pub world: WorldRef,
    #[serde(deserialize_with = "required_nullable::deserialize")]
    #[schemars(with = "Nullable<String>", extend("minLength" = 1, "pattern" = LABEL_PATTERN))]
    pub correlation_id: Option<String>,
    #[serde(deserialize_with = "required_nullable::deserialize")]
    #[schemars(with = "Nullable<String>", extend("minLength" = 1, "pattern" = LABEL_PATTERN))]
    pub causation_id: Option<String>,
    #[serde(deserialize_with = "required_nullable::deserialize")]
    #[schemars(with = "Nullable<Assignment>")]
    pub assignment: Option<Assignment>,
    #[serde(deserialize_with = "required_nullable::deserialize")]
    #[schemars(with = "Nullable<String>", extend("minLength" = 1, "pattern" = LABEL_PATTERN))]
    pub policy_version: Option<String>,
    /// Always [`ENVELOPE_SCHEMA_VERSION`] for this crate's writers.
    #[schemars(extend("const" = ENVELOPE_SCHEMA_VERSION))]
    pub schema_version: u32,
    /// The [`chain_hash`] of the previous event of this chain; `null` only
    /// for a chain's first event.
    #[serde(deserialize_with = "required_nullable::deserialize")]
    #[schemars(with = "Nullable<String>", extend("pattern" = HASH_PATTERN))]
    pub prev_hash: Option<String>,
    pub payload: Payload,
}

/// Deserialize an `Option<T>` whose key is REQUIRED: `null` reads as `None`,
/// and a missing key is an error (serde's default for a missing `Option`
/// field is `None`; naming a `deserialize_with` removes that default).
pub mod required_nullable {
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
    {
        Option::<T>::deserialize(deserializer)
    }
}

/// Schema-only stand-in for a REQUIRED nullable field: it emits exactly
/// `Option<T>`'s nullable schema, but — unlike `Option<T>` — does not tell
/// schemars the field is optional, so the field lands in `required`.
/// (`#[schemars(required)]` would instead strip the `null` from the type.)
struct Nullable<T>(PhantomData<T>);

impl<T: JsonSchema> JsonSchema for Nullable<T> {
    fn inline_schema() -> bool {
        Option::<T>::inline_schema()
    }

    fn schema_name() -> Cow<'static, str> {
        Option::<T>::schema_name()
    }

    fn schema_id() -> Cow<'static, str> {
        Option::<T>::schema_id()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        Option::<T>::json_schema(generator)
    }
}

// ============================================================================
// Validation
// ============================================================================

/// Why [`validate`] refused an envelope.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum EnvelopeRejection {
    #[error("id {0:?} is not a lower-case hyphenated UUIDv7")]
    IdNotUuidV7(String),
    #[error("schema_version {found} is not {expected}")]
    SchemaVersion { found: u32, expected: u32 },
    #[error("{field} {value:?} is not an RFC 3339 timestamp (upper-case T, Z or ±hh:mm)")]
    Timestamp { field: &'static str, value: String },
    #[error("activity {0:?} is not `<namespace>.<verb>` with both parts [a-z][a-z0-9_]*")]
    Activity(String),
    #[error("assignment probability {0} is not in (0, 1]")]
    Probability(f64),
    #[error("{0} is blank or carries surrounding whitespace or control characters")]
    BlankLabel(&'static str),
    #[error("{field} {value:?} is not `sha256-<64 lowercase hex>`")]
    Hash { field: &'static str, value: String },
}

/// Check the rules the type system cannot: a UUIDv7 id, RFC 3339 times, the
/// activity grammar, the assignment probability bound, the schema version,
/// and non-blank labels. Pure. Each rule is also in the emitted schema (see
/// the module docs). `recorded_at >= occurred_at` is NOT required —
/// `time_basis` says what `occurred_at` measures, and a declared time may
/// postdate the record.
pub fn validate(envelope: &LedgerEnvelope) -> Result<(), EnvelopeRejection> {
    if envelope.schema_version != ENVELOPE_SCHEMA_VERSION {
        return Err(EnvelopeRejection::SchemaVersion {
            found: envelope.schema_version,
            expected: ENVELOPE_SCHEMA_VERSION,
        });
    }
    if !is_uuid_v7(&envelope.id) {
        return Err(EnvelopeRejection::IdNotUuidV7(envelope.id.clone()));
    }
    check_label("source", &envelope.source)?;
    check_timestamp("occurred_at", &envelope.occurred_at)?;
    check_timestamp("recorded_at", &envelope.recorded_at)?;
    if !is_valid_activity(&envelope.activity) {
        return Err(EnvelopeRejection::Activity(envelope.activity.clone()));
    }
    for object in &envelope.objects {
        check_label("objects[].object_type", &object.object_type)?;
        check_label("objects[].object_id", &object.object_id)?;
        check_label("objects[].qualifier", &object.qualifier)?;
    }
    for (name, part) in [
        ("principal.session.value", &envelope.principal.session),
        ("principal.agent.value", &envelope.principal.agent),
        ("principal.device.value", &envelope.principal.device),
        ("principal.tenant.value", &envelope.principal.tenant),
    ] {
        if let Some(value) = &part.value {
            check_label(name, value)?;
        }
    }
    check_label("world.id", &envelope.world.id)?;
    for (name, value) in [
        ("correlation_id", &envelope.correlation_id),
        ("causation_id", &envelope.causation_id),
        ("policy_version", &envelope.policy_version),
    ] {
        if let Some(value) = value {
            check_label(name, value)?;
        }
    }
    if let Some(assignment) = &envelope.assignment {
        check_label("assignment.unit", &assignment.unit)?;
        check_label("assignment.arm", &assignment.arm)?;
        check_label("assignment.policy_version", &assignment.policy_version)?;
        check_label(
            "assignment.assignment_seed_id",
            &assignment.assignment_seed_id,
        )?;
        // `!(p > 0.0)` also refuses NaN.
        let p = assignment.probability;
        if !(p > 0.0 && p <= 1.0) {
            return Err(EnvelopeRejection::Probability(p));
        }
    }
    if let Some(prev) = &envelope.prev_hash {
        check_hash("prev_hash", prev)?;
    }
    match &envelope.payload {
        Payload::Inline { .. } => {}
        Payload::Erasable { salted_hash, store } => {
            check_hash("payload.salted_hash", salted_hash)?;
            check_label("payload.store", store)?;
        }
    }
    Ok(())
}

/// `<namespace>.<verb>`: exactly one `.`, each side `[a-z][a-z0-9_]*`
/// ([`ACTIVITY_PATTERN`]).
pub fn is_valid_activity(activity: &str) -> bool {
    fn part(s: &str) -> bool {
        let mut chars = s.chars();
        matches!(chars.next(), Some('a'..='z'))
            && chars.all(|c| matches!(c, 'a'..='z' | '0'..='9' | '_'))
    }
    match activity.split_once('.') {
        Some((namespace, verb)) => part(namespace) && part(verb),
        None => false,
    }
}

/// [`UUID_V7_PATTERN`], byte for byte.
fn is_uuid_v7(id: &str) -> bool {
    let b = id.as_bytes();
    let hex = |c: u8| matches!(c, b'0'..=b'9' | b'a'..=b'f');
    b.len() == 36
        && b.iter().enumerate().all(|(i, &c)| match i {
            8 | 13 | 18 | 23 => c == b'-',
            14 => c == b'7',
            19 => matches!(c, b'8' | b'9' | b'a' | b'b'),
            _ => hex(c),
        })
}

/// Whitespace no label may start or end with: Unicode `White_Space`, plus
/// U+FEFF (which ECMA-262's `\s` counts) — the edge class of
/// [`LABEL_PATTERN`].
fn is_label_edge_space(c: char) -> bool {
    c.is_whitespace() || c == '\u{FEFF}'
}

/// [`LABEL_PATTERN`].
fn check_label(field: &'static str, value: &str) -> Result<(), EnvelopeRejection> {
    let bad_edge = |c: Option<char>| c.is_none_or(is_label_edge_space);
    if bad_edge(value.chars().next())
        || bad_edge(value.chars().next_back())
        || value.chars().any(char::is_control)
    {
        return Err(EnvelopeRejection::BlankLabel(field));
    }
    Ok(())
}

/// [`TIMESTAMP_PATTERN`] for the shape (upper-case `T`, `Z` or `±hh:mm`, no
/// leap second), then chrono for the calendar (the `format: date-time` half).
fn check_timestamp(field: &'static str, value: &str) -> Result<(), EnvelopeRejection> {
    let reject = || EnvelopeRejection::Timestamp {
        field,
        value: value.to_owned(),
    };
    let b = value.as_bytes();
    let digits = |r: std::ops::Range<usize>| {
        b.get(r)
            .is_some_and(|s| !s.is_empty() && s.iter().all(u8::is_ascii_digit))
    };
    // yyyy-mm-ddThh:mm:ss
    let head_ok = b.len() >= 20
        && digits(0..4)
        && b[4] == b'-'
        && digits(5..7)
        && b[7] == b'-'
        && digits(8..10)
        && b[10] == b'T'
        && digits(11..13)
        && b[13] == b':'
        && digits(14..16)
        && b[16] == b':'
        && digits(17..19)
        && b[17] <= b'5';
    if !head_ok {
        return Err(reject());
    }
    let mut i = 19;
    if b[i] == b'.' {
        let start = i + 1;
        i = start;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        if i == start {
            return Err(reject());
        }
    }
    let tail = &b[i..];
    // Z, or ±hh:mm with hh <= 23 and mm <= 59 (the pattern's offset class).
    let tail_ok = tail == b"Z"
        || (tail.len() == 6
            && matches!(tail[0], b'+' | b'-')
            && tail[1..3].iter().all(u8::is_ascii_digit)
            && &tail[1..3] <= b"23".as_slice()
            && tail[3] == b':'
            && tail[4..6].iter().all(u8::is_ascii_digit)
            && tail[4] <= b'5');
    if !tail_ok {
        return Err(reject());
    }
    // Month, day-in-month, hour and minute ranges are chrono's.
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|_| ())
        .map_err(|_| reject())
}

/// [`HASH_PATTERN`].
fn check_hash(field: &'static str, value: &str) -> Result<(), EnvelopeRejection> {
    let ok = value.strip_prefix("sha256-").is_some_and(|hex| {
        hex.len() == 64 && hex.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
    });
    if ok {
        Ok(())
    } else {
        Err(EnvelopeRejection::Hash {
            field,
            value: value.to_owned(),
        })
    }
}

// ============================================================================
// Chain hashing
// ============================================================================

/// Why [`chain_hash`] refused.
#[derive(Debug, Error)]
pub enum ChainHashError {
    /// The envelope does not [`validate`]; an invalid envelope is never
    /// hashed into a chain (and a NaN or infinite probability would
    /// otherwise canonicalize as `null`).
    #[error("envelope is invalid: {0}")]
    Invalid(#[from] EnvelopeRejection),
    #[error("envelope prev_hash {found:?} is not the chain's previous hash {expected:?}")]
    PrevHashMismatch {
        expected: Option<String>,
        found: Option<String>,
    },
    #[error(transparent)]
    Canonical(#[from] CanonicalHashError),
}

/// The hash a chain's NEXT event stores as its `prev_hash`: the crate's
/// [`canonical_hash`] (RFC 8785 → SHA-256, `sha256-<hex>`) of
/// `{"prev_hash": prev, "envelope": envelope}`.
///
/// The envelope must [`validate`]. `prev` is the previous event's chain hash
/// (`None` for a chain's first event), and the envelope's own `prev_hash`
/// must equal it — so a chain cannot be hashed out of order or with a forged
/// link.
pub fn chain_hash(prev: Option<&str>, envelope: &LedgerEnvelope) -> Result<String, ChainHashError> {
    validate(envelope)?;
    if envelope.prev_hash.as_deref() != prev {
        return Err(ChainHashError::PrevHashMismatch {
            expected: prev.map(str::to_owned),
            found: envelope.prev_hash.clone(),
        });
    }
    #[derive(Serialize)]
    struct Link<'a> {
        prev_hash: Option<&'a str>,
        envelope: &'a LedgerEnvelope,
    }
    Ok(canonical_hash(&Link {
        prev_hash: prev,
        envelope,
    })?)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const V7: &str = "019a8f2e-7c3b-7d4e-8a1b-2c3d4e5f6a7b";

    fn part(value: Option<&str>, grade: EvidenceGrade) -> Value {
        json!({"value": value, "attribution_grade": grade})
    }

    fn sample() -> Value {
        json!({
            "id": V7,
            "source": "coord.process_log",
            "source_seq": 42,
            "occurred_at": "2026-11-22T10:00:00Z",
            "recorded_at": "2026-11-22T10:00:01.250+00:00",
            "time_basis": "event",
            "activity": "worktree.allocate",
            "objects": [
                {"object_type": "worktree", "object_id": "wt-1", "qualifier": "allocated"}
            ],
            "principal": {
                "session": part(Some("s-1"), EvidenceGrade::Observed),
                "agent": part(None, EvidenceGrade::Declared),
                "device": part(Some("d-1"), EvidenceGrade::Corroborated),
                "tenant": part(Some("t-1"), EvidenceGrade::GateGrade),
            },
            "evidence_grade": "observed",
            "source_class": "coord_observed",
            "world": {"id": "prod", "kind": "prod"},
            "correlation_id": "corr-1",
            "causation_id": null,
            "assignment": {
                "unit": "session:s-1",
                "arm": "treatment",
                "probability": 0.5,
                "covariates": {"device_class": "linux"},
                "policy_version": "premise-notice@1",
                "assignment_seed_id": "seed-2026-11-22"
            },
            "policy_version": null,
            "schema_version": 1,
            "prev_hash": null,
            "payload": {"kind": "inline", "value": {"note": "hi"}}
        })
    }

    fn parse(v: Value) -> Result<LedgerEnvelope, serde_json::Error> {
        serde_json::from_value(v)
    }

    #[test]
    fn sample_parses_validates_and_round_trips() {
        let env = parse(sample()).unwrap();
        validate(&env).unwrap();
        let back = serde_json::to_value(&env).unwrap();
        assert_eq!(back, sample(), "serialization must spell every field");
    }

    #[test]
    fn every_top_level_key_is_required() {
        let keys: Vec<String> = sample().as_object().unwrap().keys().cloned().collect();
        for key in keys {
            let mut v = sample();
            v.as_object_mut().unwrap().remove(&key);
            let err = parse(v).expect_err(&format!("missing `{key}` must be refused"));
            assert!(err.to_string().contains("missing field"), "`{key}`: {err}");
        }
    }

    #[test]
    fn missing_assignment_key_is_refused_but_null_is_accepted() {
        let mut v = sample();
        v.as_object_mut().unwrap().remove("assignment");
        let err = parse(v).unwrap_err();
        assert!(
            err.to_string().contains("missing field `assignment`"),
            "{err}"
        );

        let mut v = sample();
        v["assignment"] = Value::Null;
        let env = parse(v).unwrap();
        assert_eq!(env.assignment, None);
        validate(&env).unwrap();
    }

    #[test]
    fn principal_part_value_key_is_required() {
        let mut v = sample();
        v["principal"]["agent"]
            .as_object_mut()
            .unwrap()
            .remove("value");
        assert!(parse(v)
            .unwrap_err()
            .to_string()
            .contains("missing field `value`"));
    }

    #[test]
    fn unknown_fields_are_refused_everywhere() {
        for path in [
            &[][..],
            &["principal"][..],
            &["world"][..],
            &["assignment"][..],
            &["principal", "agent"][..],
            &["objects", "0"][..],
        ] {
            let mut v = sample();
            let mut target = &mut v;
            for p in path {
                target = match p.parse::<usize>() {
                    Ok(i) => &mut target[i],
                    Err(_) => &mut target[*p],
                };
            }
            target
                .as_object_mut()
                .unwrap()
                .insert("extra".into(), json!(1));
            assert!(parse(v).is_err(), "unknown field under {path:?} accepted");
        }
        let mut v = sample();
        v["payload"]["extra"] = json!(1);
        assert!(parse(v).is_err(), "unknown payload field accepted");
    }

    #[test]
    fn probability_bounds() {
        for p in [0.0, -0.1, 1.000_001, 2.0] {
            let mut v = sample();
            v["assignment"]["probability"] = json!(p);
            let env = parse(v).unwrap();
            assert_eq!(
                validate(&env),
                Err(EnvelopeRejection::Probability(p)),
                "probability {p}"
            );
        }
        for p in [1e-9, 0.5, 1.0] {
            let mut v = sample();
            v["assignment"]["probability"] = json!(p);
            validate(&parse(v).unwrap()).unwrap();
        }
        let mut env = parse(sample()).unwrap();
        env.assignment.as_mut().unwrap().probability = f64::NAN;
        assert!(matches!(
            validate(&env),
            Err(EnvelopeRejection::Probability(_))
        ));
    }

    #[test]
    fn non_v7_ids_are_refused() {
        for id in [
            "019a8f2e-7c3b-4d4e-8a1b-2c3d4e5f6a7b", // v4
            "019a8f2e-7c3b-1d4e-8a1b-2c3d4e5f6a7b", // v1
            "019a8f2e7c3b7d4e8a1b2c3d4e5f6a7b",     // v7, simple form
            "019a8f2e-7c3b-7d4e-ca1b-2c3d4e5f6a7b", // v7 nibble, wrong variant
            "019A8F2E-7C3B-7D4E-8A1B-2C3D4E5F6A7B", // v7, upper-case: one spelling
            "019a8f2e-7c3b-7d4e-8a1b-2c3d4e5f6a7B", // v7, one upper-case digit
            "not-a-uuid",
            "",
        ] {
            let mut v = sample();
            v["id"] = json!(id);
            assert_eq!(
                validate(&parse(v).unwrap()),
                Err(EnvelopeRejection::IdNotUuidV7(id.into())),
                "{id}"
            );
        }
    }

    #[test]
    fn timestamps_must_be_rfc3339() {
        for (field, bad) in [
            ("occurred_at", "2026-11-22 10:00:00"),
            ("recorded_at", "2026-11-22T10:00:00"),
            ("occurred_at", "yesterday"),
            ("occurred_at", "2026-11-22 10:00:00Z"),
            ("occurred_at", "2026-11-22t10:00:00z"),
            ("occurred_at", "2026-11-22T10:00:00z"),
            ("occurred_at", "2026-11-22T10:00:60Z"),
            ("occurred_at", "2026-02-30T10:00:00Z"),
            ("occurred_at", "2026-11-22T10:00:00.Z"),
            ("occurred_at", "2026-11-22T10:00:00+24:00"),
            ("occurred_at", "2026-11-22T10:00:00+0530"),
        ] {
            let mut v = sample();
            v[field] = json!(bad);
            assert!(
                matches!(
                    validate(&parse(v).unwrap()),
                    Err(EnvelopeRejection::Timestamp { .. })
                ),
                "{field}={bad}"
            );
        }
    }

    #[test]
    fn activity_grammar() {
        for ok in ["worktree.allocate", "train.occupancy_hourly", "ci.run2"] {
            assert!(is_valid_activity(ok), "{ok}");
        }
        for bad in [
            "allocate",
            "worktree.",
            ".allocate",
            "Worktree.allocate",
            "worktree.allocate.extra",
            "work-tree.allocate",
            "1ci.run",
            "ci. run",
        ] {
            assert!(!is_valid_activity(bad), "{bad}");
            let mut v = sample();
            v["activity"] = json!(bad);
            assert_eq!(
                validate(&parse(v).unwrap()),
                Err(EnvelopeRejection::Activity(bad.into()))
            );
        }
    }

    #[test]
    fn schema_version_must_be_current() {
        let mut v = sample();
        v["schema_version"] = json!(2);
        assert_eq!(
            validate(&parse(v).unwrap()),
            Err(EnvelopeRejection::SchemaVersion {
                found: 2,
                expected: 1
            })
        );
    }

    #[test]
    fn evidence_grade_is_ordered_declared_to_gate_grade() {
        use EvidenceGrade::*;
        assert!(Declared < Observed && Observed < Corroborated && Corroborated < GateGrade);
    }

    #[test]
    fn every_enum_token_round_trips() {
        fn rt<T>(tokens: &[(&str, T)])
        where
            T: Serialize + for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug + Copy,
        {
            for (token, value) in tokens {
                assert_eq!(serde_json::to_value(value).unwrap(), json!(token));
                assert_eq!(serde_json::from_value::<T>(json!(token)).unwrap(), *value);
            }
        }
        rt(&[
            ("declared", EvidenceGrade::Declared),
            ("observed", EvidenceGrade::Observed),
            ("corroborated", EvidenceGrade::Corroborated),
            ("gate_grade", EvidenceGrade::GateGrade),
        ]);
        rt(&[
            ("coord_observed", SourceClass::CoordObserved),
            ("derived", SourceClass::Derived),
            ("agent_reported", SourceClass::AgentReported),
            ("operator", SourceClass::Operator),
        ]);
        rt(&[
            ("event", TimeBasis::Event),
            ("insert", TimeBasis::Insert),
            ("observed", TimeBasis::Observed),
            ("declared", TimeBasis::Declared),
        ]);
        rt(&[
            ("prod", WorldKind::Prod),
            ("clone", WorldKind::Clone),
            ("fork", WorldKind::Fork),
        ]);
        assert!(serde_json::from_value::<EvidenceGrade>(json!("gate-grade")).is_err());
        assert!(serde_json::from_value::<SourceClass>(json!("coordObserved")).is_err());
    }

    #[test]
    fn payload_variants_round_trip() {
        let hash = format!("sha256-{}", "a".repeat(64));
        for payload in [
            json!({"kind": "inline", "value": [1, 2]}),
            json!({"kind": "erasable", "salted_hash": hash, "store": "rack-pg.payloads"}),
        ] {
            let mut v = sample();
            v["payload"] = payload.clone();
            let env = parse(v).unwrap();
            validate(&env).unwrap();
            assert_eq!(serde_json::to_value(&env.payload).unwrap(), payload);
        }
        let mut v = sample();
        v["payload"] = json!({"kind": "erasable", "salted_hash": "abc", "store": "s"});
        assert!(matches!(
            validate(&parse(v).unwrap()),
            Err(EnvelopeRejection::Hash {
                field: "payload.salted_hash",
                ..
            })
        ));
    }

    #[test]
    fn chain_hash_links_and_refuses_a_forged_link() {
        let first = parse(sample()).unwrap();
        let h1 = chain_hash(None, &first).unwrap();
        assert!(h1.starts_with("sha256-") && h1.len() == "sha256-".len() + 64);
        assert_eq!(h1, chain_hash(None, &first).unwrap(), "deterministic");

        let mut second = first.clone();
        second.source_seq = 43;
        second.prev_hash = Some(h1.clone());
        validate(&second).unwrap();
        let h2 = chain_hash(Some(&h1), &second).unwrap();
        assert_ne!(h1, h2);

        assert!(matches!(
            chain_hash(None, &second),
            Err(ChainHashError::PrevHashMismatch { .. })
        ));
        let mut tampered = second.clone();
        tampered.source_seq = 44;
        assert_ne!(chain_hash(Some(&h1), &tampered).unwrap(), h2);

        // An invalid envelope is never hashed: NaN / inf would otherwise
        // canonicalize as `null`.
        for p in [f64::NAN, f64::INFINITY] {
            let mut bad = first.clone();
            bad.assignment.as_mut().unwrap().probability = p;
            assert!(matches!(
                chain_hash(None, &bad),
                Err(ChainHashError::Invalid(EnvelopeRejection::Probability(_)))
            ));
        }
    }

    #[test]
    fn valid_timestamp_spellings_are_accepted() {
        for ok in [
            "2026-11-22T10:00:00Z",
            "2026-11-22T10:00:00.123456789Z",
            "2026-11-22T10:00:00+05:30",
            "2026-11-22T10:00:00-23:59",
            "2024-02-29T23:59:59Z",
        ] {
            check_timestamp("occurred_at", ok).unwrap_or_else(|e| panic!("{ok}: {e}"));
        }
    }

    #[test]
    fn labels_refuse_edge_whitespace_and_control_characters() {
        for bad in ["", " x", "x ", "\u{FEFF}x", "x\u{3000}", "a\nb", "a\u{85}b"] {
            assert!(check_label("f", bad).is_err(), "{bad:?}");
        }
        for ok in ["x", "a b", "a\u{FEFF}b", "ünïcode"] {
            check_label("f", ok).unwrap_or_else(|e| panic!("{ok:?}: {e}"));
        }
    }
}
