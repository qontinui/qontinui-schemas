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
import type { JourneyLedgerHealth } from "./JourneyLedgerHealth";
import type { JourneyNode } from "./JourneyNode";
import type { JourneySearchFrontier } from "./JourneySearchFrontier";
import type { JourneySearchFrontierNode } from "./JourneySearchFrontierNode";
import type { JourneyUnmeasuredDepartures } from "./JourneyUnmeasuredDepartures";

/**
 * The answer of [`requires_prior_knowledge`].
 */
export interface JourneyPriorKnowledge {
  /**
   * The graph's coverage.
   */
  coverage: JourneyCoverage;
  /**
   * The graph's filter.
   */
  filterEcho: JourneyFilter;
  /**
   * What the whole graph did not look at. An open frontier entry could be
   * the affordance that leads to one of `nodes`, and the absence of a
   * listed node is unknown unless this is settled.
   */
  frontier: JourneySearchFrontier;
  /**
   * Nodes that require prior knowledge, settled: they qualify, and no row
   * the graph left out could reach them by a click or a back button.
   * Sorted by key.
   */
  nodes: JourneyPriorKnowledgeNode[];
  /**
   * Nodes that qualify on their kept arrivals but cannot be settled, with
   * the counts of what could still reach them. Sorted by key.
   */
  unsettled: JourneyPriorKnowledgeUnsettledNode[];
}
/**
 * A node users only ever arrived at without clicking anything.
 */
export interface JourneyPriorKnowledgeNode {
  /**
   * Observed arrivals per navigation trigger (`initial`, `push`,
   * `replace`, `hash` only — by definition).
   */
  arrivals: {
    [k: string]: number;
  };
  /**
   * The node.
   */
  nodeKey: string;
}
/**
 * A node that qualifies on its KEPT arrivals, but that some row the graph
 * left out could still reach by a click or a back button — so whether it
 * requires prior knowledge is unknown.
 */
export interface JourneyPriorKnowledgeUnsettledNode {
  /**
   * Excluded (filtered / withdrawn) affordance or pop arrivals into it.
   */
  arrivalDoubt: JourneyArrivalDoubt;
  /**
   * Its kept URL-trigger arrivals, as in [`JourneyPriorKnowledgeNode`].
   */
  arrivals: {
    [k: string]: number;
  };
  /**
   * The node.
   */
  nodeKey: string;
  /**
   * [`JourneyGraph::unknown_destination_arrivals`] — any of them may
   * arrive here.
   */
  unknownDestinationArrivals: number;
  /**
   * [`JourneyGraph::untrusted_edge_rows`] — any of them may arrive here.
   */
  untrustedEdgeRows: number;
}
