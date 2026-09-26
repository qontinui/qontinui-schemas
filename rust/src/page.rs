//! Bounded reads — ONE page type, ONE wire envelope, ONE keyset cursor codec.
//!
//! Every list-shaped read in the fleet returns a bounded page, and until this
//! module each surface spelled "is this the whole answer?" its own way — or
//! did not spell it at all, so a capped page read as the corpus. This is the
//! Rust half of the shared contract (plan
//! `2026-09-05-every-bounded-read-is-a-page-that-reads-as-a-corpus`, §3 Layer 1
//! and Layer 2; first consumer `coord.findings`, plan
//! `2026-09-05-findings-recent-is-a-window-that-reads-as-a-corpus`, Phase 1
//! step 0).
//!
//! Three pieces:
//!
//! - [`Page<T>`] — the rows plus everything a caller needs to know about what
//!   it is holding: the cap actually applied, a [`Bound`] saying how much of
//!   the match set the page covers, the cursor to the next page (or, for a
//!   relevance-RANKED read that can never page, the door that walks the same
//!   corpus instead), whether the store answered at all, and whether a FILTER
//!   was narrowed. Generic, so it is Rust-only; its wire form is
//!   [`BoundedReadMeta`].
//! - [`BoundedReadMeta`] — the monomorphic Layer-2 envelope keys. Derives
//!   `JsonSchema` so the schemars → TypeScript / Pydantic pipeline can emit it
//!   (the registry is a hand list in qontinui-runner's `schema_export.rs`, which
//!   cannot register a generic — schemars would title it `Page_for_X`). A door
//!   `#[serde(flatten)]`s it beside its own collection key, or merges
//!   [`BoundedReadMeta::insert_into`] into a `json!` object. It never renames
//!   the caller's collection key and never moves `count` / `limit`.
//! - The keyset cursor codec — [`SortKey`], [`CursorScope`],
//!   [`ScopeFingerprint`], [`KeysetPosition`], [`CursorError`]. A versioned,
//!   filter-fingerprinted, opaque urlsafe-base64 token over a
//!   `(timestamp, uuid)` keyset position, decoded STRICTLY.
//!
//! ## Why keyset, and why the key must not move
//!
//! `OFFSET n` over a table peers write continuously shifts every later row by
//! one per concurrent insert, so a caller paging it sees a repeat and silently
//! skips a row. A keyset position is defined relative to the last row SEEN,
//! so inserts ahead of it are simply not in the descending scan.
//!
//! Keyset is immune to INSERT and DELETE. It is **not** immune to an UPDATE
//! that moves an unreached row's sort key across the cursor: that row leaves
//! the walk with no duplicate, no error, and a normally terminating
//! `next_cursor`. coord paid for this once (`2794f010d`, `session_fleet.rs`,
//! whose walk sorted on a heartbeat-driven key). [`SortKey`] is the marker a
//! surface implements to assert its key is immutable; read its doc before
//! implementing it.
//!
//! ## Counterparts — the resemblance is deliberate
//!
//! The codec is the shape three shipped paginators already use, promoted:
//!
//! - coord `crates/coord/src/session_fleet.rs` `CursorPayload` /
//!   `encode_cursor` / `decode_cursor` — the TEMPLATE: a versioned JSON payload
//!   carrying microseconds + id + a SHA-256 fingerprint of the filter scope, so
//!   a cursor replayed under different filters is refused rather than paging a
//!   different population.
//! - coord `crates/coord/src/fleet_health.rs` `AlertsCursor` — the refusal
//!   wording (`"invalid cursor — pass a `next_cursor` from a previous …"`).
//! - qontinui-web `backend/app/api/v1/endpoints/memory.py` `_encode_cursor` /
//!   `_decode_cursor` — the Python keyset cursor over `(created_at, seq)`,
//!   strict parse, 400 on anything malformed.
//!
//! These are independent stores with independent cursors, and a caller never
//! carries a token between them — so if one's encoding changes the others do
//! not have to follow. The byte layout below is specified exactly so a Python
//! implementation of THIS codec (qontinui-web `bounded_read.py`) can decode a
//! Rust-minted token and mint one Rust decodes.
//!
//! ## The token, byte for byte
//!
//! `token = base64url_nopad(json)` where `json` is the compact serde
//! serialization of, in this field order:
//!
//! ```json
//! {"v":1,"s":"<SortKey::ID>","k":<i64 microseconds since the Unix epoch>,
//!  "i":"<lowercase hyphenated uuid>","f":"<64 lowercase hex>"}
//! ```
//!
//! `f` is the [`ScopeFingerprint`]: lowercase hex SHA-256 over
//!
//! ```text
//! "qontinui.bounded-read.cursor" 0x1F  <version byte>  0x1F  <SortKey::ID utf-8>
//! then, per scope field, in the order the surface adds them:
//!   0x1E  <name utf-8>  0x1F  <value>
//! where <value> is one of
//!   'n'                                          absent (None)
//!   's' <u64 BE byte length> <utf-8 bytes>       a string
//!   'u' <16 raw bytes>                           a uuid
//!   'b' <0x00 | 0x01>                            a bool
//!   'i' <i64 BE, 8 bytes>                        an integer
//!   'l' <u64 BE count> (<u64 BE len> <bytes>)*   a string SET — sorted
//!                                                ascending by bytes, deduped
//! ```
//!
//! Every value is tagged and length-prefixed, so no two distinct scopes can
//! serialize to the same byte string (absence can never collide with a
//! sentinel string, and adjacent fields can never run together).
//!
//! The token is opaque **by contract, not by cryptography**: it is not signed
//! and needs no signature. Everything in it is a value the caller could already
//! supply, tenant scoping comes from the verified principal (a surface adds the
//! tenant to the scope so a token cannot cross tenants by accident, not as an
//! authorization boundary), and the fingerprint is an integrity check against
//! accidental cross-scope replay.
//!
//! ## Wire-type conventions
//!
//! The crate convention (dates and uuids are `String`s on the wire) holds:
//! [`BoundedReadMeta`] carries neither. The codec's Rust API takes
//! `chrono::DateTime<Utc>` and `uuid::Uuid` because it is behaviour, not a DTO,
//! and those are the types a keyset row actually has.

use std::fmt;
use std::marker::PhantomData;

use base64::Engine as _;
use chrono::{DateTime, Datelike, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

// ============================================================================
// Page<T> and Bound
// ============================================================================

/// How much of the match set a [`Page`] is known to cover — the piece that
/// lets the contract be honest about its own cost.
///
/// A window count (`COUNT(*) OVER ()`) gives an exact total and scans the whole
/// match set; a `limit + 1` probe is nearly free and says only "more exists".
/// Both are legitimate, and which one ran is a fact the caller is entitled to.
///
/// All four are relative to the page's START POSITION: on the first page that
/// is the whole match set; on a later page it is the rows at and after the
/// cursor. That is the only reading under which a keyset walk can report
/// `truncated: false` on its last page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bound {
    /// A window count ran: exactly this many rows match from this page's start
    /// position onward (the whole match set on the first page).
    Exact(i64),
    /// A `limit + 1` probe fired: at least this many rows match from this
    /// page's start position onward, so more exist beyond the page.
    AtLeast(i64),
    /// The page holds every matching row from its start position onward.
    Complete,
    /// Whether more rows exist did not resolve — the store did not answer, or
    /// the count did not decode. **Never renders as complete**: its
    /// `truncated` is `null`, not `false`, because `false` asserts "you have
    /// seen everything" on a read that could not say.
    Unknown,
}

impl Bound {
    /// The wire discriminant served as `bound_kind`.
    pub fn kind(self) -> BoundKind {
        match self {
            Bound::Exact(_) => BoundKind::Exact,
            Bound::AtLeast(_) => BoundKind::AtLeast,
            Bound::Complete => BoundKind::Complete,
            Bound::Unknown => BoundKind::Unknown,
        }
    }
}

