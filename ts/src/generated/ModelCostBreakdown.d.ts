/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Cost breakdown for a single LLM model.
 */
export interface ModelCostBreakdown {
  /**
   * Number of actions that used this model.
   */
  action_count: number;
  /**
   * Total cost in USD for this model.
   */
  cost_usd: number;
  /**
   * LLM model identifier.
   */
  model: string;
  /**
   * Provider name (e.g., `"anthropic"`, `"openai"`).
   */
  provider?: string | null;
  /**
   * Total input tokens for this model.
   */
  tokens_input: number;
  /**
   * Total output tokens for this model.
   */
  tokens_output: number;
  [k: string]: unknown;
}
