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
 * A new constraint proposed by the AI during an agentic phase.
 *
 * Serialized with `"type": "new_constraint"` via the `ConstraintProposal`
 * enum's internal tag.
 */
export interface NewConstraintProposal {
  constraint: Constraint;
  [k: string]: unknown;
}
