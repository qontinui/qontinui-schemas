/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Token usage and cost metrics for an LLM-powered action.
 */
export interface LLMMetrics {
  /**
   * Estimated cost in USD.
   */
  costUsd?: number | null;
  /**
   * Generation parameters (temperature, max_tokens, etc.).
   */
  generationParams?: {
    [k: string]: unknown;
  } | null;
  /**
   * LLM model identifier.
   */
  model?: string | null;
  /**
   * Provider name (e.g., `"anthropic"`, `"openai"`).
   */
  provider?: string | null;
  /**
   * Input/prompt token count.
   */
  tokensInput?: number | null;
  /**
   * Completion token count.
   */
  tokensOutput?: number | null;
  /**
   * Computed total token count.
   */
  tokensTotal?: number | null;
}
