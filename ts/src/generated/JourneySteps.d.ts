/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { FrontierEntry } from "./FrontierEntry";
import type { JourneyCoverage } from "./JourneyCoverage";
import type { JourneyFilter } from "./JourneyFilter";
import type { JourneyGraphEdge } from "./JourneyGraphEdge";
import type { JourneyLedgerHealth } from "./JourneyLedgerHealth";
import type { JourneyNode } from "./JourneyNode";
import type { JourneySearchFrontier } from "./JourneySearchFrontier";
import type { JourneySearchFrontierNode } from "./JourneySearchFrontierNode";
import type { JourneyTrigger } from "./JourneyTrigger";
import type { JourneyUnmeasuredDepartures } from "./JourneyUnmeasuredDepartures";

/**
 * The answer of [`steps_to`].
 */
export interface JourneySteps {
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
   * Whether the start node was observed.
   */
  fromObserved: boolean;
  /**
   * What the component reachable from the start did not look at.
   */
  frontier: JourneySearchFrontier;
  /**
   * The edges of one shortest path, in order (empty when `steps` is null
   * or 0). Among equally short paths, the one whose next node key is
   * byte-wise smallest at every step; between parallel edges, the
   * smallest edge in graph order.
   */
  path: JourneyGraphEdge[];
  /**
   * Shortest OBSERVED number of navigation edges from start to target — an
   * upper bound on the true distance while the frontier is unsettled;
   * `null` = no observed path (which is a negative only when `frontier` is settled and
   * both endpoints were observed).
   */
  steps?: number | null;
  /**
   * The target node's key.
   */
  toKey: string;
  /**
   * Whether the target node was observed.
   */
  toObserved: boolean;
}
