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
  assertionsPassed: number | null;
  /**
   * Total number of assertions.
   */
  assertionsTotal: number | null;
  /**
   * Results of individual named checks (e.g., lint, type, test).
   */
  checkResults: IndividualCheckResult[] | null;
  /**
   * Captured browser/console output.
   */
  consoleOutput: string | null;
  /**
   * Exit code of the spawned process.
   */
  exitCode: number | null;
  /**
   * Captured page snapshot (HTML or serialized representation).
   */
  pageSnapshot: string | null;
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
  stepId: string;
}
