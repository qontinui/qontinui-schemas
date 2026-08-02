/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Pixel coordinates and optional size of a matched pattern.
 *
 * Inline object on [`ActionExecutionCreate::match_location`]. Lifted to a
 * named struct so it round-trips through JSON Schema.
 */
export interface ExecutionMatchLocation {
  /**
   * Match height in pixels.
   */
  height?: number | null;
  /**
   * Match width in pixels.
   */
  width?: number | null;
  /**
   * X coordinate in pixels.
   */
  x: number;
  /**
   * Y coordinate in pixels.
   */
  y: number;
}
