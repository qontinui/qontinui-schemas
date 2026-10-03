//! [`Observation`] — the envelope every UI Bridge observation surface
//! answers in: WHAT was seen, or that nothing was there, or that the
//! producer could not look — and, in every case, where the answer came from.
//!
//! # Why the envelope exists
//!
//! [`crate::AnalyzerVerdict`] and [`crate::AssertionOutcome`] already
//! separate "checked and clean" from "could not check" for `vision/analyze`
//! and `vision/assert`. The other observation surfaces (`vision/extract`,
//! `vision/describe`, `control/page-health`, the MCP tools) answered with a
//! bare payload, where an empty list meant either "the page has nothing
//! there" or "the producer never looked" — byte-identical, and only the
//! first is a statement about the page. This type generalises the verdict
//! vocabulary to them (plan
//! `2026-09-20-ui-bridge-observations-distinguish-cannot-see-from-not-present-and-carry-provenance`,
//! D1-D3).
//!
//! # The wire shape
//!
//! ```jsonc
//! {
//!   "status": "measured" | "absent" | "unknown",   // always present
//!   "value": <T>,                                   // present iff measured
//!   "unknown": { "code": <UnknownCode>, "detail": "…" }, // present iff unknown
//!   "provenance": {                                 // always present, every key always present
//!     "producer": { "id": "…", "version": "…" },
//!     "observedAt": "<RFC3339>" | null,
//!     "evaluatedAt": "<RFC3339>",
//!     "coverage": { "considered": n, "measured": n,
//!                   "unmeasured": [{ "dimension": "…", "count": n, "code": <UnknownCode> }] },
//!     "confidence": <0..=1> | null,
//!     "cache": { "hit": bool, "storedAt": "<RFC3339>" | null, "keyInputs": ["…"] } | null,
//!     "source": { … } | null
//!   }
//! }
//! ```
//!
//! # Illegal states are unrepresentable
//!
//! The status and its payload are ONE enum, [`ObservationState`], so there
//! is no `measured` without a value, no `unknown` without an
//! [`UnknownInfo`] (and therefore without a [`UnknownCode`]), and no value
//! riding beside an `absent`. The one invariant that spans the state AND
//! the provenance — **`absent` only over full coverage** — cannot be carried
//! by the enum alone, so [`Observation`]'s fields are private and every
//! construction path checks it: [`Observation::absent`] returns `Err`, and
//! deserialization routes through the same check. "Absent" can therefore
//! never be asserted over a region the producer did not measure, whether the
//! value was built in this process or read off someone else's wire.
//!
//! Serialization is hand-written rather than derived because the wire is
//! FLAT (`status`, `value`, `unknown` and `provenance` are siblings) while
//! the Rust value is nested; a derived `#[serde(flatten)]` over an
//! internally-tagged enum would produce the same bytes but deserializes
//! through serde's untyped buffer, which accepts a `measured` whose `value`
//! key is missing whenever `T` is `Option<_>`. The hand-written path checks
//! key PRESENCE, so that shape is refused for every `T`.

use std::borrow::Cow;
use std::fmt;

use chrono::{DateTime, Utc};
use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::de::{self, Deserializer};
use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::analyzers::Analyzer;

// ---------------------------------------------------------------------------
// UnknownCode
// ---------------------------------------------------------------------------

/// WHY a producer could not answer. A closed vocabulary: an agent branches on
/// the code, and the human-readable [`UnknownInfo::detail`] is never the only
/// carrier of the reason.
///
/// Adding a member is a wire change for every consumer (runner, ui-bridge,
/// ui-bridge-mcp); the diagnostic-code registry name of each is
/// [`Self::diagnostic_code`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UnknownCode {
    /// The bridge/app could not be reached.
    AppUnreachable,
    /// The running build does not serve this capability (e.g. an SDK route
    /// that needs a runner).
    CapabilityUnavailableInBuild,
    /// The producer ran and errored (transport, HTTP status, timeout).
    ProducerFailed,
    /// The producer did not run (e.g. zero registered components — nothing
    /// to look at yet).
    ProducerNotRun,
    /// A required input was absent (no `elements` key, no bbox, no frame).
    InputMissing,
    /// Observations existed but all fell under the caller's confidence floor.
    BelowConfidenceFloor,
    /// A model reply could not be parsed or validated.
    ModelReplyUnparseable,
    /// The question needs more than one frame (e.g. `animation_settled`).
    NeedsMultiFrameInput,
    /// The input is known to be stale.
    StaleInput,
}

impl UnknownCode {
    /// Every member, in declaration order. The closed set, so a test can
    /// pin it against the wire contract rather than against itself.
    pub const ALL: [UnknownCode; 9] = [
        Self::AppUnreachable,
        Self::CapabilityUnavailableInBuild,
        Self::ProducerFailed,
        Self::ProducerNotRun,
        Self::InputMissing,
        Self::BelowConfidenceFloor,
        Self::ModelReplyUnparseable,
        Self::NeedsMultiFrameInput,
        Self::StaleInput,
    ];

