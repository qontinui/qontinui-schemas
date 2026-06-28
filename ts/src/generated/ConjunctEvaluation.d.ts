/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { PolicyStatus } from "./PolicyStatus";

/**
 * Per-conjunct evaluation result.
 */
export interface ConjunctEvaluation {
  /**
   * Free-form human-readable explanation (e.g.
   * `"3 of 17 assertions failed (max 5)"`).
   */
  evidence: string;
  /**
   * Snapshot of `PolicyConjunct.name`.
   */
  name: string;
  status: PolicyStatus;
}
