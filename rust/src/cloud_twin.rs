//! Cloud twin contracts — the local cloud (a Qontinui Cloud clone) used as a
//! substrate for the digital twin.
//!
//! Plan `2026-10-11-local-cloud-as-twin-substrate`, Phase 2 step 1. The types
//! here are the wire contract between the clone platform (the producer), the
//! rehearsal harness, and coord:
//!
//! - [`World`] / [`WorldLifecycleEvent`] — a world is an *instance dimension*
//!   (production, a clone, or a fork of either), not a sub-space.
//! - [`CloudAuditEvent`] / [`CloudAuditRollup`] — the allowlisted,
//!   CloudTrail-shaped projection coord stores from an audit stream (plan D3).
//!   **There is no field that can hold a request-parameter value**, only a
//!   [`ParameterDigest`]; the same "there is no field for a value" rule as
//!   coord's `infra_plan_ingest.rs`.
//! - [`SubstitutionBlock`] — the τ/β block every world-derived verdict carries.
//!   Its fields are private and its **only constructor is the pure
//!   [`SubstitutionBlock::compute`]** (plan D4: "τ is computed, never
//!   stamped"); deserialization refuses a block `compute` could not produce.
//! - [`RehearsalRecord`], [`ConformanceRow`] (requested amendment A4, keyed by
//!   [`ClaimKind`]) and [`VendorDriftEvent`] (requested amendment A5).
//!
//! Wire format: field names are `snake_case` (Rust's own spelling, matching
//! coord's `DriftVerdict`, which a [`SubstitutionBlock`] rides inside); enum
//! tokens are `snake_case`. Timestamps are ISO-8601 `String`s and ids are
//! `String`s, per the crate conventions. [`CloudAuditEvent`] uses
//! CloudTrail's field *meanings* in snake_case spelling (`eventTime` →
//! `event_time`), because it is a projection of a CloudTrail record, not a
//! CloudTrail record.
//!
//! **No TypeScript or Python export is registered for this module.** Plan
//! Phase 2 registers exports only where a consumer exists, and none does yet;
//! the first is Phase 8's Python harness (`RehearsalRecord`). Bindings are
//! enumerated explicitly by `add!` lines in qontinui-runner's
//! `src-tauri/src/schema_export.rs`, so adding this module emits nothing and
//! `schema-drift.yml` has nothing to regenerate.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ============================================================================
// Shared vocabulary
// ============================================================================

/// The claim kinds of the clone's claim ledger (clone plan D4). τ is kept per
/// operation family **and** claim kind (`theory/interventional-twin.md`
/// Definition 4).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ClaimKind {
    Shape,
    Semantic,
    Error,
    Ordering,
    Consistency,
    Limit,
    Auth,
    Timing,
}

/// The clone ledger's authority order (clone plan D4), **highest first**: the
/// derived `Ord` makes `RecordedRealService` the smallest value, so the
/// *weakest* authority among a set is its `max()`.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ClaimAuthority {
    /// 1. A recorded exchange with the real service (oracle-verified).
    RecordedRealService,
    /// 2. Vendor SDK and Terraform acceptance-test behaviour.
    VendorSdkAcceptance,
    /// 3. The vendor's Smithy model.
    SmithyModel,
    /// 4. Vendor documentation (doc-attested).
    Documentation,
    /// 5. A vendor tutorial.
    Tutorial,
    /// 6. Community emulator tests.
    CommunityEmulator,
}

impl ClaimAuthority {
    /// The conformance band letter for this authority rank: `A` for rank 1
    /// through `F` for rank 6. "Band B or better" is the top two ranks.
    pub fn band_letter(self) -> char {
        match self {
            Self::RecordedRealService => 'A',
            Self::VendorSdkAcceptance => 'B',
            Self::SmithyModel => 'C',
            Self::Documentation => 'D',
            Self::Tutorial => 'E',
            Self::CommunityEmulator => 'F',
        }
    }
}

/// A claim's (or an exercised operation's) verdict, in coord's
/// `confirmed | contradicted | unknown` vocabulary, plus `unsupported` for an
/// operation the world does not build (`clone:unsupported`, a recorded blind
/// spot).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ClaimVerdict {
    Confirmed,
    Contradicted,
    Unknown,
    Unsupported,
}

/// One cloud API operation, by service and operation name (e.g.
/// `ecs` / `DescribeServices`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct OperationRef {
    pub service: String,
    pub operation: String,
}

// ============================================================================
// Worlds
// ============================================================================

/// What a world is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WorldKind {
    /// Production — the real cloud.
    Prod,
    /// A world hosted by a clone release.
    Clone,
    /// A fork of another world (its `parent`).
    Fork,
}

/// The class of data a world holds. Real data and real users stay
/// production-only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DataClass {
    Synthetic,
    QontinuiOwn,
    Customer,
}

/// A world: production, a clone, or a fork. Worlds are an instance dimension
/// of the twin, not a sub-space.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct World {
    pub kind: WorldKind,
    pub id: String,
    /// The world this one was forked from. Absent for `prod` and for a world
    /// created from a template or seed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    /// The pinned clone release hosting this world. Absent for `prod`; absent
    /// on a non-prod world means it was built from an **unreleased** clone
    /// build, which the credibility firewall (plan D6) renders as
    /// [`TauState::UnreleasedClone`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clone_release: Option<String>,
    pub data_class: DataClass,
}

/// The consistency level a world snapshot was taken at (requested amendment
/// A2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotConsistency {
    ControlPlaneOnly,
    CrashConsistent,
    Quiesced,
}

/// What a world was created from (requested amendment A2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WorldSeedSource {
    Template,
    Snapshot,
    /// A value-free twin seed.
    TwinSeed,
}

/// The lifecycle transition a [`WorldLifecycleEvent`] records. Internally
/// tagged by `change`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "change", rename_all = "snake_case")]
pub enum WorldLifecycleChange {
    Created {
        world: World,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        seeded_from: Option<WorldSeedSource>,
        /// The template name, snapshot id or seed id, per `seeded_from`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        seed_ref: Option<String>,
    },
    Snapshotted {
        snapshot_id: String,
        consistency: SnapshotConsistency,
    },
    Forked {
        /// The new world; its `parent` is the event's `world_id`.
        child: World,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        from_snapshot: Option<String>,
    },
    Destroyed {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },
}

/// One world lifecycle event, as the clone platform exports it (requested
/// amendment A2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct WorldLifecycleEvent {
    pub world_id: String,
    pub occurred_at: String,
    #[serde(flatten)]
    pub change: WorldLifecycleChange,
    /// When the world expires, if it carries a TTL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ttl_at: Option<String>,
}

// ============================================================================
// Audit projection (plan D3)
// ============================================================================

/// How an audit event was attributed to Qontinui work. Each method carries
/// its own credibility: `ua_app_id` is self-declared (plan D2), a world
/// principal is authenticated (A1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AttributionMethod {
    /// `AWS_SDK_UA_APP_ID` set per session by the runner (self-declared).
    UaAppId,
    /// An authenticated world principal (inside a clone world).
    WorldPrincipal,
    /// One of coord's own observers, by its task-role principal.
    CoordObserver,
    /// A CI role.
    CiRole,
    /// The clone's oracle recorder role (clone plan D16).
    OracleRecorder,
    /// Not attributed. Explicit, never a silent default.
    None,
}

/// Qontinui attribution carried on an audit event (A1's
/// `additionalEventData.qontinui`). Every field but `method` is optional:
/// absent means *not reported*, never a default.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QontinuiAttribution {
    pub method: AttributionMethod,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work_unit: Option<String>,
    /// `owner/repo#n`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pr: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_id: Option<String>,
    /// The world control plane's log sequence number at the event, as the
    /// producer renders it (a `String`, so a Postgres `X/Y` LSN fits).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub control_plane_lsn: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
    /// The claim-ledger ids this event is conformance evidence for (A1).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conformance_claim_ids: Vec<String>,
    /// Whether the world injected a fault into this call. Absent is UNKNOWN.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fault_injected: Option<bool>,
}

/// The caller identity of an audit event (CloudTrail `userIdentity`, reduced
/// to identifiers). No access key id and no session credentials are carried.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuditPrincipal {
    /// CloudTrail `userIdentity.type` (`AssumedRole`, `IAMUser`, …).
    pub identity_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arn: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    /// For an assumed role, the role's ARN (`sessionContext.sessionIssuer.arn`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_issuer_arn: Option<String>,
}

/// A resource an audit event touched — identifiers only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuditResource {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arn: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
}

