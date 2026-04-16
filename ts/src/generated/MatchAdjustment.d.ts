/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { CoordinateSystem } from './CoordinateSystem';
import type { Coordinates } from './Coordinates';

/**
 * Match adjustment — modify the matched region before acting.
 */
export interface MatchAdjustment {
  absoluteH?: number | null;
  absoluteW?: number | null;
  addH?: number | null;
  addW?: number | null;
  addX?: number | null;
  addY?: number | null;
  targetOffset?: Coordinates | null;
  targetPosition?: string | null;
  [k: string]: unknown;
}
