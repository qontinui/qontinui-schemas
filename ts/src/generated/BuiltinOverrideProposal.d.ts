/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * A proposal to enable or disable a builtin constraint.
 *
 * Serialized with `"type": "builtin_override"` via the `ConstraintProposal`
 * enum's internal tag.
 */
export interface BuiltinOverrideProposal {
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
  [k: string]: unknown;
}