/// The outcome of an audited call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AuditOutcome {
    Success,
    Error,
    /// An authorization refusal (`AccessDenied`, `UnauthorizedOperation`, …).
    Denied,
}

/// Digest algorithm for a [`ParameterDigest`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DigestAlgorithm {
    /// SHA-256 over the RFC 8785 (JCS) canonical JSON of the request
    /// parameters.
    Sha256Jcs,
}

/// A digest of an event's request parameters. Equality of digests says two
/// calls carried the same parameters; the parameters themselves are never
/// stored (plan D3).
///
/// Fields are private and validated both by [`ParameterDigest::new`] and on
/// deserialization: for [`DigestAlgorithm::Sha256Jcs`], `hex` must be exactly
/// 64 lowercase hex characters, so the field cannot smuggle a value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(try_from = "ParameterDigestWire")]
pub struct ParameterDigest {
    algorithm: DigestAlgorithm,
    /// Lowercase hex.
    hex: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ParameterDigestWire {
    algorithm: DigestAlgorithm,
    hex: String,
}

impl TryFrom<ParameterDigestWire> for ParameterDigest {
    type Error = ContractViolation;
    fn try_from(w: ParameterDigestWire) -> Result<Self, Self::Error> {
        Self::new(w.algorithm, w.hex)
    }
}

impl ParameterDigest {
    /// Build a digest, refusing a `hex` that is not a well-formed digest for
    /// `algorithm`.
    pub fn new(
        algorithm: DigestAlgorithm,
        hex: impl Into<String>,
    ) -> Result<Self, ContractViolation> {
        let hex = hex.into();
        match algorithm {
            DigestAlgorithm::Sha256Jcs => {
                let ok = hex.len() == 64
                    && hex
                        .bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
                if !ok {
                    return Err(ContractViolation::new(
                        "parameter digest: sha256_jcs hex must be exactly 64 lowercase hex characters",
                    ));
                }
            }
        }
        Ok(Self { algorithm, hex })
    }
    pub fn algorithm(&self) -> DigestAlgorithm {
        self.algorithm
    }
    pub fn hex(&self) -> &str {
        &self.hex
    }
}

/// A wire value that breaks a `cloud_twin` contract invariant — returned by
/// validating constructors and surfaced as a deserialization error.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("cloud_twin contract violated: {0}")]
pub struct ContractViolation(String);

impl ContractViolation {
    fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
}

/// Which storage class plan D3 assigns an audit event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AuditStorageClass {
    /// Stored as a full [`CloudAuditEvent`] row.
    FullRow,
    /// Folded into an hourly [`CloudAuditRollup`].
    HourlyRollup,
}

/// One audit event, as the allowlisted CloudTrail-shaped projection coord
/// stores (plan D3, option B): who, what, when, where, the outcome, resource
/// identifiers and a **digest** of the request parameters.
///
/// There is deliberately no field for request parameters, response elements,
/// an error message (which can echo parameter values), or raw
/// `additionalEventData` (only its `qontinui` member survives, lifted to the
/// top-level [`CloudAuditEvent::qontinui`]); and the type is
/// `deny_unknown_fields`, so a producer that sends one is refused at ingest
/// rather than silently projected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CloudAuditEvent {
    /// CloudTrail `eventID`.
    pub event_id: String,
    /// CloudTrail `eventTime` (ISO-8601).
    pub event_time: String,
    /// CloudTrail `eventSource`, e.g. `rds.amazonaws.com`.
    pub event_source: String,
    /// CloudTrail `eventName`, e.g. `ModifyDBInstance`.
    pub event_name: String,
    /// CloudTrail `awsRegion`.
    pub aws_region: String,
    /// CloudTrail `recipientAccountId`.
    pub recipient_account_id: String,
    pub user_identity: AuditPrincipal,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_ip_address: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    /// CloudTrail `readOnly`. Absent is UNKNOWN, and an UNKNOWN event is
    /// stored as a full row (see [`CloudAuditEvent::storage_class`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read_only: Option<bool>,
    pub outcome: AuditOutcome,
    /// CloudTrail `errorCode` — a code, never the message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    /// CloudTrail `requestID`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub resources: Vec<AuditResource>,
    /// Absent is **UNKNOWN** — the producer reported no digest — never "the
    /// call carried no parameters". A producer digests a call with no request
    /// parameters as the JCS canonical form of the empty object `{}`, so "no
    /// parameters" is always a present digest.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_parameters_digest: Option<ParameterDigest>,
    /// Qontinui attribution. Requested amendment A1 has the producer emit it
    /// inside CloudTrail's `additionalEventData.qontinui`; the projection
    /// lifts that one member to this top-level field and keeps nothing else
    /// of `additionalEventData`.
    pub qontinui: QontinuiAttribution,
}

impl CloudAuditEvent {
    /// Plan D3's storage rule: successful read-only events fold into hourly
    /// rollups; mutating, error and denied events — and any event whose
    /// read-only flag is UNKNOWN — keep a full row.
    pub fn storage_class(&self) -> AuditStorageClass {
        match (self.read_only, self.outcome) {
            (Some(true), AuditOutcome::Success) => AuditStorageClass::HourlyRollup,
            _ => AuditStorageClass::FullRow,
        }
    }
}

/// The hourly rollup of successful read-only events (plan D3):
/// `(principal, session, event_source, event_name, count)` per hour.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CloudAuditRollup {
    /// Start of the hour (ISO-8601, truncated to the hour).
    pub hour_start: String,
    /// The principal ARN (for an assumed role, the issuer role's ARN).
    pub principal: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub event_source: String,
    pub event_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_id: Option<String>,
    pub count: u64,
}

// ============================================================================
// Substitution block (plan D4)
// ============================================================================

/// One operation a world-derived verdict relied on, with the ledger
/// authority and verdict the clone's scorecard holds for it. `authority` is
/// absent for an operation the ledger has no claim for (e.g. `unsupported`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ExercisedOp {
    pub service: String,
    pub operation: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authority: Option<ClaimAuthority>,
    pub verdict: ClaimVerdict,
}

/// Why `tau.value` is (or is not) a number. [`SubstitutionBlock::compute`]
/// picks the state by the precedence documented there.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TauState {
    /// Every exercised (family, relevant claim kind) is oracle-measured with
    /// n ≥ `min_cases`; `value` is the minimum measured conformance.
    Calibrated,
    /// `value` is null because the evidence is not enough to calibrate τ.
    /// Two cases, and `oracle_share` tells them apart:
    /// - **prior-only** — no oracle evidence at the argmin, `oracle_share`
    ///   is 0. A prior-only τ would be a per-authority constant until the
    ///   priors are calibrated against independent supervision (plan D4), so
    ///   it is not served as a number;
    /// - **partial oracle coverage** — some oracle evidence, but not measured
    ///   or below `min_cases` at the argmin, so `oracle_share` is non-zero.
    Uncalibrated,
    /// An exercised operation is unsupported, has no claim kinds, or has no
    /// conformance row for a relevant (family, kind). `value` is null.
    UnknownOp,
    /// An exercised operation is contradicted; τ is 0.
    Contradicted,
    /// The world was built from an unreleased clone build (plan D6); `value`
    /// is null.
    UnreleasedClone,
    /// The credibility firewall was violated; τ is forced to 0 (plan D6).
    FirewallViolated,
}

impl TauState {
    /// Whether this state carries a numeric `value` (and the others carry
    /// null).
    fn has_value(self) -> bool {
        matches!(
            self,
            Self::Calibrated | Self::Contradicted | Self::FirewallViolated
        )
    }
}

/// The (service, operation family, claim kind) at which τ attains its
/// minimum — the theory's argmin, reported beside τ.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TauArgmin {
    pub service: String,
    pub op_family: String,
    pub claim_kind: ClaimKind,
}

fn check_unit(name: &str, v: f64) -> Result<(), ContractViolation> {
    if v.is_finite() && (0.0..=1.0).contains(&v) {
        Ok(())
    } else {
        Err(ContractViolation::new(format!(
            "{name} must be a finite number in [0, 1], got {v}"
        )))
    }
}

/// A probability-like input that is valid only when finite and in `[0, 1]`.
fn unit_or_none(v: Option<f64>) -> Option<f64> {
    v.filter(|x| x.is_finite() && (0.0..=1.0).contains(x))
}

