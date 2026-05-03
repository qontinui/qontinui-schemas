export { a as CoordinateSystem, C as Coordinates, R as Region } from './Region.d-DtT3UphX.js';

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Spatial position of a monitor in a left-to-right layout, derived from the
 * X coordinate. Used for human-friendly display; use `index` for
 * programmatic operations.
 */
type MonitorPosition = "left" | "center" | "right";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Standardized monitor information — a physical display with its position
 * in the virtual desktop and metadata for UI display.
 */
interface Monitor {
  /**
   * Monitor height in pixels.
   */
  height: number;
  /**
   * OS-assigned monitor index (hardware enumeration order).
   */
  index: number;
  /**
   * Whether this is the primary/main monitor.
   */
  isPrimary: boolean;
  /**
   * Display name (e.g., "DELL U2720Q").
   */
  name?: string | null;
  position: MonitorPosition;
  /**
   * DPI scale factor (1.0 = 100%, 1.5 = 150%, 2.0 = 200%).
   */
  scaleFactor: number;
  /**
   * Monitor width in pixels.
   */
  width: number;
  /**
   * X position in absolute screen coordinates (can be negative).
   */
  x: number;
  /**
   * Y position in absolute screen coordinates (can be negative).
   */
  y: number;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * The combined coordinate space of all monitors.
 *
 * - Origin: `(min_x, min_y)` across all monitors
 * - Size: bounding box containing all monitors
 * - Monitors may have gaps between them or different resolutions/DPI
 */
interface VirtualDesktop {
  /**
   * List of all monitors in the virtual desktop.
   */
  monitors: Monitor[];
}

export type { Monitor, MonitorPosition, VirtualDesktop };
