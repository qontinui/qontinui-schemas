//! Journey-graph queries — pure, I/O-free, deterministic reads over OBSERVED
//! edges and the open frontier (plan
//! `2026-09-20-ui-bridge-represents-the-users-path-and-the-passage-of-time`,
//! D2 and Phase 2).
//!
//! [`build_graph`] turns ledger rows ([`JourneyEdgeObservation`],
//! [`FrontierEntry`]) into a [`JourneyGraph`]; [`reachable_from`],
//! [`steps_to`], [`dead_ends`] and [`requires_prior_knowledge`] read it.
//! Over observed edges every endpoint is already a concrete configuration, so
//! each query is a plain directed-graph read: BFS, BFS with depth,
//! out-degree zero, in-degree zero over affordance edges.
//!
//! ## The navigation edge set
//!
//! Reachability, step counts and dead ends read ONE edge set: graph edges
//! whose `trigger.navigationTrigger` is `affordance` AND whose `outcome` is
//! `changed` — a user clicking something and the configuration changing.
//! A browser back (`pop`), a typed URL (`push`/`replace`/`hash`) or a reload
//! (`initial`) is not something the app offers a user, so it is not a path.
//! The parity of these reads with multistate's reference implementation
//! (`graph_utils.py`) is pinned by `tests/journey_golden.rs`.
//!
//! ## Unexplored is not unreachable
//!
//! These functions return FACTS, not verdicts. A negative fact (no path, no
//! way out, no affordance arrival) is only a negative ANSWER when nothing
//! that was not looked at could change it. So every result carries a
//! [`JourneySearchFrontier`] naming what was not looked at inside the part of
//! the graph the query searched: the open frontier entries, grouped by
//! reason, the departures whose destination was never observed, and the
//! departures the navigation edge set left out (filtered, withdrawn,
//! rejected, `settle_timeout` / `error` to another node). The caller (the
//! runner) builds the `Observation<T>` envelope from those facts:
//! [`JourneySearchFrontier::unmeasured`]`(&coverage, subject_observed)` goes
//! into `provenance.coverage.unmeasured`, and it ALSO names a ledger that is
//! not `writing`, an empty ledger (`no_observations`) and an unobserved
//! subject (`subject_unobserved`) — so the shared `Observation::absent`
//! constructor refuses every negative it must not settle, and the answer is
//! `unknown` with [`JourneySearchFrontier::unsettled_because`]'s code (the
//! first entry's). The gate is complete: `unsettled_because(c, s)` is `None`
//! exactly when `is_settled(c, s)`. This crate does not
//! depend on `qontinui-vision-core` (which owns the envelope and pulls
//! `image`), so the wrapping is the caller's.
//!
//! Every result also repeats the [`JourneyCoverage`] the graph was built over
//! and the [`JourneyFilter`] that was applied (`filterEcho`), so a dropped
//! filter or an empty ledger is visible in the answer itself.
//!
//! ## Determinism
//!
//! Same rows in any order, same filter → byte-identical serialized output.
//! Every collection is a `BTreeMap`/`BTreeSet` or a sorted `Vec`; ties are
//! broken by byte-wise key order; where several rows describe one node, the
//! smallest node (by its fields, in declaration order) represents it; no
//! clock is read. `HashMap` iteration order never reaches an output.
//!
//! ## Rows that break the contract
//!
//! An edge or frontier row that fails its own `validate()` (or whose
//! `observedAt` is not RFC 3339) is not silently dropped and does not fail
//! the whole read: it is listed in [`JourneyGraph::rejected`] and counted in
//! [`JourneyCoverage::rows_rejected`]. Withdrawn edges (`invalidatedAt` set)
//! are excluded and counted in [`JourneyCoverage::edges_invalidated`].
//!
//! The caller passes ONE app's rows; nothing here partitions by `appId`.
//!
//! ## Wire format
//!
//! Structs are `camelCase`; the one enum is `#[schemars(inline)]`, like every
//! journey enum. Optional fields of these RESULT types always serialize
//! (`null` is a statement: "none observed" / "not applied"), and maps keyed
//! by frontier reason are `BTreeMap<String, _>` keyed by the reason's wire
//! string.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use chrono::{DateTime, SecondsFormat, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{
    EdgeOutcome, FrontierEntry, FrontierReason, JourneyEdgeObservation, JourneyLedgerHealth,
    JourneyNode, JourneyTrigger, LedgerState, NavigationTriggerKind, RunKind,
};

// ---------------------------------------------------------------------------
// Filter
// ---------------------------------------------------------------------------

/// Narrows the EDGE set a graph is built from. Staleness is a filter, not a
/// verdict: the caller decides what window to ask about, and every result
/// echoes the filter it was computed under (`filterEcho`).
///
/// The frontier is never filtered: an open frontier entry is current state,
/// and hiding one behind an edge filter would turn "not looked at" into
/// "nothing there".
///
/// `since` is a `chrono::DateTime<Utc>` (serialized RFC 3339), an exception
/// to the crate's string-timestamp convention for the same reason as
/// `page::KeysetPosition`: it is compared, not merely carried.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct JourneyFilter {
    /// Keep edges with `observedAt >= since`; `None` = no lower bound.
    #[serde(default)]
    #[schemars(with = "Option<String>")]
    pub since: Option<DateTime<Utc>>,
    /// Keep edges whose `appVersion` equals this; `None` = every version.
    /// An edge that reported no version never matches a set filter.
    #[serde(default)]
    pub app_version: Option<String>,
    /// Keep edges of this run kind; `None` = every kind.
    #[serde(default)]
    pub run_kind: Option<RunKind>,
}

impl JourneyFilter {
    fn keeps(&self, edge: &JourneyEdgeObservation, observed_at: DateTime<Utc>) -> bool {
        if self.since.is_some_and(|since| observed_at < since) {
            return false;
        }
        if let Some(version) = &self.app_version {
            if edge.app_version.as_deref() != Some(version.as_str()) {
                return false;
            }
        }
        if self.run_kind.is_some_and(|kind| edge.run_kind != kind) {
            return false;
        }
        true
    }
}

// ---------------------------------------------------------------------------
// Graph
// ---------------------------------------------------------------------------

/// One observed vertex.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct JourneyGraphNode {
    /// [`JourneyNode::key`] — the node's identity.
    pub key: String,
    /// A representative node for the key (the smallest by its fields when
    /// rows disagree on non-identity detail such as `pageLabel`).
    pub node: JourneyNode,
    /// Open frontier entries seen at this node, sorted by
    /// `affordanceFingerprint`.
    pub frontier: Vec<FrontierEntry>,
    /// How many edges left this node with `outcome: to_node_unobserved` —
    /// actions whose destination was never seen.
    pub unobserved_departures: u64,
    /// Departures from this node the navigation edge set leaves out but that
    /// may still lead somewhere (see [`JourneyUnmeasuredDepartures`]).
    pub unmeasured_departures: JourneyUnmeasuredDepartures,
}

