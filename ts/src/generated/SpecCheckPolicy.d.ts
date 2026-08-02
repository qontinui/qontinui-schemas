/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { AssertionScope } from "./AssertionScope";
import type { ConjunctRule } from "./ConjunctRule";
import type { MatchOutcome } from "./MatchOutcome";
import type { PolicyConjunct } from "./PolicyConjunct";

/**
 * AND-only policy — all conjuncts must hold for the policy to pass.
 * Per §5.7 there is no OR; compose multiple steps if you need
 * disjunction.
 */
export interface SpecCheckPolicy {
  /**
   * Conjuncts ANDed together.
   */
  conjuncts?: PolicyConjunct[];
}
