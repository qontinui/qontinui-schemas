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
  avgActionDurationMs?: number | null;
  /**
   * Number of actions with [`ActionStatus::Failed`].
   */
  failedActions: number;
  /**
   * Number of actions that used an LLM.
   */
  llmActionCount?: number | null;
  /**
   * Number of actions with [`ActionStatus::Skipped`].
   */
  skippedActions: number;
  /**
   * Number of actions with [`ActionStatus::Success`].
   */
  successfulActions: number;
  /**
   * Number of actions with [`ActionStatus::Timeout`].
   */
  timeoutActions: number;
  /**
   * Total number of actions executed.
   */
  totalActions: number;
  /**
   * Aggregate estimated cost in USD across all LLM actions.
   */
  totalCostUsd?: number | null;
  /**
   * Sum of all action durations, in milliseconds.
   */
  totalDurationMs: number;
  /**
   * Aggregate input tokens across all LLM actions.
   */
  totalTokensInput?: number | null;
  /**
   * Aggregate output tokens across all LLM actions.
   */
  totalTokensOutput?: number | null;
  [k: string]: unknown;
}
