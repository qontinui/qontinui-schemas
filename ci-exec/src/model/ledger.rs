//! The per-dispatch model ledger and the result's `summary.model` section
//! (plan `2026-10-13-ci-node-runs-model-dependent-eval-steps`, D10).
//!
//! The ledger is its own top-level result field, never part of `summary`:
//! coord replaces an oversize summary wholesale, step rows included, so a few
//! hundred call rows there would erase the step summary.
//!
//! - **No attribution.** Tenant, repo, sha and lane come from the dispatch row.
//! - **Per-call rows:** index, step, requested name, concrete model id, route,
//!   account label, input/output/cache tokens, latency, outcome, attempts.
//! - **Aggregates:** by model, by account, totals, the budget, `isolation`,
//!   and the token-coverage share.
//! - **Cost,** when the provider reports it: integer micro-USD with
//!   `cost_source: "reported"`, labelled "list-price equivalent, not your
//!   bill", with its coverage shown. It stays on tenant-authenticated reads
//!   and never enters public check-run text.
//! - **Bounded.** Serialized, it stays under [`MODEL_LEDGER_MAX_BYTES`]; when
//!   rows have to go, the aggregates still count every call and the ledger
//!   says how many rows it omitted.
//! - **UNKNOWN is an absent key, never `null`.** Coord's ingest refuses a
//!   `null` where a number belongs, because that is how serde_json writes a
//!   NaN or an infinity; so an unreported figure is left out, and coord reads
//!   an absent optional key as unreported.
//!
//! # The wire contract is a fixture
//!
//! `ci-exec/testdata/model/model_ledger_v1.json` is the exact JSON the
//! `fixture_ledger` test produces, and qontinui-coord carries the same file
//! (`crates/coord/testdata/ci_model_access/model_ledger_v1.json`) and asserts
//! its ingest validator accepts it. A change to this module's shape breaks the
//! test here; the same change to the copy breaks coord's.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::host::TokenUsage;

/// The ledger's schema tag.
pub const LEDGER_SCHEMA: &str = "qontinui.ci.model_ledger/1";

/// The largest serialized ledger a result carries — coord's ingest cap
/// (`MODEL_LEDGER_MAX_BYTES`, 1 MiB), so an over-long dispatch omits rows here
/// rather than having the whole ledger rejected there.
pub const MODEL_LEDGER_MAX_BYTES: usize = 1024 * 1024;

/// What credential separation covers for every call: the step's environment,
/// not its OS user (D11).
pub const ISOLATION: &str = "env";

/// The label every cost figure carries.
pub const COST_LABEL: &str = "list-price equivalent, not your bill";

/// The key a call with no reported model id is filed under. UNKNOWN, never a
/// guessed id.
pub const UNKNOWN_MODEL: &str = "unknown";

/// The key a call with no account (a refused call, a gateway route) is filed
/// under in `by_account`.
pub const NO_ACCOUNT: &str = "none";

/// One call's row.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CallRow {
    /// 1-based, in the order requests were admitted.
    pub index: u64,
    /// The manifest step that made the call.
    pub step: String,
    /// The model name as requested.
    pub requested: String,
    /// The concrete id the provider reported; absent is UNKNOWN.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    /// Token counts as reported; absent means the provider reported none (or
    /// the call never reached it), which is UNKNOWN, not zero.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_tokens: Option<u64>,
    pub latency_ms: u64,
    /// `ok`, or the error subtype (`budget_exhausted:calls`, `cancelled`, …).
    pub outcome: String,
    /// Provider attempts, including the host's account retries and the
    /// broker's re-tries after an exhaustion hold; 0 for a call refused before
    /// it reached the host.
    pub attempts: u32,
    /// Provider-reported cost in micro-USD, when reported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_microusd: Option<u64>,
}

impl CallRow {
    /// A row for a request refused before it reached the host.
    pub fn refused(index: u64, step: &str, requested: &str, outcome: String) -> Self {
        Self {
            index,
            step: step.to_string(),
            requested: requested.to_string(),
            model_id: None,
            route: None,
            account: None,
            input_tokens: None,
            output_tokens: None,
            cache_creation_tokens: None,
            cache_read_tokens: None,
            latency_ms: 0,
            outcome,
            attempts: 0,
            cost_microusd: None,
        }
    }

