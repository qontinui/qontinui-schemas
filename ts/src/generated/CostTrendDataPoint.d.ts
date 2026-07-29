/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Single data point in a cost-trend time series.
 */
export interface CostTrendDataPoint {
  /**
   * Total cost in USD for the period.
   */
  costUsd: number;
  /**
   * Period label in `YYYY-MM-DD` format.
   */
  date: string;
  /**
   * Number of LLM actions in the period.
   */
  llmActionCount: number;
  /**
   * Number of runs in the period.
   */
  runsCount: number;
  /**
   * Total input tokens for the period.
   */
  tokensInput: number;
  /**
   * Total output tokens for the period.
   */
  tokensOutput: number;
}
