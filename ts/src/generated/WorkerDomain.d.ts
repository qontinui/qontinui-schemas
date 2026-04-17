/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Domain assignment for a worker agent (lightweight — used inside a
 * [`VerificationPlan`]'s `worker_domains` list).
 */
export interface WorkerDomain {
  /**
   * Files / paths this worker owns (glob patterns).
   */
  filePatterns?: string[];
  /**
   * Optional specialization label (e.g., "frontend", "tests").
   */
  specialization?: string | null;
  /**
   * Additional system-prompt text to inject for this worker.
   */
  systemPromptAdditions?: string | null;
  /**
   * Worker identifier.
   */
  workerId: string;
  [k: string]: unknown;
}
