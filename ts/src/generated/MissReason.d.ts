/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Coarse-grained classification of an assertion miss.
 *
 * Per §5.3 — locked as a unit enum. Structured detail (per-field diffs,
 * near-match candidates) lives on `AssertionMiss` / `CandidateMiss`.
 */
export type MissReason =
  | "no_candidates"
  | "role_mismatch"
  | "text_mismatch"
  | "visibility_mismatch"
  | "attribute_mismatch"
  | "multiple_matches";
