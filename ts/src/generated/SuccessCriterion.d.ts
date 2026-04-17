/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { CriterionType } from './CriterionType';
import type { VerificationMethod } from './VerificationMethod';

/**
 * A success criterion that must be met for task completion.
 */
export interface SuccessCriterion {
  /**
   * Human-readable description.
   */
  description: string;
  /**
   * Domain this criterion belongs to (for multi-worker verification).
   */
  domain?: string | null;
  /**
   * For AI-evaluated criteria: the evaluation prompt.
   */
  evaluationPrompt?: string | null;
  /**
   * Unique identifier for this criterion.
   */
  id: string;
  /**
   * Whether failure of this criterion blocks task completion.
   * - `true` (default): failure blocks completion, worker must fix.
   * - `false`: failure is informational, doesn't block completion.
   */
  isCritical: boolean;
  /**
   * Whether this criterion must pass for task completion.
   */
  required: boolean;
  type: CriterionType;
  /**
   * Configuration blob for the verification method (command args, log
   * patterns, Playwright script path, etc.).
   */
  verificationConfig?: {
    [k: string]: unknown;
  };
  /**
   * For deterministic criteria: the verification method to use.
   */
  verificationMethod?: VerificationMethod | null;
  /**
   * Optional weight for partial success scoring.
   */
  weight?: number | null;
  [k: string]: unknown;
}
