/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { AssertionScope } from "./AssertionScope";
import type { ConjunctRule } from "./ConjunctRule";
import type { MatchOutcome } from "./MatchOutcome";

/**
 * One AND-conjunct: a named rule applied within a scope.
 */
export interface PolicyConjunct {
  /**
   * Human-readable name used in `ConjunctEvaluation.name`.
   */
  name: string;
  rule: ConjunctRule;
  scope: AssertionScope;
}
