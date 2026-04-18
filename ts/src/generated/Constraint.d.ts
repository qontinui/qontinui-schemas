/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ConstraintCheck } from './ConstraintCheck';
import type { ConstraintSeverity } from './ConstraintSeverity';

/**
 * A constraint definition.
 */
export interface Constraint {
  check: ConstraintCheck;
  /**
   * Why this constraint exists (shown to the AI on violation).
   */
  description: string;
  /**
   * Whether this constraint is enabled. Default: true.
   */
  enabled: boolean;
  /**
   * Unique identifier (e.g., `"builtin:no-secrets"`, `"project:no-todos"`).
   */
  id: string;
  /**
   * Human-readable name.
   */
  name: string;
  severity: ConstraintSeverity;
}