/// Departures from one node that the NAVIGATION edge set (affordance +
/// `changed`) leaves out, yet that may still lead somewhere — so a negative
/// answer over a component containing the node stays `unknown`, never
/// `absent`. The navigation counters count only `affordance`-triggered rows
/// (a typed URL is not a way the app offers); excluded back-button (`pop`)
/// rows have their own counter, [`Self::excluded_back_arrivals`]. A filtered
/// or withdrawn `no_change` row is never counted: that exclusion is a genuine
/// negative (a REJECTED row counts whatever its outcome). Each counter is its own
/// `coverage.unmeasured` dimension — see [`JourneySearchFrontier::unmeasured`]
/// for the dimension names and codes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct JourneyUnmeasuredDepartures {
    /// Live, valid affordance rows the request's filter (`since` /
    /// `appVersion` / `runKind`) excluded, outcome not `no_change`. Dimension
    /// `filtered_departures`.
    pub filtered_out: u64,
    /// Withdrawn affordance rows (`invalidatedAt` set) that pass `validate()`
    /// and whose outcome is not `no_change`. Dimension `withdrawn_departures`.
    pub withdrawn: u64,
    /// Rows that failed the contract (`validate()` or an unparseable
    /// `observedAt`) — counted whatever their trigger AND outcome, since none
    /// of the row's claims can be trusted (a row claiming `push` may really
    /// be a click). A withdrawn row that also fails `validate()` counts here
    /// too. Dimension `rejected_departures`.
    pub rejected: u64,
    /// `settle_timeout` edges to a DIFFERENT node: not a navigation step (the
    /// destination may be an intermediate render) but not a negative either.
    /// Dimension `settle_timeout_departures`.
    pub settle_timeout: u64,
    /// `error` edges to a DIFFERENT node: the action errored yet the
    /// configuration moved. Dimension `errored_departures`.
    pub errored: u64,
    /// Frontier rows at this node that failed `validate()` — treated as OPEN
    /// entries the read cannot list. A row whose `nodeKey` column disagrees
    /// with its `node` is counted at BOTH keys (the column's, and the valid
    /// node's), since either may be the true one. Dimension
    /// `rejected_frontier`.
    pub rejected_frontier: u64,
    /// Valid URL-trigger rows (`initial` / `push` / `replace` / `hash`) from
    /// this node that the filter excluded or that were withdrawn, outcome not
    /// `no_change`. A typed URL is not a navigation departure, so a question
    /// about one component ignores it; but it is an ARRIVAL that decides
    /// `requires_prior_knowledge`, and it may reveal a node no kept row
    /// shows, so the graph-wide questions (`requires_prior_knowledge`,
    /// `dead_ends`, the whole frontier) count it. Dimension
    /// `excluded_url_arrivals`.
    pub excluded_url_arrivals: u64,
    /// Back-button (`pop`) rows from this node that are not a kept, changed
    /// exit: valid `pop` rows the filter excluded or that were withdrawn
    /// (outcome not `no_change`), and kept `pop` edges to ANOTHER node whose
    /// outcome is `error` or `settle_timeout` (the same rule as an affordance
    /// edge's). A REJECTED pop row is not here — a rejected row's trigger
    /// cannot be trusted, so it counts as `rejected`. Not a navigation
    /// departure, so a question about one component ignores it; the
    /// graph-wide questions it can change — `dead_ends`' `backOnly`,
    /// `requires_prior_knowledge`, the whole frontier — count it. Dimension
    /// `excluded_back_arrivals`.
    pub excluded_back_arrivals: u64,
}

impl JourneyUnmeasuredDepartures {
    /// True when every counter is zero.
    pub fn is_zero(&self) -> bool {
        *self == Self::default()
    }

    /// The counters a NAVIGATION question reads: everything but
    /// `excludedBackArrivals` and `excludedUrlArrivals`.
    fn for_navigation(&self) -> Self {
        Self {
            excluded_back_arrivals: 0,
            excluded_url_arrivals: 0,
            ..*self
        }
    }

    fn add(&mut self, o: &Self) {
        self.excluded_back_arrivals += o.excluded_back_arrivals;
        self.excluded_url_arrivals += o.excluded_url_arrivals;
        self.filtered_out += o.filtered_out;
        self.withdrawn += o.withdrawn;
        self.rejected += o.rejected;
        self.settle_timeout += o.settle_timeout;
        self.errored += o.errored;
        self.rejected_frontier += o.rejected_frontier;
    }
}

/// Every observation of one `(from, to, trigger, outcome)` combination,
/// aggregated. `to_node_unobserved` rows are not graph edges (they have no
/// destination); they count on the from-node's `unobservedDepartures`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct JourneyGraphEdge {
    /// `fromNode.key()`.
    pub from_key: String,
    /// `toNode.key()`.
    pub to_key: String,
    /// The trigger, exactly as recorded — enough to replay the step.
    pub trigger: JourneyTrigger,
    /// How the edge closed.
    pub outcome: EdgeOutcome,
    /// How many ledger rows this edge aggregates (≥ 1).
    pub observations: u64,
    /// Earliest `observedAt` among them, RFC 3339 UTC.
    pub first_observed_at: String,
    /// Latest `observedAt` among them, RFC 3339 UTC.
    pub last_observed_at: String,
}

impl JourneyGraphEdge {
    fn is_navigation(&self) -> bool {
        self.trigger.navigation_trigger == NavigationTriggerKind::Affordance
            && self.outcome == EdgeOutcome::Changed
    }
}

/// Which kind of ledger row was rejected.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[schemars(inline)]
#[serde(rename_all = "snake_case")]
pub enum JourneyRejectedRowKind {
    /// A `journey_edge_observations` row.
    Edge,
    /// A `journey_frontier` row.
    Frontier,
}

/// A ledger row that broke the edge-observation contract and was left out of
/// the graph — reported, never silently dropped.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct JourneyRejectedRow {
    /// Edge or frontier row.
    pub kind: JourneyRejectedRowKind,
    /// The edge's `id`, or `"<nodeKey> <affordanceFingerprint>"` for a
    /// frontier row.
    pub row: String,
    /// The contract violation, as `JourneyContractError` renders it (or the
    /// timestamp parse failure).
    pub reason: String,
}

/// What a graph was built over — the resolution and freshness of every answer
/// read from it. Rides inside the runner's `provenance.coverage`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct JourneyCoverage {
    /// Distinct nodes (by key) seen as an edge endpoint, as the from-node of
    /// an unobserved departure, or as a frontier node.
    pub nodes_observed: u64,
    /// How many of those are unmodelled (`modelled: false`) — the graph's
    /// resolution, as a number.
    pub nodes_unmodelled: u64,
    /// Edge rows the graph was built from (after exclusions), INCLUDING
    /// `to_node_unobserved` rows.
    pub edges_observed: u64,
    /// How many of `edgesObserved` closed as `to_node_unobserved`.
    pub edges_to_node_unobserved: u64,
    /// Withdrawn edge rows (`invalidatedAt` set), excluded.
    pub edges_invalidated: u64,
    /// Valid, live edge rows the filter excluded.
    pub edges_filtered_out: u64,
    /// Edge and frontier rows rejected as contract violations (listed in
    /// [`JourneyGraph::rejected`]).
    pub rows_rejected: u64,
    /// Open frontier entries across the graph.
    pub frontier_open: u64,
    /// Nodes with at least one open frontier entry.
    pub frontier_nodes: u64,
    /// Open frontier entries per reason — every reason present, zeros
    /// included.
    pub frontier_by_reason: BTreeMap<String, u64>,
    /// Distinct reported `runId`s among the edges observed.
    pub runs: u64,
    /// Edges observed that reported no `runId`.
    pub edges_without_run_id: u64,
    /// Earliest `observedAt` among the edges observed; `null` = no edges.
    pub oldest_edge_at: Option<String>,
    /// Latest `observedAt` among the edges observed; `null` = no edges.
    pub newest_edge_at: Option<String>,
    /// Distinct `appVersion`s among the edges observed, sorted, `null`
    /// (= not reported) first.
    pub app_versions: Vec<Option<String>>,
    /// Whether the ledger these rows were read from is being written — the
    /// runner's `/journey/health` block, repeated here so a graph that
    /// stopped growing (`schemaAbsent` / `writeFailing`) can never read as a
    /// finished one in any query response.
    pub ledger: JourneyLedgerHealth,
}

