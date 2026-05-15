/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Tagged sum-type for every event broadcast on the Spec API channel.
 *
 * Serialization uses `serde(tag = "type", rename_all = "kebab-case")` so
 * the wire discriminator is exactly the SSE event name (e.g.
 * `"spec-changed"`, `"spec-check-invoked"`).
 */
export type SpecApiEvent =
  | (SpecChanged & {
      type: "spec-changed";
    })
  | {
      at_ms: number;
      /**
       * "http" | "mcp" | "workflow-step"
       */
      invoked_via: string;
      page_ids: string[];
      snapshot_id: string;
      type: "spec-check-invoked";
      [k: string]: unknown;
    }
  | {
      at_ms: number;
      eval_error_count: number;
      no_match_count: number;
      overall_match_rate: number;
      page_count: number;
      partial_match_count: number;
      perfect_match_count: number;
      snapshot_id: string;
      total_duration_ms: number;
      type: "spec-check-completed";
      [k: string]: unknown;
    }
  | {
      at_ms: number;
      conjunct_name: string;
      /**
       * Small scalar/struct — NOT the full result.
       */
      observed: {
        [k: string]: unknown;
      };
      page_id: string;
      /**
       * serde-tagged variant name of `ConjunctRule`
       */
      rule_kind: string;
      snapshot_id: string;
      type: "spec-check-policy-violation";
      [k: string]: unknown;
    }
  | {
      at_ms: number;
      /**
       * Expected 2 per design §6.4.
       */
      consecutive_greens: number;
      page_id: string;
      proposal_id: string;
      /**
       * The snapshot that produced the second green.
       */
      snapshot_id: string;
      type: "flywheel-proposal-promoted";
      [k: string]: unknown;
    }
  | {
      at_ms: number;
      failing_assertion_id: string;
      failing_state_id: string;
      page_id: string;
      proposal_id: string;
      snapshot_id: string;
      type: "flywheel-proposal-demoted";
      [k: string]: unknown;
    };
