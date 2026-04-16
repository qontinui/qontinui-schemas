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
  avg_duration_seconds: number;
  /**
   * Period label in `YYYY-MM-DD` format.
   */
  date: string;
  /**
   * Issues detected in the period.
   */
  issues_count: number;
  /**
   * Number of runs in the period.
   */
  runs_count: number;
  /**
   * Success rate as a percentage.
   */
  success_rate: number;
  /**
   * Total actions executed in the period.
   */
  total_actions: number;
  [k: string]: unknown;
}