/// A FILTER the surface narrowed before running the read — reported, never
/// applied silently, because a narrowed filter that reads as a complete answer
/// is the same defect class as a silent empty.
///
/// Promoted from `coord.findings`' `resource_keys_truncated`, the only
/// narrowing any surface reported when this type was written: a list-valued
/// filter truncated at a cap rather than refused (truncating a FILTER only
/// narrows a read). Minimal on purpose — which parameter, how many values
/// were actually applied, and the cap that dropped the rest. A caller seeing
/// it knows a miss is meaningless for the values beyond the cap.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FilterNarrowing {
    /// The query parameter that was narrowed, spelled as the caller spelled it
    /// on the wire (e.g. `resource_keys`).
    pub parameter: String,
    /// How many values of it the read actually applied.
    pub applied: i64,
    /// The cap the surface enforces; values past it were dropped.
    pub cap: i64,
}

/// One bounded page of rows, plus what the caller is holding.
///
/// Fields are private and every constructor keeps the invariants the envelope
/// promises, so a page that contradicts itself cannot be built:
///
/// - `next_cursor` is present only when `truncated` is `true` — never beside
///   `false` or `null`;
/// - `truncated: true` with no `next_cursor` happens ONLY on a ranked read
///   ([`Page::not_pageable`]), and then `enumerate_via` names the door that
///   walks the corpus instead;
/// - a page with a `next_cursor` has no `enumerate_via` (it IS the walk);
/// - `Bound::Unknown` never reports `truncated: false`.
///
/// Making the bad states unrepresentable (private fields, four constructors
/// that each set a consistent tuple) is the enforcement; [`Page::meta`] also
/// `debug_assert!`s the invariant, so a future constructor that breaks it fails
/// every debug test run rather than shipping a self-contradicting envelope.
#[derive(Debug, Clone, PartialEq)]
pub struct Page<T> {
    rows: Vec<T>,
    effective_limit: i64,
    bound: Bound,
    next_cursor: Option<String>,
    available: bool,
    filter_narrowed: Option<FilterNarrowing>,
    enumerate_via: Option<String>,
}

impl<T> Page<T> {
    /// Build a page from a `LIMIT effective_limit + 1` fetch — the ONLY way to
    /// get an exact `truncated` from a probe, and the arithmetic callers get
    /// wrong when they spell it themselves.
    ///
    /// `rows` is everything the statement returned, in walk order. When it
    /// holds more than `effective_limit` rows the extra row is the probe: it is
    /// dropped, the bound is [`Bound::AtLeast`]`(effective_limit + 1)`, and
    /// `cursor_of` mints `next_cursor` from the LAST KEPT row (never from the
    /// probe row — the next page starts strictly after the last row the caller
    /// saw). Otherwise the page is [`Bound::Complete`] with no cursor, even when
    /// it filled exactly: a page that fills with nothing behind it is not
    /// truncated, and saying so would cost every paging caller a guaranteed
    /// empty round trip.
    ///
    /// The probe must be counted by the SAME statement that applies every
    /// filter, or a filtered-out probe row reports a false `truncated`.
    ///
    /// `effective_limit` is the cap the statement actually applied, and must be
    /// `>= 1`; a smaller value is raised to `1` here, because a zero-row page
    /// cannot address a next position and would have to report `truncated`
    /// with no cursor.
    pub fn from_probe(
        mut rows: Vec<T>,
        effective_limit: i64,
        cursor_of: impl FnOnce(&T) -> String,
    ) -> Self {
        let effective_limit = effective_limit.max(1);
        let keep = usize::try_from(effective_limit).unwrap_or(usize::MAX);
        if rows.len() > keep {
            rows.truncate(keep);
            let next_cursor = rows.last().map(cursor_of);
            Page {
                rows,
                effective_limit,
                bound: Bound::AtLeast(effective_limit.saturating_add(1)),
                next_cursor,
                available: true,
                filter_narrowed: None,
                enumerate_via: None,
            }
        } else {
            Page::complete(rows, effective_limit)
        }
    }

    /// Build a page of a RANKED read (relevance order, an RRF top-N — e.g.
    /// `POST /memory/query`) from a `LIMIT effective_limit + 1` fetch. A
    /// ranking has no stable position to resume from, so it can never hand out
    /// a cursor: when the probe row arrived the page is honestly
    /// `truncated: true, next_cursor: null`, and `enumerate_via` names the door
    /// that walks the same corpus by an immutable key (see [`SortKey`]) — the
    /// only way a caller can get past the top N. Set on every page of the read,
    /// truncated or not, because it describes the READ, not this page.
    ///
    /// Same probe arithmetic, and the same `effective_limit >= 1` floor, as
    /// [`Page::from_probe`].
    pub fn not_pageable(
        mut rows: Vec<T>,
        effective_limit: i64,
        enumerate_via: impl Into<String>,
    ) -> Self {
        let effective_limit = effective_limit.max(1);
        let keep = usize::try_from(effective_limit).unwrap_or(usize::MAX);
        let bound = if rows.len() > keep {
            rows.truncate(keep);
            Bound::AtLeast(effective_limit.saturating_add(1))
        } else {
            Bound::Complete
        };
        Page {
            rows,
            effective_limit,
            bound,
            next_cursor: None,
            available: true,
            filter_narrowed: None,
            enumerate_via: Some(enumerate_via.into()),
        }
    }

    /// Build a page whose statement carried an exact window count
    /// (`COUNT(*) OVER ()`) of the rows matching from this page's start
    /// position onward. `truncated` is `total > rows.len()`, and `cursor_of`
    /// mints the next cursor from the last row only then.
    ///
    /// A count that claims more rows than an EMPTY page holds is
    /// self-contradictory — the window count rides on the returned rows, so
    /// with none there was nothing to carry it — and there is no row to
    /// address a next position from. That page is [`Bound::Unknown`], never a
    /// `truncated: true` with no way forward.
    pub fn from_window_count(
        rows: Vec<T>,
        effective_limit: i64,
        total: i64,
        cursor_of: impl FnOnce(&T) -> String,
    ) -> Self {
        let truncated = total > rows.len() as i64;
        let (bound, next_cursor) = match (truncated, rows.last()) {
            (true, Some(last)) => (Bound::Exact(total), Some(cursor_of(last))),
            (true, None) => (Bound::Unknown, None),
            (false, _) => (Bound::Exact(total), None),
        };
        Page {
            rows,
            effective_limit,
            bound,
            next_cursor,
            available: true,
            filter_narrowed: None,
            enumerate_via: None,
        }
    }

    /// A page that holds every matching row by construction — e.g. a by-id
    /// read, which returns 0 or 1 row and has no next page.
    pub fn complete(rows: Vec<T>, effective_limit: i64) -> Self {
        Page {
            rows,
            effective_limit,
            bound: Bound::Complete,
            next_cursor: None,
            available: true,
            filter_narrowed: None,
            enumerate_via: None,
        }
    }

    /// The store is not provisioned (missing table or column): no rows, and
    /// the bound is [`Bound::Unknown`] — an empty page here is UNKNOWN, never
    /// "nothing matched".
    pub fn unavailable(effective_limit: i64) -> Self {
        Page {
            rows: Vec::new(),
            effective_limit,
            bound: Bound::Unknown,
            next_cursor: None,
            available: false,
            filter_narrowed: None,
            enumerate_via: None,
        }
    }

    /// Record that a FILTER was narrowed before the read ran (see
    /// [`FilterNarrowing`]). `None` leaves the page unchanged.
    pub fn with_filter_narrowed(mut self, narrowing: Option<FilterNarrowing>) -> Self {
        self.filter_narrowed = narrowing;
        self
    }

    /// The rows, in walk order, at most `effective_limit` of them.
    pub fn rows(&self) -> &[T] {
        &self.rows
    }

    /// Take the rows out of the page.
    pub fn into_rows(self) -> Vec<T> {
        self.rows
    }

