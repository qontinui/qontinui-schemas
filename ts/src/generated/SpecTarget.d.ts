/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * What was observed and when.
 */
export interface SpecTarget {
  /**
   * ISO-8601 UTC timestamp; stamped post-run when the observation completed.
   */
  observedAt?: string | null;
  /**
   * The source website URL the spec was synthesized from.
   */
  sourceUrl: string;
  [k: string]: unknown;
}
