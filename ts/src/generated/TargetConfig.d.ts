/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { AccessibilityRoleCriterion } from './AccessibilityRoleCriterion';
import type { CoordinateSystem } from './CoordinateSystem';
import type { Coordinates } from './Coordinates';
import type { MatchAdjustment } from './MatchAdjustment';
import type { MatchMethod } from './MatchMethod';
import type { OcrEngine } from './OcrEngine';
import type { PatternOptions } from './PatternOptions';
import type { PollingConfig } from './PollingConfig';
import type { Region } from './Region';
import type { SearchOptions } from './SearchOptions';
import type { SearchStrategy } from './SearchStrategy';
import type { TextMatchType } from './TextMatchType';
import type { TextSearchOptions } from './TextSearchOptions';

/**
 * The discriminated union over all 14 target configurations. Wire format is
 * a flat object tagged by the `type` field (`"image"`, `"stateImage"`, …).
 */
export type TargetConfig =
  | {
      imageIds: string[];
      searchOptions?: SearchOptions | null;
      type: "image";
      [k: string]: unknown;
    }
  | {
      region: Region;
      type: "region";
      [k: string]: unknown;
    }
  | {
      searchOptions?: SearchOptions | null;
      text: string;
      textOptions?: TextSearchOptions | null;
      type: "text";
      [k: string]: unknown;
    }
  | {
      coordinates: Coordinates;
      type: "coordinates";
      [k: string]: unknown;
    }
  | {
      stateId: string;
      stringIds: string[];
      type: "stateString";
      useAll?: boolean | null;
      [k: string]: unknown;
    }
  | {
      regionId: string;
      type: "stateRegion";
      [k: string]: unknown;
    }
  | {
      locationId: string;
      type: "stateLocation";
      [k: string]: unknown;
    }
  | {
      imageIds: string[];
      imageNames?: string[] | null;
      stateId: string;
      stateName?: string | null;
      type: "stateImage";
      [k: string]: unknown;
    }
  | {
      type: "currentPosition";
      [k: string]: unknown;
    }
  | {
      type: "lastFindResult";
      [k: string]: unknown;
    }
  | {
      index: number;
      type: "resultIndex";
      [k: string]: unknown;
    }
  | {
      type: "allResults";
      [k: string]: unknown;
    }
  | {
      imageId: string;
      type: "resultByImage";
      [k: string]: unknown;
    }
  | {
      captureFirst: boolean;
      cdpHost: string;
      cdpPort: number;
      isInteractive?: boolean | null;
      name?: string | null;
      nameContains?: string | null;
      ref?: string | null;
      role?: AccessibilityRoleCriterion | null;
      type: "accessibility";
      [k: string]: unknown;
    };
