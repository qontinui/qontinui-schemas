/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Which of the THREE things happened to an assertion.
 *
 * `passed: bool` alone cannot say them: it collapses "evaluated, and the
 * expectation was violated" together with "could not be evaluated at all".
 * Those are different facts and call for different action — the first is a
 * bug in the page, the second is a gap in the snapshot (or in the caller's
 * inputs), and only the first should send anyone looking at the UI.
 *
 * The bit is kept alongside as the GATE, and every consumer that reads it
 * keeps its current behaviour: [`AssertionOutcome::Unknown`] is
 * `passed = false`. That is deliberate and is the safe direction — a
 * vacuous pass over an un-evaluated assertion is a silent false negative
 * for exactly the bug the assertion was written to catch.
 */
export type AssertionOutcome = "passed" | "failed" | "unknown";