impl JourneyCoverage {
    /// The `UnknownCode` wire string a NEGATIVE answer must carry because of
    /// the ledger's state, or `None` when the ledger is `writing`.
    pub fn ledger_unknown_code(&self) -> Option<&'static str> {
        match self.ledger.state {
            LedgerState::Writing => None,
            LedgerState::SchemaAbsent => Some("ledger_schema_absent"),
            LedgerState::WriteFailing => Some("ledger_write_failing"),
        }
    }
}

/// The journey graph derived from ledger rows under one filter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct JourneyGraph {
    /// The filter the edge set was narrowed by.
    pub filter_echo: JourneyFilter,
    /// Observed nodes, sorted by key.
    pub nodes: Vec<JourneyGraphNode>,
    /// Aggregated edges, sorted by `(fromKey, toKey, trigger, outcome)`.
    pub edges: Vec<JourneyGraphEdge>,
    /// Rows left out as contract violations, sorted.
    pub rejected: Vec<JourneyRejectedRow>,
    /// Unmeasured departures (filtered, withdrawn, rejected, rejected
    /// frontier, excluded back arrivals) placed at a node key that is NOT in
    /// `nodes`. No component query can reach such a node, but a graph-wide
    /// question (`dead_ends`, `requires_prior_knowledge`, the whole frontier)
    /// counts them.
    pub off_graph_departures: JourneyUnmeasuredDepartures,
    /// Untrusted edge rows — live rows that failed `validate()` or carry an
    /// unparseable `observedAt`, and withdrawn rows failing either check —
    /// whose `fromNode` itself is invalid, so they cannot be placed at any
    /// node. Their trigger cannot be trusted, so each may be a click
    /// anywhere: EVERY negative answer counts them, component and graph-wide
    /// alike (dimension `unplaced_rejected_rows`).
    pub unplaced_rejected_rows: u64,
    /// Distinct rejected frontier rows whose node is invalid AND whose
    /// `nodeKey` column names no observed node, so they cannot be placed at
    /// any node: each may be an open affordance anywhere, and EVERY negative
    /// answer counts them (dimension `unplaced_rejected_frontier`).
    pub unplaced_rejected_frontier: u64,
    /// Rejected frontier rows counted at TWO node keys (their `nodeKey`
    /// column and their valid node's own key disagree). Per-node counts
    /// include both placements; graph-wide totals subtract this, so a
    /// graph-wide `rejectedFrontier` plus `unplacedRejectedFrontier` is the
    /// number of DISTINCT rejected frontier rows in `rejected`.
    pub rejected_frontier_double_placed: u64,
    /// What the graph was built over.
    pub coverage: JourneyCoverage,
}

impl JourneyGraph {
    /// The node with this key, if observed.
    pub fn node(&self, key: &str) -> Option<&JourneyGraphNode> {
        self.nodes
            .binary_search_by(|n| n.key.as_str().cmp(key))
            .ok()
            .map(|i| &self.nodes[i])
    }

    /// The navigation adjacency (affordance + changed), from-key → to-keys.
    fn navigation_adjacency(&self) -> BTreeMap<&str, BTreeSet<&str>> {
        let mut adj: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
        for e in self.edges.iter().filter(|e| e.is_navigation()) {
            adj.entry(e.from_key.as_str())
                .or_default()
                .insert(e.to_key.as_str());
        }
        adj
    }

    /// The reverse navigation adjacency, to-key → from-keys.
    fn reverse_navigation_adjacency(&self) -> BTreeMap<&str, BTreeSet<&str>> {
        let mut adj: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
        for e in self.edges.iter().filter(|e| e.is_navigation()) {
            adj.entry(e.to_key.as_str())
                .or_default()
                .insert(e.from_key.as_str());
        }
        adj
    }

    /// The frontier summary over the given node keys. `graph_wide` adds the
    /// off-graph departures; the unplaced rejected rows are always added.
    fn search_frontier<'a>(
        &self,
        keys: impl IntoIterator<Item = &'a str>,
        graph_wide: bool,
    ) -> JourneySearchFrontier {
        let mut summary = JourneySearchFrontier::empty();
        summary.unplaced_rejected_rows = self.unplaced_rejected_rows;
        summary.unplaced_rejected_frontier = self.unplaced_rejected_frontier;
        if graph_wide {
            summary.off_graph_departures = self.off_graph_departures;
            summary
                .unmeasured_departures
                .add(&self.off_graph_departures);
        }
        for key in keys {
            let Some(node) = self.node(key) else { continue };
            let unmeasured = if graph_wide {
                node.unmeasured_departures
            } else {
                node.unmeasured_departures.for_navigation()
            };
            if node.frontier.is_empty() && node.unobserved_departures == 0 && unmeasured.is_zero() {
                continue;
            }
            let mut open_by_reason: BTreeMap<String, Vec<FrontierEntry>> = BTreeMap::new();
            for entry in &node.frontier {
                open_by_reason
                    .entry(entry.reason.as_str().to_string())
                    .or_default()
                    .push(entry.clone());
                *summary
                    .by_reason
                    .get_mut(entry.reason.as_str())
                    .expect("every reason is pre-seeded") += 1;
                summary.open_entries += 1;
            }
            summary.unobserved_departures += node.unobserved_departures;
            summary.unmeasured_departures.add(&unmeasured);
            summary.nodes.push(JourneySearchFrontierNode {
                node_key: node.key.clone(),
                open_by_reason,
                unobserved_departures: node.unobserved_departures,
                unmeasured_departures: unmeasured,
            });
        }
        if graph_wide {
            // A row placed at two keys is still one row.
            summary.unmeasured_departures.rejected_frontier -= self.rejected_frontier_double_placed;
        }
        summary.nodes.sort_by(|a, b| a.node_key.cmp(&b.node_key));
        summary
    }

    fn all_keys(&self) -> impl Iterator<Item = &str> {
        self.nodes.iter().map(|n| n.key.as_str())
    }
}

/// `(spec_id, state_ids, modelled, pathname_template, page_label)` — the
/// order a representative node is chosen by.
type NodeOrder<'a> = (
    &'a Option<String>,
    &'a Vec<String>,
    bool,
    &'a Option<String>,
    &'a Option<String>,
);

impl JourneyGraphNode {
    /// No open frontier entry, no unobserved departure and no unmeasured
    /// departure: everything that leaves this node was observed.
    fn is_fully_explored(&self) -> bool {
        self.frontier.is_empty()
            && self.unobserved_departures == 0
            && self.unmeasured_departures.for_navigation().is_zero()
    }
}

fn node_order(n: &JourneyNode) -> NodeOrder<'_> {
    (
        &n.spec_id,
        &n.state_ids,
        n.modelled,
        &n.pathname_template,
        &n.page_label,
    )
}

/// The aggregation and sort key of a graph edge.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct EdgeKey {
    from: String,
    to: String,
    action_type: String,
    target_fingerprint: Option<String>,
    target_role: Option<String>,
    declared_effect: Option<String>,
    navigation_trigger: &'static str,
    choke_point: &'static str,
    outcome: &'static str,
}

impl EdgeKey {
    fn new(from: String, to: String, t: &JourneyTrigger, outcome: EdgeOutcome) -> Self {
        Self {
            from,
            to,
            action_type: t.action_type.clone(),
            target_fingerprint: t.target_fingerprint.clone(),
            target_role: t.target_role.clone(),
            declared_effect: t
                .declared_effect
                .map(|e| serde_json::to_string(&e).expect("IrEffect serializes")),
            navigation_trigger: t.navigation_trigger.as_str(),
            choke_point: t.choke_point.as_str(),
            outcome: outcome.as_str(),
        }
    }
}

