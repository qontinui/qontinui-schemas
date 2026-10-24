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
