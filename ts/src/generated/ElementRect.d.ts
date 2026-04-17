/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Bounding rectangle of a DOM element, mirroring the output of
 * `Element.getBoundingClientRect()`.
 *
 * Contains both the origin+size pair (`x`, `y`, `width`, `height`) and the
 * edge offsets (`top`, `right`, `bottom`, `left`) for consumer convenience.
 */
export interface ElementRect {
  /**
   * Distance from the top of the viewport plus `height`.
   */
  bottom: number;
  /**
   * Height of the element in CSS pixels.
   */
  height: number;
  /**
   * Distance from the left of the viewport.
   */
  left: number;
  /**
   * Distance from the left of the viewport plus `width`.
   */
  right: number;
  /**
   * Distance from the top of the viewport.
   */
  top: number;
  /**
   * Width of the element in CSS pixels.
   */
  width: number;
  /**
   * X coordinate of the element's origin (same as `left`).
   */
  x: number;
  /**
   * Y coordinate of the element's origin (same as `top`).
   */
  y: number;
  [k: string]: unknown;
}
