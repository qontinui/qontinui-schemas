/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { CoordinateSystem } from './CoordinateSystem';
import type { Coordinates } from './Coordinates';
import type { MatchAdjustment } from './MatchAdjustment';
import type { MatchMethod } from './MatchMethod';
import type { PatternOptions } from './PatternOptions';
import type { PollingConfig } from './PollingConfig';
import type { Region } from './Region';
import type { SearchStrategy } from './SearchStrategy';

/**
 * Search options for target finding.
 */
export interface SearchOptions {
  adjustment?: MatchAdjustment | null;
  captureImage?: boolean | null;
  maxMatches?: number | null;
  maxMatchesToActOn?: number | null;
  minMatches?: number | null;
  pattern?: PatternOptions | null;
  polling?: PollingConfig | null;
  searchRegions?: Region[] | null;
  searchStrategy?: SearchStrategy | null;
  similarity?: number | null;
  timeout?: number | null;
  useDefinedRegion?: boolean | null;
}
