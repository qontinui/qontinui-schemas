/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ScreenshotAnnotationShape } from './ScreenshotAnnotationShape';

/**
 * Annotation overlaid on a screenshot (box, circle, arrow, or text).
 */
export interface ScreenshotAnnotation {
  /**
   * CSS-style color string (e.g., `"#FF0000"`).
   */
  color?: string | null;
  /**
   * Annotation height in pixels.
   */
  height?: number | null;
  /**
   * Free-form label for the annotation.
   */
  label?: string | null;
  type: ScreenshotAnnotationShape;
  /**
   * Annotation width in pixels.
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
  [k: string]: unknown;
}
