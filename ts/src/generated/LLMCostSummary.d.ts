/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ModelCostBreakdown } from './ModelCostBreakdown';

/**
 * Aggregate LLM cost summary for an execution run.
 */
export interface LLMCostSummary {
  /**
   * Number of actions that used an LLM.
   */
  llm_action_count: number;
  /**
   * Per-model cost breakdowns.
   */
  per_model?: ModelCostBreakdown[];
  /**
   * Associated run ID.
   */
  run_id: string;
  /**
   * Total estimated cost in USD.
   */
  total_cost_usd: number;
  /**
   * Total input tokens across all models.
   */
  total_tokens_input: number;
  /**
   * Total output tokens across all models.
   */
  total_tokens_output: number;
  [k: string]: unknown;
}
