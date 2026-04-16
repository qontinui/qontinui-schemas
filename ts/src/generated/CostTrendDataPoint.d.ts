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
  cost_usd: number;
  /**
   * Period label in `YYYY-MM-DD` format.
   */
  date: string;
  /**
   * Number of LLM actions in the period.
   */
  llm_action_count: number;
  /**
   * Number of runs in the period.
   */
  runs_count: number;
  /**
   * Total input tokens for the period.
   */
  tokens_input: number;
  /**
   * Total output tokens for the period.
   */
  tokens_output: number;
  [k: string]: unknown;
}
