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
   * What the whole graph did not look at: any open frontier entry could be
   * the affordance that leads to one of `nodes`.
   */
  frontier: JourneySearchFrontier;
  /**
   * Matching nodes, sorted by key.
   */
  nodes: JourneyPriorKnowledgeNode[];
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
