/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { Constraint } from './Constraint';
import type { ConstraintCheck } from './ConstraintCheck';
import type { ConstraintSeverity } from './ConstraintSeverity';

/**
 * Response for `POST /constraints/validate`.
 */
export interface ValidateConfigResponse {
  /**
   * Successfully parsed constraints (may be partial if some were skipped).
   */
  constraints: Constraint[];
  /**
   * Parse errors or non-fatal warnings (e.g., constraints skipped due to bad regex).
   */
  errors: string[];
  /**
   * Whether the config is fully valid (parseable with no errors or warnings).
   */
  valid: boolean;
}
