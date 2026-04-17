/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Single data point in an execution-trend time series.
 */
export interface ExecutionTrendDataPoint {
  /**
   * Mean run duration in seconds.
   */
  avgDurationSeconds: number;
  /**
   * Period label in `YYYY-MM-DD` format.
   */
  date: string;
  /**
   * Issues detected in the period.
   */
  issuesCount: number;
  /**
   * Number of runs in the period.
   */
  runsCount: number;
  /**
   * Success rate as a percentage.
   */
  successRate: number;
  /**
   * Total actions executed in the period.
   */
  totalActions: number;
  [k: string]: unknown;
}