    /// The cap ACTUALLY applied — not the one requested. A `limit` in the
    /// envelope that disagreed with the statement's `LIMIT` would misreport the
    /// very boundary `truncated` is about.
    pub fn effective_limit(&self) -> i64 {
        self.effective_limit
    }

    /// See [`Bound`].
    pub fn bound(&self) -> Bound {
        self.bound
    }

    /// The opaque token for the following page. `Some` only when
    /// [`Self::truncated`] is `Some(true)` — and then always, unless the read is
    /// a ranking ([`Self::enumerate_via`]).
    pub fn next_cursor(&self) -> Option<&str> {
        self.next_cursor.as_deref()
    }

    /// For a RANKED read that cannot be paged ([`Page::not_pageable`]): the
    /// door that walks the same corpus by an immutable key. `None` on a walk.
    pub fn enumerate_via(&self) -> Option<&str> {
        self.enumerate_via.as_deref()
    }

    /// The envelope invariant (see the type doc). Private: the constructors
    /// make a violation unrepresentable, and [`Self::meta`] asserts it.
    fn envelope_is_consistent(&self) -> bool {
        let cursor = self.next_cursor.is_some();
        let via = self.enumerate_via.is_some();
        match (self.truncated(), cursor, via) {
            // A cursor IS the walk, so it never names another one.
            (_, true, true) => false,
            // A cursor only ever sits beside `truncated: true`.
            (truncated, true, false) => truncated == Some(true),
            // Truncated with no way forward is only honest for a ranking
            // that names the door which walks the corpus.
            (Some(true), false, via) => via,
            _ => true,
        }
    }

    /// `false` ONLY when the store is unprovisioned.
    pub fn available(&self) -> bool {
        self.available
    }

    /// See [`FilterNarrowing`].
    pub fn filter_narrowed(&self) -> Option<&FilterNarrowing> {
        self.filter_narrowed.as_ref()
    }

    /// Whether matching rows exist beyond this page. `None` for
    /// [`Bound::Unknown`] — a page that could not observe the match set is not
    /// entitled to claim either way.
    pub fn truncated(&self) -> Option<bool> {
        match self.bound {
            Bound::Exact(total) => Some(total > self.rows.len() as i64),
            Bound::AtLeast(_) => Some(true),
            Bound::Complete => Some(false),
            Bound::Unknown => None,
        }
    }

    /// The Layer-2 wire envelope keys for this page.
    pub fn meta(&self) -> BoundedReadMeta {
        debug_assert!(
            self.envelope_is_consistent(),
            "Page envelope invariant broken: truncated={:?} next_cursor={} enumerate_via={:?}",
            self.truncated(),
            self.next_cursor.is_some(),
            self.enumerate_via
        );
        let shown = self.rows.len() as i64;
        let truncated = self.truncated();
        BoundedReadMeta {
            count: shown,
            limit: self.effective_limit,
            shown,
            total: match self.bound {
                Bound::Exact(total) => Some(total),
                Bound::AtLeast(_) | Bound::Complete | Bound::Unknown => None,
            },
            truncated,
            bound_kind: self.bound.kind(),
            // The constructors already keep this true; restated here so the
            // wire can never carry a cursor beside `truncated: false`.
            next_cursor: if truncated == Some(false) {
                None
            } else {
                self.next_cursor.clone()
            },
            available: self.available,
            filter_narrowed: self.filter_narrowed.clone(),
            enumerate_via: self.enumerate_via.clone(),
        }
    }
}

// ============================================================================
// BoundedReadMeta — the Layer-2 wire envelope
// ============================================================================

/// The wire discriminant of a [`Bound`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BoundKind {
    /// A window count ran; `total` is a number.
    Exact,
    /// A `limit + 1` probe fired; more rows exist, `total` is `null`.
    AtLeast,
    /// The page holds every matching row from its start position.
    Complete,
    /// Did not resolve; `truncated` and `total` are `null`.
    Unknown,
}

/// The schema of `Option<T>` — `T` or `null` — for a field that is also
/// `#[schemars(required)]`. With `required` alone schemars 1 emits `T`'s bare
/// schema and the `null` this wire sends vanishes from the contract; this keeps
/// both: the key must be present, and its value may be `null` (a scalar gets
/// `"type": [..., "null"]`, a struct `anyOf: [{"$ref": ...}, {"type": "null"}]`).
fn nullable<T: JsonSchema>(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
    generator.subschema_for::<Option<T>>()
}

/// The envelope keys every bounded read serves beside its own collection key
/// (plan `2026-09-05-every-bounded-read-is-a-page-that-reads-as-a-corpus`,
/// §3 Layer 2).
///
/// **Every key is ALWAYS present, and `null` is a value here, not an
/// absence.** This deliberately departs from the crate's
/// `skip_serializing_if = "Option::is_none"` convention: `total: null` says "no
/// count ran", `truncated: null` says "unknown", and `next_cursor: null` says
/// "there is no next page" — each a positive statement a reader must be able
/// to see, not a key it might be missing because of an older server.
///
/// Enforced on all three sides, because an absent key that silently reads as
/// `null` is exactly the "absent is UNKNOWN, never a default" defect:
/// serialization never skips a field; every `Option` field carries
/// `#[schemars(required, schema_with = "nullable::<T>")]` — `required` because
/// schemars 1 otherwise omits an `Option` from `required` (the generated
/// Pydantic model would then default a missing key to `None`), and
/// `schema_with` because `required` alone makes schemars emit the INNER type's
/// schema, dropping `null` so generated models reject the value this wire
/// sends; and deserialization uses `Option::deserialize`, which refuses a
/// MISSING key instead of defaulting it (serde's derive otherwise treats an
/// absent `Option` as `None`). Pinning tests assert, structurally, that
/// `required` is exactly the field set and that every nullable key admits
/// `null`.
///
/// `count` and `limit` keep the spellings list doors already serve (coord
/// #1897 put `count` on every agent list door; web proxies and runner pollers
/// read both), so merging this into an existing response rewrites them with
/// the same values rather than moving them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BoundedReadMeta {
    /// Rows in this page (the legacy spelling of `shown`).
    pub count: i64,
    /// The cap actually applied to this page.
    pub limit: i64,
    /// Rows in this page.
    pub shown: i64,
    /// The exact match count from this page's start position — a number only
    /// when `bound_kind` is `exact`, else `null`.
    #[serde(deserialize_with = "Option::deserialize")]
    #[schemars(required, schema_with = "nullable::<i64>")]
    pub total: Option<i64>,
    /// Whether matching rows exist beyond this page; `null` when
    /// `bound_kind` is `unknown`.
    #[serde(deserialize_with = "Option::deserialize")]
    #[schemars(required, schema_with = "nullable::<bool>")]
    pub truncated: Option<bool>,
    /// Which kind of bound produced `total` / `truncated`.
    pub bound_kind: BoundKind,
    /// The opaque token for the next page — pass it back verbatim as
    /// `cursor`. `null` iff `truncated` is not `true`, OR the read is a
    /// RANKING that cannot be paged — then `truncated` may be `true` with
    /// `next_cursor: null`, and `enumerate_via` names the door that walks the
    /// corpus by an immutable key.
    #[serde(deserialize_with = "Option::deserialize")]
    #[schemars(required, schema_with = "nullable::<String>")]
    pub next_cursor: Option<String>,
    /// `false` ONLY when the store is unprovisioned: an empty page with
    /// `available: false` is UNKNOWN, not "nothing matched".
    pub available: bool,
    /// A FILTER the surface narrowed before the read ran, or `null`.
    #[serde(deserialize_with = "Option::deserialize")]
    #[schemars(required, schema_with = "nullable::<FilterNarrowing>")]
    pub filter_narrowed: Option<FilterNarrowing>,
    /// For a relevance-RANKED read, which can never hand out a cursor: the
    /// door that enumerates the same corpus by an immutable sort key (plan
    /// `2026-09-05-every-bounded-read-is-a-page-that-reads-as-a-corpus` D8), so
    /// a `truncated: true, next_cursor: null` answer still says how to reach
    /// the rest. `null` on a keyset walk, whose `next_cursor` is the way on.
    #[serde(deserialize_with = "Option::deserialize")]
    #[schemars(required, schema_with = "nullable::<String>")]
    pub enumerate_via: Option<String>,
}

