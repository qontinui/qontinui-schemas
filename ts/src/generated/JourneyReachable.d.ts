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
 * The answer of [`reachable_from`].
 */
export interface JourneyReachable {
  /**
   * The graph's coverage.
   */
  coverage: JourneyCoverage;
  /**
   * The graph's filter.
   */
  filterEcho: JourneyFilter;
  /**
   * The start node's key.
   */
  fromKey: string;
  /**
   * Whether the start node was observed at all. `false` means the
   * question was asked about a node the ledger has never seen — the
   * caller answers `unknown` (`no_observations`), never "reaches nothing".
   */
  fromObserved: boolean;
  /**
   * What the searched component (exactly the `reachable` nodes) did not
   * look at. Unsettled ⇒ "X is not reachable" is unknown.
   */
  frontier: JourneySearchFrontier;
  /**
   * Every node reachable from the start, INCLUDING the start at 0 steps
   * (empty when `fromObserved` is false), sorted by key.
   */
  reachable: JourneyReachedNode[];
}
/**
 * A node reached, and its BFS distance.
 */
export interface JourneyReachedNode {
  /**
   * The node.
   */
  nodeKey: string;
  /**
   * Shortest OBSERVED number of navigation edges from the start (0 = the
   * start) — an upper bound on the true distance while the frontier is
   * unsettled.
   */
  steps: number;
}