    /// The snake_case wire value.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AppUnreachable => "app_unreachable",
            Self::CapabilityUnavailableInBuild => "capability_unavailable_in_build",
            Self::ProducerFailed => "producer_failed",
            Self::ProducerNotRun => "producer_not_run",
            Self::InputMissing => "input_missing",
            Self::BelowConfidenceFloor => "below_confidence_floor",
            Self::ModelReplyUnparseable => "model_reply_unparseable",
            Self::NeedsMultiFrameInput => "needs_multi_frame_input",
            Self::StaleInput => "stale_input",
        }
    }

    /// The ui-bridge diagnostic-registry name: `UB-OBS-<CODE-IN-UPPER-KEBAB>`,
    /// e.g. `UB-OBS-INPUT-MISSING`.
    pub fn diagnostic_code(self) -> String {
        format!(
            "UB-OBS-{}",
            self.as_str().to_ascii_uppercase().replace('_', "-")
        )
    }
}

impl fmt::Display for UnknownCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The payload of an `unknown` observation: a typed code an agent branches
/// on, plus prose for a human.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UnknownInfo {
    pub code: UnknownCode,
    /// Human-readable explanation. Never parsed; [`Self::code`] is the
    /// machine-readable reason.
    pub detail: String,
}

impl UnknownInfo {
    pub fn new(code: UnknownCode, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Provenance
// ---------------------------------------------------------------------------

/// Which producer answered, and at what version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Producer {
    /// e.g. `vision-core/layout`, `runner/page-health`, `sdk/page-health`.
    pub id: String,
    /// The producing crate's or package's version.
    pub version: String,
}

impl Producer {
    pub fn new(id: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            version: version.into(),
        }
    }

    /// `vision-core/<analyzer>` at THIS crate's version — the producer id
    /// the wire contract assigns to each analyzer.
    pub fn vision_core(analyzer: Analyzer) -> Self {
        Self::new(
            format!("vision-core/{}", analyzer.name()),
            env!("CARGO_PKG_VERSION"),
        )
    }
}

/// One dimension the producer could NOT measure, how many items it affected,
/// and why.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UnmeasuredDimension {
    /// What went unmeasured, e.g. `bbox`, `stacking_order`, `text`.
    pub dimension: String,
    /// How many considered items lacked it.
    pub count: u64,
    pub code: UnknownCode,
}

impl UnmeasuredDimension {
    pub fn new(dimension: impl Into<String>, count: u64, code: UnknownCode) -> Self {
        Self {
            dimension: dimension.into(),
            count,
            code,
        }
    }
}

/// How much of what the producer looked at it could actually measure.
///
/// A non-empty [`Self::unmeasured`] is what makes a `measured` observation
/// DEGRADED (legal: the value stands, some dimension was not ruled out), and
/// what makes an `absent` observation ILLEGAL (nothing can be claimed absent
/// from a region that was not measured).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ObservationCoverage {
    /// Items the producer considered.
    pub considered: u64,
    /// Items it could measure in full.
    pub measured: u64,
    /// Named dimensions it could not measure. Always serialized — an empty
    /// list is the statement "full coverage".
    pub unmeasured: Vec<UnmeasuredDimension>,
}

impl ObservationCoverage {
    /// Every one of `n` considered items was measured in full.
    pub fn full(n: u64) -> Self {
        Self {
            considered: n,
            measured: n,
            unmeasured: Vec::new(),
        }
    }

    /// True when no dimension went unmeasured — the precondition for
    /// `absent`.
    pub fn is_full(&self) -> bool {
        self.unmeasured.is_empty()
    }
}

/// Provenance of a cached answer.
///
/// Every key always serializes: `storedAt: null` states "the cache holds no
/// storage time for this entry", which is different from a producer that
/// said nothing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CacheProvenance {
    /// Whether this answer was served from the cache.
    pub hit: bool,
    /// When the served entry was stored. `null` on a miss (nothing was
    /// served from storage) or when the cache keeps no timestamp.
    #[serde(deserialize_with = "required_nullable")]
    #[schemars(with = "Nullable<DateTime<Utc>>")]
    pub stored_at: Option<DateTime<Utc>>,
    /// The inputs the cache key was derived from (e.g. `frame_sha256`,
    /// `prompt_version`), so a reader can judge what a hit is keyed on.
    pub key_inputs: Vec<String>,
}

