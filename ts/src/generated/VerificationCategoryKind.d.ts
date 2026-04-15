/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Verification-depth category for a step.
 *
 * Mirrors the TS `VerificationCategory` literal union. Kept local to this
 * module because it is only referenced from [`BaseStepFields`].
 */
export type VerificationCategoryKind =
  | "existence"
  | "uniqueness"
  | "referential_integrity"
  | "semantic_correctness"
  | "runtime_behavior";