/// τ — the substitution fidelity of a clone world for a production question.
/// Fields are private, like [`SubstitutionBlock`]'s: only
/// [`SubstitutionBlock::compute`] produces one, and deserialization enforces
/// the invariants `compute` guarantees.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(try_from = "TauWire")]
pub struct Tau {
    /// Null unless `state` is `calibrated`, `contradicted` or
    /// `firewall_violated` (the last two force 0).
    value: Option<f64>,
    state: TauState,
    /// The number of oracle cases behind the argmin.
    n: u64,
    /// Absent only when no (family, kind) was evaluated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    argmin: Option<TauArgmin>,
    /// The argmin's oracle share in `[0, 1]`; 0 for a prior-only τ.
    oracle_share: f64,
}

#[derive(Deserialize, JsonSchema)]
struct TauWire {
    value: Option<f64>,
    state: TauState,
    n: u64,
    #[serde(default)]
    argmin: Option<TauArgmin>,
    oracle_share: f64,
}

impl TryFrom<TauWire> for Tau {
    type Error = ContractViolation;
    fn try_from(w: TauWire) -> Result<Self, Self::Error> {
        Tau::checked(w.value, w.state, w.n, w.argmin, w.oracle_share)
    }
}

impl Tau {
    /// The one place τ's invariants live; `compute` and deserialization both
    /// go through it.
    fn checked(
        value: Option<f64>,
        state: TauState,
        n: u64,
        argmin: Option<TauArgmin>,
        oracle_share: f64,
    ) -> Result<Self, ContractViolation> {
        check_unit("tau.oracle_share", oracle_share)?;
        if let Some(v) = value {
            check_unit("tau.value", v)?;
        }
        match (state.has_value(), value) {
            (true, None) => {
                return Err(ContractViolation::new(format!(
                    "tau.state {state:?} requires a value"
                )))
            }
            (false, Some(_)) => {
                return Err(ContractViolation::new(format!(
                    "tau.state {state:?} must carry a null value"
                )))
            }
            _ => {}
        }
        if matches!(state, TauState::Contradicted | TauState::FirewallViolated)
            && value != Some(0.0)
        {
            return Err(ContractViolation::new(format!(
                "tau.state {state:?} forces value 0"
            )));
        }
        if state == TauState::Calibrated && argmin.is_none() {
            return Err(ContractViolation::new(
                "a calibrated tau must name its argmin",
            ));
        }
        if argmin.is_none() && (n != 0 || oracle_share != 0.0) {
            return Err(ContractViolation::new(
                "tau with no argmin must have n = 0 and oracle_share = 0",
            ));
        }
        Ok(Self {
            value,
            state,
            n,
            argmin,
            oracle_share,
        })
    }

    pub fn value(&self) -> Option<f64> {
        self.value
    }
    pub fn state(&self) -> TauState {
        self.state
    }
    pub fn n(&self) -> u64 {
        self.n
    }
    pub fn argmin(&self) -> Option<&TauArgmin> {
        self.argmin.as_ref()
    }
    pub fn oracle_share(&self) -> f64 {
        self.oracle_share
    }
}

/// What β, the base fidelity, was judged against (theory §3.4, rule R1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BetaBasis {
    /// An HCL base judged against the HCL (β = 1).
    Hcl,
    /// A twin seed, measured at seed time.
    TwinSeed,
    /// A world snapshot.
    Snapshot,
}

/// β — the base fidelity of the world's starting state. Private fields, as
/// for [`Tau`], validated on deserialization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(try_from = "BetaWire")]
pub struct Beta {
    /// Null (UNKNOWN) where β is not measured.
    value: Option<f64>,
    basis: BetaBasis,
}

#[derive(Deserialize, JsonSchema)]
struct BetaWire {
    value: Option<f64>,
    basis: BetaBasis,
}

impl TryFrom<BetaWire> for Beta {
    type Error = ContractViolation;
    fn try_from(w: BetaWire) -> Result<Self, Self::Error> {
        if let Some(v) = w.value {
            check_unit("beta.value", v)?;
        }
        if w.basis == BetaBasis::Hcl && w.value != Some(1.0) {
            return Err(ContractViolation::new(
                "beta with basis hcl must have value 1",
            ));
        }
        Ok(Self {
            value: w.value,
            basis: w.basis,
        })
    }
}

impl Beta {
    pub fn value(&self) -> Option<f64> {
        self.value
    }
    pub fn basis(&self) -> BetaBasis {
        self.basis
    }
}

/// The credibility-firewall verdict on a rehearsal (plan D6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FirewallStatus {
    Ok,
    Violated,
    Unknown,
}

/// The β input to [`SubstitutionBlock::compute`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BetaInput {
    /// An HCL base judged against the HCL: β = 1.
    Hcl,
    /// A twin seed: β is the value measured at seed time, or null when it was
    /// not measured (or the measurement is not a finite number in `[0, 1]`).
    TwinSeed { measured: Option<f64> },
    /// A snapshot base: β is not measured, so null.
    Snapshot,
}

/// One operation a rehearsal relied on, as input to
/// [`SubstitutionBlock::compute`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExercisedOpInput<'a> {
    pub service: &'a str,
    pub operation: &'a str,
    /// The ledger's operation family the operation belongs to — the key
    /// conformance rows are looked up by.
    pub op_family: &'a str,
    pub authority: Option<ClaimAuthority>,
    pub verdict: ClaimVerdict,
    /// The claim kinds relevant to this rehearsal's use of the operation.
    pub claim_kinds: &'a [ClaimKind],
}

/// Everything [`SubstitutionBlock::compute`] reads. Plain data: coord's
/// Phase 6 module loads the conformance snapshot and the firewall verdict and
/// passes them in.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SubstitutionInput<'a> {
    pub world_id: &'a str,
    /// `None` means the world was built from an unreleased clone build.
    pub clone_release: Option<&'a str>,
    pub exercised_ops: &'a [ExercisedOpInput<'a>],
    /// The conformance snapshot. Only rows whose `scorecard_sha` equals
    /// `scorecard_sha` (when given) and whose `clone_release` equals the
    /// world's (when released) are read.
    pub conformance: &'a [ConformanceRow],
    pub scorecard_sha: Option<&'a str>,
    /// The fewest oracle cases a (family, kind) needs to count as measured.
    pub min_cases: u64,
    pub beta: BetaInput,
    pub firewall: FirewallStatus,
}

/// How good the evidence for one exercised (service, family, kind) is.
/// Ordered worst first, mirroring the state precedence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum CellTier {
    Unknown,
    Contradicted,
    Uncalibrated,
    Calibrated,
}

#[derive(Debug, Clone, Copy)]
struct Cell<'a> {
    service: &'a str,
    op_family: &'a str,
    kind: ClaimKind,
    tier: CellTier,
    /// Valid measured conformance (only meaningful for `Calibrated`).
    measured: f64,
    share: f64,
    cases: u64,
    authority: Option<ClaimAuthority>,
}

impl Cell<'_> {
    /// Total order, worst first: tier; then measured conformance (calibrated
    /// cells); then oracle share; then weaker authority (no authority is
    /// weakest); then service, family (lexical) and claim kind (declaration
    /// order) so the argmin is deterministic.
    fn worse_first(&self, other: &Self) -> std::cmp::Ordering {
        let weakness =
            |a: Option<ClaimAuthority>| std::cmp::Reverse(a.map_or(u8::MAX, |a| a as u8));
        self.tier
            .cmp(&other.tier)
            .then_with(|| {
                if self.tier == CellTier::Calibrated {
                    self.measured.total_cmp(&other.measured)
                } else {
                    std::cmp::Ordering::Equal
                }
            })
            .then_with(|| self.share.total_cmp(&other.share))
            .then_with(|| weakness(self.authority).cmp(&weakness(other.authority)))
            .then_with(|| self.service.cmp(other.service))
            .then_with(|| self.op_family.cmp(other.op_family))
            .then_with(|| self.kind.cmp(&other.kind))
    }
}

/// The weakest authority among `ops`, or `None` (UNKNOWN) when there are no
/// ops or any op has no authority.
fn weakest_authority(
    mut authorities: impl Iterator<Item = Option<ClaimAuthority>>,
) -> Option<ClaimAuthority> {
    let first = authorities.next()??;
    authorities.try_fold(first, |acc, a| a.map(|a| acc.max(a)))
}

