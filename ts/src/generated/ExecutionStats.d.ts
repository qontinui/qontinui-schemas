/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Aggregate execution statistics for a completed run.
 */
export interface ExecutionStats {
  /**
   * Mean per-action duration, in milliseconds.
   */
  avg_action_duration_ms?: number | null;
  /**
   * Number of actions with [`ActionStatus::Failed`].
   */
  failed_actions: number;
  /**
   * Number of actions that used an LLM.
   */
  llm_action_count?: number | null;
  /**
   * Number of actions with [`ActionStatus::Skipped`].
   */
  skipped_actions: number;
  /**
   * Number of actions with [`ActionStatus::Success`].
   */
  successful_actions: number;
  /**
   * Number of actions with [`ActionStatus::Timeout`].
   */
  timeout_actions: number;
  /**
   * Total number of actions executed.
   */
  total_actions: number;
  /**
   * Aggregate estimated cost in USD across all LLM actions.
   */
  total_cost_usd?: number | null;
  /**
   * Sum of all action durations, in milliseconds.
   */
  total_duration_ms: number;
  /**
   * Aggregate input tokens across all LLM actions.
   */
  total_tokens_input?: number | null;
  /**
   * Aggregate output tokens across all LLM actions.
   */
  total_tokens_output?: number | null;
  [k: string]: unknown;
}
