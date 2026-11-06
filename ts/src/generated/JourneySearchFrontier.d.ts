/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { FrontierEntry } from "./FrontierEntry";
import type { JourneyNode } from "./JourneyNode";
import type { JourneySearchFrontierNode } from "./JourneySearchFrontierNode";
import type { JourneyUnmeasuredDepartures } from "./JourneyUnmeasuredDepartures";

/**
 * What the searched part of the graph did NOT look at. Empty
 * ([`Self::is_settled`]) means a negative answer over that part is a real
 * negative; otherwise it is unknown.
 */
export interface JourneySearchFrontier {
  /**
   * Open frontier entries per reason — every reason present, zeros
   * included.
   */
  byReason: {
    [k: string]: number;
  };
  /**
   * Nodes with open frontier entries, unobserved departures or unmeasured
   * departures, sorted by key.
   */
  nodes: JourneySearchFrontierNode[];
  /**
   * The off-graph part included in `unmeasuredDepartures` (all zero for a
   * question about one component).
   */
  offGraphDepartures: JourneyUnmeasuredDepartures;
  /**
   * Open frontier entries across those nodes.
   */
  openEntries: number;
  /**
   * Unmeasured departures across those nodes, plus — for a graph-wide
   * question — `offGraphDepartures`.
   */
  unmeasuredDepartures: JourneyUnmeasuredDepartures;
  /**
   * Unobserved departures across those nodes.
   */
  unobservedDepartures: number;
  /**
   * Rejected frontier rows that could not be placed at any node (counted
   * by every question).
   */
  unplacedRejectedFrontier: number;
  /**
   * Untrusted edge rows that could not be placed at any node (counted by
   * every question).
   */
  unplacedRejectedRows: number;
}
