/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { CacheProvenance } from "./CacheProvenance";
import type { EdgeTiming } from "./EdgeTiming";
import type { JourneyFilter } from "./JourneyFilter";
import type { JourneyLedgerHealth } from "./JourneyLedgerHealth";
import type { MetricDistribution } from "./MetricDistribution";
import type { Observation } from "./Observation";
import type { ObservationCoverage } from "./ObservationCoverage";
import type { Producer } from "./Producer";
import type { Provenance } from "./Provenance";
import type { UnknownCode } from "./UnknownCode";
import type { UnknownInfo } from "./UnknownInfo";
import type { UnmeasuredDimension } from "./UnmeasuredDimension";

/**
 * The answer value of `GET /apps/{app_id}/journey/edge-timing`.
 *
 * **Samples POOL every trigger and outcome of the `(from, to)` pair**:
 * affordance clicks and typed URLs alike, and `changed`, `settle_timeout`
 * and `error` closes alike. Keeping only `affordance` + `changed` would drop
 * exactly the slow and failing interactions (a `settle_timeout` close is the
 * slowest sample there is) and bias every percentile fast. The pooled
 * composition is reported in `samplesByOutcome` / `samplesByTrigger`, so a
 * reader who wants one slice can see whether the pool is dominated by
 * another.
 */
export interface JourneyEdgeTimingAnswer {
  /**
   * Live (not withdrawn) observations of this edge the filter removed.
   */
  edgesFilteredOut: number;
  /**
   * Withdrawn observations of this edge, excluded.
   */
  edgesInvalidated: number;
  /**
   * The filter the observations were narrowed by.
   */
  filterEcho: JourneyFilter;
  /**
   * The from-node key asked about.
   */
  fromKey: string;
  /**
   * The ledger the rows were read from.
   */
  ledger: JourneyLedgerHealth;
  /**
   * Observations of this edge left out as contract violations (a row that
   * fails `validate()` or whose `observedAt` is not RFC 3339) — the same
   * rows the graph routes list as `rejected`; never samples, never
   * "filtered out".
   */
  rowsRejected: number;
  /**
   * Samples per edge outcome (wire string), pooled into `timing`.
   */
  samplesByOutcome: {
    [k: string]: number;
  };
  /**
   * Samples per navigation trigger (wire string), pooled into `timing`.
   */
  samplesByTrigger: {
    [k: string]: number;
  };
  /**
   * Observations whose stored timeline did not parse (counted in `timing`
   * as without a timeline).
   */
  timelinesUnparseable: number;
  /**
   * The distribution, with its sample size.
   */
  timing: EdgeTiming;
  /**
   * The to-node key asked about.
   */
  toKey: string;
}
