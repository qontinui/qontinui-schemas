/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Departures from one node that the NAVIGATION edge set (affordance +
 * `changed`) leaves out, yet that may still lead somewhere — so a negative
 * answer over a component containing the node stays `unknown`, never
 * `absent`. The navigation counters count only `affordance`-triggered rows
 * (a typed URL is not a way the app offers); excluded back-button (`pop`)
 * rows have their own counter, [`Self::excluded_back_arrivals`]. A filtered
 * or withdrawn click or back that closed `no_change`, or `error` on its own
 * node, is never counted: that exclusion is a genuine negative (a URL-trigger
 * row never is one). (A REJECTED row is not counted here at all: nothing in it is
 * trusted, so it is placed at no node — see
 * [`JourneyGraph::unplaced_rejected_rows`].) Each counter is its own
 * `coverage.unmeasured` dimension — see [`JourneySearchFrontier::unmeasured`]
 * for the dimension names and codes.
 */
export interface JourneyUnmeasuredDepartures {
  /**
   * `error` edges to a DIFFERENT node: the action errored yet the
   * configuration moved. Dimension `errored_departures`.
   */
  errored: number;
  /**
   * Back-button (`pop`) rows from this node that are not a kept, changed
   * exit: valid `pop` rows the filter excluded or that were withdrawn
   * (unless a genuine negative: `no_change`, or `error` on its own node),
   * kept `pop` edges to ANOTHER node whose outcome is `error` or
   * `settle_timeout`, and kept `pop` self-loops that closed
   * `settle_timeout` (the same rules as an affordance edge's). A REJECTED pop row is not here — nothing in a rejected row
   * is trusted, so it counts in `JourneyGraph::unplaced_rejected_rows`.
   * Not a navigation
   * departure, so a question about one component ignores it; the
   * graph-wide questions it can change — `dead_ends`' `backOnly`,
   * `requires_prior_knowledge`, the whole frontier — count it. Dimension
   * `excluded_back_arrivals`.
   */
  excludedBackArrivals: number;
  /**
   * Valid URL-trigger rows (`initial` / `push` / `replace` / `hash`) from
   * this node that the filter excluded or that were withdrawn, WHATEVER
   * their outcome (a URL row is an arrival, never a genuine negative). A typed URL is not a navigation departure, so a question
   * about one component ignores it; but it is an ARRIVAL that decides
   * `requires_prior_knowledge`, and it may reveal a node no kept row
   * shows, so the graph-wide questions (`requires_prior_knowledge`,
   * `dead_ends`, the whole frontier) count it. Dimension
   * `excluded_url_arrivals`.
   */
  excludedUrlArrivals: number;
  /**
   * Live, valid affordance rows the request's filter (`since` /
   * `appVersion` / `runKind`) excluded, unless they are a genuine negative
   * (`no_change`, or `error` on their own node). Dimension
   * `filtered_departures`.
   */
  filteredOut: number;
  /**
   * Frontier rows at this node that failed `validate()` — treated as OPEN
   * entries the read cannot list. A row whose `nodeKey` column disagrees
   * with its `node` is counted at BOTH keys (the column's, and the valid
   * node's), since either may be the true one. Dimension
   * `rejected_frontier`.
   */
  rejectedFrontier: number;
  /**
   * Affordance edges that closed `settle_timeout` — to a DIFFERENT node
   * (the destination may be an intermediate render), or to the SAME node
   * (the destination may not have rendered before the snapshot). Not a
   * navigation step, but not a negative either. Dimension
   * `settle_timeout_departures`.
   */
  settleTimeout: number;
  /**
   * Withdrawn affordance rows (`invalidatedAt` set) that pass `validate()`,
   * unless they are a genuine negative (`no_change`, or `error` on their
   * own node). Dimension `withdrawn_departures`.
   */
  withdrawn: number;
}