impl BoundedReadMeta {
    /// Merge these keys into an existing JSON response object, overwriting
    /// only the keys named here. For doors that build their answer with
    /// `serde_json::json!` rather than a struct that could `#[serde(flatten)]`
    /// this; every other key — the collection, the filter echoes — is left
    /// exactly as the door wrote it.
    pub fn insert_into(&self, target: &mut serde_json::Map<String, serde_json::Value>) {
        if let Ok(serde_json::Value::Object(keys)) = serde_json::to_value(self) {
            target.extend(keys);
        }
    }
}

// ============================================================================
// The keyset cursor codec
// ============================================================================

/// Wire version of the cursor payload. A token carrying any other version is
/// refused, never reinterpreted: a future revision that changes the payload,
/// the fingerprint layout or the tiebreak produces tokens whose fields mean
/// something else, and decoding one under the new meaning is the silently
/// wrong page this codec exists to prevent. Bump it on any such change.
pub const CURSOR_WIRE_VERSION: u8 = 1;

/// The longest token [`ScopeFingerprint::decode`] will look at. A token this
/// codec mints is ~300 characters with a short [`SortKey::ID`]; anything far
/// longer is not one of ours, and bounding it keeps a hostile query string from
/// buying a large base64 + JSON decode.
pub const CURSOR_MAX_TOKEN_LEN: usize = 1024;

/// Domain separator for [`ScopeFingerprint`] — pins the fingerprint's meaning
/// to THIS layout, so it cannot collide with another module's digest of the
/// same values.
const FINGERPRINT_DOMAIN: &[u8] = b"qontinui.bounded-read.cursor";

/// Marker for a keyset sort key this codec may walk. **Implementing it is an
/// assertion about the schema**, so read all four clauses:
///
/// 1. **The key is IMMUTABLE after insert.** The timestamp column has no
///    UPDATE site anywhere (`created_at`, `authored_at`, `started_at` — never
///    `updated_at`, never a heartbeat, never a `COALESCE` over a mutable
///    column). Keyset survives INSERT and DELETE, but an UPDATE that moves an
///    unreached row's key across the cursor drops that row from the walk
///    silently: no duplicate, no error, a normally terminating cursor. coord
///    found this in `session_fleet.rs` (`2794f010d`). A surface whose natural
///    order IS mutable ("recently updated") keeps that order for DISPLAY only
///    and walks on an immutable key.
/// 2. **The tiebreak is the row's unique id**, so `(key, id)` is a total order
///    and the row comparison `(key, id) < ($k, $i)` neither drops nor repeats
///    rows that share a timestamp.
/// 3. **The statement orders by exactly `key DESC, id DESC`** and filters with
///    exactly that row comparison. A walk that disagrees with its sort skips
///    and repeats rows.
/// 4. **[`Self::ID`] names the sequence and changes when the key does.** It is
///    bound into every token and into the fingerprint, so a token minted
///    against an old key decodes as malformed instead of paging silently
///    wrong. Keep it short (it rides in the token): `<store>:<key>,<tiebreak>`,
///    e.g. `coord.findings:created_at,finding_id`.
pub trait SortKey {
    /// The stable name of the ordered sequence. See the trait doc, clause 4.
    const ID: &'static str;
}

/// A decoded keyset position: the last served row's sort key and id. The next
/// page is every row strictly BEFORE it in `(at DESC, id DESC)` order.
///
/// `at` round-trips at microsecond resolution — PostgreSQL `timestamptz`'s
/// own — so the tie arm of the row comparison matches the row it came from.
/// A finer timestamp is truncated to microseconds by [`ScopeFingerprint::encode`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeysetPosition {
    /// The last served row's (immutable) sort-key timestamp.
    pub at: DateTime<Utc>,
    /// The last served row's unique id — the tiebreak.
    pub id: Uuid,
}

/// Builds the [`ScopeFingerprint`] of a read: everything that changes WHICH
/// rows the ordered sequence contains. Add every such filter — the tenant
/// included — as the surface APPLIED it (after any truncation), and nothing
/// that only changes which SLICE is taken: `limit` stays out, so resizing a
/// page mid-walk is legitimate and needs no new cursor.
///
/// The byte layout is specified in the module doc.
pub struct CursorScope<K: SortKey> {
    hasher: Sha256,
    _key: PhantomData<fn() -> K>,
}

impl<K: SortKey> Default for CursorScope<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: SortKey> CursorScope<K> {
    /// Start a scope for sort key `K`.
    pub fn new() -> Self {
        let mut hasher = Sha256::new();
        hasher.update(FINGERPRINT_DOMAIN);
        hasher.update([0x1F, CURSOR_WIRE_VERSION, 0x1F]);
        hasher.update(K::ID.as_bytes());
        CursorScope {
            hasher,
            _key: PhantomData,
        }
    }

    fn field(&mut self, name: &str) {
        self.hasher.update([0x1E]);
        self.hasher.update(name.as_bytes());
        self.hasher.update([0x1F]);
    }

    fn put_bytes(&mut self, bytes: &[u8]) {
        self.hasher.update((bytes.len() as u64).to_be_bytes());
        self.hasher.update(bytes);
    }

    /// A string-valued filter; `None` is hashed as ABSENT, distinct from every
    /// string including the empty one.
    pub fn opt_str(mut self, name: &str, value: Option<&str>) -> Self {
        self.field(name);
        match value {
            Some(v) => {
                self.hasher.update(b"s");
                self.put_bytes(v.as_bytes());
            }
            None => self.hasher.update(b"n"),
        }
        self
    }

    /// A uuid-valued scope member (the tenant, a device).
    pub fn uuid(mut self, name: &str, value: Uuid) -> Self {
        self.field(name);
        self.hasher.update(b"u");
        self.hasher.update(value.as_bytes());
        self
    }

    /// An optional uuid filter; `None` is hashed as ABSENT.
    pub fn opt_uuid(mut self, name: &str, value: Option<Uuid>) -> Self {
        match value {
            Some(v) => self.uuid(name, v),
            None => {
                self.field(name);
                self.hasher.update(b"n");
                self
            }
        }
    }

    /// A boolean filter.
    pub fn bool(mut self, name: &str, value: bool) -> Self {
        self.field(name);
        self.hasher.update(b"b");
        self.hasher.update([u8::from(value)]);
        self
    }

    /// An integer filter (a mode, an enum discriminant).
    pub fn i64(mut self, name: &str, value: i64) -> Self {
        self.field(name);
        self.hasher.update(b"i");
        self.hasher.update(value.to_be_bytes());
        self
    }

    /// A SET-valued filter (array overlap, `IN (…)`): hashed sorted and
    /// deduplicated, because `{a, b}` and `{b, a, a}` select the same rows and
    /// a cursor minted under one must be honoured under the other.
    pub fn str_set(mut self, name: &str, values: &[String]) -> Self {
        let mut set: Vec<&[u8]> = values.iter().map(String::as_bytes).collect();
        set.sort_unstable();
        set.dedup();
        self.field(name);
        self.hasher.update(b"l");
        self.hasher.update((set.len() as u64).to_be_bytes());
        for v in set {
            self.put_bytes(v);
        }
        self
    }

    /// Finish the scope.
    pub fn finish(self) -> ScopeFingerprint<K> {
        ScopeFingerprint {
            hex: hex::encode(self.hasher.finalize()),
            _key: PhantomData,
        }
    }
}

/// The fingerprint of one read's scope under sort key `K` — and the codec.
/// A cursor is minted and checked against a fingerprint, so a token can only
/// resume the sequence it came from.
pub struct ScopeFingerprint<K: SortKey> {
    hex: String,
    _key: PhantomData<fn() -> K>,
}

