/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { AssertionMiss } from './AssertionMiss';
import type { CandidateMiss } from './CandidateMiss';
import type { FieldDiff } from './FieldDiff';
import type { MatchedElement } from './MatchedElement';
import type { MissReason } from './MissReason';

/**
 * Internally-tagged on `status` — `{ "status": "pass", "matched": {...} }`
 * or `{ "status": "fail", "miss": {...} }`. Distinct from the
 * worker-output `{ "type": ..., "data": ... }` envelope used elsewhere
 * in the codebase.
 */
export type AssertionOutcome =
  | {
      matched: MatchedElement;
      status: "pass";
      [k: string]: unknown;
    }
  | {
      miss: AssertionMiss;
      status: "fail";
      [k: string]: unknown;
    };
