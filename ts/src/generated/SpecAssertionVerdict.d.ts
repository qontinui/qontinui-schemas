/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { AssertionMiss } from "./AssertionMiss";
import type { CandidateMiss } from "./CandidateMiss";
import type { FieldDiff } from "./FieldDiff";
import type { MatchedElement } from "./MatchedElement";
import type { MissReason } from "./MissReason";
import type { UnknownCode } from "./UnknownCode";

/**
 * Internally-tagged on `status` — `{ "status": "pass", "matched": {...} }`,
 * `{ "status": "fail", "miss": {...} }` or
 * `{ "status": "unknown", "code": "no_expectation" }`. Distinct from the
 * worker-output `{ "type": ..., "data": ... }` envelope used elsewhere
 * in the codebase.
 *
 * Named `SpecAssertionVerdict` (it was `AssertionOutcome`) so the generated
 * bindings' `AssertionOutcome` is the shared three-state
 * [`crate::outcome::AssertionOutcome`].
 */
export type SpecAssertionVerdict =
  | {
      /**
       * The element that satisfied the assertion.
       */
      matched: MatchedElement;
      status: "pass";
      [k: string]: unknown;
    }
  | {
      /**
       * Diagnostic detail.
       */
      miss: AssertionMiss;
      status: "fail";
      [k: string]: unknown;
    }
  | {
      /**
       * WHY it could not be evaluated.
       */
      code: UnknownCode;
      status: "unknown";
      [k: string]: unknown;
    };
