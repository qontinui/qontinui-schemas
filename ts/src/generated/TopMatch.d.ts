/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { MatchLocation } from './MatchLocation';

/**
 * A single match result with confidence and location.
 */
export interface TopMatch {
  confidence: number;
  dimensions?: MatchLocation | null;
  location: MatchLocation;
  [k: string]: unknown;
}
