/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { FrontierEntry } from "./FrontierEntry";
import type { JourneyArrivalDoubt } from "./JourneyArrivalDoubt";
import type { JourneyCoverage } from "./JourneyCoverage";
import type { JourneyFilter } from "./JourneyFilter";
import type { JourneyGraphEdge } from "./JourneyGraphEdge";
import type { JourneyLedgerHealth } from "./JourneyLedgerHealth";
import type { JourneyNode } from "./JourneyNode";
import type { JourneyTrigger } from "./JourneyTrigger";
import type { JourneyUnmeasuredDepartures } from "./JourneyUnmeasuredDepartures";

/**
 * The journey graph derived from ledger rows under one filter.
 */
export interface JourneyGraph {
  /**
   * What the graph was built over.
   */
  coverage: JourneyCoverage;
  /**
   * Aggregated edges, sorted by `(fromKey, toKey, trigger, outcome)`.
   */
  edges: JourneyGraphEdge[];
  /**
   * The filter the edge set was narrowed by.
   */
  filterEcho: JourneyFilter;
  /**
   * Observed nodes, sorted by key.
   */
  nodes: JourneyGraphNode[];
  /**
   * Unmeasured departures (filtered, withdrawn, rejected frontier with a
   * valid node, excluded back and URL arrivals) placed at a node key that
   * is NOT in
   * `nodes`. No component query can reach such a node, but a graph-wide
   * question (`dead_ends`, `requires_prior_knowledge`, the whole frontier)
   * counts them.
   */
  offGraphDepartures: JourneyUnmeasuredDepartures;
  /**
   * Rows left out as contract violations, live or withdrawn (a withdrawn one
   * is also counted in `coverage.edgesInvalidated`), sorted.
   */
  rejected: JourneyRejectedRow[];
  /**
   * Rejected frontier rows counted at TWO node keys (their `nodeKey`
   * column and their valid node's own key disagree). Per-node counts
   * include both placements; graph-wide totals subtract this, so a
   * graph-wide `rejectedFrontier` plus `unplacedRejectedFrontier` is the
   * number of DISTINCT rejected frontier rows by full content. (`rejected`
   * lists rows by `(kind, row, reason)`, so two rows differing only in
   * their node JSON share one entry there but are both counted.)
   */
  rejectedFrontierDoublePlaced: number;
  /**
   * Affordance or `pop` rows whose real destination is unknown: kept or
   * trusted-excluded ones with no `toNode` (`to_node_unobserved`), ones
   * that closed `settle_timeout` or `error` to a DIFFERENT node (that node
   * may be an intermediate state), and ones that closed `settle_timeout`
   * on the SAME node (the destination may not have rendered yet). Each
   * may arrive at any node, so `requires_prior_knowledge` lists no node as
   * settled while this is non-zero.
   */
  unknownDestinationArrivals: number;
  /**
   * Distinct rejected frontier rows whose node fails `validate()`.
   * Nothing in such a row is trusted, its `nodeKey` column included, so it
   * is placed at NO node: each may be an open affordance anywhere, and
   * EVERY negative answer counts it (dimension
   * `unplaced_rejected_frontier`).
   */
  unplacedRejectedFrontier: number;
  /**
   * Distinct UNTRUSTED edge rows — live or withdrawn rows that fail
   * [`trusted_edge_row`] (a failed `validate()` or an unparseable
   * `observedAt`). Nothing in such a row is trusted, its `fromNode` and
   * trigger included, so it is placed at NO node: it may be a click from
   * and to anywhere, and EVERY negative answer counts it, component and
   * graph-wide alike (dimension `unplaced_rejected_rows`). While it is
   * non-zero, `dead_ends` lists no dead end and `requires_prior_knowledge`
   * lists no settled node.
   */
  unplacedRejectedRows: number;
}
/**
 * One observed vertex.
 */
export interface JourneyGraphNode {
  /**
   * Excluded affordance / pop rows arriving here from another node.
   */
  arrivalDoubt: JourneyArrivalDoubt;
  /**
   * Open frontier entries seen at this node, sorted by
   * `affordanceFingerprint`.
   */
  frontier: FrontierEntry[];
  /**
   * [`JourneyNode::key`] — the node's identity.
   */
  key: string;
  /**
   * A representative node for the key (the smallest by its fields when
   * rows disagree on non-identity detail such as `pageLabel`).
   */
  node: JourneyNode;
  /**
   * `true` when the node has at least one kept arrival (an edge from a
   * DIFFERENT node whose `toNode` is this node — a self-loop is not an
   * arrival), and NONE of them closed `changed` or
   * `no_change`, whatever its trigger. Only those two outcomes confirm the
   * node is a real destination: a `settle_timeout` or `error` arrival's
   * observed node may be an intermediate state (a spinner, an error
   * render). Such a node is never a settled dead end, nor a settled
   * requires-prior-knowledge node.
   */
  unconfirmedDestination: boolean;
  /**
   * Departures from this node the navigation edge set leaves out but that
   * may still lead somewhere (see [`JourneyUnmeasuredDepartures`]).
   */
  unmeasuredDepartures: JourneyUnmeasuredDepartures;
  /**
   * How many edges left this node with `outcome: to_node_unobserved` —
   * actions whose destination was never seen.
   */
  unobservedDepartures: number;
}
/**
 * A ledger row that broke the edge-observation contract and was left out of
 * the graph — reported, never silently dropped.
 */
export interface JourneyRejectedRow {
  /**
   * Edge or frontier row.
   */
  kind: "edge" | "frontier";
  /**
   * The contract violation, as `JourneyContractError` renders it (or the
   * timestamp parse failure).
   */
  reason: string;
  /**
   * The edge's `id`, or `"<nodeKey> <affordanceFingerprint>"` for a
   * frontier row.
   */
  row: string;
  /**
   * How many DISTINCT rows (by full serialized content) this entry stands
   * for. Two rows that share `(kind, row, reason)` but differ anywhere
   * else — a live and a withdrawn copy, or two fromNodes under one id —
   * are one entry here with `rows: 2`. The same full-row identity is the
   * one `JourneyGraph::unplaced_rejected_rows` and
   * `unplaced_rejected_frontier` count by, so for edges the `rows` of the
   * edge entries sum to `unplaced_rejected_rows`.
   */
  rows: number;
}
