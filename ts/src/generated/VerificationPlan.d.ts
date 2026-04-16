/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { CriterionType } from './CriterionType';
import type { SuccessCriterion } from './SuccessCriterion';
import type { VerificationMethod } from './VerificationMethod';
import type { WorkerDomain } from './WorkerDomain';

/**
 * The verification plan created by the planning agent.
 */
export interface VerificationPlan {
  /**
   * Execution steps to run before verification (GUI automation / setup).
   * Stored as raw JSON values because the step discriminated union spans
   * several types that are outside this module's scope.
   */
  execution_steps?: unknown[];
  /**
   * Summary of the goal.
   */
  goal_summary: string;
  /**
   * All success criteria that must be verified.
   */
  success_criteria: SuccessCriterion[];
  /**
   * Suggested number of worker agents.
   */
  suggested_worker_count: number;
  /**
   * Plan version (incremented on replan).
   */
  version: number;
  /**
   * Domain assignments for multiple workers.
   */
  worker_domains?: WorkerDomain[] | null;
  [k: string]: unknown;
}