impl<K: SortKey> Clone for ScopeFingerprint<K> {
    fn clone(&self) -> Self {
        ScopeFingerprint {
            hex: self.hex.clone(),
            _key: PhantomData,
        }
    }
}

impl<K: SortKey> fmt::Debug for ScopeFingerprint<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScopeFingerprint")
            .field("sort_key", &K::ID)
            .field("hex", &self.hex)
            .finish()
    }
}

impl<K: SortKey> PartialEq for ScopeFingerprint<K> {
    fn eq(&self, other: &Self) -> bool {
        self.hex == other.hex
    }
}

impl<K: SortKey> Eq for ScopeFingerprint<K> {}

/// The decoded body of a token. One-letter fields because it rides in a query
/// string; `deny_unknown_fields` because a payload with extra keys is not one
/// this codec minted.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CursorPayload {
    /// [`CURSOR_WIRE_VERSION`].
    v: u8,
    /// [`SortKey::ID`].
    s: String,
    /// The sort key as microseconds since the Unix epoch — `timestamptz`'s own
    /// resolution, so the value → token → bound parameter trip is exact.
    k: i64,
    /// The unique tiebreak id.
    i: Uuid,
    /// The [`ScopeFingerprint`] hex.
    f: String,
}

fn cursor_engine() -> base64::engine::general_purpose::GeneralPurpose {
    // URL-safe and unpadded: the token travels in a query string, where `+`,
    // `/` and `=` are reserved or mangled. The NO_PAD engine also REFUSES a
    // padded token, which keeps decode a strict inverse of encode.
    base64::engine::general_purpose::URL_SAFE_NO_PAD
}

impl<K: SortKey> ScopeFingerprint<K> {
    /// The lowercase hex digest.
    pub fn as_hex(&self) -> &str {
        &self.hex
    }

    /// Mint the token addressing the position immediately AFTER `pos` in the
    /// walk. `pos.at` is carried at microsecond resolution.
    pub fn encode(&self, pos: KeysetPosition) -> String {
        let payload = CursorPayload {
            v: CURSOR_WIRE_VERSION,
            s: K::ID.to_string(),
            k: pos.at.timestamp_micros(),
            i: pos.id,
            f: self.hex.clone(),
        };
        // Five owned fields of serializable primitives: `to_vec` cannot fail
        // for this payload, and falling back to the empty token (which decodes
        // as malformed) is preferred over a panic inside a request handler.
        let json = serde_json::to_vec(&payload).unwrap_or_default();
        cursor_engine().encode(json)
    }

    /// Decode a caller-supplied token STRICTLY: any token this fingerprint did
    /// not mint — garbage, empty, over-long, padded, another wire version,
    /// another sort key, another scope, an out-of-range instant — is a
    /// [`CursorError`], never a panic and never a clamp. There is no nearest
    /// valid cursor: a clamped one would resume the walk at an arbitrary point
    /// with rows silently missing.
    pub fn decode(&self, token: &str) -> Result<KeysetPosition, CursorError> {
        let token = token.trim();
        if token.is_empty() {
            return Err(CursorError::new(CursorRejection::Empty));
        }
        if token.len() > CURSOR_MAX_TOKEN_LEN {
            return Err(CursorError::new(CursorRejection::TooLong));
        }
        let bytes = cursor_engine()
            .decode(token)
            .map_err(|_| CursorError::new(CursorRejection::Encoding))?;
        let payload: CursorPayload = serde_json::from_slice(&bytes)
            .map_err(|_| CursorError::new(CursorRejection::Payload))?;
        if payload.v != CURSOR_WIRE_VERSION {
            return Err(CursorError::new(CursorRejection::Version));
        }
        if payload.s != K::ID {
            return Err(CursorError::new(CursorRejection::SortKey));
        }
        if payload.f != self.hex {
            return Err(CursorError::new(CursorRejection::Scope));
        }
        let at = DateTime::<Utc>::from_timestamp_micros(payload.k)
            .ok_or(CursorError::new(CursorRejection::Timestamp))?;
        // Every row a store here can hold is AD 1 … 9999. chrono accepts year
        // 0 and below, which PostgreSQL's Gregorian calendar either rejects or
        // reads as BC — a token no server of ours could have minted.
        if !(1..=9999).contains(&at.year()) {
            return Err(CursorError::new(CursorRejection::Timestamp));
        }
        Ok(KeysetPosition { at, id: payload.i })
    }
}

/// Why a supplied cursor was refused. Every reason is the same refusal on the
/// wire — the token is not one this read minted, so restart the walk — and the
/// reason is carried for the detail line and for tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorRejection {
    /// Blank.
    Empty,
    /// Longer than [`CURSOR_MAX_TOKEN_LEN`].
    TooLong,
    /// Not unpadded urlsafe base64.
    Encoding,
    /// Not this payload shape.
    Payload,
    /// Another [`CURSOR_WIRE_VERSION`].
    Version,
    /// Minted for another [`SortKey`].
    SortKey,
    /// Minted under different filters or another tenant.
    Scope,
    /// The instant is out of range.
    Timestamp,
}

impl CursorRejection {
    /// A short human reason. Echoes NOTHING from the token: a cursor is
    /// caller-supplied, and reflecting it into an error body turns the
    /// response into a mirror.
    pub fn as_str(self) -> &'static str {
        match self {
            CursorRejection::Empty => "the cursor is blank",
            CursorRejection::TooLong => "the cursor is longer than any token this read mints",
            CursorRejection::Encoding => "the cursor is not unpadded urlsafe base64",
            CursorRejection::Payload => "the cursor does not decode to a cursor payload",
            CursorRejection::Version => "the cursor was minted by a different version of this read",
            CursorRejection::SortKey => "the cursor was minted for a different sort order",
            CursorRejection::Scope => {
                "the cursor was minted under different filters or a different tenant; a cursor \
                 addresses a position in ONE ordered sequence"
            }
            CursorRejection::Timestamp => "the cursor's position is out of range",
        }
    }
}

impl fmt::Display for CursorRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A malformed cursor — the typed refusal every door answers with `400` (or a
/// typed tool error), naming the `cursor` parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error(
    "invalid cursor — pass a `next_cursor` from a previous response verbatim, or omit `cursor` \
     for the first page ({reason})"
)]
pub struct CursorError {
    reason: CursorRejection,
}

impl CursorError {
    fn new(reason: CursorRejection) -> Self {
        CursorError { reason }
    }

    /// Why the token was refused.
    pub fn reason(&self) -> CursorRejection {
        self.reason
    }