/// The substitution block a world-derived verdict carries: which world, which
/// clone release, the operations relied on, the conformance band, τ with its
/// argmin and oracle share, β, the scorecard it was computed from, and the
/// firewall verdict.
///
/// # One constructor: `compute`
///
/// Plan D4 resolves that τ is **computed, never stamped**: the only way to
/// build a block is [`SubstitutionBlock::compute`], a pure function (no I/O)
/// over a [`SubstitutionInput`]. It lives in this crate, not in coord,
/// because coord cannot add an inherent method to a type defined here
/// (E0116); plan Phase 6's coord module loads the conformance snapshot and
/// the firewall verdict and calls it. Every field is private, there is no
/// `new`, no `Default` and no builder, and read access is through accessors.
/// A struct literal or a struct-update expression outside this module does
/// not compile:
///
/// ```compile_fail,E0451
/// use qontinui_types::cloud_twin::SubstitutionBlock;
/// fn restamp(b: SubstitutionBlock) -> SubstitutionBlock {
///     // E0451: fields of `SubstitutionBlock` are private.
///     SubstitutionBlock { world_id: String::from("w"), ..b }
/// }
/// ```
///
/// It still derives `Serialize`/`Deserialize`, because coord ingests a block
/// and re-serializes it (the rehearsal table's `substitution` column, the
/// `DriftVerdict` it rides in). Deserialization is a transport path, not a
/// computation path, and it **refuses** a block that `compute` could not have
/// produced: τ's and β's invariants (see [`Tau`], [`Beta`]), the band against
/// the exercised operations, and the firewall / unreleased-clone states
/// against the fields they are derived from.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(try_from = "SubstitutionBlockWire")]
pub struct SubstitutionBlock {
    world_id: String,
    /// Absent when the world was built from an unreleased clone build.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    clone_release: Option<String>,
    exercised_ops: Vec<ExercisedOp>,
    /// The weakest ledger authority among the exercised operations. Absent
    /// when there are none or any of them has no authority (UNKNOWN, never a
    /// default band).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    band: Option<ClaimAuthority>,
    tau: Tau,
    beta: Beta,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    scorecard_sha: Option<String>,
    firewall: FirewallStatus,
}

#[derive(Deserialize, JsonSchema)]
struct SubstitutionBlockWire {
    world_id: String,
    #[serde(default)]
    clone_release: Option<String>,
    exercised_ops: Vec<ExercisedOp>,
    #[serde(default)]
    band: Option<ClaimAuthority>,
    tau: Tau,
    beta: Beta,
    #[serde(default)]
    scorecard_sha: Option<String>,
    firewall: FirewallStatus,
}

impl TryFrom<SubstitutionBlockWire> for SubstitutionBlock {
    type Error = ContractViolation;
    fn try_from(w: SubstitutionBlockWire) -> Result<Self, Self::Error> {
        let state = w.tau.state();
        if (w.firewall == FirewallStatus::Violated) != (state == TauState::FirewallViolated) {
            return Err(ContractViolation::new(
                "firewall violated if and only if tau.state is firewall_violated",
            ));
        }
        if state == TauState::UnreleasedClone && w.clone_release.is_some() {
            return Err(ContractViolation::new(
                "tau.state unreleased_clone with a clone_release",
            ));
        }
        if w.clone_release.is_none()
            && !matches!(
                state,
                TauState::UnreleasedClone | TauState::FirewallViolated
            )
        {
            return Err(ContractViolation::new(
                "no clone_release requires tau.state unreleased_clone (or firewall_violated)",
            ));
        }
        if w.band != weakest_authority(w.exercised_ops.iter().map(|o| o.authority)) {
            return Err(ContractViolation::new(
                "band must be the weakest authority among the exercised operations",
            ));
        }
        Ok(Self {
            world_id: w.world_id,
            clone_release: w.clone_release,
            exercised_ops: w.exercised_ops,
            band: w.band,
            tau: w.tau,
            beta: w.beta,
            scorecard_sha: w.scorecard_sha,
            firewall: w.firewall,
        })
    }
}

impl SubstitutionBlock {
    /// Compute the substitution block — the only constructor (plan D4; plan
    /// Phase 6 step 3).
    ///
    /// Each exercised operation is expanded into one *cell* per relevant
    /// claim kind, keyed `(service, op_family, claim_kind)` and matched to the
    /// snapshot's [`ConformanceRow`]s. A cell is:
    /// - **unknown** — the operation's verdict is `unsupported`, or no row
    ///   matches;
    /// - **contradicted** — the verdict is `contradicted`, or the row has
    ///   `contradictions_open > 0`;
    /// - **calibrated** — the row's `measured_conformance` is a finite number
    ///   in `[0, 1]` and `oracle_cases ≥ min_cases`;
    /// - **uncalibrated** — otherwise.
    ///
    /// When several rows match a cell, the worst one is used.
    ///
    /// `tau.state` and `tau.value` follow this **precedence** (the first that
    /// applies wins):
    /// 1. firewall `violated` → `firewall_violated`, value 0;
    /// 2. no `clone_release` → `unreleased_clone`, null;
    /// 3. any unknown cell, or any exercised operation with no claim kinds →
    ///    `unknown_op`, null;
    /// 4. any contradicted cell → `contradicted`, value 0;
    /// 5. every cell calibrated → `calibrated`, value = the minimum measured
    ///    conformance (never an average over breadth);
    /// 6. otherwise → `uncalibrated`, null (also when nothing was exercised).
    ///
    /// Whatever the state, `argmin` is the worst cell under a total order —
    /// tier (in the precedence order above), then measured conformance, then
    /// oracle share, then weaker authority, then service and family
    /// (lexical) and claim kind (declaration order) — and `oracle_share` and
    /// `n` are that cell's row's `oracle_verified_share` and `oracle_cases`
    /// (0 with no row). `band` is the weakest exercised authority.
    ///
    /// Numeric inputs that are not finite numbers in `[0, 1]` (a measured
    /// conformance, an oracle share, a measured β) are treated as *not
    /// measured* — never clamped — so the result always satisfies the
    /// deserialization invariants. A firewall verdict of `unknown` does not
    /// by itself change τ; it is carried for the reader.
    pub fn compute(input: SubstitutionInput<'_>) -> SubstitutionBlock {
        let rows: Vec<&ConformanceRow> = input
            .conformance
            .iter()
            .filter(|r| input.scorecard_sha.is_none_or(|sha| r.scorecard_sha == sha))
            .filter(|r| input.clone_release.is_none_or(|rel| r.clone_release == rel))
            .collect();

        let mut cells: Vec<Cell<'_>> = Vec::new();
        let mut kindless_op = false;
        for op in input.exercised_ops {
            if op.claim_kinds.is_empty() {
                kindless_op = true;
            }
            for &kind in op.claim_kinds {
                let base = Cell {
                    service: op.service,
                    op_family: op.op_family,
                    kind,
                    tier: CellTier::Unknown,
                    measured: 0.0,
                    share: 0.0,
                    cases: 0,
                    authority: op.authority,
                };
                let candidates = rows
                    .iter()
                    .filter(|r| {
                        r.service == op.service
                            && r.op_family == op.op_family
                            && r.claim_kind == kind
                    })
                    .map(|r| {
                        let measured = unit_or_none(r.measured_conformance);
                        let tier = if op.verdict == ClaimVerdict::Unsupported {
                            CellTier::Unknown
                        } else if op.verdict == ClaimVerdict::Contradicted
                            || r.contradictions_open > 0
                        {
                            CellTier::Contradicted
                        } else if measured.is_some() && r.oracle_cases >= input.min_cases {
                            CellTier::Calibrated
                        } else {
                            CellTier::Uncalibrated
                        };
                        Cell {
                            tier,
                            measured: measured.unwrap_or(0.0),
                            share: unit_or_none(Some(r.oracle_verified_share)).unwrap_or(0.0),
                            cases: r.oracle_cases,
                            ..base
                        }
                    });
                cells.push(candidates.min_by(|a, b| a.worse_first(b)).unwrap_or(base));
            }
        }

        let worst = cells.iter().min_by(|a, b| a.worse_first(b)).copied();
        let (state, value) = if input.firewall == FirewallStatus::Violated {
            (TauState::FirewallViolated, Some(0.0))
        } else if input.clone_release.is_none() {
            (TauState::UnreleasedClone, None)
        } else if kindless_op || worst.is_some_and(|c| c.tier == CellTier::Unknown) {
            (TauState::UnknownOp, None)
        } else {
            match worst {
                Some(c) if c.tier == CellTier::Contradicted => (TauState::Contradicted, Some(0.0)),
                Some(c) if c.tier == CellTier::Calibrated => {
                    (TauState::Calibrated, Some(c.measured))
                }
                _ => (TauState::Uncalibrated, None),
            }
        };
        let argmin = worst.map(|c| TauArgmin {
            service: c.service.to_owned(),
            op_family: c.op_family.to_owned(),
            claim_kind: c.kind,
        });
        let tau = Tau::checked(
            value,
            state,
            worst.map_or(0, |c| c.cases),
            argmin,
            worst.map_or(0.0, |c| c.share),
        )
        .expect("compute upholds tau's invariants by construction");

        let beta = match input.beta {
            BetaInput::Hcl => Beta {
                value: Some(1.0),
                basis: BetaBasis::Hcl,
            },
            BetaInput::TwinSeed { measured } => Beta {
                value: unit_or_none(measured),
                basis: BetaBasis::TwinSeed,
            },
            BetaInput::Snapshot => Beta {
                value: None,
                basis: BetaBasis::Snapshot,
            },
        };

        SubstitutionBlock {
            world_id: input.world_id.to_owned(),
            clone_release: input.clone_release.map(str::to_owned),
            exercised_ops: input
                .exercised_ops
                .iter()
                .map(|o| ExercisedOp {
                    service: o.service.to_owned(),
                    operation: o.operation.to_owned(),
                    authority: o.authority,
                    verdict: o.verdict,
                })
                .collect(),
            band: weakest_authority(input.exercised_ops.iter().map(|o| o.authority)),
            tau,
            beta,
            scorecard_sha: input.scorecard_sha.map(str::to_owned),
            firewall: input.firewall,
        }
    }