/// Where an [`Observation`] came from.
///
/// **Every key always serializes.** `observedAt`, `confidence`, `cache` and
/// `source` are `null` when they have nothing to say, and that `null` is a
/// statement rather than an omission (plan `489ed69e` lesson 1): a producer
/// that states "no sample was taken" / "this is a deduction" / "I have no
/// cache" / "I have no source attribution" is distinguishable on the wire
/// from one that said nothing — the latter is a payload that fails to parse.
//
// (Rust-only note, kept out of the generated schema description:
// `confidence` is private because it carries an invariant the other fields
// do not — finite and in `0.0..=1.0` whenever it is `Some`. See
// `Provenance::with_confidence`.)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Provenance {
    pub producer: Producer,
    /// When the underlying state was sampled (frame `capturedAt`, snapshot
    /// time). `None` = no sample was taken (e.g. `input_missing`).
    #[serde(deserialize_with = "required_nullable")]
    #[schemars(with = "Nullable<DateTime<Utc>>")]
    pub observed_at: Option<DateTime<Utc>>,
    /// When the producer ran. Never absent.
    pub evaluated_at: DateTime<Utc>,
    pub coverage: ObservationCoverage,
    /// How far the answer rests on an ESTIMATED input. `None` is a positive
    /// statement: the answer is a deduction, not an estimate — the same
    /// reading as [`crate::Finding::confidence`].
    #[serde(
        serialize_with = "serialize_confidence",
        deserialize_with = "deserialize_confidence"
    )]
    #[schemars(with = "Nullable<f64>", range(min = 0.0, max = 1.0))]
    confidence: Option<f64>,
    /// `None` = this producer has no cache.
    #[serde(deserialize_with = "required_nullable")]
    #[schemars(with = "Nullable<CacheProvenance>")]
    pub cache: Option<CacheProvenance>,
    /// Source attribution as a JSON object (a `SnapshotAttribution`, a
    /// `FrameSource` projection, …). `None` = the producer has none to give.
    #[serde(deserialize_with = "required_nullable")]
    #[schemars(with = "Nullable<Map<String, Value>>")]
    pub source: Option<Map<String, Value>>,
}

impl Provenance {
    /// Provenance for a producer that ran at `evaluated_at` over `coverage`.
    /// Every optional key starts as its explicit `null` statement; set the
    /// ones that apply with the `with_*` methods.
    pub fn new(
        producer: Producer,
        evaluated_at: DateTime<Utc>,
        coverage: ObservationCoverage,
    ) -> Self {
        Self {
            producer,
            observed_at: None,
            evaluated_at,
            coverage,
            confidence: None,
            cache: None,
            source: None,
        }
    }

    pub fn with_observed_at(mut self, at: DateTime<Utc>) -> Self {
        self.observed_at = Some(at);
        self
    }

    /// Record that the answer rests on an estimated input, at `c`.
    ///
    /// A NON-FINITE `c` is filtered to `None`, never stored: `serde_json`
    /// writes a non-finite float as `null` anyway, and filtering here keeps
    /// the in-memory value and the wire from disagreeing about it (the same
    /// filter [`crate::AssertionResult::confidence`]'s OCR fold applies).
    /// The binding wire contract for this envelope names that outcome
    /// explicitly — "non-finite -> null".
    ///
    /// A finite `c` outside `0.0..=1.0` is a caller bug: it trips a
    /// `debug_assert` and is clamped in release, so a release build can never
    /// put an out-of-contract number on the wire.
    pub fn with_confidence(mut self, c: f64) -> Self {
        self.confidence = finite_confidence(c);
        self
    }

    pub fn with_cache(mut self, cache: CacheProvenance) -> Self {
        self.cache = Some(cache);
        self
    }

    pub fn with_source(mut self, source: Map<String, Value>) -> Self {
        self.source = Some(source);
        self
    }

    /// The confidence, finite and in `0.0..=1.0` when present.
    pub fn confidence(&self) -> Option<f64> {
        self.confidence
    }
}

/// Schema stand-in for an always-present, nullable key: the schema of
/// `Option<T>` (so `null` is admitted), but NOT an option to schemars, so the
/// key lands in `required`. Plain `Option<T>` would be emitted as an optional
/// key, and the generated TS/Python would type it `field?: T | null` — the
/// "said nothing" reading the always-present contract exists to refuse.
struct Nullable<T>(std::marker::PhantomData<T>);

impl<T: JsonSchema> JsonSchema for Nullable<T> {
    fn inline_schema() -> bool {
        true
    }

    fn schema_name() -> Cow<'static, str> {
        format!("Nullable_{}", T::schema_name()).into()
    }

    fn schema_id() -> Cow<'static, str> {
        format!("qontinui_vision_core::Nullable<{}>", T::schema_id()).into()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        <Option<T>>::json_schema(generator)
    }
}

/// Reads an `Option` whose KEY is required: `null` is `None`, a missing key
/// is an error. Plain `Option` fields would read a missing key as `None`,
/// which is exactly the "said nothing" the always-present contract refuses.
fn required_nullable<'de, D, T>(d: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(d)
}

