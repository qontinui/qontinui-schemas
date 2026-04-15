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
  is_primary: boolean;
  /**
   * Display name (e.g., "DELL U2720Q").
   */
  name?: string | null;
  position: MonitorPosition;
  /**
   * DPI scale factor (1.0 = 100%, 1.5 = 150%, 2.0 = 200%).
   */
  scale_factor: number;
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
  [k: string]: unknown;
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
  [k: string]: unknown;
}

export type { CoordinateSystem, Coordinates, Monitor, MonitorPosition, Region, VirtualDesktop };