    pub fn world_id(&self) -> &str {
        &self.world_id
    }
    pub fn clone_release(&self) -> Option<&str> {
        self.clone_release.as_deref()
    }
    pub fn exercised_ops(&self) -> &[ExercisedOp] {
        &self.exercised_ops
    }
    pub fn band(&self) -> Option<ClaimAuthority> {
        self.band
    }
    pub fn tau(&self) -> &Tau {
        &self.tau
    }
    pub fn beta(&self) -> &Beta {
        &self.beta
    }
    pub fn scorecard_sha(&self) -> Option<&str> {
        self.scorecard_sha.as_deref()
    }
    pub fn firewall(&self) -> FirewallStatus {
        self.firewall
    }
}

// ============================================================================
// Rehearsal, conformance and vendor drift
// ============================================================================

/// Which door a rehearsal record came through (plan D5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RehearsalProvenance {
    /// A session that ran the harness itself (`coord_record_rehearsal`).
    Session,
    /// The credential-free PR check (never reaches coord in practice).
    PrCheck,
    /// The trusted push-to-main job — the verdict of record.
    TrustedCi,
    /// An on-demand coord dispatch (`coord_rehearse`).
    Dispatch,
}

/// The rehearsal harness's record of one rehearsal (plan Phase 8). The harness
/// classifies; coord builds the `DriftVerdict` envelope and attaches the
/// [`SubstitutionBlock`] at record time — neither is carried here.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RehearsalRecord {
    pub subject_repo: String,
    pub subject_ref: String,
    pub subject_sha: String,
    /// The `Session-Id` behind the subject change (read by the firewall).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject_session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject_work_unit: Option<String>,
    /// The base the world was forked from (template, snapshot or seed ref).
    pub base: String,
    pub world_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clone_release: Option<String>,
    /// coord's `CanonicalDriftClass` token (`none`, `benign_add`, `pending`,
    /// `in_place`, `active_negation`, `divergent`, `unknown`), mapped by plan
    /// D8. Carried as a string because coord owns that vocabulary.
    pub drift_class: String,
    /// Covered scope in `[0, 1]`.
    pub coverage: f64,
    #[serde(default)]
    pub exercised_ops: Vec<OperationRef>,
    /// Resource types or operations the world does not build — the
    /// "UNKNOWN: non-rehearsable types" annotation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub non_rehearsable: Vec<String>,
    /// Predicted Δ (from `terraform show -json`).
    pub predicted_delta: serde_json::Value,
    /// Observed Δ (S0 → S1 state diff plus the window's audit events).
    pub observed_delta: serde_json::Value,
    pub active_negation: bool,
    pub replan_clean: bool,
    pub provenance: RehearsalProvenance,
    pub ran_at: String,
}

/// A service's compatibility level (clone plan D5).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
pub enum CompatLevel {
    L0,
    L1,
    L2,
    L3,
}

/// A count of ledger claims at one authority and verdict.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ClaimTally {
    pub authority: ClaimAuthority,
    pub verdict: ClaimVerdict,
    pub count: u64,
}

/// One scorecard row per `(service, op_family, claim_kind)` (requested
/// amendment A4; coord table `clone_conformance_observations`, plan Phase 6).
/// Breadth is never reported without depth beside it: the authority tallies,
/// oracle share and open contradictions travel with every row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ConformanceRow {
    pub clone_release: String,
    pub scorecard_sha: String,
    pub service: String,
    pub op_family: String,
    pub claim_kind: ClaimKind,
    pub declared_level: CompatLevel,
    #[serde(default)]
    pub claims_by_authority: Vec<ClaimTally>,
    /// Share of claims that are oracle-verified, in `[0, 1]`.
    pub oracle_verified_share: f64,
    /// Usage-weighted oracle conformance. Null unless oracle-measured.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub measured_conformance: Option<f64>,
    /// Oracle cases behind `measured_conformance` (compared to `min_cases`).
    #[serde(default)]
    pub oracle_cases: u64,
    pub contradictions_open: u64,
    pub observed_at: String,
}

