/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * A conditional routing rule that selects model/provider based on runtime
 * context.
 *
 * Rules are evaluated in order; the first matching rule wins. Condition
 * syntax: `"<variable> <op> <value>"` where:
 * - Variables: `verification_failures`, `iteration`, `stage_index`
 * - Operators: `>=`, `>`, `<=`, `<`, `==`, `!=`
 */
export interface RoutingRule {
  /**
   * Condition expression, e.g. `"verification_failures >= 2"`.
   */
  condition: string;
  /**
   * Max tokens override when this rule matches.
   */
  maxTokens?: number | null;
  /**
   * Model to use when this rule matches.
   */
  model?: string | null;
  /**
   * Provider to use when this rule matches.
   */
  provider?: string | null;
  /**
   * Temperature override when this rule matches.
   */
  temperature?: number | null;
  [k: string]: unknown;
}
