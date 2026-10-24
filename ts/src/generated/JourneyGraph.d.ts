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
   * Unmeasured departures (filtered, withdrawn, rejected, rejected
   * frontier, excluded back arrivals) placed at a node key that is NOT in
   * `nodes`. No component query can reach such a node, but a graph-wide
   * question (`dead_ends`, `requires_prior_knowledge`, the whole frontier)
   * counts them.
   */
  offGraphDepartures: JourneyUnmeasuredDepartures;
  /**
   * Rows left out as contract violations, sorted.
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
   * Affordance or `pop` rows — kept `to_node_unobserved` ones, and trusted
   * excluded ones with no `toNode` — whose destination is unknown. Each
   * may arrive at any node, so `requires_prior_knowledge` lists no node as
   * settled while this is non-zero.
   */
  unknownDestinationArrivals: number;
  /**
   * Distinct rejected frontier rows whose node is invalid AND whose
   * `nodeKey` column names no observed node, so they cannot be placed at
   * any node: each may be an open affordance anywhere, and EVERY negative
   * answer counts them (dimension `unplaced_rejected_frontier`).
   */
  unplacedRejectedFrontier: number;
  /**
   * Untrusted edge rows — live rows that failed `validate()` or carry an
   * unparseable `observedAt`, and withdrawn rows failing either check —
   * whose `fromNode` itself is invalid, so they cannot be placed at any
   * node. Their trigger cannot be trusted, so each may be a click
   * anywhere: EVERY negative answer counts them, component and graph-wide
   * alike (dimension `unplaced_rejected_rows`).
   */
  unplacedRejectedRows: number;
  /**
   * Distinct untrusted edge rows (placed or unplaced). Each may be a click
   * from and to anywhere, so `requires_prior_knowledge` lists no node as
   * settled while this is non-zero.
   */
  untrustedEdgeRows: number;
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
}