/// One vendor change seen by the clone's drift watcher (requested amendment
/// A5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct VendorDriftEvent {
    /// The vendor artifact that changed (a Smithy model, an SDK, a doc page…).
    pub artifact: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    #[serde(default)]
    pub changed_operations: Vec<OperationRef>,
    /// Ledger claims the change reopened.
    #[serde(default)]
    pub reopened_claim_ids: Vec<String>,
    pub detected_at: String,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::DeserializeOwned;
    use serde_json::{json, Value};

    fn round_trip<T>(v: &T) -> T
    where
        T: Serialize + DeserializeOwned + PartialEq + std::fmt::Debug,
    {
        let s = serde_json::to_string(v).unwrap();
        let back: T = serde_json::from_str(&s).unwrap();
        assert_eq!(&back, v);
        back
    }

    fn world() -> World {
        World {
            kind: WorldKind::Fork,
            id: "w-1".into(),
            parent: Some("w-0".into()),
            clone_release: Some("qcloud-0.3.0".into()),
            data_class: DataClass::Synthetic,
        }
    }

    fn attribution() -> QontinuiAttribution {
        QontinuiAttribution {
            method: AttributionMethod::WorldPrincipal,
            session_id: Some("s".into()),
            agent_id: Some("a".into()),
            work_unit: Some("wu".into()),
            pr: Some("qontinui/qontinui-coord#1".into()),
            world_id: Some("w-1".into()),
            control_plane_lsn: Some("0/16B3748".into()),
            correlation_id: Some("c".into()),
            conformance_claim_ids: vec!["claim-1".into()],
            fault_injected: Some(false),
        }
    }

    fn audit_event() -> CloudAuditEvent {
        CloudAuditEvent {
            event_id: "e".into(),
            event_time: "2026-10-11T00:00:00Z".into(),
            event_source: "rds.amazonaws.com".into(),
            event_name: "ModifyDBInstance".into(),
            aws_region: "us-east-1".into(),
            recipient_account_id: "000000000000".into(),
            user_identity: AuditPrincipal {
                identity_type: "AssumedRole".into(),
                arn: Some("arn:aws:sts::0:assumed-role/r/s".into()),
                principal_id: Some("AROA:s".into()),
                account_id: Some("0".into()),
                session_issuer_arn: Some("arn:aws:iam::0:role/r".into()),
            },
            source_ip_address: Some("10.0.0.1".into()),
            user_agent: Some("aws-sdk-rust app/qontinui-s".into()),
            read_only: Some(false),
            outcome: AuditOutcome::Success,
            error_code: None,
            request_id: Some("r".into()),
            resources: vec![AuditResource {
                arn: Some("arn:aws:rds:us-east-1:0:db:coord".into()),
                resource_type: Some("AWS::RDS::DBInstance".into()),
                account_id: None,
            }],
            request_parameters_digest: Some(
                ParameterDigest::new(DigestAlgorithm::Sha256Jcs, "ab".repeat(32)).unwrap(),
            ),
            qontinui: attribution(),
        }
    }

    fn block_json() -> Value {
        json!({
            "world_id": "w-1",
            "clone_release": "qcloud-0.3.0",
            "exercised_ops": [
                {"service": "rds", "operation": "ModifyDBInstance",
                 "authority": "documentation", "verdict": "unknown"}
            ],
            "band": "documentation",
            "tau": {
                "value": null, "state": "uncalibrated", "n": 0,
                "argmin": {"service": "rds", "op_family": "db-instance", "claim_kind": "semantic"},
                "oracle_share": 0.0
            },
            "beta": {"value": 1.0, "basis": "hcl"},
            "scorecard_sha": "deadbeef",
            "firewall": "ok"
        })
    }

    #[test]
    fn world_and_lifecycle_round_trip() {
        round_trip(&world());
        let events = [
            WorldLifecycleChange::Created {
                world: world(),
                seeded_from: Some(WorldSeedSource::TwinSeed),
                seed_ref: Some("seed-1".into()),
            },
            WorldLifecycleChange::Snapshotted {
                snapshot_id: "snap".into(),
                consistency: SnapshotConsistency::CrashConsistent,
            },
            WorldLifecycleChange::Forked {
                child: world(),
                from_snapshot: None,
            },
            WorldLifecycleChange::Destroyed { reason: None },
        ];
        for change in events {
            let ev = WorldLifecycleEvent {
                world_id: "w-0".into(),
                occurred_at: "2026-10-11T00:00:00Z".into(),
                change,
                ttl_at: Some("2026-10-12T00:00:00Z".into()),
            };
            round_trip(&ev);
        }
        let v = serde_json::to_value(WorldLifecycleEvent {
            world_id: "w".into(),
            occurred_at: "t".into(),
            change: WorldLifecycleChange::Snapshotted {
                snapshot_id: "s".into(),
                consistency: SnapshotConsistency::ControlPlaneOnly,
            },
            ttl_at: None,
        })
        .unwrap();
        assert_eq!(v["change"], "snapshotted");
        assert_eq!(v["consistency"], "control_plane_only");
    }

    #[test]
    fn audit_event_and_rollup_round_trip() {
        round_trip(&audit_event());
        round_trip(&CloudAuditRollup {
            hour_start: "2026-10-11T00:00:00Z".into(),
            principal: "arn:aws:iam::0:role/r".into(),
            session_id: None,
            event_source: "ecs.amazonaws.com".into(),
            event_name: "DescribeServices".into(),
            world_id: None,
            count: 42,
        });
    }

    fn collect_keys(v: &Value, out: &mut Vec<String>) {
        match v {
            Value::Object(m) => {
                for (k, child) in m {
                    out.push(k.clone());
                    collect_keys(child, out);
                }
            }
            Value::Array(a) => a.iter().for_each(|c| collect_keys(c, out)),
            _ => {}
        }
    }

    /// Plan D3: a fully populated event's JSON — and its JSON Schema — name
    /// no field that could carry a request-parameter value.
    #[test]
    fn audit_event_has_no_parameter_value_field() {
        let mut keys = Vec::new();
        collect_keys(&serde_json::to_value(audit_event()).unwrap(), &mut keys);
        let schema = serde_json::to_value(schemars::schema_for!(CloudAuditEvent)).unwrap();
        collect_keys(&schema, &mut keys);
        for k in &keys {
            let lower = k.to_ascii_lowercase();
            let forbidden = [
                "request_parameters",
                "requestparameters",
                "response_elements",
                "responseelements",
                "error_message",
                "errormessage",
                "additional_event_data",
                "additionaleventdata",
            ];
            if forbidden.iter().any(|f| lower.contains(f)) {
                assert_eq!(
                    k, "request_parameters_digest",
                    "forbidden audit field `{k}`"
                );
            }
        }
        assert!(keys.iter().any(|k| k == "request_parameters_digest"));
    }

    /// A producer that sends a value field is refused, not silently projected.
    #[test]
    fn audit_event_refuses_a_parameter_value_field() {
        for (field, value) in [
            ("request_parameters", json!({"masterUserPassword": "x"})),
            ("response_elements", json!({})),
            ("error_message", json!("x")),
        ] {
            let mut v = serde_json::to_value(audit_event()).unwrap();
            v[field] = value;
            let err = serde_json::from_value::<CloudAuditEvent>(v).unwrap_err();
            assert!(err.to_string().contains("unknown field"), "{field}: {err}");
        }
    }

    #[test]
    fn audit_storage_class_follows_d3() {
        let mut e = audit_event();
        assert_eq!(e.storage_class(), AuditStorageClass::FullRow);
        e.read_only = Some(true);
        assert_eq!(e.storage_class(), AuditStorageClass::HourlyRollup);
        e.outcome = AuditOutcome::Denied;
        assert_eq!(e.storage_class(), AuditStorageClass::FullRow);
        e.outcome = AuditOutcome::Success;
        e.read_only = None;
        assert_eq!(e.storage_class(), AuditStorageClass::FullRow);
    }

    #[test]
    fn substitution_block_round_trips_and_reads_through_accessors() {
        let b: SubstitutionBlock = serde_json::from_value(block_json()).unwrap();
        let b = round_trip(&b);
        assert_eq!(serde_json::to_value(&b).unwrap(), block_json());
        assert_eq!(b.world_id(), "w-1");
        assert_eq!(b.clone_release(), Some("qcloud-0.3.0"));
        assert_eq!(b.exercised_ops().len(), 1);
        assert_eq!(b.band(), Some(ClaimAuthority::Documentation));
        assert_eq!(b.band().map(ClaimAuthority::band_letter), Some('D'));
        assert_eq!(b.tau().value(), None);
        assert_eq!(b.tau().state(), TauState::Uncalibrated);
        assert_eq!(b.tau().n(), 0);
        assert_eq!(b.tau().argmin().unwrap().claim_kind, ClaimKind::Semantic);
        assert_eq!(b.tau().oracle_share(), 0.0);
        assert_eq!(b.beta().value(), Some(1.0));
        assert_eq!(b.beta().basis(), BetaBasis::Hcl);
        assert_eq!(b.scorecard_sha(), Some("deadbeef"));
        assert_eq!(b.firewall(), FirewallStatus::Ok);
    }

    #[test]
    fn authority_order_makes_weakest_the_max() {
        let weakest = [
            ClaimAuthority::VendorSdkAcceptance,
            ClaimAuthority::Documentation,
            ClaimAuthority::RecordedRealService,
        ]
        .into_iter()
        .max();
        assert_eq!(weakest, Some(ClaimAuthority::Documentation));
    }

    #[test]
    fn rehearsal_conformance_and_vendor_drift_round_trip() {
        round_trip(&RehearsalRecord {
            subject_repo: "qontinui/qontinui-stack".into(),
            subject_ref: "refs/heads/main".into(),
            subject_sha: "abc".into(),
            subject_session_id: Some("s".into()),
            subject_work_unit: None,
            base: "snap-0".into(),
            world_id: "w-1".into(),
            clone_release: Some("qcloud-0.3.0".into()),
            drift_class: "benign_add".into(),
            coverage: 0.75,
            exercised_ops: vec![OperationRef {
                service: "rds".into(),
                operation: "ModifyDBInstance".into(),
            }],
            non_rehearsable: vec!["aws_budgets_budget".into()],
            predicted_delta: json!({"changes": 1}),
            observed_delta: json!({"changes": 2}),
            active_negation: false,
            replan_clean: true,
            provenance: RehearsalProvenance::TrustedCi,
            ran_at: "2026-10-11T00:00:00Z".into(),
        });
        let row = ConformanceRow {
            clone_release: "qcloud-0.3.0".into(),
            scorecard_sha: "deadbeef".into(),
            service: "iam".into(),
            op_family: "role-policy".into(),
            claim_kind: ClaimKind::Auth,
            declared_level: CompatLevel::L1,
            claims_by_authority: vec![ClaimTally {
                authority: ClaimAuthority::Documentation,
                verdict: ClaimVerdict::Confirmed,
                count: 3,
            }],
            oracle_verified_share: 0.0,
            measured_conformance: None,
            oracle_cases: 0,
            contradictions_open: 1,
            observed_at: "2026-10-11T00:00:00Z".into(),
        };
        round_trip(&row);
        assert_eq!(serde_json::to_value(&row).unwrap()["declared_level"], "L1");
        round_trip(&VendorDriftEvent {
            artifact: "smithy:aws-models/ecs".into(),
            version: Some("2026-10-01".into()),
            commit: None,
            changed_operations: vec![OperationRef {
                service: "ecs".into(),
                operation: "DescribeServices".into(),
            }],
            reopened_claim_ids: vec!["claim-9".into()],
            detected_at: "2026-10-11T00:00:00Z".into(),
        });
    }

    #[test]
    fn attribution_method_tokens() {
        assert_eq!(
            serde_json::to_value(AttributionMethod::UaAppId).unwrap(),
            "ua_app_id"
        );
        assert_eq!(
            serde_json::to_value(AttributionMethod::None).unwrap(),
            "none"
        );
        assert_eq!(
            serde_json::to_value(AttributionMethod::OracleRecorder).unwrap(),
            "oracle_recorder"
        );
    }

    // ---------------------------------------------------------------- M3

    #[test]
    fn parameter_digest_refuses_a_non_digest() {
        let ok = "0123456789abcdef".repeat(4);
        assert!(ParameterDigest::new(DigestAlgorithm::Sha256Jcs, ok.clone()).is_ok());
        for bad in [
            "not-a-digest".to_string(),
            ok.to_uppercase(),
            ok[..63].to_string(),
            format!("{ok}0"),
            format!("{}g", &ok[..63]),
        ] {
            assert!(
                ParameterDigest::new(DigestAlgorithm::Sha256Jcs, bad.clone()).is_err(),
                "{bad}"
            );
            let mut v = serde_json::to_value(audit_event()).unwrap();
            v["request_parameters_digest"]["hex"] = json!(bad);
            let err = serde_json::from_value::<CloudAuditEvent>(v).unwrap_err();
            assert!(err.to_string().contains("64 lowercase hex"), "{err}");
        }
    }

    // ---------------------------------------------------------------- H1

    const REL: &str = "qcloud-0.3.0";
    const SHA: &str = "sc1";

    fn row(
        service: &str,
        family: &str,
        kind: ClaimKind,
        measured: Option<f64>,
        cases: u64,
        share: f64,
        contradictions: u64,
    ) -> ConformanceRow {
        ConformanceRow {
            clone_release: REL.into(),
            scorecard_sha: SHA.into(),
            service: service.into(),
            op_family: family.into(),
            claim_kind: kind,
            declared_level: CompatLevel::L1,
            claims_by_authority: vec![],
            oracle_verified_share: share,
            measured_conformance: measured,
            oracle_cases: cases,
            contradictions_open: contradictions,
            observed_at: "2026-10-11T00:00:00Z".into(),
        }
    }

    const RDS_KINDS: &[ClaimKind] = &[ClaimKind::Semantic, ClaimKind::Error];
    const ECS_KINDS: &[ClaimKind] = &[ClaimKind::Shape];

    fn rds_op(verdict: ClaimVerdict) -> ExercisedOpInput<'static> {
        ExercisedOpInput {
            service: "rds",
            operation: "ModifyDBInstance",
            op_family: "db-instance",
            authority: Some(ClaimAuthority::RecordedRealService),
            verdict,
            claim_kinds: RDS_KINDS,
        }
    }

    fn ecs_op(
        authority: Option<ClaimAuthority>,
        verdict: ClaimVerdict,
    ) -> ExercisedOpInput<'static> {
        ExercisedOpInput {
            service: "ecs",
            operation: "DescribeServices",
            op_family: "service",
            authority,
            verdict,
            claim_kinds: ECS_KINDS,
        }
    }

    /// The all-oracle-measured snapshot.
    fn measured_rows() -> Vec<ConformanceRow> {
        vec![
            row(
                "rds",
                "db-instance",
                ClaimKind::Semantic,
                Some(0.95),
                40,
                1.0,
                0,
            ),
            row(
                "rds",
                "db-instance",
                ClaimKind::Error,
                Some(0.90),
                35,
                0.8,
                0,
            ),
            row("ecs", "service", ClaimKind::Shape, Some(0.99), 50, 1.0, 0),
        ]
    }

    fn both_ops_ok() -> [ExercisedOpInput<'static>; 2] {
        [
            rds_op(ClaimVerdict::Confirmed),
            ecs_op(
                Some(ClaimAuthority::VendorSdkAcceptance),
                ClaimVerdict::Confirmed,
            ),
        ]
    }

    /// Compute, and require that the result survives its own validation.
    fn compute(
        ops: &[ExercisedOpInput<'_>],
        rows: &[ConformanceRow],
        clone_release: Option<&str>,
        firewall: FirewallStatus,
    ) -> SubstitutionBlock {
        let b = SubstitutionBlock::compute(SubstitutionInput {
            world_id: "w-1",
            clone_release,
            exercised_ops: ops,
            conformance: rows,
            scorecard_sha: Some(SHA),
            min_cases: 30,
            beta: BetaInput::Hcl,
            firewall,
        });
        round_trip(&b)
    }

    fn assert_tau(
        b: &SubstitutionBlock,
        value: Option<f64>,
        state: TauState,
        argmin: (&str, &str, ClaimKind),
        share: f64,
    ) {
        let t = b.tau();
        assert_eq!(t.value(), value);
        assert_eq!(t.state(), state);
        let a = t
            .argmin()
            .expect("argmin is filled when something was exercised");
        assert_eq!(
            (a.service.as_str(), a.op_family.as_str(), a.claim_kind),
            argmin
        );
        assert_eq!(t.oracle_share(), share);
    }

    #[test]
    fn compute_all_oracle_measured_is_calibrated_at_the_minimum() {
        let b = compute(
            &both_ops_ok(),
            &measured_rows(),
            Some(REL),
            FirewallStatus::Ok,
        );
        assert_tau(
            &b,
            Some(0.90),
            TauState::Calibrated,
            ("rds", "db-instance", ClaimKind::Error),
            0.8,
        );
        assert_eq!(b.tau().n(), 35);
        assert_eq!(b.band(), Some(ClaimAuthority::VendorSdkAcceptance));
        assert_eq!(b.scorecard_sha(), Some(SHA));
        assert_eq!(b.beta().value(), Some(1.0));
    }

    #[test]
    fn compute_one_doc_attested_op_is_uncalibrated_prior_only() {
        let mut rows = measured_rows();
        rows[2] = row("ecs", "service", ClaimKind::Shape, None, 0, 0.0, 0);
        let ops = [
            rds_op(ClaimVerdict::Confirmed),
            ecs_op(Some(ClaimAuthority::Documentation), ClaimVerdict::Unknown),
        ];
        let b = compute(&ops, &rows, Some(REL), FirewallStatus::Ok);
        assert_tau(
            &b,
            None,
            TauState::Uncalibrated,
            ("ecs", "service", ClaimKind::Shape),
            0.0,
        );
        assert_eq!(b.band(), Some(ClaimAuthority::Documentation));
        assert_eq!(b.band().map(ClaimAuthority::band_letter), Some('D'));
    }

    #[test]
    fn compute_partial_oracle_coverage_below_min_cases_is_uncalibrated_with_a_share() {
        let mut rows = measured_rows();
        rows[1] = row(
            "rds",
            "db-instance",
            ClaimKind::Error,
            Some(0.90),
            10,
            0.8,
            0,
        );
        let b = compute(&both_ops_ok(), &rows, Some(REL), FirewallStatus::Ok);
        assert_tau(
            &b,
            None,
            TauState::Uncalibrated,
            ("rds", "db-instance", ClaimKind::Error),
            0.8,
        );
        assert_eq!(b.tau().n(), 10);
    }

    #[test]
    fn compute_one_contradicted_is_zero() {
        let mut rows = measured_rows();
        rows[1] = row(
            "rds",
            "db-instance",
            ClaimKind::Error,
            Some(0.90),
            35,
            0.8,
            1,
        );
        let b = compute(&both_ops_ok(), &rows, Some(REL), FirewallStatus::Ok);
        assert_tau(
            &b,
            Some(0.0),
            TauState::Contradicted,
            ("rds", "db-instance", ClaimKind::Error),
            0.8,
        );
        // A contradicted op verdict alone is enough too.
        let b = compute(
            &[rds_op(ClaimVerdict::Contradicted)],
            &measured_rows(),
            Some(REL),
            FirewallStatus::Ok,
        );
        assert_eq!(b.tau().state(), TauState::Contradicted);
        assert_eq!(b.tau().value(), Some(0.0));
    }

    #[test]
    fn compute_one_unsupported_is_unknown_op() {
        let ops = [
            rds_op(ClaimVerdict::Confirmed),
            ecs_op(None, ClaimVerdict::Unsupported),
        ];
        let b = compute(&ops, &measured_rows(), Some(REL), FirewallStatus::Ok);
        assert_tau(
            &b,
            None,
            TauState::UnknownOp,
            ("ecs", "service", ClaimKind::Shape),
            1.0,
        );
        assert_eq!(
            b.band(),
            None,
            "an op with no authority leaves the band UNKNOWN"
        );

        // A (family, kind) with no conformance row is unknown too, share 0.
        let rows: Vec<_> = measured_rows().into_iter().take(2).collect();
        let b = compute(&both_ops_ok(), &rows, Some(REL), FirewallStatus::Ok);
        assert_tau(
            &b,
            None,
            TauState::UnknownOp,
            ("ecs", "service", ClaimKind::Shape),
            0.0,
        );

        // Rows from another scorecard are not read.
        let mut rows = measured_rows();
        rows[2].scorecard_sha = "other".into();
        let b = compute(&both_ops_ok(), &rows, Some(REL), FirewallStatus::Ok);
        assert_eq!(b.tau().state(), TauState::UnknownOp);
    }

    #[test]
    fn compute_firewall_violation_wins_over_everything() {
        let ops = [
            rds_op(ClaimVerdict::Confirmed),
            ecs_op(None, ClaimVerdict::Unsupported),
        ];
        let b = compute(&ops, &measured_rows(), None, FirewallStatus::Violated);
        assert_tau(
            &b,
            Some(0.0),
            TauState::FirewallViolated,
            ("ecs", "service", ClaimKind::Shape),
            1.0,
        );
        assert_eq!(b.firewall(), FirewallStatus::Violated);
    }

    #[test]
    fn compute_unreleased_clone_is_null() {
        let b = compute(&both_ops_ok(), &measured_rows(), None, FirewallStatus::Ok);
        assert_tau(
            &b,
            None,
            TauState::UnreleasedClone,
            ("rds", "db-instance", ClaimKind::Error),
            0.8,
        );
        assert_eq!(b.clone_release(), None);
    }

    #[test]
    fn compute_nothing_exercised_is_uncalibrated_with_no_argmin() {
        let b = compute(&[], &measured_rows(), Some(REL), FirewallStatus::Ok);
        assert_eq!(b.tau().state(), TauState::Uncalibrated);
        assert_eq!(b.tau().value(), None);
        assert!(b.tau().argmin().is_none());
        assert_eq!(b.band(), None);
    }

    #[test]
    fn compute_argmin_ties_break_lexically() {
        let rows = vec![
            row("sqs", "queue", ClaimKind::Shape, None, 0, 0.0, 0),
            row("sns", "topic", ClaimKind::Shape, None, 0, 0.0, 0),
        ];
        let op = |service: &'static str, family: &'static str| ExercisedOpInput {
            service,
            operation: "Op",
            op_family: family,
            authority: Some(ClaimAuthority::Documentation),
            verdict: ClaimVerdict::Unknown,
            claim_kinds: ECS_KINDS,
        };
        let ops = [op("sqs", "queue"), op("sns", "topic")];
        let b = compute(&ops, &rows, Some(REL), FirewallStatus::Ok);
        assert_eq!(b.tau().argmin().unwrap().service, "sns");
    }

    #[test]
    fn compute_beta_follows_its_basis() {
        let beta = |input| {
            SubstitutionBlock::compute(SubstitutionInput {
                world_id: "w",
                clone_release: Some(REL),
                exercised_ops: &[],
                conformance: &[],
                scorecard_sha: None,
                min_cases: 30,
                beta: input,
                firewall: FirewallStatus::Ok,
            })
            .beta()
            .clone()
        };
        assert_eq!(beta(BetaInput::Hcl).value(), Some(1.0));
        assert_eq!(
            beta(BetaInput::TwinSeed {
                measured: Some(0.7)
            })
            .value(),
            Some(0.7)
        );
        assert_eq!(beta(BetaInput::TwinSeed { measured: None }).value(), None);
        assert_eq!(
            beta(BetaInput::TwinSeed {
                measured: Some(f64::NAN)
            })
            .value(),
            None
        );
        assert_eq!(beta(BetaInput::Snapshot).value(), None);
        assert_eq!(beta(BetaInput::Snapshot).basis(), BetaBasis::Snapshot);
    }

    // ---------------------------------------------------------------- M2

    /// Mutate the valid `block_json()` fixture and require a refusal.
    fn refused(mutate: impl FnOnce(&mut Value)) -> String {
        let mut v = block_json();
        assert!(serde_json::from_value::<SubstitutionBlock>(v.clone()).is_ok());
        mutate(&mut v);
        serde_json::from_value::<SubstitutionBlock>(v)
            .expect_err("an invariant-breaking block must be refused")
            .to_string()
    }

    #[test]
    fn tau_refuses_a_value_on_a_null_state() {
        let e = refused(|v| v["tau"]["value"] = json!(0.97));
        assert!(e.contains("null value"), "{e}");
    }

    #[test]
    fn tau_refuses_a_calibrated_state_without_a_value() {
        let e = refused(|v| v["tau"]["state"] = json!("calibrated"));
        assert!(e.contains("requires a value"), "{e}");
    }

    #[test]
    fn tau_refuses_a_non_zero_contradicted_value() {
        let e = refused(|v| {
            v["tau"]["state"] = json!("contradicted");
            v["tau"]["value"] = json!(0.5);
        });
        assert!(e.contains("forces value 0"), "{e}");
    }

    #[test]
    fn tau_refuses_a_non_zero_firewall_violated_value() {
        let e = refused(|v| {
            v["tau"]["state"] = json!("firewall_violated");
            v["tau"]["value"] = json!(0.3);
            v["firewall"] = json!("violated");
        });
        assert!(e.contains("forces value 0"), "{e}");
    }

    #[test]
    fn tau_refuses_a_value_outside_the_unit_interval() {
        let e = refused(|v| {
            v["tau"]["state"] = json!("calibrated");
            v["tau"]["value"] = json!(1.5);
        });
        assert!(e.contains("tau.value"), "{e}");
    }

    #[test]
    fn tau_refuses_a_share_outside_the_unit_interval() {
        let e = refused(|v| v["tau"]["oracle_share"] = json!(-0.1));
        assert!(e.contains("tau.oracle_share"), "{e}");
    }

    /// JSON cannot carry NaN or infinity, so the finiteness rule is pinned on
    /// the shared check both doors (compute and deserialization) go through.
    #[test]
    fn tau_refuses_non_finite_numbers() {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(Tau::checked(Some(bad), TauState::Calibrated, 1, None, 0.0).is_err());
            assert!(Tau::checked(None, TauState::Uncalibrated, 0, None, bad).is_err());
        }
    }

    #[test]
    fn tau_refuses_cases_without_an_argmin() {
        let e = refused(|v| {
            v["tau"]["argmin"] = Value::Null;
            v["tau"]["n"] = json!(5);
        });
        assert!(e.contains("no argmin"), "{e}");
    }

    #[test]
    fn tau_refuses_a_calibrated_value_without_an_argmin() {
        let e = refused(|v| {
            v["tau"]["argmin"] = Value::Null;
            v["tau"]["state"] = json!("calibrated");
            v["tau"]["value"] = json!(0.9);
        });
        assert!(e.contains("argmin"), "{e}");
    }

    #[test]
    fn beta_refuses_hcl_other_than_one() {
        let e = refused(|v| v["beta"]["value"] = json!(0.9));
        assert!(e.contains("hcl"), "{e}");
    }

    #[test]
    fn beta_refuses_a_value_outside_the_unit_interval() {
        let e = refused(|v| v["beta"] = json!({"value": 1.5, "basis": "twin_seed"}));
        assert!(e.contains("beta.value"), "{e}");
    }

    #[test]
    fn block_refuses_a_firewall_mismatch() {
        let e = refused(|v| v["firewall"] = json!("violated"));
        assert!(e.contains("firewall"), "{e}");
        let e = refused(|v| {
            v["tau"]["state"] = json!("firewall_violated");
            v["tau"]["value"] = json!(0.0);
        });
        assert!(e.contains("firewall"), "{e}");
    }

    #[test]
    fn block_refuses_unreleased_clone_with_a_release() {
        let e = refused(|v| v["tau"]["state"] = json!("unreleased_clone"));
        assert!(e.contains("unreleased_clone"), "{e}");
    }

    #[test]
    fn block_refuses_a_missing_release_on_a_released_state() {
        let e = refused(|v| {
            v.as_object_mut().unwrap().remove("clone_release");
        });
        assert!(e.contains("clone_release"), "{e}");
    }

    #[test]
    fn block_refuses_a_band_that_is_not_the_weakest_authority() {
        let e = refused(|v| v["band"] = json!("recorded_real_service"));
        assert!(e.contains("band"), "{e}");
    }
}
