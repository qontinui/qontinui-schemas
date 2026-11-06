/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { JourneyTrigger } from "./JourneyTrigger";

/**
 * Every observation of one `(from, to, trigger, outcome)` combination,
 * aggregated. `to_node_unobserved` rows are not graph edges (they have no
 * destination); they count on the from-node's `unobservedDepartures`.
 */
export interface JourneyGraphEdge {
  /**
   * Earliest `observedAt` among them, RFC 3339 UTC.
   */
  firstObservedAt: string;
  /**
   * `fromNode.key()`.
   */
  fromKey: string;
  /**
   * Latest `observedAt` among them, RFC 3339 UTC.
   */
  lastObservedAt: string;
  /**
   * How many ledger rows this edge aggregates (≥ 1).
   */
  observations: number;
  /**
   * How the edge closed.
   */
  outcome:
    "changed" | "no_change" | "error" | "settle_timeout" | "to_node_unobserved";
  /**
   * `toNode.key()`.
   */
  toKey: string;
  /**
   * The trigger, exactly as recorded — enough to replay the step.
   */
  trigger: JourneyTrigger;
}