/// One aggregated edge while the graph is built: (trigger, outcome,
/// observations, first observedAt, last observedAt).
type EdgeAgg = (
    JourneyTrigger,
    EdgeOutcome,
    u64,
    DateTime<Utc>,
    DateTime<Utc>,
);

fn rfc3339(t: DateTime<Utc>) -> String {
    t.to_rfc3339_opts(SecondsFormat::AutoSi, true)
}

fn keep_smallest(slot: &mut Option<JourneyNode>, candidate: &JourneyNode) {
    match slot {
        Some(current) if node_order(current) <= node_order(candidate) => {}
        _ => *slot = Some(candidate.clone()),
    }
}

/// Derive the journey graph from one app's ledger rows under `filter`.
///
/// Pure and order-independent: the same rows in any order give an equal
/// graph. Withdrawn edges are excluded and counted; rows that fail their
/// contract `validate()` are rejected and listed; the filter narrows edges
/// only (see [`JourneyFilter`]). `ledger` is the state of the store the rows
/// came from; it is carried into [`JourneyCoverage::ledger`] and so into
/// every query result.
pub fn build_graph(
    edges: &[JourneyEdgeObservation],
    frontier: &[FrontierEntry],
    ledger: &JourneyLedgerHealth,
    filter: &JourneyFilter,
) -> JourneyGraph {
    let mut rejected: BTreeSet<JourneyRejectedRow> = BTreeSet::new();
    let mut node_repr: BTreeMap<String, Option<JourneyNode>> = BTreeMap::new();
    let mut unobserved: BTreeMap<String, u64> = BTreeMap::new();
    let mut agg: BTreeMap<EdgeKey, EdgeAgg> = BTreeMap::new();

    let mut edges_observed = 0u64;
    let mut edges_to_node_unobserved = 0u64;
    let mut edges_invalidated = 0u64;
    let mut edges_filtered_out = 0u64;
    let mut runs: BTreeSet<&str> = BTreeSet::new();
    let mut edges_without_run_id = 0u64;
    let mut oldest: Option<DateTime<Utc>> = None;
    let mut newest: Option<DateTime<Utc>> = None;
    let mut app_versions: BTreeSet<Option<&str>> = BTreeSet::new();

    // Unmeasured departures by from-node key, attached to graph nodes (or
    // folded into `off_graph`) once the node set is known.
    let mut excluded: BTreeMap<String, JourneyUnmeasuredDepartures> = BTreeMap::new();
    let mut unplaced_rejected_rows = 0u64;
    let mut rejected_frontier_double_placed = 0u64;
    // How one excluded edge row is counted.
    //
    // - A row that FAILED `validate()` (`trusted == false`): none of its
    //   claims can be trusted, its trigger included, so it counts as
    //   `rejected` whatever its trigger or outcome — at its fromNode when
    //   that is valid, else as an unplaced row every negative counts.
    // - A row that passed `validate()` (filtered, or withdrawn): its trigger
    //   decides. An affordance row counts by its cause (`bump`), a pop row
    //   as an excluded back arrival, a URL-trigger row (initial / push /
    //   replace / hash) as an excluded URL arrival. A `no_change` row is a
    //   genuine negative and is skipped by the callers.
    let mut place =
        |e: &JourneyEdgeObservation, trusted: bool, bump: fn(&mut JourneyUnmeasuredDepartures)| {
            let bump: fn(&mut JourneyUnmeasuredDepartures) = if !trusted {
                |d| d.rejected += 1
            } else {
                match e.trigger.navigation_trigger {
                    NavigationTriggerKind::Affordance => bump,
                    NavigationTriggerKind::Pop => |d| d.excluded_back_arrivals += 1,
                    NavigationTriggerKind::Initial
                    | NavigationTriggerKind::Push
                    | NavigationTriggerKind::Replace
                    | NavigationTriggerKind::Hash => |d| d.excluded_url_arrivals += 1,
                }
            };
            if e.from_node.validate().is_ok() {
                bump(excluded.entry(e.from_node.key()).or_default());
            } else {
                unplaced_rejected_rows += 1;
            }
        };

    for edge in edges {
        if edge.invalidated_at.is_some() {
            edges_invalidated += 1;
            // A withdrawn row is judged like a live one: failing validate()
            // OR carrying an unparseable observedAt makes it untrusted.
            if edge.validate().is_err() || DateTime::parse_from_rfc3339(&edge.observed_at).is_err()
            {
                place(edge, false, |_| {});
            } else if edge.outcome != EdgeOutcome::NoChange {
                place(edge, true, |d| d.withdrawn += 1);
            }
            continue;
        }
        if let Err(err) = edge.validate() {
            // Counted once per DISTINCT rejected row, so the counters agree
            // with `rows_rejected`.
            if rejected.insert(JourneyRejectedRow {
                kind: JourneyRejectedRowKind::Edge,
                row: edge.id.clone(),
                reason: err.to_string(),
            }) {
                place(edge, false, |_| {});
            }
            continue;
        }
        let observed_at = match DateTime::parse_from_rfc3339(&edge.observed_at) {
            Ok(t) => t.with_timezone(&Utc),
            Err(err) => {
                if rejected.insert(JourneyRejectedRow {
                    kind: JourneyRejectedRowKind::Edge,
                    row: edge.id.clone(),
                    reason: format!("observedAt {:?} is not RFC 3339: {err}", edge.observed_at),
                }) {
                    place(edge, false, |_| {});
                }
                continue;
            }
        };
        if !filter.keeps(edge, observed_at) {
            edges_filtered_out += 1;
            if edge.outcome != EdgeOutcome::NoChange {
                place(edge, true, |d| d.filtered_out += 1);
            }
            continue;
        }

        edges_observed += 1;
        match &edge.run_id {
            Some(run) => {
                runs.insert(run.as_str());
            }
            None => edges_without_run_id += 1,
        }
        oldest = Some(oldest.map_or(observed_at, |o| o.min(observed_at)));
        newest = Some(newest.map_or(observed_at, |n| n.max(observed_at)));
        app_versions.insert(edge.app_version.as_deref());

        let from_key = edge.from_node.key();
        keep_smallest(
            node_repr.entry(from_key.clone()).or_default(),
            &edge.from_node,
        );
        match &edge.to_node {
            None => {
                edges_to_node_unobserved += 1;
                *unobserved.entry(from_key).or_default() += 1;
            }
            Some(to) => {
                let to_key = to.key();
                keep_smallest(node_repr.entry(to_key.clone()).or_default(), to);
                let key = EdgeKey::new(from_key, to_key, &edge.trigger, edge.outcome);
                agg.entry(key)
                    .and_modify(|(_, _, n, first, last)| {
                        *n += 1;
                        *first = (*first).min(observed_at);
                        *last = (*last).max(observed_at);
                    })
                    .or_insert_with(|| {
                        (
                            edge.trigger.clone(),
                            edge.outcome,
                            1,
                            observed_at,
                            observed_at,
                        )
                    });
            }
        }
    }

    // Frontier: validated, deduplicated on the table's primary key (keeping
    // the greatest row by its serialized form, so duplicates in the input
    // cannot make the result order-dependent).
    let mut frontier_rows: BTreeMap<(String, String), (String, FrontierEntry)> = BTreeMap::new();
    // Distinct rejected frontier rows: (nodeKey column, the node's key when
    // the node is valid). Placed once the node set is known.
    let mut rejected_frontier_rows: Vec<(String, Option<String>)> = Vec::new();
    for entry in frontier {
        if let Err(err) = entry.validate() {
            let distinct = rejected.insert(JourneyRejectedRow {
                kind: JourneyRejectedRowKind::Frontier,
                row: format!("{} {}", entry.node_key, entry.affordance_fingerprint),
                reason: err.to_string(),
            });
            if distinct {
                let node_key = entry.node.validate().is_ok().then(|| entry.node.key());
                rejected_frontier_rows.push((entry.node_key.clone(), node_key));
            }
            continue;
        }
        let text = serde_json::to_string(entry).expect("FrontierEntry serializes");
        let key = (entry.node_key.clone(), entry.affordance_fingerprint.clone());
        match frontier_rows.get(&key) {
            Some((existing, _)) if *existing >= text => {}
            _ => {
                frontier_rows.insert(key, (text, entry.clone()));
            }
        }
    }
    let mut frontier_by_node: BTreeMap<String, Vec<FrontierEntry>> = BTreeMap::new();
    for ((node_key, _), (_, entry)) in frontier_rows {
        keep_smallest(node_repr.entry(node_key.clone()).or_default(), &entry.node);
        frontier_by_node.entry(node_key).or_default().push(entry);
    }

    // A rejected frontier row is not listable, but not explored either: an
    // OPEN entry. It is counted at its `nodeKey` column ALWAYS, and also at
    // its node's key when that node is valid and the key differs — either may
    // be the node the affordance really belongs to. A row whose node is
    // INVALID and whose key column names no observed node cannot be placed
    // at all: it is unplaced, and every negative counts it.
    let mut unplaced_rejected_frontier = 0u64;
    for (column, node_key) in rejected_frontier_rows {
        match node_key {
            None if !node_repr.contains_key(&column) => unplaced_rejected_frontier += 1,
            None => excluded.entry(column).or_default().rejected_frontier += 1,
            Some(k) => {
                if k != column {
                    excluded.entry(column).or_default().rejected_frontier += 1;
                    rejected_frontier_double_placed += 1;
                }
                excluded.entry(k).or_default().rejected_frontier += 1;
            }
        }
    }

    let mut frontier_by_reason: BTreeMap<String, u64> = FrontierReason::ALL
        .iter()
        .map(|r| (r.as_str().to_string(), 0))
        .collect();
    let mut frontier_open = 0u64;
    let mut nodes_unmodelled = 0u64;
    let nodes: Vec<JourneyGraphNode> = node_repr
        .into_iter()
        .map(|(key, node)| {
            let node = node.expect("every key was inserted with a node");
            if !node.modelled {
                nodes_unmodelled += 1;
            }
            // Already sorted by fingerprint: the BTreeMap key's second half.
            let frontier = frontier_by_node.remove(&key).unwrap_or_default();
            for entry in &frontier {
                frontier_open += 1;
                *frontier_by_reason
                    .get_mut(entry.reason.as_str())
                    .expect("every reason is pre-seeded") += 1;
            }
            JourneyGraphNode {
                unobserved_departures: unobserved.get(&key).copied().unwrap_or(0),
                unmeasured_departures: excluded.remove(&key).unwrap_or_default(),
                key,
                node,
                frontier,
            }
        })
        .collect();
    let mut off_graph_departures = JourneyUnmeasuredDepartures::default();
    for rest in excluded.values() {
        off_graph_departures.add(rest);
    }
    let frontier_nodes = nodes.iter().filter(|n| !n.frontier.is_empty()).count() as u64;

    let graph_edges: Vec<JourneyGraphEdge> = agg
        .into_iter()
        .map(
            |(key, (trigger, outcome, observations, first, last))| JourneyGraphEdge {
                from_key: key.from,
                to_key: key.to,
                trigger,
                outcome,
                observations,
                first_observed_at: rfc3339(first),
                last_observed_at: rfc3339(last),
            },
        )
        .collect();

    // settle_timeout / error edges to a DIFFERENT node: excluded from
    // navigation (affordance) or from the kept back exits (pop), never a
    // negative.
    let mut nodes = nodes;
    for e in &graph_edges {
        if e.from_key == e.to_key {
            continue;
        }
        let Ok(i) = nodes.binary_search_by(|n| n.key.cmp(&e.from_key)) else {
            continue;
        };
        let d = &mut nodes[i].unmeasured_departures;
        match (e.trigger.navigation_trigger, e.outcome) {
            (NavigationTriggerKind::Affordance, EdgeOutcome::SettleTimeout) => {
                d.settle_timeout += e.observations
            }
            (NavigationTriggerKind::Affordance, EdgeOutcome::Error) => d.errored += e.observations,
            (NavigationTriggerKind::Pop, EdgeOutcome::SettleTimeout | EdgeOutcome::Error) => {
                d.excluded_back_arrivals += e.observations
            }
            _ => {}
        }
    }

    let coverage = JourneyCoverage {
        nodes_observed: nodes.len() as u64,
        nodes_unmodelled,
        edges_observed,
        edges_to_node_unobserved,
        edges_invalidated,
        edges_filtered_out,
        rows_rejected: rejected.len() as u64,
        frontier_open,
        frontier_nodes,
        frontier_by_reason,
        runs: runs.len() as u64,
        edges_without_run_id,
        oldest_edge_at: oldest.map(rfc3339),
        newest_edge_at: newest.map(rfc3339),
        app_versions: app_versions
            .into_iter()
            .map(|v| v.map(str::to_string))
            .collect(),
        ledger: ledger.clone(),
    };

    JourneyGraph {
        filter_echo: filter.clone(),
        nodes,
        edges: graph_edges,
        rejected: rejected.into_iter().collect(),
        off_graph_departures,
        unplaced_rejected_rows,
        unplaced_rejected_frontier,
        rejected_frontier_double_placed,
        coverage,
    }
}