fn finite_confidence(c: f64) -> Option<f64> {
    if !c.is_finite() {
        return None;
    }
    debug_assert!(
        (0.0..=1.0).contains(&c),
        "confidence {c} is outside the documented 0.0..=1.0"
    );
    Some(c.clamp(0.0, 1.0))
}

/// Belt and braces over the private field: even a value that somehow held a
/// non-finite float reaches the wire as `null`, never as a number.
fn serialize_confidence<S: Serializer>(c: &Option<f64>, s: S) -> Result<S::Ok, S::Error> {
    match c.and_then(finite_confidence) {
        Some(v) => s.serialize_some(&v),
        None => s.serialize_none(),
    }
}

/// The key is REQUIRED (no `#[serde(default)]`): a payload with no
/// `confidence` key is a producer that did not state one, and the contract
/// has no such producer. A present number must be in range.
fn deserialize_confidence<'de, D: Deserializer<'de>>(d: D) -> Result<Option<f64>, D::Error> {
    match Option::<f64>::deserialize(d)? {
        None => Ok(None),
        Some(c) if c.is_finite() && (0.0..=1.0).contains(&c) => Ok(Some(c)),
        Some(c) => Err(de::Error::custom(format!(
            "confidence {c} is outside 0.0..=1.0"
        ))),
    }
}

// ---------------------------------------------------------------------------
// Observation
// ---------------------------------------------------------------------------

/// The three answers, as the `status` tag spells them on the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ObservationStatus {
    /// The producer looked and measured a value.
    Measured,
    /// The producer looked, with FULL coverage, and found nothing.
    Absent,
    /// The producer could not answer.
    Unknown,
}

/// The status and its payload, as one value so neither can exist without
/// the other.
#[derive(Debug, Clone, PartialEq)]
pub enum ObservationState<T> {
    Measured(T),
    Absent,
    Unknown(UnknownInfo),
}

impl<T> ObservationState<T> {
    pub fn status(&self) -> ObservationStatus {
        match self {
            Self::Measured(_) => ObservationStatus::Measured,
            Self::Absent => ObservationStatus::Absent,
            Self::Unknown(_) => ObservationStatus::Unknown,
        }
    }
}

/// Why an [`Observation`] could not be built.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ObservationError {
    /// `absent` was claimed over coverage that left dimensions unmeasured.
    #[error(
        "`absent` requires full coverage, but {} dimension(s) went unmeasured ({}); \
         report `measured` with the unmeasured dimensions, or `unknown`",
        .dimensions.len(),
        .dimensions.join(", ")
    )]
    AbsentOverUnmeasuredCoverage { dimensions: Vec<String> },
}

/// One observation: a [`ObservationState`] plus its [`Provenance`]. See the
/// module doc for the wire shape and the invariants.
#[derive(Debug, Clone, PartialEq)]
pub struct Observation<T> {
    state: ObservationState<T>,
    provenance: Provenance,
}

impl<T> Observation<T> {
    /// The producer looked and measured `value`. Legal over any coverage — a
    /// non-empty `coverage.unmeasured` is the DEGRADED case: the value
    /// stands, and the named dimensions were not ruled out.
    pub fn measured(value: T, provenance: Provenance) -> Self {
        Self {
            state: ObservationState::Measured(value),
            provenance,
        }
    }

    /// The producer looked with FULL coverage and found nothing.
    ///
    /// # Errors
    ///
    /// [`ObservationError::AbsentOverUnmeasuredCoverage`] when
    /// `provenance.coverage.unmeasured` is non-empty: nothing can be claimed
    /// absent from a region that was not measured.
    pub fn absent(provenance: Provenance) -> Result<Self, ObservationError> {
        if !provenance.coverage.is_full() {
            return Err(ObservationError::AbsentOverUnmeasuredCoverage {
                dimensions: provenance
                    .coverage
                    .unmeasured
                    .iter()
                    .map(|d| d.dimension.clone())
                    .collect(),
            });
        }
        Ok(Self {
            state: ObservationState::Absent,
            provenance,
        })
    }

    /// The producer could not answer, for the typed reason `code`.
    pub fn unknown(code: UnknownCode, detail: impl Into<String>, provenance: Provenance) -> Self {
        Self {
            state: ObservationState::Unknown(UnknownInfo::new(code, detail)),
            provenance,
        }
    }

    pub fn status(&self) -> ObservationStatus {
        self.state.status()
    }

    pub fn state(&self) -> &ObservationState<T> {
        &self.state
    }

    /// The measured value; `None` unless the status is `measured`.
    pub fn value(&self) -> Option<&T> {
        match &self.state {
            ObservationState::Measured(v) => Some(v),
            _ => None,
        }
    }

    /// The typed reason; `None` unless the status is `unknown`.
    pub fn unknown_info(&self) -> Option<&UnknownInfo> {
        match &self.state {
            ObservationState::Unknown(u) => Some(u),
            _ => None,
        }
    }

