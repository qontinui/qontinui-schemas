/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ConstraintSeverity } from './ConstraintSeverity';
import type { ConstraintViolation } from './ConstraintViolation';

/**
 * Result of evaluating a single constraint.
 */
export interface ConstraintResult {
  /**
   * The id of the constraint that was evaluated.
   */
  constraintId: string;
  /**
   * The human-readable name of the constraint that was evaluated.
   */
  constraintName: string;
  /**
   * Whether the constraint passed.
   */
  passed: boolean;
  severity: ConstraintSeverity;
  /**
   * Details about the violation (empty if passed).
   */
  violations: ConstraintViolation[];
  [k: string]: unknown;
}
