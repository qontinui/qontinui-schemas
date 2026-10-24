/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { FrontierEntry } from "./FrontierEntry";
import type { JourneyCoverage } from "./JourneyCoverage";
import type { JourneyFilter } from "./JourneyFilter";
import type { JourneyLedgerHealth } from "./JourneyLedgerHealth";
import type { JourneyNode } from "./JourneyNode";
import type { JourneySearchFrontier } from "./JourneySearchFrontier";
import type { JourneySearchFrontierNode } from "./JourneySearchFrontierNode";
import type { JourneyUnmeasuredDepartures } from "./JourneyUnmeasuredDepartures";

/**
 * The answer of [`dead_ends`].
 */
export interface JourneyDeadEnds {
  /**
   * The graph's coverage.
   */
  coverage: JourneyCoverage;
  /**
   * Nodes with no navigation edge to a DIFFERENT node, an empty frontier,
   * no unobserved departure and no unmeasured NAVIGATION departure (an
   * excluded back arrival only makes `backOnly` unknown), sorted by key.
   */
  deadEnds: JourneyDeadEnd[];
  /**
   * The graph's filter.
   */
  filterEcho: JourneyFilter;
  /**
   * What the whole graph did not look at.
   */
  frontier: JourneySearchFrontier;
  /**
   * Nodes with no navigation edge to a different node that still have an
   * open frontier, an unobserved departure or an unmeasured navigation
   * departure — or whose arrival is unconfirmed (`unconfirmedDestination`:
   * no arrival from another node closed `changed` / `no_change`, so it may
   * be an intermediate state), or that the graph as a whole cannot settle
   * (`unsettledByGraph`) — never dead ends. Sorted.
   */
  unexplored: JourneyUnexploredNode[];
}
/**
 * A node a user cannot leave by any affordance.
 */
export interface JourneyDeadEnd {
  /**
   * `true` when a kept `pop` edge with outcome `changed` leaves this node
   * for a different node — the only way out is the browser's back button,
   * a journey finding rather than an exit. `null` = unknown: no such edge
   * was kept, but a `pop` row from this node was filtered or withdrawn, a
   * kept `pop` edge to another node ended `error` / `settle_timeout`, or a
   * kept `pop` self-loop ended `settle_timeout`
   * (`unmeasuredDepartures.excludedBackArrivals > 0`). `false` otherwise.
   * (A rejected `pop`-claiming row is untrusted and counts in
   * `JourneyGraph::unplaced_rejected_rows`, which lists no node as a dead
   * end at all.)
   */
  backOnly?: boolean | null;
  /**
   * The node.
   */
  nodeKey: string;
}
/**
 * A node with no observed affordance exit that was NOT fully looked at, so
 * it cannot be called a dead end.
 */
export interface JourneyUnexploredNode {
  /**
   * The node.
   */
  nodeKey: string;
  /**
   * Its open frontier entries per reason (only reasons present).
   */
  openByReason: {
    [k: string]: number;
  };
  /**
   * `true` when no observed arrival at the node closed `changed` or
   * `no_change` (they were all `settle_timeout` / `error`), so nothing
   * confirms it is a real destination rather than an intermediate state
   * ([`JourneyGraphNode::unconfirmed_destination`]).
   */
  unconfirmedDestination: boolean;
  /**
   * Departures from it the navigation edge set left out (filtered,
   * withdrawn, `settle_timeout` / `error` to another node, `settle_timeout`
   * self-loops, rejected frontier rows with a valid node).
   */
  unmeasuredDepartures: JourneyUnmeasuredDepartures;
  /**
   * Departures from it whose destination was never observed.
   */
  unobservedDepartures: number;
  /**
   * `true` when the node itself was fully explored but the GRAPH cannot
   * support a dead-end verdict: an unplaced untrusted row (edge or
   * frontier) may belong to it, or the ledger is not `writing`.
   */
  unsettledByGraph: boolean;
}
