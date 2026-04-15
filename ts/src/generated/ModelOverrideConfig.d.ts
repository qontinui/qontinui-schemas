/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { RoutingRule } from './RoutingRule';

/**
 * Per-phase model override configuration.
 *
 * Each phase can independently specify a provider and/or model, along with
 * optional temperature, max_tokens, fallback config, and conditional routing
 * rules.
 */
export interface ModelOverrideConfig {
  /**
   * Fallback model if the primary fails with a retryable error.
   */
  fallback_model?: string | null;
  /**
   * Fallback provider if the primary fails with a retryable error.
   */
  fallback_provider?: string | null;
  /**
   * Max output tokens override for this phase.
   */
  max_tokens?: number | null;
  /**
   * Model override for this phase.
   */
  model?: string | null;
  /**
   * Provider override for this phase.
   */
  provider?: string | null;
  /**
   * Conditional routing rules evaluated at runtime. First matching rule
   * wins; unmatched falls back to this config's static fields.
   */
  routing_rules?: RoutingRule[] | null;
  /**
   * Temperature override for this phase (`0.0`–`1.0`).
   */
  temperature?: number | null;
  [k: string]: unknown;
}
