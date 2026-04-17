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
  actionCount: number;
  /**
   * Total cost in USD for this model.
   */
  costUsd: number;
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
  tokensInput: number;
  /**
   * Total output tokens for this model.
   */
  tokensOutput: number;
  [k: string]: unknown;
}
