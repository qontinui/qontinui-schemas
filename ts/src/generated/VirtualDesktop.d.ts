/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { Monitor } from './Monitor';
import type { MonitorPosition } from './MonitorPosition';

/**
 * The combined coordinate space of all monitors.
 *
 * - Origin: `(min_x, min_y)` across all monitors
 * - Size: bounding box containing all monitors
 * - Monitors may have gaps between them or different resolutions/DPI
 */
export interface VirtualDesktop {
  /**
   * List of all monitors in the virtual desktop.
   */
  monitors: Monitor[];
}