    fn usage(&self) -> Option<TokenUsage> {
        Some(TokenUsage {
            input: self.input_tokens?,
            output: self.output_tokens?,
            cache_creation: self.cache_creation_tokens?,
            cache_read: self.cache_read_tokens?,
        })
    }
}

/// Convert a provider-reported USD figure to integer micro-USD. A negative or
/// non-finite figure is not a cost: `None`.
pub fn usd_to_microusd(usd: f64) -> Option<u64> {
    (usd.is_finite() && usd >= 0.0).then(|| (usd * 1_000_000.0).round() as u64)
}

/// One aggregate bucket.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Aggregate {
    pub calls: u64,
    pub ok: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
    pub cost_microusd: u64,
}

impl Aggregate {
    fn add(&mut self, row: &CallRow) {
        self.calls += 1;
        if row.outcome == "ok" {
            self.ok += 1;
        }
        if let Some(u) = row.usage() {
            self.input_tokens += u.input;
            self.output_tokens += u.output;
            self.cache_creation_tokens += u.cache_creation;
            self.cache_read_tokens += u.cache_read;
        }
        self.cost_microusd += row.cost_microusd.unwrap_or(0);
    }
}

/// The budget, declared and spent.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct BudgetReport {
    pub max_calls: u64,
    pub max_tokens: u64,
    pub max_wall_secs: u64,
    /// Calls that reached the host.
    pub calls_used: u64,
    pub tokens_used: u64,
    /// From the first call to the broker's stop; 0 when no call was made.
    pub wall_secs_used: u64,
    /// How far the tokens used went past `max_tokens`. Calls already in
    /// flight when the limit is crossed finish, so this is bounded by the
    /// in-flight cap times one call's spend; the ledger shows it rather than
    /// hiding it.
    pub tokens_overshoot: u64,
}

/// How many calls reported what the ledger sums.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Coverage {
    /// Calls that reached the host.
    pub calls: u64,
    /// Of those, the calls that reported the figure.
    pub reported: u64,
    /// `reported / calls`, or absent when no call reached the host (UNKNOWN,
    /// not 0 % and not 100 %).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub share: Option<f64>,
}

impl Coverage {
    fn of(calls: u64, reported: u64) -> Self {
        Self {
            calls,
            reported,
            share: (calls > 0).then(|| reported as f64 / calls as f64),
        }
    }
}

/// The labelled cost.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CostReport {
    /// Sum over the calls that reported a cost.
    pub total_microusd: u64,
    pub cost_source: &'static str,
    pub label: &'static str,
    pub coverage: Coverage,
}

/// The whole ledger.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ModelLedger {
    pub schema: &'static str,
    pub isolation: &'static str,
    pub calls: Vec<CallRow>,
    /// Rows omitted to keep the ledger under its byte bound. The aggregates
    /// still count them.
    pub rows_omitted: u64,
    /// Rows of requests refused before they reached the host (a harness
    /// looping on `budget_exhausted`) that were counted but not stored once
    /// [`MAX_STORED_REFUSALS`] were. The aggregates and
    /// `refusals_by_outcome` still count them.
    pub refused_rows_dropped: u64,
    /// Every refusal before the host, by outcome, stored or not.
    pub refusals_by_outcome: BTreeMap<String, u64>,
    /// `false` when any row was omitted or dropped.
    pub rows_complete: bool,
    pub by_model: BTreeMap<String, Aggregate>,
    pub by_account: BTreeMap<String, Aggregate>,
    pub totals: Aggregate,
    pub budget: BudgetReport,
    pub token_coverage: Coverage,
    pub cost: CostReport,
}

impl ModelLedger {
    /// The serialized size coord measures against its ingest cap.
    pub fn serialized_len(&self) -> usize {
        serde_json::to_vec(self)
            .map(|v| v.len())
            .unwrap_or(usize::MAX)
    }

