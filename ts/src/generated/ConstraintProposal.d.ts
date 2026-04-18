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
 * A constraint proposal from the AI.
 *
 * Internally tagged by `type` so the on-the-wire shape matches the TypeScript
 * discriminated union `{ type: "new_constraint" | "builtin_override", ... }`.
 */
export type ConstraintProposal =
  | {
      constraint: Constraint;
      type: "new_constraint";
    }
  | {
      /**
       * Builtin suffix (e.g., `"no-secrets"`, `"no-debug-statements"`).
       */
      builtinSuffix: string;
      /**
       * Whether the builtin should be enabled.
       */
      enabled: boolean;
      /**
       * Human-readable justification for the override.
       */
      reason: string;
      type: "builtin_override";
    };
