import { a as Coordinates, R as Region } from './Region.d-Ba5dWM7L.js';

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Role selector criterion — either a single role name or a list of roles.
 * Matches the Python `str | list[str] | None` type on
 * `AccessibilityTarget.role`.
 */
type AccessibilityRoleCriterion = string | string[];

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Match adjustment — modify the matched region before acting.
 */
interface MatchAdjustment {
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

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Template matching method used by OpenCV-style correlation search.
 */
type MatchMethod = "CORRELATION" | "CORRELATION_NORMED" | "SQUARED_DIFFERENCE" | "SQUARED_DIFFERENCE_NORMED";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * OCR engine for text search.
 */
type OcrEngine = "TESSERACT" | "EASYOCR" | "PADDLEOCR" | "NATIVE";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Advanced pattern-matching options.
 */
interface PatternOptions {
  colorTolerance?: number | null;
  edgeThreshold1?: number | null;
  edgeThreshold2?: number | null;
  matchMethod?: MatchMethod | null;
  maxRotation?: number | null;
  maxScale?: number | null;
  minDistanceBetweenMatches?: number | null;
  minRotation?: number | null;
  minScale?: number | null;
  nmsThreshold?: number | null;
  nonMaxSuppression?: boolean | null;
  rotationInvariant?: boolean | null;
  rotationStep?: number | null;
  scaleInvariant?: boolean | null;
  scaleStep?: number | null;
  useColorReduction?: boolean | null;
  useEdges?: boolean | null;
  useGrayscale?: boolean | null;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Polling configuration for search operations.
 */
interface PollingConfig {
  interval?: number | null;
  maxAttempts?: number | null;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Strategy for selecting among multiple matches in a FIND action.
 *
 * Variants serialize uppercase (`"FIRST"`, `"ALL"`, `"BEST"`, `"EACH"`) to
 * match the existing Python enum and all stored action configs.
 */
type SearchStrategy = "FIRST" | "ALL" | "BEST" | "EACH";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Search options for target finding.
 */
interface SearchOptions {
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
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Text-match comparison mode.
 */
type TextMatchType = "EXACT" | "CONTAINS" | "STARTS_WITH" | "ENDS_WITH" | "REGEX" | "FUZZY";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Text search options for OCR-based finding.
 */
interface TextSearchOptions {
  blacklistChars?: string | null;
  caseSensitive?: boolean | null;
  confidenceThreshold?: number | null;
  editDistance?: number | null;
  fuzzyThreshold?: number | null;
  ignoreWhitespace?: boolean | null;
  language?: string | null;
  matchType?: TextMatchType | null;
  normalizeUnicode?: boolean | null;
  ocrEngine?: OcrEngine | null;
  oemMode?: number | null;
  preprocessing?: string[] | null;
  psmMode?: number | null;
  scaleFactor?: number | null;
  whitelistChars?: string | null;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * The discriminated union over all 14 target configurations. Wire format is
 * a flat object tagged by the `type` field (`"image"`, `"stateImage"`, …).
 */
type TargetConfig =
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

/**
 * Target Configuration Types
 *
 * Discriminated union of 14 action-target variants (image, region, text,
 * coordinates, stateImage, stateRegion, stateLocation, stateString,
 * currentPosition, lastFindResult, resultIndex, allResults, resultByImage,
 * accessibility) + supporting search/pattern/OCR options.
 *
 * Source of truth: qontinui-schemas/rust/src/targets.rs.
 *
 * Wire format: `type` discriminator uses camelCase ("stateImage",
 * "currentPosition", "resultByImage", etc.).
 */

type ImageTarget = Extract<TargetConfig, {
    type: "image";
}>;
type RegionTarget = Extract<TargetConfig, {
    type: "region";
}>;
type TextTarget = Extract<TargetConfig, {
    type: "text";
}>;
type CoordinatesTarget = Extract<TargetConfig, {
    type: "coordinates";
}>;
type StateStringTarget = Extract<TargetConfig, {
    type: "stateString";
}>;
type StateRegionTarget = Extract<TargetConfig, {
    type: "stateRegion";
}>;
type StateLocationTarget = Extract<TargetConfig, {
    type: "stateLocation";
}>;
type StateImageTarget = Extract<TargetConfig, {
    type: "stateImage";
}>;
type CurrentPositionTarget = Extract<TargetConfig, {
    type: "currentPosition";
}>;
type LastFindResultTarget = Extract<TargetConfig, {
    type: "lastFindResult";
}>;
type ResultIndexTarget = Extract<TargetConfig, {
    type: "resultIndex";
}>;
type AllResultsTarget = Extract<TargetConfig, {
    type: "allResults";
}>;
type ResultByImageTarget = Extract<TargetConfig, {
    type: "resultByImage";
}>;
type AccessibilityTarget = Extract<TargetConfig, {
    type: "accessibility";
}>;
/** Wire-format type discriminator string for `TargetConfig`. */
type TargetType = TargetConfig["type"];

export type { AccessibilityTarget, AllResultsTarget, CoordinatesTarget, CurrentPositionTarget, ImageTarget, LastFindResultTarget, MatchAdjustment, MatchMethod, OcrEngine, PatternOptions, PollingConfig, RegionTarget, ResultByImageTarget, ResultIndexTarget, SearchOptions, SearchStrategy, StateImageTarget, StateLocationTarget, StateRegionTarget, StateStringTarget, TargetConfig, TargetType, TextMatchType, TextSearchOptions, TextTarget };
