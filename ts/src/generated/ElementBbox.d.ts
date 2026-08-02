/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Viewport-relative bounding box in CSS pixels.
 *
 * This is the live on-screen geometry captured at snapshot time via
 * `Element.getBoundingClientRect()`. Present only when the SDK has a
 * live DOM ref for the element; absent when the element is registered
 * without a ref or when the snapshot is served from the DOM-fallback
 * scanner.
 *
 * Click target for a hit is `(x + width/2, y + height/2)`.
 */
export interface ElementBbox {
  /**
   * Height of the bbox in CSS pixels.
   */
  height: number;
  /**
   * Width of the bbox in CSS pixels.
   */
  width: number;
  /**
   * X coordinate of the bbox origin in viewport CSS pixels.
   */
  x: number;
  /**
   * Y coordinate of the bbox origin in viewport CSS pixels.
   */
  y: number;
}
