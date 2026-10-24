/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * One metric over every sample. The order statistics are over the MEASURED
 * values only (degraded ones included — their value stands) and are `null`
 * when none was measured; `absent` and `unknownByCode` say what the rest
 * were. `measured + absent + Σ unknownByCode + withoutTimeline = n`.
 */
export interface MetricDistribution {
  /**
   * Samples whose producer looked and found the thing did not occur.
   */
  absent: number;
  /**
   * Of those, how many were degraded (`coverage.unmeasured` non-empty).
   */
  degraded: number;
  /**
   * Largest measured value.
   */
  max?: number | null;
  /**
   * Samples with a measured value.
   */
  measured: number;
  /**
   * Nearest-rank 50th percentile (the lower middle value of an even count).
   */
  median?: number | null;
  /**
   * Smallest measured value.
   */
  min?: number | null;
  /**
   * Nearest-rank 90th percentile.
   */
  p90?: number | null;
  /**
   * Samples the producer could not answer, per `UnknownCode` wire value.
   */
  unknownByCode: {
    [k: string]: number;
  };
  /**
   * Samples with no timeline at all.
   */
  withoutTimeline: number;
  [k: string]: unknown;
}
