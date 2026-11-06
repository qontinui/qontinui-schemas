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
 * The answer of [`frontier`] — the open frontier of one node, or of the
 * whole graph.
 */
export interface JourneyFrontier {
  /**
   * The graph's coverage.
   */
  coverage: JourneyCoverage;
  /**
   * The open, listable frontier entries, sorted by `(nodeKey,
   * affordanceFingerprint)`.
   */
  entries: FrontierEntry[];
  /**
   * The graph's filter.
   */
  filterEcho: JourneyFilter;
  /**
   * Everything not looked at in the part asked about — including rows
   * that could not be listed (`rejected_frontier`). An EMPTY `entries` is
   * a real "nothing left to explore" only when this is settled and
   * [`JourneySearchFrontier::unsettled_because`] returns `None`.
   */
  frontier: JourneySearchFrontier;
  /**
   * The node asked about; `null` = the whole graph.
   */
  nodeKey?: string | null;
  /**
   * Whether that node was observed (always `true` for the whole graph).
   * `false` means "never seen", never "nothing open there".
   */
  nodeObserved: boolean;
}
