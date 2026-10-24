/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { CacheProvenance } from "./CacheProvenance";
import type { MetricDistribution } from "./MetricDistribution";
import type { Observation } from "./Observation";
import type { ObservationCoverage } from "./ObservationCoverage";
import type { Producer } from "./Producer";
import type { Provenance } from "./Provenance";
import type { UnknownCode } from "./UnknownCode";
import type { UnknownInfo } from "./UnknownInfo";
import type { UnmeasuredDimension } from "./UnmeasuredDimension";

/**
 * One UI Bridge observation: `measured` (with `value`), `absent` (the
 * producer considered at least one item, measured all of it, and found
 * nothing), or `unknown` (the producer could not answer; `unknown.code` says
 * why). `provenance` is always present and every one of its keys is always
 * present.
 *
 * This type describes the wire SHAPE only; it is not a validator. It
 * accepts envelopes the canonical parser refuses: `absent` carrying `value`
 * or `unknown`, a present `"unknown": null`, and `absent` over non-empty
 * `provenance.coverage.unmeasured` or over `provenance.coverage.considered`
 * of 0. Parse an envelope you did not build with
 * `qontinui_vision_core::Observation` (Rust) or a parser enforcing the same
 * rules.
 */
export type Observation2 =
  | {
      provenance: Provenance;
      status: "measured";
      value: number;
      [k: string]: unknown;
    }
  | {
      provenance: Provenance;
      status: "absent";
      [k: string]: unknown;
    }
  | {
      provenance: Provenance;
      status: "unknown";
      unknown: UnknownInfo;
      [k: string]: unknown;
    };

/**
 * The answer of [`edge_timing`].
 */
export interface EdgeTiming {
  /**
   * Distinct reported app versions, sorted, `null` (= not reported) first.
   */
  appVersions: (string | null)[];
  /**
   * `summary.firstResponseMs`.
   */
  firstResponseMs: MetricDistribution;
  /**
   * `summary.layoutShiftAfterFirstResponseMs`.
   */
  layoutShiftAfterFirstResponseMs: MetricDistribution;
  /**
   * `summary.layoutShiftCount`.
   */
  layoutShiftCount: MetricDistribution;
  /**
   * `summary.layoutShiftTotal`.
   */
  layoutShiftTotal: MetricDistribution;
  /**
   * `summary.longTaskTotalMs`.
   */
  longTaskTotalMs: MetricDistribution;
  /**
   * Samples (ledger rows) the answer is over.
   */
  n: number;
  /**
   * Latest `observedAt`; `null` when `n` is 0.
   */
  newestObservedAt?: string | null;
  /**
   * Every observation's raw values, ordered by `(observedAt,
   * observationId)`, ties broken by the serialized `(appVersion,
   * timeline)`.
   */
  observations: EdgeTimingRow[];
  /**
   * Earliest `observedAt`; `null` when `n` is 0.
   */
  oldestObservedAt?: string | null;
  /**
   * `summary.settleMs`.
   */
  settleMs: MetricDistribution;
  /**
   * Of those, rows with no timeline.
   */
  withoutTimeline: number;
  [k: string]: unknown;
}
/**
 * One observation's values, exactly as its timeline summary stated them.
 * `null` (for every metric) = the row has no timeline.
 */
export interface EdgeTimingRow {
  /**
   * `null` = not reported.
   */
  appVersion?: string | null;
  /**
   * `summary.firstResponseMs`.
   */
  firstResponseMs?: Observation | null;
  /**
   * Whether the row carried a timeline.
   */
  hasTimeline: boolean;
  /**
   * `summary.layoutShiftAfterFirstResponseMs`.
   */
  layoutShiftAfterFirstResponseMs?: Observation | null;
  /**
   * `summary.layoutShiftCount`.
   */
  layoutShiftCount?: Observation2 | null;
  /**
   * `summary.layoutShiftTotal`.
   */
  layoutShiftTotal?: Observation | null;
  /**
   * `summary.longTaskTotalMs`.
   */
  longTaskTotalMs?: Observation | null;
  /**
   * The ledger row id — a `/vision/assert` `timeline_ref`.
   */
  observationId: string;
  /**
   * RFC 3339 UTC.
   */
  observedAt: string;
  /**
   * `summary.settleMs`.
   */
  settleMs?: Observation | null;
  [k: string]: unknown;
}