    pub fn provenance(&self) -> &Provenance {
        &self.provenance
    }

    pub fn into_parts(self) -> (ObservationState<T>, Provenance) {
        (self.state, self.provenance)
    }

    /// Transform a measured value, keeping status and provenance. Cannot
    /// break the absent-over-full-coverage invariant: coverage is untouched.
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Observation<U> {
        Observation {
            state: match self.state {
                ObservationState::Measured(v) => ObservationState::Measured(f(v)),
                ObservationState::Absent => ObservationState::Absent,
                ObservationState::Unknown(u) => ObservationState::Unknown(u),
            },
            provenance: self.provenance,
        }
    }
}

impl<T: Serialize> Serialize for Observation<T> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut st = s.serialize_struct("Observation", 3)?;
        st.serialize_field("status", &self.state.status())?;
        match &self.state {
            ObservationState::Measured(v) => st.serialize_field("value", v)?,
            ObservationState::Absent => {}
            ObservationState::Unknown(u) => st.serialize_field("unknown", u)?,
        }
        st.serialize_field("provenance", &self.provenance)?;
        st.end()
    }
}

/// The flat wire form, read with KEY PRESENCE preserved for `value` and
/// `unknown`: a `measured` whose `value` key is missing is refused even when
/// `T` would accept `null` (`Option<_>`, `serde_json::Value`), and a present
/// `"unknown": null` on a non-unknown status is refused rather than read as
/// absent.
///
/// Unrecognised sibling keys are IGNORED, not refused, so the envelope can
/// grow without a flag day: the nested provenance types already accept extra
/// keys, and refusing at the top level would drop the whole observation —
/// value and unknown code alike — at every reader older than the writer.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(bound(deserialize = "T: Deserialize<'de>"))]
struct ObservationWire<T> {
    status: ObservationStatus,
    #[serde(default, deserialize_with = "deserialize_present")]
    value: Option<T>,
    #[serde(default, deserialize_with = "deserialize_present")]
    unknown: Option<Option<UnknownInfo>>,
    provenance: Provenance,
}

/// `Some(T)` whenever the key is PRESENT (including an explicit `null`), so
/// the `#[serde(default)]` `None` means exactly "the key was absent".
fn deserialize_present<'de, D, T>(d: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(d).map(Some)
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Observation<T> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let w = ObservationWire::<T>::deserialize(d)?;
        let state = match (w.status, w.value, w.unknown) {
            (ObservationStatus::Measured, Some(v), None) => ObservationState::Measured(v),
            (ObservationStatus::Measured, None, _) => {
                return Err(de::Error::custom(
                    "a `measured` observation must carry `value`",
                ))
            }
            (ObservationStatus::Absent, None, None) => ObservationState::Absent,
            (ObservationStatus::Unknown, None, Some(Some(u))) => ObservationState::Unknown(u),
            (ObservationStatus::Unknown, _, None | Some(None)) => {
                return Err(de::Error::custom(
                    "an `unknown` observation must carry `unknown: { code, detail }`",
                ))
            }
            (status, _, _) => {
                return Err(de::Error::custom(format!(
                    "`value` is legal only on `measured` and `unknown` only on `unknown`; \
                     got both or the wrong one for status `{}`",
                    match status {
                        ObservationStatus::Measured => "measured",
                        ObservationStatus::Absent => "absent",
                        ObservationStatus::Unknown => "unknown",
                    }
                )))
            }
        };
        if matches!(state, ObservationState::Absent) && !w.provenance.coverage.is_full() {
            return Err(de::Error::custom(
                Observation::<T>::absent(w.provenance)
                    .err()
                    .expect("coverage is not full, so absent refuses"),
            ));
        }
        Ok(Self {
            state,
            provenance: w.provenance,
        })
    }
}

// Schema-only mirror of the wire: one object per status, each requiring
// exactly the keys that status carries. Never constructed; it exists so the
// generated TS/Python bindings are a discriminated union on `status` rather
// than an all-optional bag. Its doc comment below is the published schema
// description of `Observation`.
/// One UI Bridge observation: `measured` (with `value`), `absent` (the
/// producer looked with full coverage and found nothing), or `unknown` (the
/// producer could not answer; `unknown.code` says why). `provenance` is
/// always present and every one of its keys is always present.
#[derive(JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
#[allow(dead_code)]
enum ObservationWireSchema<T> {
    Measured {
        value: T,
        provenance: Provenance,
    },
    Absent {
        provenance: Provenance,
    },
    Unknown {
        unknown: UnknownInfo,
        provenance: Provenance,
    },
}

