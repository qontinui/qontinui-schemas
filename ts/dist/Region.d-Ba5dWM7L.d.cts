/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Coordinate system identifier. Always specify which system you are working
 * with to avoid confusion in multi-monitor setups.
 */
type CoordinateSystem = "screen" | "virtual" | "monitor_relative";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * X,Y coordinates with optional coordinate system specification.
 *
 * Coordinates without an explicit `system` default to
 * [`CoordinateSystem::Screen`] for backward compatibility.
 */
interface Coordinates {
  /**
   * Monitor index (required when `system` is `MonitorRelative`).
   */
  monitor_index?: number | null;
  /**
   * Coordinate system. `None` defaults to `Screen`.
   */
  system?: CoordinateSystem | null;
  /**
   * X coordinate (horizontal position).
   */
  x: number;
  /**
   * Y coordinate (vertical position).
   */
  y: number;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Rectangular region on screen. Like [`Coordinates`], can optionally specify
 * which coordinate system the `x`/`y` are in.
 */
interface Region {
  /**
   * Height of the region (must be positive).
   */
  height: number;
  /**
   * Monitor index (required when `system` is `MonitorRelative`).
   */
  monitor_index?: number | null;
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

export type { CoordinateSystem as C, Region as R, Coordinates as a };
