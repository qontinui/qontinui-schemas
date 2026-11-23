/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { AssertionOutcome } from "./AssertionOutcome";
import type { OutcomeCounts } from "./OutcomeCounts";
import type { UnknownCode } from "./UnknownCode";

/**
 * The fold's answer. Serialized camelCase (`outcomeCounts`, `unknownCodes`),
 * the names the ui-bridge SDK already ships.
 *
 * Only [`fold_outcomes`] builds one, and deserialization re-checks what the
 * fold guarantees: the outcome is the one the counts imply, an empty fold is
 * `unknown` with exactly `[no_results]`, and codes are present iff something
 * was unknown. So a `FoldedOutcome` that Rust deserializes off the wire
 * cannot say `passed` over zero results, or `passed` beside a failure. The
 * generated TS and Python bindings carry the shape only, not these checks.
 */
export interface FoldedOutcome {
  outcome: AssertionOutcome;
  outcomeCounts: OutcomeCounts;
  /**
   * The distinct codes of the `unknown` inputs, in first-seen order —
   * `[no_results]` for an empty input, `[]` when nothing was unknown. Kept
   * on a `failed` fold too: the failure decides the outcome, and the
   * unknowns beside it are still work someone owes.
   */
  unknownCodes: UnknownCode[];
}
