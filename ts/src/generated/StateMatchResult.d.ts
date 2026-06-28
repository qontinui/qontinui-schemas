/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { AssertionMiss } from "./AssertionMiss";
import type { AssertionOutcome } from "./AssertionOutcome";
import type { AssertionResult } from "./AssertionResult";
import type { CandidateMiss } from "./CandidateMiss";
import type { FieldDiff } from "./FieldDiff";
import type { MatchedElement } from "./MatchedElement";
import type { MissReason } from "./MissReason";

/**
 * Per-state evaluation result.
 */
export interface StateMatchResult {
  /**
   * One entry per IR assertion in the state.
   */
  assertions: AssertionResult[];
  /**
   * Fraction of assertions that passed (0.0..=1.0).
   */
  matchRate: number;
  /**
   * Identifier of the IR state.
   */
  stateId: string;
  /**
   * Human-readable state name (snapshot of `IrState.name`).
   */
  stateName: string;
}
