/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { FrontierEntry } from "./FrontierEntry";
import type { JourneyNode } from "./JourneyNode";
import type { JourneyUnmeasuredDepartures } from "./JourneyUnmeasuredDepartures";

/**
 * One node, inside a query's searched component, that was not fully looked
 * at.
 */
export interface JourneySearchFrontierNode {
  /**
   * The node.
   */
  nodeKey: string;
  /**
   * Its open frontier entries, grouped by reason (only reasons present),
   * each list sorted by `affordanceFingerprint` — the affordances that
   * would have to be explored to settle the question.
   */
  openByReason: {
    [k: string]: FrontierEntry[];
  };
  /**
   * Departures from it the navigation edge set left out.
   */
  unmeasuredDepartures: JourneyUnmeasuredDepartures;
  /**
   * Departures from it whose destination was never observed.
   */
  unobservedDepartures: number;
}