// ---------------------------------------------------------------------------
// Search frontier — what a negative answer was NOT able to look at
// ---------------------------------------------------------------------------

/// One node, inside a query's searched component, that was not fully looked
/// at.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct JourneySearchFrontierNode {
    /// The node.
    pub node_key: String,
    /// Its open frontier entries, grouped by reason (only reasons present),
    /// each list sorted by `affordanceFingerprint` — the affordances that
    /// would have to be explored to settle the question.
    pub open_by_reason: BTreeMap<String, Vec<FrontierEntry>>,
    /// Departures from it whose destination was never observed.
    pub unobserved_departures: u64,
    /// Departures from it the navigation edge set left out.
    pub unmeasured_departures: JourneyUnmeasuredDepartures,
}

/// What the searched part of the graph did NOT look at. Empty
/// ([`Self::is_settled`]) means a negative answer over that part is a real
/// negative; otherwise it is unknown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct JourneySearchFrontier {
    /// Nodes with open frontier entries, unobserved departures or unmeasured
    /// departures, sorted by key.
    pub nodes: Vec<JourneySearchFrontierNode>,
    /// Open frontier entries across those nodes.
    pub open_entries: u64,
    /// Open frontier entries per reason — every reason present, zeros
    /// included.
    pub by_reason: BTreeMap<String, u64>,
    /// Unobserved departures across those nodes.
    pub unobserved_departures: u64,
    /// Unmeasured departures across those nodes, plus — for a graph-wide
    /// question — `offGraphDepartures`.
    pub unmeasured_departures: JourneyUnmeasuredDepartures,
    /// The off-graph part included in `unmeasuredDepartures` (all zero for a
    /// question about one component).
    pub off_graph_departures: JourneyUnmeasuredDepartures,
    /// Untrusted edge rows that could not be placed at any node (counted by
    /// every question).
    pub unplaced_rejected_rows: u64,
    /// Rejected frontier rows that could not be placed at any node (counted
    /// by every question).
    pub unplaced_rejected_frontier: u64,
}