    /// The stable machine code — one code for every reason, because every
    /// reason has the one remedy.
    pub fn code(&self) -> &'static str {
        "cursor_malformed"
    }

    /// The refusal text naming the surface the cursor should have come from,
    /// in the fleet's standard wording (`fleet_health`'s `/coord/alerts`).
    pub fn refusal(&self, surface: &str) -> String {
        format!(
            "invalid cursor — pass a `next_cursor` from a previous {surface} response verbatim, \
             or omit `cursor` for the first page ({})",
            self.reason
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    struct TestKey;
    impl SortKey for TestKey {
        const ID: &'static str = "test.rows:created_at,id";
    }

    struct OtherKey;
    impl SortKey for OtherKey {
        const ID: &'static str = "test.rows:authored_at,id";
    }

    fn scope(tenant: Uuid, topic: Option<&str>) -> ScopeFingerprint<TestKey> {
        CursorScope::<TestKey>::new()
            .uuid("tenant", tenant)
            .opt_str("topic", topic)
            .finish()
    }

    fn pos() -> KeysetPosition {
        KeysetPosition {
            at: Utc
                .timestamp_micros(1_790_000_000_123_456)
                .single()
                .unwrap(),
            id: Uuid::from_u128(0x0123_4567_89ab_cdef_0123_4567_89ab_cdef),
        }
    }

    fn raw_token(payload: serde_json::Value) -> String {
        cursor_engine().encode(serde_json::to_vec(&payload).unwrap())
    }

    // ---- codec -------------------------------------------------------------

    #[test]
    fn cursor_round_trips_exactly_at_microsecond_resolution() {
        let fp = scope(Uuid::nil(), Some("merge-engine"));
        let token = fp.encode(pos());
        assert!(!token.contains('='), "unpadded: {token}");
        assert!(
            token
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
            "urlsafe alphabet only: {token}"
        );
        assert_eq!(fp.decode(&token), Ok(pos()));
        // Surrounding whitespace a query parser left behind is tolerated.
        assert_eq!(fp.decode(&format!("  {token}\n")), Ok(pos()));
    }

    #[test]
    fn sub_microsecond_precision_is_truncated_not_rounded_into_a_different_row() {
        let fp = scope(Uuid::nil(), None);
        let nanos = Utc
            .timestamp_opt(1_790_000_000, 123_456_789)
            .single()
            .unwrap();
        let decoded = fp
            .decode(&fp.encode(KeysetPosition {
                at: nanos,
                id: Uuid::nil(),
            }))
            .unwrap();
        assert_eq!(decoded.at.timestamp_micros(), nanos.timestamp_micros());
    }

    #[test]
    fn a_token_from_another_wire_version_is_malformed() {
        let fp = scope(Uuid::nil(), None);
        let token = raw_token(serde_json::json!({
            "v": CURSOR_WIRE_VERSION + 1,
            "s": TestKey::ID,
            "k": 1_790_000_000_123_456_i64,
            "i": Uuid::nil(),
            "f": fp.as_hex(),
        }));
        let err = fp.decode(&token).unwrap_err();
        assert_eq!(err.reason(), CursorRejection::Version);
        assert_eq!(err.code(), "cursor_malformed");
    }

    #[test]
    fn a_token_minted_for_another_sort_key_is_malformed() {
        let other = CursorScope::<OtherKey>::new()
            .uuid("tenant", Uuid::nil())
            .finish();
        let token = other.encode(pos());
        let err = scope(Uuid::nil(), None).decode(&token).unwrap_err();
        assert_eq!(err.reason(), CursorRejection::SortKey);
    }

    #[test]
    fn a_token_replayed_under_different_filters_or_tenant_is_malformed() {
        let minted = scope(Uuid::nil(), Some("a")).encode(pos());
        for other in [
            scope(Uuid::nil(), Some("b")),
            scope(Uuid::nil(), None),
            scope(Uuid::from_u128(1), Some("a")),
        ] {
            assert_eq!(
                other.decode(&minted).unwrap_err().reason(),
                CursorRejection::Scope
            );
        }
    }

    #[test]
    fn garbage_empty_padded_and_over_long_tokens_are_malformed_never_a_panic() {
        let fp = scope(Uuid::nil(), None);
        let good = fp.encode(pos());
        let cases: Vec<(String, CursorRejection)> = vec![
            (String::new(), CursorRejection::Empty),
            ("   ".into(), CursorRejection::Empty),
            (
                "x".repeat(CURSOR_MAX_TOKEN_LEN + 1),
                CursorRejection::TooLong,
            ),
            ("not base64!!".into(), CursorRejection::Encoding),
            (format!("{good}="), CursorRejection::Encoding),
            (format!("{good}=="), CursorRejection::Encoding),
            // A standard-alphabet token is not ours either.
            ("ab+/".into(), CursorRejection::Encoding),
            (
                cursor_engine().encode(b"not json"),
                CursorRejection::Payload,
            ),
            (cursor_engine().encode(b"{}"), CursorRejection::Payload),
            (
                cursor_engine().encode([0xff, 0xfe, 0x00]),
                CursorRejection::Payload,
            ),
            // memory.py's `<iso>|<seq>` shape is a different codec.
            (
                cursor_engine().encode(b"2026-09-05T00:00:00+00:00|12"),
                CursorRejection::Payload,
            ),
            (
                raw_token(serde_json::json!({
                    "v": CURSOR_WIRE_VERSION, "s": TestKey::ID, "k": 1, "i": Uuid::nil(),
                    "f": fp.as_hex(), "extra": true,
                })),
                CursorRejection::Payload,
            ),
            (
                raw_token(serde_json::json!({
                    "v": CURSOR_WIRE_VERSION, "s": TestKey::ID, "k": "1", "i": Uuid::nil(),
                    "f": fp.as_hex(),
                })),
                CursorRejection::Payload,
            ),
            (
                raw_token(serde_json::json!({
                    "v": CURSOR_WIRE_VERSION, "s": TestKey::ID, "k": 1, "i": "not-a-uuid",
                    "f": fp.as_hex(),
                })),
                CursorRejection::Payload,
            ),
            (
                raw_token(serde_json::json!({
                    "v": CURSOR_WIRE_VERSION, "s": TestKey::ID, "k": i64::MAX,
                    "i": Uuid::nil(), "f": fp.as_hex(),
                })),
                CursorRejection::Timestamp,
            ),
            (
                // Year 0 — representable in chrono, never a row of ours.
                raw_token(serde_json::json!({
                    "v": CURSOR_WIRE_VERSION, "s": TestKey::ID,
                    "k": -62_167_219_200_000_000_i64, "i": Uuid::nil(), "f": fp.as_hex(),
                })),
                CursorRejection::Timestamp,
            ),
        ];
        for (token, want) in cases {
            let err = fp
                .decode(&token)
                .expect_err(&format!("{token:?} must be refused"));
            assert_eq!(err.reason(), want, "{token:?}");
        }
    }

    #[test]
    fn refusal_names_the_cursor_and_echoes_nothing_from_the_token() {
        let fp = scope(Uuid::nil(), None);
        let secret = "c2VjcmV0LXZhbHVl";
        let err = fp.decode(secret).unwrap_err();
        let msg = err.refusal("/coord/agent-findings");
        assert!(
            msg.starts_with(
                "invalid cursor — pass a `next_cursor` from a previous /coord/agent-findings"
            ),
            "{msg}"
        );
        assert!(msg.contains("`cursor`"), "{msg}");
        assert!(!msg.contains(secret), "{msg}");
        assert!(!err.to_string().contains(secret));
    }

    #[test]
    fn a_minted_token_stays_well_inside_the_length_bound() {
        struct LongKey;
        impl SortKey for LongKey {
            // A generously long ID still leaves the token under the bound.
            const ID: &'static str = "a-very-long-store-name.with-a-schema:authored_at_timestamp,a_tiebreak_identifier_column_name";
        }
        let fp = CursorScope::<LongKey>::new().finish();
        let token = fp.encode(pos());
        assert!(token.len() < CURSOR_MAX_TOKEN_LEN / 2, "{}", token.len());
        assert_eq!(fp.decode(&token), Ok(pos()));
    }

    // ---- the fingerprint ---------------------------------------------------

    #[test]
    fn str_set_is_order_and_duplicate_insensitive() {
        let a = CursorScope::<TestKey>::new()
            .str_set("keys", &["b".into(), "a".into()])
            .finish();
        let b = CursorScope::<TestKey>::new()
            .str_set("keys", &["a".into(), "b".into(), "a".into()])
            .finish();
        assert_eq!(a, b);
    }

    #[test]
    fn absent_is_distinct_from_every_present_value_and_fields_cannot_run_together() {
        let fp =
            |f: fn(CursorScope<TestKey>) -> CursorScope<TestKey>| f(CursorScope::new()).finish();
        let all = [
            fp(|s| s.opt_str("topic", None)),
            fp(|s| s.opt_str("topic", Some(""))),
            fp(|s| s.opt_str("topic", Some("n"))),
            fp(|s| s.opt_str("topic", Some("ab")).opt_str("kind", Some("c"))),
            fp(|s| s.opt_str("topic", Some("a")).opt_str("kind", Some("bc"))),
            fp(|s| s.str_set("keys", &[])),
            fp(|s| s.str_set("keys", &["".into()])),
            fp(|s| s.str_set("keys", &["ab".into()])),
            fp(|s| s.str_set("keys", &["a".into(), "b".into()])),
            fp(|s| s.opt_uuid("device", None)),
            fp(|s| s.opt_uuid("device", Some(Uuid::nil()))),
            fp(|s| s.bool("closed", false)),
            fp(|s| s.bool("closed", true)),
            fp(|s| s.i64("mode", 0)),
            fp(|s| s.i64("mode", 1)),
        ];
        for (i, a) in all.iter().enumerate() {
            for b in &all[i + 1..] {
                assert_ne!(a, b);
            }
        }
    }

    #[test]
    fn the_fingerprint_is_bound_to_the_sort_key_and_the_wire_version() {
        let a = CursorScope::<TestKey>::new().finish();
        let b = CursorScope::<OtherKey>::new().finish();
        assert_ne!(a.as_hex(), b.as_hex());
        assert_eq!(a.as_hex().len(), 64);
        // Pinned: the layout in the module doc is a cross-language contract,
        // so an accidental change to it must fail here, not in Python.
        let pinned = CursorScope::<TestKey>::new()
            .uuid("tenant", Uuid::nil())
            .opt_str("topic", Some("merge-engine"))
            .str_set("keys", &["b".into(), "a".into()])
            .finish();
        let mut h = Sha256::new();
        h.update(b"qontinui.bounded-read.cursor\x1f\x01\x1ftest.rows:created_at,id");
        h.update(b"\x1etenant\x1fu");
        h.update([0u8; 16]);
        h.update(b"\x1etopic\x1fs");
        h.update(12u64.to_be_bytes());
        h.update(b"merge-engine");
        h.update(b"\x1ekeys\x1fl");
        h.update(2u64.to_be_bytes());
        h.update(1u64.to_be_bytes());
        h.update(b"a");
        h.update(1u64.to_be_bytes());
        h.update(b"b");
        assert_eq!(pinned.as_hex(), hex::encode(h.finalize()));
    }

    #[test]
    fn the_token_payload_is_the_documented_json() {
        let fp = scope(Uuid::nil(), None);
        let token = fp.encode(pos());
        let json: serde_json::Value =
            serde_json::from_slice(&cursor_engine().decode(token).unwrap()).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "v": 1,
                "s": "test.rows:created_at,id",
                "k": 1_790_000_000_123_456_i64,
                "i": "01234567-89ab-cdef-0123-456789abcdef",
                "f": fp.as_hex(),
            })
        );
    }

    // ---- Page / meta -------------------------------------------------------

    fn probe(n: usize, limit: i64) -> Page<usize> {
        Page::from_probe((0..n).collect(), limit, |last| format!("after-{last}"))
    }

    #[test]
    fn from_probe_fewer_than_limit_is_complete_with_no_cursor() {
        let page = probe(3, 5);
        assert_eq!(page.rows(), &[0, 1, 2]);
        assert_eq!(page.bound(), Bound::Complete);
        assert_eq!(page.truncated(), Some(false));
        assert_eq!(page.next_cursor(), None);
    }

    #[test]
    fn from_probe_exactly_limit_is_not_truncated() {
        // The assertion the naive `rows.len() == limit` spelling fails.
        let page = probe(5, 5);
        assert_eq!(page.rows().len(), 5);
        assert_eq!(page.bound(), Bound::Complete);
        assert_eq!(page.truncated(), Some(false));
        assert_eq!(page.next_cursor(), None);
    }

    #[test]
    fn from_probe_limit_plus_one_drops_the_probe_and_cursors_from_the_last_kept_row() {
        let page = probe(6, 5);
        assert_eq!(page.rows(), &[0, 1, 2, 3, 4]);
        assert_eq!(page.bound(), Bound::AtLeast(6));
        assert_eq!(page.truncated(), Some(true));
        assert_eq!(page.next_cursor(), Some("after-4"));
    }

    #[test]
    fn from_probe_raises_a_non_positive_limit_to_one() {
        let page = probe(2, 0);
        assert_eq!(page.effective_limit(), 1);
        assert_eq!(page.rows(), &[0]);
        assert_eq!(page.next_cursor(), Some("after-0"));
    }

    #[test]
    fn from_window_count_is_exact_and_truncated_only_when_rows_remain() {
        let more = Page::from_window_count(vec![1, 2], 2, 7, |l| format!("after-{l}"));
        assert_eq!(more.truncated(), Some(true));
        assert_eq!(more.next_cursor(), Some("after-2"));
        let meta = more.meta();
        assert_eq!(meta.total, Some(7));
        assert_eq!(meta.bound_kind, BoundKind::Exact);

        let last = Page::from_window_count(vec![1, 2], 5, 2, |l| format!("after-{l}"));
        assert_eq!(last.truncated(), Some(false));
        assert_eq!(last.next_cursor(), None);
    }

    fn keys(map: &serde_json::Map<String, serde_json::Value>) -> Vec<&str> {
        let mut k: Vec<&str> = map.keys().map(String::as_str).collect();
        k.sort_unstable();
        k
    }

    fn meta_json(page: &Page<usize>) -> serde_json::Map<String, serde_json::Value> {
        match serde_json::to_value(page.meta()).unwrap() {
            serde_json::Value::Object(m) => m,
            other => panic!("meta must serialize to an object, got {other}"),
        }
    }

    #[test]
    fn meta_serves_every_layer_two_key_always_with_null_as_a_value() {
        let want = [
            "available",
            "bound_kind",
            "count",
            "enumerate_via",
            "filter_narrowed",
            "limit",
            "next_cursor",
            "shown",
            "total",
            "truncated",
        ];
        for page in [
            probe(6, 5),
            probe(2, 5),
            Page::unavailable(5),
            Page::from_window_count(vec![1], 5, 1, |_| String::new()),
        ] {
            assert_eq!(keys(&meta_json(&page)), want);
        }
    }

    #[test]
    fn meta_null_rules_per_bound() {
        let truncated = meta_json(&probe(6, 5));
        assert_eq!(truncated["truncated"], true);
        assert_eq!(truncated["bound_kind"], "at_least");
        assert!(truncated["total"].is_null(), "a probe never counts");
        assert_eq!(truncated["next_cursor"], "after-4");
        assert_eq!(truncated["count"], 5);
        assert_eq!(truncated["shown"], 5);
        assert_eq!(truncated["limit"], 5);
        assert_eq!(truncated["available"], true);
        assert!(truncated["filter_narrowed"].is_null());

        let complete = meta_json(&probe(2, 5));
        assert_eq!(complete["truncated"], false);
        assert_eq!(complete["bound_kind"], "complete");
        assert!(complete["total"].is_null());
        assert!(complete["next_cursor"].is_null());

        let unknown = meta_json(&Page::unavailable(5));
        assert!(
            unknown["truncated"].is_null(),
            "UNKNOWN must never render as `truncated: false`"
        );
        assert_eq!(unknown["bound_kind"], "unknown");
        assert!(unknown["total"].is_null());
        assert!(unknown["next_cursor"].is_null());
        assert_eq!(unknown["available"], false);
        assert_eq!(unknown["count"], 0);
    }

    #[test]
    fn filter_narrowing_rides_the_meta() {
        let page = probe(1, 5).with_filter_narrowed(Some(FilterNarrowing {
            parameter: "resource_keys".into(),
            applied: 100,
            cap: 100,
        }));
        assert_eq!(
            meta_json(&page)["filter_narrowed"],
            serde_json::json!({"parameter": "resource_keys", "applied": 100, "cap": 100})
        );
    }

    #[test]
    fn insert_into_adds_the_envelope_and_leaves_the_callers_keys_alone() {
        let mut answer = serde_json::json!({
            "findings": [1, 2],
            "count": 2,
            "limit": 20,
            "kind_applied": null,
            "resource_keys_truncated": false,
        });
        let page = Page::from_probe(vec![1, 2], 20, |_| String::new());
        page.meta()
            .insert_into(answer.as_object_mut().expect("object"));
        assert_eq!(answer["findings"], serde_json::json!([1, 2]));
        assert_eq!(answer["count"], 2);
        assert_eq!(answer["limit"], 20);
        assert!(answer["kind_applied"].is_null());
        assert_eq!(answer["resource_keys_truncated"], false);
        assert_eq!(answer["truncated"], false);
        assert!(answer["next_cursor"].is_null());
        assert_eq!(answer["bound_kind"], "complete");
    }

    #[test]
    fn not_pageable_is_truncated_without_a_cursor_and_names_the_walk() {
        let more = Page::not_pageable((0..6).collect::<Vec<usize>>(), 5, "GET /memory/records");
        assert_eq!(more.rows(), &[0, 1, 2, 3, 4]);
        assert_eq!(more.truncated(), Some(true));
        assert_eq!(more.next_cursor(), None);
        assert_eq!(more.enumerate_via(), Some("GET /memory/records"));
        let meta = meta_json(&more);
        assert_eq!(meta["truncated"], true);
        assert!(meta["next_cursor"].is_null());
        assert_eq!(meta["enumerate_via"], "GET /memory/records");
        assert_eq!(meta["bound_kind"], "at_least");

        let all = Page::not_pageable((0..3).collect::<Vec<usize>>(), 5, "GET /memory/records");
        assert_eq!(all.truncated(), Some(false));
        assert_eq!(meta_json(&all)["enumerate_via"], "GET /memory/records");
    }

    #[test]
    fn every_constructor_keeps_the_envelope_invariant() {
        let cursor = |l: &usize| format!("after-{l}");
        let pages: Vec<Page<usize>> = vec![
            Page::from_probe(vec![], 5, cursor),
            Page::from_probe((0..3).collect(), 5, cursor),
            Page::from_probe((0..5).collect(), 5, cursor),
            Page::from_probe((0..6).collect(), 5, cursor),
            Page::from_probe((0..2).collect(), 0, cursor),
            Page::from_window_count((0..2).collect(), 2, 7, cursor),
            Page::from_window_count((0..2).collect(), 5, 2, cursor),
            // A count that claims rows an empty page cannot hold: UNKNOWN, not
            // a truncated page with no way forward.
            Page::from_window_count(vec![], 5, 3, cursor),
            Page::complete((0..1).collect(), 5),
            Page::unavailable(5),
            Page::not_pageable((0..6).collect(), 5, "door"),
            Page::not_pageable((0..2).collect(), 5, "door"),
        ];
        for page in &pages {
            assert!(page.envelope_is_consistent(), "{page:?}");
            let meta = page.meta();
            // truncated && no cursor => a ranking that names its walk.
            if meta.truncated == Some(true) && meta.next_cursor.is_none() {
                assert!(meta.enumerate_via.is_some(), "{page:?}");
            }
            // a cursor => it IS the walk.
            if meta.next_cursor.is_some() {
                assert!(meta.enumerate_via.is_none(), "{page:?}");
                assert_eq!(meta.truncated, Some(true), "{page:?}");
            }
        }
        assert_eq!(pages[7].bound(), Bound::Unknown);
        assert_eq!(pages[7].truncated(), None);
    }

    #[test]
    fn the_invariant_check_can_fail() {
        // Hand-built past the constructors (private fields are reachable from
        // this module only): each broken shape must be caught.
        let base = Page::from_probe((0..6).collect::<Vec<usize>>(), 5, |l| format!("{l}"));
        let mut cursor_and_via = base.clone();
        cursor_and_via.enumerate_via = Some("door".into());
        let mut truncated_dead_end = base.clone();
        truncated_dead_end.next_cursor = None;
        let mut cursor_on_complete = Page::complete(vec![1usize], 5);
        cursor_on_complete.next_cursor = Some("x".into());
        for broken in [cursor_and_via, truncated_dead_end, cursor_on_complete] {
            assert!(!broken.envelope_is_consistent(), "{broken:?}");
        }
    }

    fn schema_required_and_properties(schema: &serde_json::Value) -> (Vec<String>, Vec<String>) {
        let mut required: Vec<String> = schema["required"]
            .as_array()
            .expect("required list")
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        let mut properties: Vec<String> = schema["properties"]
            .as_object()
            .expect("properties")
            .keys()
            .cloned()
            .collect();
        required.sort_unstable();
        properties.sort_unstable();
        (required, properties)
    }

    #[test]
    fn bounded_read_meta_schema_is_monomorphic_and_requires_every_key() {
        let schema = serde_json::to_value(schemars::schema_for!(BoundedReadMeta)).unwrap();
        assert_eq!(schema["title"], "BoundedReadMeta");
        let (required, properties) = schema_required_and_properties(&schema);
        // The struct's ACTUAL field set, read off a serialized instance, so a
        // field added later without `#[schemars(required)]` fails here.
        let mut fields: Vec<String> = meta_json(&probe(1, 5)).keys().cloned().collect();
        fields.sort_unstable();
        assert_eq!(fields.len(), 10, "{fields:?}");
        assert_eq!(properties, fields, "every field is a schema property");
        assert_eq!(
            required, fields,
            "EVERY key is required — an absent key must never default to null"
        );
        // Structurally, never by grepping the text (a description that
        // merely MENTIONS null would pass that): each scalar's `type` array
        // carries "null", and the struct is `anyOf` a `$ref` to the one
        // FilterNarrowing definition plus `{"type": "null"}`.
        for (key, scalar) in [
            ("total", "integer"),
            ("truncated", "boolean"),
            ("next_cursor", "string"),
            ("enumerate_via", "string"),
        ] {
            let types = schema["properties"][key]["type"]
                .as_array()
                .unwrap_or_else(|| panic!("{key} must carry a type ARRAY: {schema}"));
            assert!(
                types.contains(&serde_json::json!(scalar)),
                "{key}: {types:?}"
            );
            assert!(
                types.contains(&serde_json::json!("null")),
                "{key}: {types:?}"
            );
        }
        let narrowed = schema["properties"]["filter_narrowed"]["anyOf"]
            .as_array()
            .unwrap_or_else(|| panic!("filter_narrowed must be anyOf: {schema}"));
        assert!(
            narrowed.contains(&serde_json::json!({"$ref": "#/$defs/FilterNarrowing"})),
            "{narrowed:?}"
        );
        assert!(
            narrowed.contains(&serde_json::json!({"type": "null"})),
            "{narrowed:?}"
        );
        // Exactly one definition, so a generator emits one class, not a
        // second inlined `FilterNarrowed` copy.
        let defs = schema["$defs"].as_object().expect("$defs");
        let mut def_names: Vec<&String> = defs.keys().collect();
        def_names.sort_unstable();
        assert_eq!(def_names, ["BoundKind", "FilterNarrowing"], "{defs:?}");
    }

    #[test]
    fn filter_narrowing_schema_requires_every_key() {
        let schema = serde_json::to_value(schemars::schema_for!(FilterNarrowing)).unwrap();
        let (required, properties) = schema_required_and_properties(&schema);
        assert_eq!(properties, ["applied", "cap", "parameter"]);
        assert_eq!(required, properties);
    }

    #[test]
    fn bounded_read_meta_refuses_a_missing_nullable_key_rather_than_defaulting_it() {
        let full = serde_json::to_value(probe(2, 5).meta()).unwrap();
        assert!(serde_json::from_value::<BoundedReadMeta>(full.clone()).is_ok());
        for key in [
            "total",
            "truncated",
            "next_cursor",
            "filter_narrowed",
            "enumerate_via",
        ] {
            let mut missing = full.clone();
            missing.as_object_mut().unwrap().remove(key);
            assert!(
                serde_json::from_value::<BoundedReadMeta>(missing).is_err(),
                "a missing `{key}` must be refused, never read as null"
            );
        }
    }
}
