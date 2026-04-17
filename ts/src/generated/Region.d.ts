/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { CoordinateSystem } from './CoordinateSystem';

/**
 * Rectangular region on screen. Like [`Coordinates`], can optionally specify
 * which coordinate system the `x`/`y` are in.
 */
export interface Region {
  /**
   * Height of the region (must be positive).
   */
  height: number;
  /**
   * Monitor index (required when `system` is `MonitorRelative`).
   */
  monitorIndex?: number | null;
  /**
   * Coordinate system. `None` defaults to `Screen`.
   */
  system?: CoordinateSystem | null;
  /**
   * Width of the region (must be positive).
   */
  width: number;
  /**
   * X coordinate of the top-left corner.
   */
  x: number;
  /**
   * Y coordinate of the top-left corner.
   */
  y: number;
  [k: string]: unknown;
}