impl JourneySearchFrontier {
    fn empty() -> Self {
        Self {
            nodes: Vec::new(),
            open_entries: 0,
            by_reason: FrontierReason::ALL
                .iter()
                .map(|r| (r.as_str().to_string(), 0))
                .collect(),
            unobserved_departures: 0,
            unmeasured_departures: JourneyUnmeasuredDepartures::default(),
            off_graph_departures: JourneyUnmeasuredDepartures::default(),
            unplaced_rejected_rows: 0,
            unplaced_rejected_frontier: 0,
        }
    }

    /// True iff a negative answer over the searched part may be reported
    /// `absent`: the ledger is `writing`, the graph was built over at least
    /// one edge, the subject of the question was observed, and nothing in the
    /// searched part was left unexplored (no open or rejected frontier entry,
    /// no unobserved or unmeasured departure, no unplaced rejected row).
    /// Exactly `self.unmeasured(coverage, subject_observed).is_empty()`, and
    /// exactly `self.unsettled_because(coverage, subject_observed).is_none()`.
    ///
    /// `subject_observed` is the question's subject: the start node; for
    /// `steps_to`, both endpoints; for one node's frontier, that node
    /// (`nodeObserved`); `true` for a graph-wide question.
    pub fn is_settled(&self, coverage: &JourneyCoverage, subject_observed: bool) -> bool {
        self.unmeasured(coverage, subject_observed).is_empty()
    }

    /// What the answer did not look at, as the envelope's
    /// `provenance.coverage.unmeasured` entries:
    /// `(dimension, count, UnknownCode wire string)`, only for non-zero
    /// counts, in this order:
    ///
    /// | dimension | counts | code |
    /// |---|---|---|
    /// | `ledger` | 1 (the ledger) when `coverage.ledger` is not `writing` | `ledger_schema_absent` / `ledger_write_failing` |
    /// | `no_observations` | 1 (the ledger) when `coverage.edgesObserved == 0` | `no_observations` |
    /// | `subject_unobserved` | 1 when `subject_observed` is false | `no_observations` |
    /// | `frontier` | `openEntries` | `frontier_open` |
    /// | `rejected_frontier` | invalid frontier rows, treated as open | `frontier_open` |
    /// | `unplaced_rejected_frontier` | invalid frontier rows placeable at no node | `frontier_open` |
    /// | `unobserved_departures` | `to_node_unobserved` departures | `to_node_unobserved` |
    /// | `settle_timeout_departures` | `settle_timeout` affordance edges to another node | `no_event_before_timeout` |
    /// | `rejected_departures` | contract-violating edge rows (any trigger, any outcome) | `producer_failed` |
    /// | `unplaced_rejected_rows` | untrusted edge rows with an invalid `fromNode` | `producer_failed` |
    /// | `errored_departures` | `error` affordance edges to another node | `input_missing` |
    /// | `filtered_departures` | affordance rows the filter excluded | `input_missing` |
    /// | `withdrawn_departures` | withdrawn affordance rows | `input_missing` |
    /// | `excluded_back_arrivals` | excluded or unsettled `pop` rows (graph-wide questions only) | `input_missing` |
    /// | `excluded_url_arrivals` | excluded URL-trigger rows (graph-wide questions only) | `input_missing` |
    ///
    /// Empty iff [`Self::is_settled`]. Putting these in `coverage.unmeasured`
    /// is what makes the shared `Observation::absent` constructor REFUSE a
    /// negative over a non-writing or empty ledger, an unobserved subject or
    /// an unexplored part, so the rule is enforced once, by the envelope.
    pub fn unmeasured(
        &self,
        coverage: &JourneyCoverage,
        subject_observed: bool,
    ) -> Vec<(&'static str, u64, &'static str)> {
        let mut out = Vec::new();
        if let Some(code) = coverage.ledger_unknown_code() {
            out.push(("ledger", 1, code));
        }
        if coverage.edges_observed == 0 {
            out.push(("no_observations", 1, "no_observations"));
        }
        if !subject_observed {
            out.push(("subject_unobserved", 1, "no_observations"));
        }
        let d = &self.unmeasured_departures;
        out.extend(
            [
                ("frontier", self.open_entries, "frontier_open"),
                ("rejected_frontier", d.rejected_frontier, "frontier_open"),
                (
                    "unplaced_rejected_frontier",
                    self.unplaced_rejected_frontier,
                    "frontier_open",
                ),
                (
                    "unobserved_departures",
                    self.unobserved_departures,
                    "to_node_unobserved",
                ),
                (
                    "settle_timeout_departures",
                    d.settle_timeout,
                    "no_event_before_timeout",
                ),
                ("rejected_departures", d.rejected, "producer_failed"),
                (
                    "unplaced_rejected_rows",
                    self.unplaced_rejected_rows,
                    "producer_failed",
                ),
                ("errored_departures", d.errored, "input_missing"),
                ("filtered_departures", d.filtered_out, "input_missing"),
                ("withdrawn_departures", d.withdrawn, "input_missing"),
                (
                    "excluded_back_arrivals",
                    d.excluded_back_arrivals,
                    "input_missing",
                ),
                (
                    "excluded_url_arrivals",
                    d.excluded_url_arrivals,
                    "input_missing",
                ),
            ]
            .into_iter()
            .filter(|(_, n, _)| *n > 0),
        );
        out
    }

    /// Why a NEGATIVE answer over this searched part cannot be settled: the
    /// code of the FIRST [`Self::unmeasured`] entry, or `None` when it may be
    /// reported `absent`. So the precedence is the table's order — ledger
    /// state; `no_observations` (no edge at all, then the subject unseen);
    /// `frontier_open`; `to_node_unobserved`; `no_event_before_timeout`;
    /// `producer_failed`; `input_missing` — and
    /// `unsettled_because(c, s).is_none() == is_settled(c, s)`.
    ///
    /// A POSITIVE answer (a path found, a node listed) is `measured` whatever
    /// this says; the caller still reports the coverage beside it.
    pub fn unsettled_because(
        &self,
        coverage: &JourneyCoverage,
        subject_observed: bool,
    ) -> Option<&'static str> {
        self.unmeasured(coverage, subject_observed)
            .first()
            .map(|(_, _, code)| *code)
    }
}

// ---------------------------------------------------------------------------
// reachable_from
// ---------------------------------------------------------------------------

/// A node reached, and its BFS distance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct JourneyReachedNode {
    /// The node.
    pub node_key: String,
    /// Shortest number of navigation edges from the start (0 = the start).
    pub steps: u64,
}

/// The answer of [`reachable_from`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct JourneyReachable {
    /// The start node's key.
    pub from_key: String,
    /// Whether the start node was observed at all. `false` means the
    /// question was asked about a node the ledger has never seen — the
    /// caller answers `unknown` (`no_observations`), never "reaches nothing".
    pub from_observed: bool,
    /// Every node reachable from the start, INCLUDING the start at 0 steps
    /// (empty when `fromObserved` is false), sorted by key.
    pub reachable: Vec<JourneyReachedNode>,
    /// What the searched component (exactly the `reachable` nodes) did not
    /// look at. Unsettled ⇒ "X is not reachable" is unknown.
    pub frontier: JourneySearchFrontier,
    /// The graph's filter.
    pub filter_echo: JourneyFilter,
    /// The graph's coverage.
    pub coverage: JourneyCoverage,
}

impl JourneyReachable {
    /// The distance to `key`, if reached.
    pub fn steps_to(&self, key: &str) -> Option<u64> {
        self.reachable
            .binary_search_by(|r| r.node_key.as_str().cmp(key))
            .ok()
            .map(|i| self.reachable[i].steps)
    }
}

