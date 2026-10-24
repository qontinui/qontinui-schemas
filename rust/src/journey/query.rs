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
//! reason, and the departures whose destination was never observed
//! (`to_node_unobserved`). The caller (the runner) builds the
//! `Observation<T>` envelope from those facts: a negative over an unsettled
//! frontier ([`JourneySearchFrontier::is_settled`] false) is `unknown`
//! (`frontier_open` / `to_node_unobserved`), never `absent`. This crate does
//! not depend on `qontinui-vision-core` (which owns the envelope and pulls
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

    /// The frontier summary over the given node keys.
    fn search_frontier<'a>(
        &self,
        keys: impl IntoIterator<Item = &'a str>,
    ) -> JourneySearchFrontier {
        let mut summary = JourneySearchFrontier::empty();
        for key in keys {
            let Some(node) = self.node(key) else { continue };
            if node.frontier.is_empty() && node.unobserved_departures == 0 {
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
            summary.nodes.push(JourneySearchFrontierNode {
                node_key: node.key.clone(),
                open_by_reason,
                unobserved_departures: node.unobserved_departures,
            });
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

    for edge in edges {
        if edge.invalidated_at.is_some() {
            edges_invalidated += 1;
            continue;
        }
        if let Err(err) = edge.validate() {
            rejected.insert(JourneyRejectedRow {
                kind: JourneyRejectedRowKind::Edge,
                row: edge.id.clone(),
                reason: err.to_string(),
            });
            continue;
        }
        let observed_at = match DateTime::parse_from_rfc3339(&edge.observed_at) {
            Ok(t) => t.with_timezone(&Utc),
            Err(err) => {
                rejected.insert(JourneyRejectedRow {
                    kind: JourneyRejectedRowKind::Edge,
                    row: edge.id.clone(),
                    reason: format!("observedAt {:?} is not RFC 3339: {err}", edge.observed_at),
                });
                continue;
            }
        };
        if !filter.keeps(edge, observed_at) {
            edges_filtered_out += 1;
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
    for entry in frontier {
        if let Err(err) = entry.validate() {
            rejected.insert(JourneyRejectedRow {
                kind: JourneyRejectedRowKind::Frontier,
                row: format!("{} {}", entry.node_key, entry.affordance_fingerprint),
                reason: err.to_string(),
            });
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
                key,
                node,
                frontier,
            }
        })
        .collect();
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
}

/// What the searched part of the graph did NOT look at. Empty
/// ([`Self::is_settled`]) means a negative answer over that part is a real
/// negative; otherwise it is unknown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct JourneySearchFrontier {
    /// Nodes with open frontier entries or unobserved departures, sorted by
    /// key.
    pub nodes: Vec<JourneySearchFrontierNode>,
    /// Open frontier entries across those nodes.
    pub open_entries: u64,
    /// Open frontier entries per reason — every reason present, zeros
    /// included.
    pub by_reason: BTreeMap<String, u64>,
    /// Unobserved departures across those nodes.
    pub unobserved_departures: u64,
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
        }
    }

    /// True iff nothing in the searched part was left unexplored: no open
    /// frontier entry and no departure with an unobserved destination. Only
    /// then may a negative answer be reported as `absent`.
    pub fn is_settled(&self) -> bool {
        self.open_entries == 0 && self.unobserved_departures == 0
    }

    /// What the searched part did not look at, as the envelope's
    /// `provenance.coverage.unmeasured` entries:
    /// `(dimension, count, UnknownCode wire string)`, only for non-zero
    /// counts — `("frontier", openEntries, "frontier_open")` and
    /// `("unobserved_departures", unobservedDepartures,
    /// "to_node_unobserved")`. Empty iff [`Self::is_settled`]. Putting these
    /// in `coverage.unmeasured` is what makes the shared
    /// `Observation::absent` constructor REFUSE a negative over an open
    /// frontier, so the rule is enforced once, by the envelope.
    pub fn unmeasured(&self) -> Vec<(&'static str, u64, &'static str)> {
        let mut out = Vec::new();
        if self.open_entries > 0 {
            out.push(("frontier", self.open_entries, "frontier_open"));
        }
        if self.unobserved_departures > 0 {
            out.push((
                "unobserved_departures",
                self.unobserved_departures,
                "to_node_unobserved",
            ));
        }
        out
    }

    /// Why a NEGATIVE answer over this searched part cannot be settled, as an
    /// `UnknownCode` wire string, or `None` when it may be reported
    /// `absent`. Precedence, first match wins:
    ///
    /// 1. the ledger is not `writing` — `ledger_schema_absent` /
    ///    `ledger_write_failing` ([`JourneyCoverage::ledger_unknown_code`]);
    /// 2. the subject of the question (the start node; for `steps_to`, either
    ///    endpoint) was never observed — `no_observations`;
    /// 3. an open frontier entry — `frontier_open`;
    /// 4. a departure whose destination was never seen — `to_node_unobserved`.
    ///
    /// A POSITIVE answer (a path found, a node listed) is `measured` whatever
    /// this says; the caller still reports the coverage beside it.
    pub fn unsettled_because(
        &self,
        coverage: &JourneyCoverage,
        subject_observed: bool,
    ) -> Option<&'static str> {
        if let Some(code) = coverage.ledger_unknown_code() {
            return Some(code);
        }
        if !subject_observed {
            return Some("no_observations");
        }
        if self.open_entries > 0 {
            return Some("frontier_open");
        }
        if self.unobserved_departures > 0 {
            return Some("to_node_unobserved");
        }
        None
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
        let frontier = graph.search_frontier(dist.keys().copied());
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
        frontier = graph.search_frontier(forward.keys().copied());

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
    /// True when the only observed way out is the browser's back button (a
    /// `pop` edge that changed the configuration to a different node) — a
    /// journey finding, not an exit.
    pub back_only: bool,
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
}

/// The answer of [`dead_ends`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct JourneyDeadEnds {
    /// Nodes with no navigation edge to a DIFFERENT node, an empty frontier
    /// and no unobserved departure, sorted by key.
    pub dead_ends: Vec<JourneyDeadEnd>,
    /// Nodes with no navigation edge to a different node that still have an
    /// open frontier or an unobserved departure — never dead ends. Sorted.
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
/// A node with an open frontier, or with a departure whose destination was
/// never observed, is `unexplored`, never a dead end.
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
        if node.frontier.is_empty() && node.unobserved_departures == 0 {
            dead.push(JourneyDeadEnd {
                node_key: node.key.clone(),
                back_only: pop_exits.contains(key),
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
            });
        }
    }

    JourneyDeadEnds {
        dead_ends: dead,
        unexplored,
        frontier: graph.search_frontier(graph.all_keys()),
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
/// a user must already know the URL — with no affordance edge into them from
/// any observed node.
///
/// Conservative on purpose: an `affordance` edge from a DIFFERENT node into
/// the target disqualifies it whatever its outcome (an errored click that
/// still landed there is still a way in), and so does a `pop` arrival from a
/// different node (back-navigation means the user had been there by some
/// other way). An `affordance` or `pop` edge from a node to itself neither
/// qualifies nor disqualifies it; an `initial | push | replace | hash` edge
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
        frontier: graph.search_frontier(graph.all_keys()),
        filter_echo: graph.filter_echo.clone(),
        coverage: graph.coverage.clone(),
    }
}

#[cfg(test)]
mod tests;
