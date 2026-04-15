/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { BuiltinOverrideProposal } from './BuiltinOverrideProposal';
import type { Constraint } from './Constraint';
import type { ConstraintCheck } from './ConstraintCheck';
import type { ConstraintSeverity } from './ConstraintSeverity';
import type { NewConstraintProposal } from './NewConstraintProposal';

/**
 * A constraint proposal from the AI.
 *
 * Internally tagged by `type` so the on-the-wire shape matches the TypeScript
 * discriminated union `{ type: "new_constraint" | "builtin_override", ... }`.
 */
export type ConstraintProposal =
  | (NewConstraintProposal & {
      type: "new_constraint";
    })
  | (BuiltinOverrideProposal & {
      type: "builtin_override";
    });
