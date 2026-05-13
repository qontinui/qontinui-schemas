/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { MatchOutcome } from './MatchOutcome';

/**
 * Internally-tagged on `kind`. AND-conjuncts only per §5.7 — no OR.
 * Unknown variants are rejected at the wire.
 */
export type ConjunctRule =
  | {
      kind: "all_pass";
      [k: string]: unknown;
    }
  | {
      /**
       * Maximum allowed failures.
       */
      count: number;
      kind: "max_failures";
      [k: string]: unknown;
    }
  | {
      kind: "failure_rate_below";
      /**
       * Threshold in [0.0, 1.0].
       */
      rate: number;
      [k: string]: unknown;
    }
  | {
      kind: "state_match_rate_at_least";
      /**
       * Threshold in [0.0, 1.0].
       */
      rate: number;
      [k: string]: unknown;
    }
  | {
      kind: "any_state_match_rate_at_least";
      /**
       * Threshold in [0.0, 1.0].
       */
      rate: number;
      [k: string]: unknown;
    }
  | {
      kind: "match_outcome_at_least";
      outcome: MatchOutcome;
      [k: string]: unknown;
    };
