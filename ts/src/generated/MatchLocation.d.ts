/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Location of a pattern match on screen.
 */
export interface MatchLocation {
  /**
   * Height of the matched region (optional — point matches omit it).
   */
  h?: number | null;
  /**
   * Width of the matched region (optional — point matches omit it).
   */
  w?: number | null;
  x: number;
  y: number;
}