    /// The ledger, if its serialized size is within [`MODEL_LEDGER_MAX_BYTES`];
    /// otherwise its size. Row omission keeps every ledger under the bound
    /// except one whose aggregates alone exceed it (thousands of distinct,
    /// very long model ids or account labels). Such a ledger is not sent at
    /// all, so it can never push the result body past coord's ceiling and
    /// cost the verdict.
    pub fn transmittable(self) -> Result<Self, usize> {
        match self.serialized_len() {
            n if n <= MODEL_LEDGER_MAX_BYTES => Ok(self),
            n => Err(n),
        }
    }
}

/// How many rows of requests refused before they reached the host a ledger
/// stores during a run. Calls that reached the host are bounded by
/// `max_calls`; refusals are bounded only by how often a harness asks, so past
/// this they are counted, not stored — memory stays bounded however long a
/// harness loops on `budget_exhausted`.
pub const MAX_STORED_REFUSALS: usize = 64;

/// Collects rows during a dispatch.
#[derive(Debug, Default)]
pub struct LedgerBuilder {
    rows: Vec<CallRow>,
    stored_refusals: usize,
    dropped_refusals: u64,
    refusals: BTreeMap<String, u64>,
}

impl LedgerBuilder {
    /// Record one row. A row that never reached the host (`attempts == 0`)
    /// is a refusal: it is counted by outcome, and stored only while fewer
    /// than [`MAX_STORED_REFUSALS`] are. Returns how many refusals with this
    /// row's outcome there have been so far (0 for a call that reached the
    /// host), so the caller can rate-limit its log line.
    pub fn push(&mut self, row: CallRow) -> u64 {
        if row.attempts > 0 {
            self.rows.push(row);
            return 0;
        }
        let seen = self.refusals.entry(row.outcome.clone()).or_insert(0);
        *seen += 1;
        let seen = *seen;
        if self.stored_refusals < MAX_STORED_REFUSALS {
            self.stored_refusals += 1;
            self.rows.push(row);
        } else {
            self.dropped_refusals += 1;
        }
        seen
    }

    pub fn rows(&self) -> &[CallRow] {
        &self.rows
    }

    /// The finished ledger, under [`MODEL_LEDGER_MAX_BYTES`].
    pub fn finish(self, budget: BudgetReport) -> ModelLedger {
        self.finish_bounded(budget, MODEL_LEDGER_MAX_BYTES)
    }

    /// The finished ledger, with rows dropped from the END until the
    /// serialized whole fits `max_bytes`. The aggregates are computed over
    /// every row first, so they never lose a call.
    pub fn finish_bounded(self, budget: BudgetReport, max_bytes: usize) -> ModelLedger {
        let mut by_model: BTreeMap<String, Aggregate> = BTreeMap::new();
        let mut by_account: BTreeMap<String, Aggregate> = BTreeMap::new();
        let mut totals = Aggregate::default();
        let (mut reached, mut usage_reported, mut cost_reported) = (0u64, 0u64, 0u64);
        for row in &self.rows {
            totals.add(row);
            by_model
                .entry(
                    row.model_id
                        .clone()
                        .unwrap_or_else(|| UNKNOWN_MODEL.to_string()),
                )
                .or_default()
                .add(row);
            by_account
                .entry(
                    row.account
                        .clone()
                        .unwrap_or_else(|| NO_ACCOUNT.to_string()),
                )
                .or_default()
                .add(row);
            if row.attempts > 0 {
                reached += 1;
                if row.usage().is_some() {
                    usage_reported += 1;
                }
                if row.cost_microusd.is_some() {
                    cost_reported += 1;
                }
            }
        }
        // Dropped refusals reached no model and no account.
        if self.dropped_refusals > 0 {
            let dropped = self.dropped_refusals;
            totals.calls += dropped;
            by_model.entry(UNKNOWN_MODEL.to_string()).or_default().calls += dropped;
            by_account.entry(NO_ACCOUNT.to_string()).or_default().calls += dropped;
        }
        let mut ledger = ModelLedger {
            schema: LEDGER_SCHEMA,
            isolation: ISOLATION,
            calls: Vec::new(),
            rows_omitted: 0,
            refused_rows_dropped: self.dropped_refusals,
            refusals_by_outcome: self.refusals,
            rows_complete: true,
            by_model,
            by_account,
            cost: CostReport {
                total_microusd: totals.cost_microusd,
                cost_source: "reported",
                label: COST_LABEL,
                coverage: Coverage::of(reached, cost_reported),
            },
            totals,
            budget,
            token_coverage: Coverage::of(reached, usage_reported),
        };
        // Size every row once, then keep the longest prefix that fits. The
        // fixed part is measured with the omission count at its widest so
        // writing the real count cannot push the total over.
        let total_rows = self.rows.len() as u64;
        ledger.rows_omitted = total_rows;
        ledger.rows_complete = false;
        let fixed = serde_json::to_vec(&ledger).map(|v| v.len()).unwrap_or(0);
        let mut used = fixed;
        let mut kept = 0usize;
        for row in &self.rows {
            // +1 for the separating comma.
            let len = serde_json::to_vec(row).map(|v| v.len()).unwrap_or(0) + 1;
            if used + len > max_bytes {
                break;
            }
            used += len;
            kept += 1;
        }
        let mut rows = self.rows;
        rows.truncate(kept);
        ledger.rows_omitted = total_rows - kept as u64;
        ledger.rows_complete = ledger.rows_omitted == 0 && ledger.refused_rows_dropped == 0;
        ledger.calls = rows;
        ledger
    }
}

