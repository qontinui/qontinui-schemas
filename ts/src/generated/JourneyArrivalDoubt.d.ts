/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Trusted affordance or `pop` rows, excluded from the graph, that ARRIVE at
 * a node from a different node (outcome not `no_change`). Any of them may be
 * a way in that the user did not need a URL for, so
 * [`requires_prior_knowledge`] cannot list the node as settled.
 */
export interface JourneyArrivalDoubt {
  /**
   * Arrivals the request's filter excluded.
   */
  filtered: number;
  /**
   * Withdrawn arrivals.
   */
  withdrawn: number;
}
