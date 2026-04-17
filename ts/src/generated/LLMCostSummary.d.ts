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
  llmActionCount: number;
  /**
   * Per-model cost breakdowns.
   */
  perModel?: ModelCostBreakdown[];
  /**
   * Associated run ID.
   */
  runId: string;
  /**
   * Total estimated cost in USD.
   */
  totalCostUsd: number;
  /**
   * Total input tokens across all models.
   */
  totalTokensInput: number;
  /**
   * Total output tokens across all models.
   */
  totalTokensOutput: number;
  [k: string]: unknown;
}
