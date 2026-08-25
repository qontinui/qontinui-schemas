/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ConjunctEvaluation } from "./ConjunctEvaluation";
import type { PolicyStatus } from "./PolicyStatus";

/**
 * Top-level wrapper for evaluating a `SpecCheckPolicy` against a
 * `SpecCheckResult`.
 */
export interface PolicyEvaluation {
  /**
   * One result per conjunct in the policy, in policy declaration
   * order.
   */
  conjunctResults: ConjunctEvaluation[];
  /**
   * Aggregate of all conjunct statuses — fails if any conjunct fails;
   * indeterminate if any conjunct is indeterminate and none failed.
   */
  overallStatus: PolicyStatus;
}