impl<T: JsonSchema> JsonSchema for Observation<T> {
    fn schema_name() -> Cow<'static, str> {
        "Observation".into()
    }

    fn schema_id() -> Cow<'static, str> {
        format!("qontinui_vision_core::Observation<{}>", T::schema_id()).into()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        ObservationWireSchema::<T>::json_schema(generator)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use serde_json::json;

    fn t(sec: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 9, 30, 12, 0, sec).unwrap()
    }

    fn prov(coverage: ObservationCoverage) -> Provenance {
        Provenance::new(Producer::vision_core(Analyzer::Layout), t(1), coverage)
    }

    fn obj(v: &Value) -> &Map<String, Value> {
        v.as_object().expect("an object")
    }

    /// Asserts on the serialized VALUE, not a round trip — a round trip
    /// passes whether or not the key is on the wire.
    fn assert_null_key(o: &Map<String, Value>, key: &str) {
        assert!(
            o.contains_key(key),
            "`{key}` is missing from the wire: {o:?}"
        );
        assert!(o[key].is_null(), "`{key}` should be null: {:?}", o[key]);
    }

    #[test]
    fn unknown_code_is_exactly_the_contract_set() {
        let wire: Vec<Value> = UnknownCode::ALL
            .iter()
            .map(|c| serde_json::to_value(c).unwrap())
            .collect();
        assert_eq!(
            wire,
            vec![
                json!("app_unreachable"),
                json!("capability_unavailable_in_build"),
                json!("producer_failed"),
                json!("producer_not_run"),
                json!("input_missing"),
                json!("below_confidence_floor"),
                json!("model_reply_unparseable"),
                json!("needs_multi_frame_input"),
                json!("stale_input"),
            ]
        );
        for c in UnknownCode::ALL {
            // `as_str` and serde agree, and the value round-trips.
            assert_eq!(serde_json::to_value(c).unwrap(), json!(c.as_str()));
            let back: UnknownCode = serde_json::from_value(json!(c.as_str())).unwrap();
            assert_eq!(back, c);
        }
        assert!(serde_json::from_value::<UnknownCode>(json!("healthy")).is_err());
        assert_eq!(
            UnknownCode::CapabilityUnavailableInBuild.diagnostic_code(),
            "UB-OBS-CAPABILITY-UNAVAILABLE-IN-BUILD"
        );
        assert_eq!(
            UnknownCode::InputMissing.diagnostic_code(),
            "UB-OBS-INPUT-MISSING"
        );
    }

    #[test]
    fn a_deduction_states_every_null_provenance_key_on_the_wire() {
        let o = Observation::measured(json!({"elements": 3}), prov(ObservationCoverage::full(3)));
        let v = serde_json::to_value(&o).unwrap();
        let top = obj(&v);
        assert_eq!(top["status"], json!("measured"));
        assert_eq!(top["value"], json!({"elements": 3}));
        assert!(!top.contains_key("unknown"));
        let p = obj(&top["provenance"]);
        assert_null_key(p, "confidence");
        assert_null_key(p, "cache");
        assert_null_key(p, "observedAt");
        assert_null_key(p, "source");
        assert_eq!(p["evaluatedAt"], json!("2026-09-30T12:00:01Z"));
        assert_eq!(
            p["producer"],
            json!({"id": "vision-core/layout", "version": env!("CARGO_PKG_VERSION")})
        );
        assert_eq!(
            p["coverage"],
            json!({"considered": 3, "measured": 3, "unmeasured": []})
        );
    }

    #[test]
    fn a_cache_miss_states_a_null_stored_at() {
        let p = prov(ObservationCoverage::full(1)).with_cache(CacheProvenance {
            hit: false,
            stored_at: None,
            key_inputs: vec!["frame_sha256".into()],
        });
        let v = serde_json::to_value(&p).unwrap();
        let cache = obj(&obj(&v)["cache"]);
        assert_eq!(cache["hit"], json!(false));
        assert_null_key(cache, "storedAt");
        assert_eq!(cache["keyInputs"], json!(["frame_sha256"]));
    }

    #[test]
    fn a_nan_confidence_serializes_null_and_never_a_number() {
        let p = prov(ObservationCoverage::full(1)).with_confidence(f64::NAN);
        assert_eq!(p.confidence(), None);
        let v = serde_json::to_value(&p).unwrap();
        assert_null_key(obj(&v), "confidence");

        let p = prov(ObservationCoverage::full(1)).with_confidence(f64::INFINITY);
        let v = serde_json::to_value(&p).unwrap();
        assert_null_key(obj(&v), "confidence");

        // The serializer's own filter, independent of the setter: a
        // non-finite value planted in the private field still reaches the
        // wire as null.
        let mut p = prov(ObservationCoverage::full(1));
        p.confidence = Some(f64::NAN);
        let v = serde_json::to_value(&p).unwrap();
        assert_null_key(obj(&v), "confidence");

        let p = prov(ObservationCoverage::full(1)).with_confidence(0.75);
        let v = serde_json::to_value(&p).unwrap();
        assert_eq!(obj(&v)["confidence"], json!(0.75));
    }

    #[test]
    fn absent_over_unmeasured_coverage_is_refused() {
        let cov = ObservationCoverage {
            considered: 7,
            measured: 5,
            unmeasured: vec![UnmeasuredDimension::new(
                "bbox",
                2,
                UnknownCode::InputMissing,
            )],
        };
        let err = Observation::<Value>::absent(prov(cov)).unwrap_err();
        assert_eq!(
            err,
            ObservationError::AbsentOverUnmeasuredCoverage {
                dimensions: vec!["bbox".into()]
            }
        );
        assert!(Observation::<Value>::absent(prov(ObservationCoverage::full(7))).is_ok());
    }

    #[test]
    fn measured_over_unmeasured_coverage_is_the_legal_degraded_case() {
        let cov = ObservationCoverage {
            considered: 7,
            measured: 5,
            unmeasured: vec![UnmeasuredDimension::new(
                "bbox",
                2,
                UnknownCode::InputMissing,
            )],
        };
        let o = Observation::measured(json!([]), prov(cov));
        let v = serde_json::to_value(&o).unwrap();
        assert_eq!(
            v["provenance"]["coverage"]["unmeasured"],
            json!([{"dimension": "bbox", "count": 2, "code": "input_missing"}])
        );
    }

    #[test]
    fn the_three_states_have_exactly_their_keys() {
        let m = serde_json::to_value(Observation::measured(
            1u32,
            prov(ObservationCoverage::full(1)),
        ))
        .unwrap();
        let a = serde_json::to_value(
            Observation::<u32>::absent(prov(ObservationCoverage::full(1))).unwrap(),
        )
        .unwrap();
        let u = serde_json::to_value(Observation::<u32>::unknown(
            UnknownCode::AppUnreachable,
            "connection refused",
            prov(ObservationCoverage::default()),
        ))
        .unwrap();
        let keys = |v: &Value| {
            let mut k: Vec<String> = obj(v).keys().cloned().collect();
            k.sort();
            k
        };
        assert_eq!(keys(&m), ["provenance", "status", "value"]);
        assert_eq!(keys(&a), ["provenance", "status"]);
        assert_eq!(keys(&u), ["provenance", "status", "unknown"]);
        assert_eq!(a["status"], json!("absent"));
        assert_eq!(
            u["unknown"],
            json!({"code": "app_unreachable", "detail": "connection refused"})
        );
        // No verdict or score leaks into the envelope.
        for v in [&m, &a, &u] {
            let s = v.to_string();
            assert!(!s.contains("healthy") && !s.contains("score"), "{s}");
        }
    }

    #[test]
    fn every_state_round_trips() {
        let p = prov(ObservationCoverage::full(2))
            .with_observed_at(t(0))
            .with_confidence(0.5)
            .with_cache(CacheProvenance {
                hit: true,
                stored_at: Some(t(0)),
                key_inputs: vec!["frame_sha256".into(), "prompt_version".into()],
            })
            .with_source(obj(&json!({"kind": "snapshot"})).clone());
        for o in [
            Observation::measured(json!({"x": 1}), p.clone()),
            Observation::absent(p.clone()).unwrap(),
            Observation::unknown(UnknownCode::StaleInput, "old", p.clone()),
        ] {
            let back: Observation<Value> =
                serde_json::from_value(serde_json::to_value(&o).unwrap()).unwrap();
            assert_eq!(back, o);
        }
    }

    fn wire_provenance() -> Value {
        serde_json::to_value(prov(ObservationCoverage::full(0))).unwrap()
    }

    #[test]
    fn an_unknown_without_a_code_fails_to_deserialize() {
        let no_code = json!({
            "status": "unknown",
            "unknown": {"detail": "no bridge"},
            "provenance": wire_provenance(),
        });
        assert!(serde_json::from_value::<Observation<Value>>(no_code).is_err());

        let no_unknown = json!({"status": "unknown", "provenance": wire_provenance()});
        assert!(serde_json::from_value::<Observation<Value>>(no_unknown).is_err());

        let bogus_code = json!({
            "status": "unknown",
            "unknown": {"code": "who_knows", "detail": "x"},
            "provenance": wire_provenance(),
        });
        assert!(serde_json::from_value::<Observation<Value>>(bogus_code).is_err());
    }

    #[test]
    fn a_measured_without_a_value_fails_to_deserialize_even_for_nullable_t() {
        let w = json!({"status": "measured", "provenance": wire_provenance()});
        assert!(serde_json::from_value::<Observation<Value>>(w.clone()).is_err());
        // The shape a flattened derive would have let through: `T` accepts
        // `null`, so a missing key could silently become `None`.
        assert!(serde_json::from_value::<Observation<Option<u32>>>(w).is_err());
        // An explicit `null` value is a present value and IS legal for such a T.
        let explicit =
            json!({"status": "measured", "value": null, "provenance": wire_provenance()});
        let o: Observation<Option<u32>> = serde_json::from_value(explicit).unwrap();
        assert_eq!(o.value(), Some(&None));
    }

    #[test]
    fn cross_state_keys_and_absent_over_partial_coverage_fail_to_deserialize() {
        let p = wire_provenance();
        for bad in [
            json!({"status": "absent", "value": 1, "provenance": p}),
            json!({"status": "absent", "unknown": {"code": "stale_input", "detail": ""}, "provenance": p}),
            json!({"status": "measured", "value": 1, "unknown": {"code": "stale_input", "detail": ""}, "provenance": p}),
            json!({"status": "unknown", "value": 1, "unknown": {"code": "stale_input", "detail": ""}, "provenance": p}),
            json!({"value": 1, "provenance": p}),
            // A present `null` is not an absent key.
            json!({"status": "absent", "unknown": null, "provenance": p}),
            json!({"status": "measured", "value": 1, "unknown": null, "provenance": p}),
            json!({"status": "unknown", "unknown": null, "provenance": p}),
        ] {
            assert!(
                serde_json::from_value::<Observation<Value>>(bad.clone()).is_err(),
                "accepted {bad}"
            );
        }
        let mut partial = wire_provenance();
        partial["coverage"] = json!({
            "considered": 3, "measured": 1,
            "unmeasured": [{"dimension": "bbox", "count": 2, "code": "input_missing"}]
        });
        let absent = json!({"status": "absent", "provenance": partial});
        let err = serde_json::from_value::<Observation<Value>>(absent).unwrap_err();
        assert!(err.to_string().contains("full coverage"), "{err}");
    }

    #[test]
    fn an_unrecognised_sibling_key_is_ignored_so_the_envelope_can_grow() {
        let v = json!({"status": "measured", "value": 1, "diagnostics": {"x": 1}, "provenance": wire_provenance()});
        let o = serde_json::from_value::<Observation<Value>>(v).expect("extra key must not refuse");
        assert_eq!(o.status(), ObservationStatus::Measured);
    }

    #[test]
    fn a_cache_missing_stored_at_fails_to_deserialize() {
        let mut p = wire_provenance();
        p["cache"] = json!({"hit": false, "keyInputs": []});
        assert!(serde_json::from_value::<Provenance>(p).is_err());
    }

    #[test]
    fn a_provenance_missing_a_null_key_fails_to_deserialize() {
        for key in ["confidence", "cache", "observedAt", "source"] {
            let mut p = wire_provenance();
            p.as_object_mut().unwrap().remove(key);
            assert!(
                serde_json::from_value::<Provenance>(p).is_err(),
                "a provenance with no `{key}` key parsed"
            );
        }
        let mut p = wire_provenance();
        p["confidence"] = json!(1.5);
        assert!(serde_json::from_value::<Provenance>(p).is_err());
    }

    /// The generated TS/Python bindings are only as strict as the schema:
    /// every provenance key must be REQUIRED there too (an optional key
    /// would generate `confidence?: number | null`, the "said nothing"
    /// reading), while the nullable ones still admit `null`.
    #[test]
    fn the_schema_requires_every_provenance_key_and_admits_null_where_stated() {
        let schema = serde_json::to_value(schemars::schema_for!(Provenance)).unwrap();
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
                "cache",
                "confidence",
                "coverage",
                "evaluatedAt",
                "observedAt",
                "producer",
                "source"
            ]
        );
        let props = &schema["properties"];
        for key in ["confidence", "observedAt", "source"] {
            assert!(
                props[key]["type"]
                    .as_array()
                    .is_some_and(|t| t.contains(&json!("null"))),
                "`{key}` must admit null: {}",
                props[key]
            );
        }
        assert!(
            props["cache"]["anyOf"]
                .as_array()
                .is_some_and(|a| a.contains(&json!({"type": "null"}))),
            "`cache` must admit null: {}",
            props["cache"]
        );
        assert_eq!(props["evaluatedAt"]["type"], json!("string"));

        let cache = serde_json::to_value(schemars::schema_for!(CacheProvenance)).unwrap();
        assert!(cache["required"]
            .as_array()
            .unwrap()
            .contains(&json!("storedAt")));

        // The envelope is a union discriminated on `status`, one arm per state.
        let obs = serde_json::to_value(schemars::schema_for!(Observation<Value>)).unwrap();
        assert_eq!(obs["title"], json!("Observation"));
        let arms = obs["oneOf"].as_array().unwrap();
        let consts: Vec<&Value> = arms
            .iter()
            .map(|a| &a["properties"]["status"]["const"])
            .collect();
        assert_eq!(
            consts,
            [&json!("measured"), &json!("absent"), &json!("unknown")]
        );
        assert_eq!(
            arms[0]["required"],
            json!(["status", "value", "provenance"])
        );
        assert_eq!(
            arms[2]["required"],
            json!(["status", "unknown", "provenance"])
        );
    }
}
