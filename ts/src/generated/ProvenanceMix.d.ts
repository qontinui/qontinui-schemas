/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Count of nodes by provenance, within a section or overall. The
 * `observed + inferred` sum is the coverage denominator; `assumed` is tracked
 * separately for the assumption-fill rate.
 */
export interface ProvenanceMix {
  assumed: number;
  inferred: number;
  observed: number;
  [k: string]: unknown;
}
