/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { CheckIssueDetail } from './CheckIssueDetail';
import type { IndividualCheckResult } from './IndividualCheckResult';

/**
 * Detailed output captured for a single verification step.
 */
export interface VerificationStepDetails {
  /**
   * Number of assertions that passed.
   */
  assertions_passed: number | null;
  /**
   * Total number of assertions.
   */
  assertions_total: number | null;
  /**
   * Results of individual named checks (e.g., lint, type, test).
   */
  check_results: IndividualCheckResult[] | null;
  /**
   * Captured browser/console output.
   */
  console_output: string | null;
  /**
   * Exit code of the spawned process.
   */
  exit_code: number | null;
  /**
   * Captured page snapshot (HTML or serialized representation).
   */
  page_snapshot: string | null;
  /**
   * Phase the step belongs to (e.g., `"setup"`, `"verification"`).
   */
  phase: string;
  /**
   * Captured stderr, if any.
   */
  stderr: string | null;
  /**
   * Captured stdout, if any.
   */
  stdout: string | null;
  /**
   * ID of the step this detail belongs to.
   */
  step_id: string;
  [k: string]: unknown;
}
