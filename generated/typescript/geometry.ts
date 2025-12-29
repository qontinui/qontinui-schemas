/**
 * Auto-generated TypeScript types from qontinui-schemas
 * DO NOT EDIT - regenerate with: poetry run python scripts/generate_typescript.py
 */

export enum CoordinateSystem {
  SCREEN = "screen",
  VIRTUAL = "virtual",
  MONITOR_RELATIVE = "monitor_relative",
}

export interface Coordinates {
  /** X coordinate (horizontal position) */
  x: number;
  /** Y coordinate (vertical position) */
  y: number;
  /** Coordinate system. None defaults to SCREEN for backward compatibility. */
  system?: CoordinateSystem | null;
  /** Monitor index (required when system is MONITOR_RELATIVE) */
  monitor_index?: number | null;
}

export interface Region {
  /** X coordinate of top-left corner */
  x: number;
  /** Y coordinate of top-left corner */
  y: number;
  /** Width of the region */
  width: number;
  /** Height of the region */
  height: number;
  /** Coordinate system. None defaults to SCREEN for backward compatibility. */
  system?: CoordinateSystem | null;
  /** Monitor index (required when system is MONITOR_RELATIVE) */
  monitor_index?: number | null;
}

export interface Monitor {
  /** OS-assigned monitor index (hardware enumeration order) */
  index: number;
  /** X position in absolute screen coordinates (can be negative) */
  x: number;
  /** Y position in absolute screen coordinates (can be negative) */
  y: number;
  /** Monitor width in pixels */
  width: number;
  /** Monitor height in pixels */
  height: number;
  /** Spatial position based on X coordinate (for UI display) */
  position: "left" | "center" | "right";
  /** Whether this is the primary/main monitor */
  is_primary?: boolean;
  /** DPI scale factor (1.0 = 100%, 1.5 = 150%, 2.0 = 200%) */
  scale_factor?: number;
  /** Display name (e.g., 'DELL U2720Q') */
  name?: string | null;
}

export interface VirtualDesktop {
  /** List of all monitors in the virtual desktop */
  monitors: Monitor[];
}
