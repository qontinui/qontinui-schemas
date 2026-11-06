/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { JourneyLedgerHealth } from "./JourneyLedgerHealth";

/**
 * What a graph was built over — the resolution and freshness of every answer
 * read from it. Rides inside the runner's `provenance.coverage`.
 */
export interface JourneyCoverage {
  /**
   * Distinct `appVersion`s among the edges observed, sorted, `null`
   * (= not reported) first.
   */
  appVersions: (string | null)[];
  /**
   * Valid, live edge rows the filter excluded.
   */
  edgesFilteredOut: number;
  /**
   * Withdrawn edge rows (`invalidatedAt` set), excluded.
   */
  edgesInvalidated: number;
  /**
   * Edge rows the graph was built from (after exclusions), INCLUDING
   * `to_node_unobserved` rows.
   */
  edgesObserved: number;
  /**
   * How many of `edgesObserved` closed as `to_node_unobserved`.
   */
  edgesToNodeUnobserved: number;
  /**
   * Edges observed that reported no `runId`.
   */
  edgesWithoutRunId: number;
  /**
   * Open frontier entries per reason — every reason present, zeros
   * included.
   */
  frontierByReason: {
    [k: string]: number;
  };
  /**
   * Nodes with at least one open frontier entry.
   */
  frontierNodes: number;
  /**
   * Open frontier entries across the graph.
   */
  frontierOpen: number;
  /**
   * Whether the ledger these rows were read from is being written — the
   * runner's `/journey/health` block, repeated here so a graph that
   * stopped growing (`schemaAbsent` / `writeFailing`) can never read as a
   * finished one in any query response.
   */
  ledger: JourneyLedgerHealth;
  /**
   * Latest `observedAt` among the edges observed; `null` = no edges.
   */
  newestEdgeAt?: string | null;
  /**
   * Distinct nodes (by key) seen as an edge endpoint, as the from-node of
   * an unobserved departure, or as a frontier node.
   */
  nodesObserved: number;
  /**
   * How many of those are unmodelled (`modelled: false`) — the graph's
   * resolution, as a number.
   */
  nodesUnmodelled: number;
  /**
   * Earliest `observedAt` among the edges observed; `null` = no edges.
   */
  oldestEdgeAt?: string | null;
  /**
   * ENTRIES in [`JourneyGraph::rejected`] — one per `(kind, row,
   * reason)`. The sum of their `rows` gives the number of distinct
   * rejected rows (by full content).
   */
  rowsRejected: number;
  /**
   * Distinct reported `runId`s among the edges observed.
   */
  runs: number;
}