/// The result's `summary.model` block: always present.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ModelSection {
    /// The run ended before the model checks ran (the manifest was not read,
    /// or the job never reached provisioning) and its job declares model
    /// steps — or the job is not known yet. A different statement from
    /// `not_requested`.
    NotEvaluated,
    /// The job has no `model = true` step.
    NotRequested,
    /// Model access was granted; `calls` reached the host.
    /// `ledger_omitted_bytes` is set — and only then serialized — when the
    /// ledger was NOT sent because it serialized to that many bytes, over
    /// coord's ingest cap: a reader of the stored result can tell "dropped
    /// for size" from "lost".
    Granted {
        calls: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        ledger_omitted_bytes: Option<u64>,
    },
    /// Model access was withheld for the closed `reason`; the steps ran
    /// without it.
    Refused { reason: String },
    /// Access was granted, but at least one call failed because every model
    /// account was exhausted past the wall budget.
    /// `ledger_omitted_bytes` as for `Granted`.
    Exhausted {
        calls: u64,
        reset_at: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        ledger_omitted_bytes: Option<u64>,
    },
}

impl ModelSection {
    /// One line for a terminal summary.
    pub fn summary_line(&self) -> String {
        match self {
            ModelSection::NotEvaluated => "model: not evaluated".to_string(),
            ModelSection::NotRequested => "model: not requested".to_string(),
            ModelSection::Granted {
                calls,
                ledger_omitted_bytes,
            } => format!(
                "model: granted ({calls} call(s)){}",
                ledger_note(*ledger_omitted_bytes)
            ),
            ModelSection::Refused { reason } => format!("model: refused ({reason})"),
            ModelSection::Exhausted {
                calls,
                reset_at,
                ledger_omitted_bytes,
            } => format!(
                "model: every account exhausted ({calls} call(s); earliest reset at unix \
                 {reset_at}){}",
                ledger_note(*ledger_omitted_bytes)
            ),
        }
    }
}