fn bfs_distances<'a>(
    adj: &BTreeMap<&'a str, BTreeSet<&'a str>>,
    start: &'a str,
) -> BTreeMap<&'a str, u64> {
    let mut dist: BTreeMap<&str, u64> = BTreeMap::new();
    dist.insert(start, 0);
    let mut queue = VecDeque::from([start]);
    while let Some(current) = queue.pop_front() {
        let d = dist[current];
        for &next in adj.get(current).into_iter().flatten() {
            if !dist.contains_key(next) {
                dist.insert(next, d + 1);
                queue.push_back(next);
            }
        }
    }
    dist
}

/// Every node reachable from `from_key` over the navigation edge set
/// (affordance + changed), with its shortest step count.
pub fn reachable_from(graph: &JourneyGraph, from_key: &str) -> JourneyReachable {
    let from_observed = graph.node(from_key).is_some();
    let (reachable, frontier) = if from_observed {
        let adj = graph.navigation_adjacency();
        let start = graph.node(from_key).expect("observed").key.as_str();
        let dist = bfs_distances(&adj, start);
        let frontier = graph.search_frontier(dist.keys().copied(), false);
        let reachable = dist
            .into_iter()
            .map(|(k, steps)| JourneyReachedNode {
                node_key: k.to_string(),
                steps,
            })
            .collect();
        (reachable, frontier)
    } else {
        (Vec::new(), JourneySearchFrontier::empty())
    };
    JourneyReachable {
        from_key: from_key.to_string(),
        from_observed,
        reachable,
        frontier,
        filter_echo: graph.filter_echo.clone(),
        coverage: graph.coverage.clone(),
    }
}

// ---------------------------------------------------------------------------
// steps_to
// ---------------------------------------------------------------------------

/// The answer of [`steps_to`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct JourneySteps {
    /// The start node's key.
    pub from_key: String,
    /// The target node's key.
    pub to_key: String,
    /// Whether the start node was observed.
    pub from_observed: bool,
    /// Whether the target node was observed.
    pub to_observed: bool,
    /// Shortest number of navigation edges from start to target; `null` = no
    /// observed path (which is a negative only when `frontier` is settled and
    /// both endpoints were observed).
    pub steps: Option<u64>,
    /// The edges of one shortest path, in order (empty when `steps` is null
    /// or 0). Among equally short paths, the one whose next node key is
    /// byte-wise smallest at every step; between parallel edges, the
    /// smallest edge in graph order.
    pub path: Vec<JourneyGraphEdge>,
    /// What the component reachable from the start did not look at.
    pub frontier: JourneySearchFrontier,
    /// The graph's filter.
    pub filter_echo: JourneyFilter,
    /// The graph's coverage.
    pub coverage: JourneyCoverage,
}

/// Shortest navigation path from `from_key` to `to_key` (BFS hop count), with
/// the edge list of one deterministic shortest path.
pub fn steps_to(graph: &JourneyGraph, from_key: &str, to_key: &str) -> JourneySteps {
    let from_observed = graph.node(from_key).is_some();
    let to_observed = graph.node(to_key).is_some();
    let mut steps = None;
    let mut path = Vec::new();
    let mut frontier = JourneySearchFrontier::empty();

    if from_observed {
        let adj = graph.navigation_adjacency();
        let start = graph.node(from_key).expect("observed").key.as_str();
        let forward = bfs_distances(&adj, start);
        frontier = graph.search_frontier(forward.keys().copied(), false);

        if to_observed && forward.contains_key(to_key) {
            // Distances TO the target, so the walk can pick, at every step,
            // the smallest next key that is still on a shortest path.
            let reverse = graph.reverse_navigation_adjacency();
            let target = graph.node(to_key).expect("observed").key.as_str();
            let to_target = bfs_distances(&reverse, target);
            let total = to_target[start];
            let mut current = start;
            while current != target {
                let want = to_target[current] - 1;
                let next = adj[current]
                    .iter()
                    .copied()
                    .find(|n| to_target.get(n) == Some(&want))
                    .expect("a node on a shortest path has a successor one step closer");
                let edge = graph
                    .edges
                    .iter()
                    .find(|e| e.is_navigation() && e.from_key == current && e.to_key == next)
                    .expect("adjacency is derived from these edges");
                path.push(edge.clone());
                current = next;
            }
            steps = Some(total);
        }
    }

    JourneySteps {
        from_key: from_key.to_string(),
        to_key: to_key.to_string(),
        from_observed,
        to_observed,
        steps,
        path,
        frontier,
        filter_echo: graph.filter_echo.clone(),
        coverage: graph.coverage.clone(),
    }
}

// ---------------------------------------------------------------------------
// dead_ends
// ---------------------------------------------------------------------------

/// A node a user cannot leave by any affordance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct JourneyDeadEnd {
    /// The node.
    pub node_key: String,
    /// `true` when a kept `pop` edge with outcome `changed` leaves this node
    /// for a different node — the only way out is the browser's back button,
    /// a journey finding rather than an exit. `null` = unknown: no such edge
    /// was kept, but a `pop` row from this node was filtered or withdrawn, or
    /// a kept `pop` edge to another node ended `error` / `settle_timeout`
    /// (`unmeasuredDepartures.excludedBackArrivals > 0`). `false` otherwise.
    /// (A rejected `pop`-claiming row is untrusted and counts as `rejected`,
    /// which makes the node `unexplored`, never a dead end.)
    pub back_only: Option<bool>,
}

/// A node with no observed affordance exit that was NOT fully looked at, so
/// it cannot be called a dead end.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct JourneyUnexploredNode {
    /// The node.
    pub node_key: String,
    /// Its open frontier entries per reason (only reasons present).
    pub open_by_reason: BTreeMap<String, u64>,
    /// Departures from it whose destination was never observed.
    pub unobserved_departures: u64,
    /// Departures from it the navigation edge set left out (filtered,
    /// withdrawn, rejected, `settle_timeout` / `error` to another node,
    /// rejected frontier rows).
    pub unmeasured_departures: JourneyUnmeasuredDepartures,
    /// `true` when the node itself was fully explored but the GRAPH cannot
    /// support a dead-end verdict: an unplaced untrusted row (edge or
    /// frontier) may belong to it, or the ledger is not `writing`.
    pub unsettled_by_graph: bool,
}

/// The answer of [`dead_ends`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct JourneyDeadEnds {
    /// Nodes with no navigation edge to a DIFFERENT node, an empty frontier,
    /// no unobserved departure and no unmeasured NAVIGATION departure (an
    /// excluded back arrival only makes `backOnly` unknown), sorted by key.
    pub dead_ends: Vec<JourneyDeadEnd>,
    /// Nodes with no navigation edge to a different node that still have an
    /// open frontier, an unobserved departure or an unmeasured navigation
    /// departure — or that the graph as a whole cannot settle
    /// (`unsettledByGraph`) — never dead ends. Sorted.
    pub unexplored: Vec<JourneyUnexploredNode>,
    /// What the whole graph did not look at.
    pub frontier: JourneySearchFrontier,
    /// The graph's filter.
    pub filter_echo: JourneyFilter,
    /// The graph's coverage.
    pub coverage: JourneyCoverage,
}

