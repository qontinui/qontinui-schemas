/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Trusted affordance or `pop` rows, excluded from the graph, that ARRIVE at
 * a node from a different node with outcome `changed`. Any of them may be a
 * way in that the user did not need a URL for, so
 * [`requires_prior_knowledge`] cannot list the node as settled. An excluded
 * row that closed `error` or `settle_timeout` (or has no `toNode`) is not
 * here: its recorded destination may be an intermediate state, so it routes
 * to the graph-wide [`JourneyGraph::unknown_destination_arrivals`].
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
