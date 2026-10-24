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
 * or withdrawn `no_change` row is never counted: that exclusion is a genuine
 * negative (a REJECTED row counts whatever its outcome). Each counter is its own
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
   * (outcome not `no_change`), and kept `pop` edges to ANOTHER node whose
   * outcome is `error` or `settle_timeout` (the same rule as an affordance
   * edge's). A REJECTED pop row is not here — a rejected row's trigger
   * cannot be trusted, so it counts as `rejected`. Not a navigation
   * departure, so a question about one component ignores it; the
   * graph-wide questions it can change — `dead_ends`' `backOnly`,
   * `requires_prior_knowledge`, the whole frontier — count it. Dimension
   * `excluded_back_arrivals`.
   */
  excludedBackArrivals: number;
  /**
   * Valid URL-trigger rows (`initial` / `push` / `replace` / `hash`) from
   * this node that the filter excluded or that were withdrawn, outcome not
   * `no_change`. A typed URL is not a navigation departure, so a question
   * about one component ignores it; but it is an ARRIVAL that decides
   * `requires_prior_knowledge`, and it may reveal a node no kept row
   * shows, so the graph-wide questions (`requires_prior_knowledge`,
   * `dead_ends`, the whole frontier) count it. Dimension
   * `excluded_url_arrivals`.
   */
  excludedUrlArrivals: number;
  /**
   * Live, valid affordance rows the request's filter (`since` /
   * `appVersion` / `runKind`) excluded, outcome not `no_change`. Dimension
   * `filtered_departures`.
   */
  filteredOut: number;
  /**
   * Rows that failed the contract (`validate()` or an unparseable
   * `observedAt`) — counted whatever their trigger AND outcome, since none
   * of the row's claims can be trusted (a row claiming `push` may really
   * be a click). A withdrawn row that also fails `validate()` counts here
   * too. Dimension `rejected_departures`.
   */
  rejected: number;
  /**
   * Frontier rows at this node that failed `validate()` — treated as OPEN
   * entries the read cannot list. A row whose `nodeKey` column disagrees
   * with its `node` is counted at BOTH keys (the column's, and the valid
   * node's), since either may be the true one. Dimension
   * `rejected_frontier`.
   */
  rejectedFrontier: number;
  /**
   * `settle_timeout` edges to a DIFFERENT node: not a navigation step (the
   * destination may be an intermediate render) but not a negative either.
   * Dimension `settle_timeout_departures`.
   */
  settleTimeout: number;
  /**
   * Withdrawn affordance rows (`invalidatedAt` set) that pass `validate()`
   * and whose outcome is not `no_change`. Dimension `withdrawn_departures`.
   */
  withdrawn: number;
}
