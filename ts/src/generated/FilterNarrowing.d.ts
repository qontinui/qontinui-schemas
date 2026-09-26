/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * A FILTER the surface narrowed before running the read — reported, never
 * applied silently, because a narrowed filter that reads as a complete answer
 * is the same defect class as a silent empty.
 *
 * Promoted from `coord.findings`' `resource_keys_truncated`, the only
 * narrowing any surface reported when this type was written: a list-valued
 * filter truncated at a cap rather than refused (truncating a FILTER only
 * narrows a read). Minimal on purpose — which parameter, how many values
 * were actually applied, and the cap that dropped the rest. A caller seeing
 * it knows a miss is meaningless for the values beyond the cap.
 */
export interface FilterNarrowing {
  /**
   * How many values of it the read actually applied.
   */
  applied: number;
  /**
   * The cap the surface enforces; values past it were dropped.
   */
  cap: number;
  /**
   * The query parameter that was narrowed, spelled as the caller spelled it
   * on the wire (e.g. `resource_keys`).
   */
  parameter: string;
  [k: string]: unknown;
}
