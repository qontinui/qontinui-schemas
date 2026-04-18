/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Bounding box for a discovered image element.
 *
 * Pixel-space rectangle on a source screenshot. `width` / `height` are `> 0`
 * on the Python side (validator not duplicated here — this is a wire-format
 * layer).
 */
export interface DiscoveryBoundingBox {
  /**
   * Height of the bounding box (pixels, positive).
   */
  height: number;
  /**
   * Width of the bounding box (pixels, positive).
   */
  width: number;
  /**
   * X coordinate of the top-left corner (pixels).
   */
  x: number;
  /**
   * Y coordinate of the top-left corner (pixels).
   */
  y: number;
}
