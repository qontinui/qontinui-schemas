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
   * departure — or that the graph as a whole cannot settle
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
   * was kept, but a `pop` row from this node was filtered or withdrawn, or
   * a kept `pop` edge to another node ended `error` / `settle_timeout`
   * (`unmeasuredDepartures.excludedBackArrivals > 0`). `false` otherwise.
   * (A rejected `pop`-claiming row is untrusted and counts as `rejected`,
   * which makes the node `unexplored`, never a dead end.)
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
   * Departures from it the navigation edge set left out (filtered,
   * withdrawn, rejected, `settle_timeout` / `error` to another node,
   * rejected frontier rows).
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
