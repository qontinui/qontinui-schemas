/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { MonitorPosition } from './MonitorPosition';

/**
 * Standardized monitor information — a physical display with its position
 * in the virtual desktop and metadata for UI display.
 */
export interface Monitor {
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