/// The summary-line suffix for a ledger dropped for size.
fn ledger_note(omitted: Option<u64>) -> String {
    match omitted {
        Some(bytes) => {
            format!("; ledger not sent: {bytes} bytes, over the {MODEL_LEDGER_MAX_BYTES}-byte cap")
        }
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok_row(index: u64, model: &str, account: &str, tokens: u64, cost: Option<u64>) -> CallRow {
        CallRow {
            index,
            step: "b2".to_string(),
            requested: "claude-sonnet".to_string(),
            model_id: Some(model.to_string()),
            route: Some("subscription".to_string()),
            account: Some(account.to_string()),
            input_tokens: Some(tokens),
            output_tokens: Some(1),
            cache_creation_tokens: Some(0),
            cache_read_tokens: Some(0),
            latency_ms: 10,
            outcome: "ok".to_string(),
            attempts: 1,
            cost_microusd: cost,
        }
    }

    /// Rows for ok, error and refused calls, aggregated by model and account,
    /// with coverage counted over the calls that reached the host only.
    #[test]
    fn rows_for_ok_error_and_refused_aggregate_with_coverage() {
        let mut b = LedgerBuilder::default();
        b.push(ok_row(1, "claude-sonnet-4-5", "acct-a", 10, Some(1500)));
        let mut error = CallRow::refused(2, "b2", "claude-sonnet", "provider_error".to_string());
        error.attempts = 1;
        error.route = Some("subscription".to_string());
        error.account = Some("acct-b".to_string());
        b.push(error);
        b.push(CallRow::refused(
            3,
            "b2",
            "claude-sonnet",
            "budget_exhausted:calls".to_string(),
        ));
        let l = b.finish(BudgetReport {
            max_calls: 2,
            calls_used: 2,
            ..Default::default()
        });
        assert_eq!(l.schema, LEDGER_SCHEMA);
        assert_eq!(l.isolation, "env");
        assert_eq!(l.calls.len(), 3);
        assert!(l.rows_complete);
        assert_eq!(l.totals.calls, 3);
        assert_eq!(l.totals.ok, 1);
        assert_eq!(l.totals.input_tokens, 10);
        assert_eq!(l.by_model["claude-sonnet-4-5"].calls, 1);
        assert_eq!(l.by_model[UNKNOWN_MODEL].calls, 2);
        assert_eq!(l.by_account["acct-b"].calls, 1);
        assert_eq!(l.by_account[NO_ACCOUNT].calls, 1);
        // Two calls reached the host; one reported usage and a cost.
        assert_eq!(l.token_coverage.calls, 2);
        assert_eq!(l.token_coverage.reported, 1);
        assert_eq!(l.token_coverage.share, Some(0.5));
        assert_eq!(l.cost.total_microusd, 1500);
        assert_eq!(l.cost.label, COST_LABEL);
        assert_eq!(l.cost.cost_source, "reported");
        let v = serde_json::to_value(&l).unwrap();
        assert_eq!(v["calls"][2]["outcome"], "budget_exhausted:calls");
        assert!(
            v["calls"][2].get("input_tokens").is_none(),
            "unreported is absent, not 0 and not null"
        );
        assert!(v["calls"][2].get("model_id").is_none());
        for key in ["tenant", "repo", "sha", "lane"] {
            assert!(
                v.get(key).is_none(),
                "the ledger carries no attribution ({key})"
            );
        }
    }

    /// A ledger with no call reaching the host reports coverage as UNKNOWN.
    #[test]
    fn an_empty_ledger_has_unknown_coverage() {
        let l = LedgerBuilder::default().finish(BudgetReport::default());
        assert_eq!(l.token_coverage.share, None);
        let v = serde_json::to_value(&l).unwrap();
        assert!(
            v["token_coverage"].get("share").is_none(),
            "an UNKNOWN share is absent: coord reads a null number as non-finite"
        );
        assert!(l.calls.is_empty());
        assert!(l.rows_complete);
    }

    /// The byte bound: rows are dropped from the end, the omission is said,
    /// and the aggregates still count every call.
    #[test]
    fn the_byte_bound_omits_rows_and_says_so() {
        let mut b = LedgerBuilder::default();
        for i in 1..=500 {
            b.push(ok_row(i, "claude-sonnet-4-5", "acct-a", 7, None));
        }
        let bound = 16 * 1024;
        let l = b.finish_bounded(BudgetReport::default(), bound);
        let size = serde_json::to_vec(&l).unwrap().len();
        assert!(size <= bound, "{size} > {bound}");
        assert!(!l.rows_complete);
        assert!(l.rows_omitted > 0);
        assert_eq!(l.calls.len() as u64 + l.rows_omitted, 500);
        assert_eq!(l.calls.last().unwrap().index, l.calls.len() as u64);
        assert_eq!(l.totals.calls, 500, "aggregates count every call");
        assert_eq!(l.totals.input_tokens, 3500);
    }

    /// Refusals past the stored cap are counted, not stored: memory stays
    /// bounded however long a harness loops on a spent budget.
    #[test]
    fn refusals_past_the_cap_are_counted_not_stored() {
        let mut b = LedgerBuilder::default();
        b.push(ok_row(1, "claude-sonnet-4-5", "acct-a", 7, None));
        let mut last = 0;
        for i in 0..1000u64 {
            last = b.push(CallRow::refused(
                i + 2,
                "b2",
                "claude-sonnet",
                "budget_exhausted:calls".to_string(),
            ));
        }
        assert_eq!(last, 1000, "push reports the running count per outcome");
        assert_eq!(b.rows().len(), 1 + MAX_STORED_REFUSALS);
        let l = b.finish(BudgetReport::default());
        assert_eq!(l.calls.len(), 1 + MAX_STORED_REFUSALS);
        assert_eq!(l.refused_rows_dropped, 1000 - MAX_STORED_REFUSALS as u64);
        assert!(!l.rows_complete);
        assert_eq!(l.refusals_by_outcome["budget_exhausted:calls"], 1000);
        assert_eq!(l.totals.calls, 1001, "the aggregates count every request");
        assert_eq!(l.by_model[UNKNOWN_MODEL].calls, 1000);
        assert_eq!(l.by_account[NO_ACCOUNT].calls, 1000);
    }

    /// The wire contract, byte-for-shape: every row kind (ok with and
    /// without a reported cost, an error that reached the host with nothing
    /// reported, a refusal before the host), both aggregate maps, the
    /// refusal counts, the budget with an overshoot, and fractional
    /// coverage shares.
    fn fixture_ledger() -> ModelLedger {
        let mut b = LedgerBuilder::default();
        b.push(ok_row(1, "claude-sonnet-4-5", "acct-a", 1200, Some(4200)));
        let mut haiku = ok_row(2, "claude-haiku-4-5", "acct-b", 800, None);
        haiku.step = "grade".to_string();
        haiku.requested = "claude-haiku".to_string();
        haiku.cache_read_tokens = Some(512);
        haiku.attempts = 2;
        b.push(haiku);
        let mut error = CallRow::refused(3, "b2", "claude-sonnet", "provider_error".to_string());
        error.route = Some("subscription".to_string());
        error.account = Some("acct-b".to_string());
        error.latency_ms = 30_000;
        error.attempts = 1;
        b.push(error);
        let mut exhausted = CallRow::refused(
            4,
            "b2",
            "claude-sonnet",
            "all_accounts_exhausted".to_string(),
        );
        exhausted.latency_ms = 12;
        exhausted.attempts = 1;
        b.push(exhausted);
        b.push(CallRow::refused(
            5,
            "b2",
            "claude-sonnet",
            "budget_exhausted:tokens".to_string(),
        ));
        b.push(CallRow::refused(
            6,
            "grade",
            "claude-opus",
            "model_not_declared".to_string(),
        ));
        b.finish(BudgetReport {
            max_calls: 400,
            max_tokens: 2000,
            max_wall_secs: 3600,
            calls_used: 4,
            tokens_used: 2512 + 2,
            wall_secs_used: 41,
            tokens_overshoot: 514,
        })
    }

    const LEDGER_FIXTURE: &str = include_str!("../../testdata/model/model_ledger_v1.json");

    /// What this module sends is exactly the fixture coord's ingest test
    /// accepts. On a deliberate shape change, regenerate the fixture from the
    /// panic message AND copy it to qontinui-coord's
    /// `crates/coord/testdata/ci_model_access/model_ledger_v1.json`.
    #[test]
    fn the_ledger_matches_the_shared_wire_fixture() {
        let fixture: serde_json::Value = serde_json::from_str(LEDGER_FIXTURE).unwrap();
        let ours = serde_json::to_value(fixture_ledger()).unwrap();
        assert_eq!(
            ours,
            fixture,
            "the ledger's wire shape drifted from the shared fixture; produced:\n{}",
            serde_json::to_string_pretty(&ours).unwrap()
        );
        assert_eq!(fixture["schema"], LEDGER_SCHEMA);
    }

    /// A ledger whose aggregates alone exceed the bound is not transmittable;
    /// an ordinary one is, unchanged.
    #[test]
    fn only_a_ledger_within_the_bound_is_transmittable() {
        let small = fixture_ledger();
        assert_eq!(small.clone().transmittable(), Ok(small));
        let mut b = LedgerBuilder::default();
        // Distinct ids too long to fit even with every row omitted.
        for i in 1..=600u64 {
            let id = format!("{i:04}-{}", "x".repeat(4096));
            b.push(ok_row(i, &id, "acct-a", 1, None));
        }
        let big = b.finish(BudgetReport::default());
        assert!(big.calls.is_empty(), "every row was omitted");
        let size = big.serialized_len();
        assert!(size > MODEL_LEDGER_MAX_BYTES);
        assert_eq!(big.transmittable(), Err(size));
    }

    #[test]
    fn cost_conversion_refuses_what_is_not_a_cost() {
        assert_eq!(usd_to_microusd(0.0123), Some(12_300));
        assert_eq!(usd_to_microusd(-1.0), None);
        assert_eq!(usd_to_microusd(f64::NAN), None);
        assert_eq!(usd_to_microusd(f64::INFINITY), None);
    }

    /// `summary.model` stays a small block (plan R6): coord caps the whole
    /// stored summary at 64 KiB and replaces an oversize one, step rows
    /// included. Its largest form — the longest closed reason, the widest
    /// integers — is a few dozen bytes; 256 is the bound this test holds.
    #[test]
    fn the_section_is_small_in_every_form() {
        let reasons = crate::model::COORD_REFUSALS
            .iter()
            .chain(crate::model::RUNNER_REFUSALS);
        let mut forms: Vec<ModelSection> = reasons
            .map(|r| ModelSection::Refused {
                reason: r.to_string(),
            })
            .collect();
        forms.extend([
            ModelSection::NotEvaluated,
            ModelSection::NotRequested,
            ModelSection::Granted {
                calls: u64::MAX,
                ledger_omitted_bytes: Some(u64::MAX),
            },
            ModelSection::Exhausted {
                calls: u64::MAX,
                reset_at: u64::MAX,
                ledger_omitted_bytes: Some(u64::MAX),
            },
        ]);
        for form in forms {
            let n = serde_json::to_vec(&form).unwrap().len();
            assert!(n <= 256, "{form:?} serializes to {n} bytes");
        }
    }

    #[test]
    fn the_section_speaks_the_wire_vocabulary() {
        let v = |s: ModelSection| serde_json::to_value(s).unwrap();
        assert_eq!(v(ModelSection::NotRequested)["status"], "not_requested");
        assert_eq!(v(ModelSection::NotEvaluated)["status"], "not_evaluated");
        let granted = v(ModelSection::Granted {
            calls: 2,
            ledger_omitted_bytes: None,
        });
        assert_eq!(granted["status"], "granted");
        assert!(
            granted.get("ledger_omitted_bytes").is_none(),
            "the marker is serialized only when set: {granted}"
        );
        let dropped = ModelSection::Granted {
            calls: 2,
            ledger_omitted_bytes: Some(2_000_000),
        };
        assert_eq!(v(dropped.clone())["ledger_omitted_bytes"], 2_000_000);
        assert!(
            dropped
                .summary_line()
                .contains("ledger not sent: 2000000 bytes"),
            "{}",
            dropped.summary_line()
        );
        let refused = v(ModelSection::Refused {
            reason: "fork_pr".to_string(),
        });
        assert_eq!(refused["status"], "refused");
        assert_eq!(refused["reason"], "fork_pr");
        let ex = v(ModelSection::Exhausted {
            calls: 1,
            reset_at: 9,
            ledger_omitted_bytes: None,
        });
        assert_eq!(ex["status"], "exhausted");
        assert_eq!(ex["reset_at"], 9);
        assert!(ex.get("ledger_omitted_bytes").is_none());
        let ex_dropped = ModelSection::Exhausted {
            calls: 1,
            reset_at: 9,
            ledger_omitted_bytes: Some(7),
        };
        assert_eq!(v(ex_dropped.clone())["ledger_omitted_bytes"], 7);
        assert!(ex_dropped
            .summary_line()
            .contains("ledger not sent: 7 bytes"));
    }
}
