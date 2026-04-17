/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Configuration for the context-summarization subsystem.
 *
 * When the loop's rolling context approaches the token budget, older
 * iterations are compressed into a summary to keep the prompt small.
 */
export interface SummarizationConfig {
  /**
   * Whether summarization is active.
   */
  enabled: boolean;
  /**
   * Target maximum tokens for the full loop context.
   */
  maxTokensBudget: number;
  /**
   * Number of most-recent iterations to keep verbatim (never summarized).
   */
  preserveLastNIterations: number;
  /**
   * Cap on tokens emitted by a single summarization pass.
   */
  summaryMaxTokens: number;
  /**
   * Fraction of the token budget (0.0–1.0) at which summarization triggers.
   */
  tokenThresholdPct: number;
  [k: string]: unknown;
}
