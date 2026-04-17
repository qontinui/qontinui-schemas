/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { CoordinateSystem } from './CoordinateSystem';

/**
 * X,Y coordinates with optional coordinate system specification.
 *
 * Coordinates without an explicit `system` default to
 * [`CoordinateSystem::Screen`] for backward compatibility.
 */
export interface Coordinates {
  /**
   * Monitor index (required when `system` is `MonitorRelative`).
   */
  monitorIndex?: number | null;
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
