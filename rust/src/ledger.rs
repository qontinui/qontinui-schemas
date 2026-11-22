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
//! version's file is never edited). Non-Rust writers (shell, Python) validate
//! against that file; Rust writers call [`validate`].
//!
//! # Every field is required — absence is spelled `null`, never omitted
//!
//! Every field of the envelope must be **present** on the wire. A field that
//! may have no value (`assignment`, `causation_id`, `prev_hash`, a principal
//! part's `value`, …) is spelled `null`, and a missing key is refused at
//! deserialization: a reader must never have to guess whether an absent
//! `assignment` meant "not an experiment" or "a writer that predates the
//! field". [`required_nullable`] is how a Rust `Option` gets that behaviour
//! (serde would otherwise default a missing `Option` to `None`), and the
//! emitted schema lists every such field in `required` with a nullable type.
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
//! are RFC 3339. No TypeScript or Python export is registered — bindings are
//! added where a consumer exists (the `cloud_twin` precedent).
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
    #[schemars(with = "Nullable<String>")]
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
    pub object_type: String,
    pub object_id: String,
    /// The OCEL qualifier: the role this object plays in the event.
    pub qualifier: String,
}

/// The world an event happened in — production is a world. References a
/// [`crate::cloud_twin::World`] by id and kind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WorldRef {
    pub id: String,
    pub kind: WorldKind,
}

/// The D12 assignment block: present on every event a randomised policy
/// decided, because assignment cannot be back-filled.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Assignment {
    /// The randomisation unit (a session, a PR, a switchback window, …).
    pub unit: String,
    /// The arm the unit was assigned to.
    pub arm: String,
    /// The logged probability of this arm for this unit, in `(0, 1]`.
    pub probability: f64,
    /// The covariates recorded at assignment time.
    pub covariates: BTreeMap<String, Value>,
    /// The version of the policy that made the assignment.
    pub policy_version: String,
    /// The id of the seed the assignment was drawn from.
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
        salted_hash: String,
        /// The side store holding the payload.
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
    /// A UUIDv7.
    pub id: String,
    /// The writer that produced the event.
    pub source: String,
    /// The writer's own monotone sequence number (idempotent replay key with
    /// `source`).
    pub source_seq: u64,
    /// RFC 3339; what it measures is `time_basis`.
    pub occurred_at: String,
    /// RFC 3339; when the ledger recorded the event.
    pub recorded_at: String,
    pub time_basis: TimeBasis,
    /// `<namespace>.<verb>`, both `[a-z][a-z0-9_]*`.
    pub activity: String,
    pub objects: Vec<ObjectRef>,
    pub principal: Principal,
    pub evidence_grade: EvidenceGrade,
    pub source_class: SourceClass,
    pub world: WorldRef,
    #[serde(deserialize_with = "required_nullable::deserialize")]
    #[schemars(with = "Nullable<String>")]
    pub correlation_id: Option<String>,
    #[serde(deserialize_with = "required_nullable::deserialize")]
    #[schemars(with = "Nullable<String>")]
    pub causation_id: Option<String>,
    #[serde(deserialize_with = "required_nullable::deserialize")]
    #[schemars(with = "Nullable<Assignment>")]
    pub assignment: Option<Assignment>,
    #[serde(deserialize_with = "required_nullable::deserialize")]
    #[schemars(with = "Nullable<String>")]
    pub policy_version: Option<String>,
    /// Always [`ENVELOPE_SCHEMA_VERSION`] for this crate's writers.
    pub schema_version: u32,
    /// The [`chain_hash`] of the previous event of this chain; `null` only
    /// for a chain's first event.
    #[serde(deserialize_with = "required_nullable::deserialize")]
    #[schemars(with = "Nullable<String>")]
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
    #[error("id {0:?} is not a UUIDv7")]
    IdNotUuidV7(String),
    #[error("schema_version {found} is not {expected}")]
    SchemaVersion { found: u32, expected: u32 },
    #[error("{field} {value:?} is not an RFC 3339 timestamp")]
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
/// and non-blank labels. Pure. `recorded_at >= occurred_at` is NOT required —
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

/// `<namespace>.<verb>`: exactly one `.`, each side `[a-z][a-z0-9_]*`.
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

fn is_uuid_v7(id: &str) -> bool {
    // Hyphenated form only, so one id has one spelling.
    id.len() == 36
        && uuid::Uuid::try_parse(id)
            .map(|u| u.get_version_num() == 7 && u.get_variant() == uuid::Variant::RFC4122)
            .unwrap_or(false)
}

fn check_label(field: &'static str, value: &str) -> Result<(), EnvelopeRejection> {
    if value.is_empty() || value.trim() != value || value.chars().any(char::is_control) {
        return Err(EnvelopeRejection::BlankLabel(field));
    }
    Ok(())
}

fn check_timestamp(field: &'static str, value: &str) -> Result<(), EnvelopeRejection> {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|_| ())
        .map_err(|_| EnvelopeRejection::Timestamp {
            field,
            value: value.to_owned(),
        })
}

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
/// `prev` is the previous event's chain hash (`None` for a chain's first
/// event), and the envelope's own `prev_hash` must equal it — so a chain
/// cannot be hashed out of order or with a forged link.
pub fn chain_hash(prev: Option<&str>, envelope: &LedgerEnvelope) -> Result<String, ChainHashError> {
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
        ] {
            let mut v = sample();
            let mut target = &mut v;
            for p in path {
                target = &mut target[*p];
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
    }
}
