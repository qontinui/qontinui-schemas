/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { AssertionMiss } from "./AssertionMiss";
import type { AssertionOutcome } from "./AssertionOutcome";
import type { CandidateMiss } from "./CandidateMiss";
import type { FieldDiff } from "./FieldDiff";
import type { MatchedElement } from "./MatchedElement";
import type { MissReason } from "./MissReason";

/**
 * Per-assertion evaluation result wrapper.
 */
export interface AssertionResult {
  /**
   * Identifier of the IR assertion.
   */
  assertionId: string;
  /**
   * Free-form category — `"layout"`, `"a11y"`, etc.
   */
  category: string;
  /**
   * Snapshot of `IrAssertion.description`.
   */
  description: string;
  outcome: AssertionOutcome;
  /**
   * Free-form severity string — `"critical"` / `"error"` / `"warning"`
   * / `"info"`. Free-form so policy adapters can introduce new tiers
   * without bumping the schema.
   */
  severity: string;
}