/// Observed nodes with no way out by any affordance.
///
/// A node with an open frontier, a departure whose destination was never
/// observed, or a navigation departure the edge set left out
/// ([`JourneyUnmeasuredDepartures`] other than `excludedBackArrivals`) is
/// `unexplored`, never a dead end. So is EVERY node without an exit while
/// the graph has an unplaced untrusted row (edge or frontier) or the ledger
/// is not `writing` (`unsettledByGraph: true`). A dead end's `backOnly` is
/// `true` with a kept, changed `pop` exit to another node, `null` when it has
/// none but an excluded or unsettled one (`excludedBackArrivals > 0`), else
/// `false`.
pub fn dead_ends(graph: &JourneyGraph) -> JourneyDeadEnds {
    let adj = graph.navigation_adjacency();
    let mut pop_exits: BTreeSet<&str> = BTreeSet::new();
    for e in &graph.edges {
        if e.trigger.navigation_trigger == NavigationTriggerKind::Pop
            && e.outcome == EdgeOutcome::Changed
            && e.from_key != e.to_key
        {
            pop_exits.insert(e.from_key.as_str());
        }
    }

    // An unplaced untrusted row may belong to any node, and a ledger that is
    // not being written may be missing any row: neither allows a settled
    // dead-end verdict for ANY node.
    let graph_unsettled = graph.unplaced_rejected_rows > 0
        || graph.unplaced_rejected_frontier > 0
        || graph.coverage.ledger.state != LedgerState::Writing;
    let mut dead = Vec::new();
    let mut unexplored = Vec::new();
    for node in &graph.nodes {
        let key = node.key.as_str();
        let has_exit = adj
            .get(key)
            .is_some_and(|tos| tos.iter().any(|&to| to != key));
        if has_exit {
            continue;
        }
        if node.is_fully_explored() && !graph_unsettled {
            dead.push(JourneyDeadEnd {
                node_key: node.key.clone(),
                back_only: if pop_exits.contains(key) {
                    Some(true)
                } else if node.unmeasured_departures.excluded_back_arrivals > 0 {
                    None
                } else {
                    Some(false)
                },
            });
        } else {
            let mut open_by_reason: BTreeMap<String, u64> = BTreeMap::new();
            for entry in &node.frontier {
                *open_by_reason
                    .entry(entry.reason.as_str().to_string())
                    .or_default() += 1;
            }
            unexplored.push(JourneyUnexploredNode {
                node_key: node.key.clone(),
                open_by_reason,
                unobserved_departures: node.unobserved_departures,
                unmeasured_departures: node.unmeasured_departures,
                unsettled_by_graph: node.is_fully_explored(),
            });
        }
    }

    JourneyDeadEnds {
        dead_ends: dead,
        unexplored,
        frontier: graph.search_frontier(graph.all_keys(), true),
        filter_echo: graph.filter_echo.clone(),
        coverage: graph.coverage.clone(),
    }
}

// ---------------------------------------------------------------------------
// frontier
// ---------------------------------------------------------------------------

/// The answer of [`frontier`] — the open frontier of one node, or of the
/// whole graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct JourneyFrontier {
    /// The node asked about; `null` = the whole graph.
    pub node_key: Option<String>,
    /// Whether that node was observed (always `true` for the whole graph).
    /// `false` means "never seen", never "nothing open there".
    pub node_observed: bool,
    /// The open, listable frontier entries, sorted by `(nodeKey,
    /// affordanceFingerprint)`.
    pub entries: Vec<FrontierEntry>,
    /// Everything not looked at in the part asked about — including rows
    /// that could not be listed (`rejected_frontier`). An EMPTY `entries` is
    /// a real "nothing left to explore" only when this is settled and
    /// [`JourneySearchFrontier::unsettled_because`] returns `None`.
    pub frontier: JourneySearchFrontier,
    /// The graph's filter.
    pub filter_echo: JourneyFilter,
    /// The graph's coverage.
    pub coverage: JourneyCoverage,
}

/// The open frontier at `node_key`, or across the whole graph when `None`.
pub fn frontier(graph: &JourneyGraph, node_key: Option<&str>) -> JourneyFrontier {
    let (node_observed, entries, frontier) = match node_key {
        Some(key) => match graph.node(key) {
            Some(n) => (
                true,
                n.frontier.clone(),
                graph.search_frontier([n.key.as_str()], false),
            ),
            None => (false, Vec::new(), JourneySearchFrontier::empty()),
        },
        None => (
            true,
            graph
                .nodes
                .iter()
                .flat_map(|n| n.frontier.iter().cloned())
                .collect(),
            graph.search_frontier(graph.all_keys(), true),
        ),
    };
    JourneyFrontier {
        node_key: node_key.map(str::to_string),
        node_observed,
        entries,
        frontier,
        filter_echo: graph.filter_echo.clone(),
        coverage: graph.coverage.clone(),
    }
}

// ---------------------------------------------------------------------------
// requires_prior_knowledge
// ---------------------------------------------------------------------------

/// A node users only ever arrived at without clicking anything.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct JourneyPriorKnowledgeNode {
    /// The node.
    pub node_key: String,
    /// Observed arrivals per navigation trigger (`initial`, `push`,
    /// `replace`, `hash` only — by definition).
    pub arrivals: BTreeMap<String, u64>,
}

/// The answer of [`requires_prior_knowledge`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct JourneyPriorKnowledge {
    /// Matching nodes, sorted by key.
    pub nodes: Vec<JourneyPriorKnowledgeNode>,
    /// What the whole graph did not look at: any open frontier entry could be
    /// the affordance that leads to one of `nodes`.
    pub frontier: JourneySearchFrontier,
    /// The graph's filter.
    pub filter_echo: JourneyFilter,
    /// The graph's coverage.
    pub coverage: JourneyCoverage,
}

/// Nodes whose ONLY observed arrivals are `initial | push | replace | hash` —
/// a user must already know the URL.
///
/// The rule, exactly: a node qualifies when it has at least one observed
/// `initial | push | replace | hash` arrival and NO arrival by an affordance
/// or pop edge from a different observed node. That disqualifying edge counts
/// whatever its outcome (an errored click that still landed there is still a
/// way in; back-navigation means the user had been there by some other way).
/// An `affordance` or `pop` edge from a node to itself neither qualifies nor
/// disqualifies it; an `initial | push | replace | hash` edge
/// counts as an arrival whatever its from-node (a reload is `initial` from
/// the node itself).
pub fn requires_prior_knowledge(graph: &JourneyGraph) -> JourneyPriorKnowledge {
    // to_key -> arrival kind -> count
    let mut arrivals: BTreeMap<&str, BTreeMap<&'static str, u64>> = BTreeMap::new();
    let mut disqualified: BTreeSet<&str> = BTreeSet::new();
    for e in &graph.edges {
        let to = e.to_key.as_str();
        match e.trigger.navigation_trigger {
            NavigationTriggerKind::Affordance | NavigationTriggerKind::Pop => {
                if e.from_key != e.to_key {
                    disqualified.insert(to);
                }
            }
            kind @ (NavigationTriggerKind::Initial
            | NavigationTriggerKind::Push
            | NavigationTriggerKind::Replace
            | NavigationTriggerKind::Hash) => {
                *arrivals
                    .entry(to)
                    .or_default()
                    .entry(kind.as_str())
                    .or_default() += e.observations;
            }
        }
    }
    let nodes = arrivals
        .into_iter()
        .filter(|(to, _)| !disqualified.contains(to))
        .map(|(to, by_kind)| JourneyPriorKnowledgeNode {
            node_key: to.to_string(),
            arrivals: by_kind
                .into_iter()
                .map(|(k, n)| (k.to_string(), n))
                .collect(),
        })
        .collect();
    JourneyPriorKnowledge {
        nodes,
        frontier: graph.search_frontier(graph.all_keys(), true),
        filter_echo: graph.filter_echo.clone(),
        coverage: graph.coverage.clone(),
    }
}

#[cfg(test)]
mod tests;
