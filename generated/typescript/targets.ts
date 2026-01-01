/**
 * Auto-generated TypeScript types from qontinui-schemas
 * DO NOT EDIT - regenerate with: poetry run python scripts/generate_typescript.py
 */

import type { Region, Coordinates } from './geometry';

export enum SearchStrategy {
  FIRST = "FIRST",
  ALL = "ALL",
  BEST = "BEST",
  EACH = "EACH",
}

export interface PollingConfig {
  interval?: number | null;
  maxAttempts?: number | null;
}

export interface PatternOptions {
  matchMethod?: "CORRELATION" | "CORRELATION_NORMED" | "SQUARED_DIFFERENCE" | "SQUARED_DIFFERENCE_NORMED" | null;
  scaleInvariant?: boolean | null;
  rotationInvariant?: boolean | null;
  minScale?: number | null;
  maxScale?: number | null;
  scaleStep?: number | null;
  minRotation?: number | null;
  maxRotation?: number | null;
  rotationStep?: number | null;
  useGrayscale?: boolean | null;
  useColorReduction?: boolean | null;
  colorTolerance?: number | null;
  useEdges?: boolean | null;
  edgeThreshold1?: number | null;
  edgeThreshold2?: number | null;
  nonMaxSuppression?: boolean | null;
  nmsThreshold?: number | null;
  minDistanceBetweenMatches?: number | null;
}

export interface MatchAdjustment {
  targetPosition?: string | null;
  targetOffset?: Coordinates | null;
  addW?: number | null;
  addH?: number | null;
  absoluteW?: number | null;
  absoluteH?: number | null;
  addX?: number | null;
  addY?: number | null;
}

export interface SearchOptions {
  similarity?: number | null;
  timeout?: number | null;
  searchRegions?: Region[] | null;
  searchStrategy?: SearchStrategy | null;
  useDefinedRegion?: boolean | null;
  maxMatchesToActOn?: number | null;
  minMatches?: number | null;
  maxMatches?: number | null;
  polling?: PollingConfig | null;
  pattern?: PatternOptions | null;
  adjustment?: MatchAdjustment | null;
  captureImage?: boolean | null;
}

export interface TextSearchOptions {
  ocrEngine?: "TESSERACT" | "EASYOCR" | "PADDLEOCR" | "NATIVE" | null;
  language?: string | null;
  whitelistChars?: string | null;
  blacklistChars?: string | null;
  matchType?: "EXACT" | "CONTAINS" | "STARTS_WITH" | "ENDS_WITH" | "REGEX" | "FUZZY" | null;
  caseSensitive?: boolean | null;
  ignoreWhitespace?: boolean | null;
  normalizeUnicode?: boolean | null;
  fuzzyThreshold?: number | null;
  editDistance?: number | null;
  preprocessing?: string[] | null;
  scaleFactor?: number | null;
  psmMode?: number | null;
  oemMode?: number | null;
  confidenceThreshold?: number | null;
}

export interface ImageTarget {
  type?: "image";
  imageIds: string[];
  searchOptions?: SearchOptions | null;
}

export interface RegionTarget {
  type?: "region";
  region: Region;
}

export interface TextTarget {
  type?: "text";
  text: string;
  searchOptions?: SearchOptions | null;
  textOptions?: TextSearchOptions | null;
}

export interface CoordinatesTarget {
  type?: "coordinates";
  coordinates: Coordinates;
}

export interface StateStringTarget {
  type?: "stateString";
  stateId: string;
  stringIds: string[];
  useAll?: boolean | null;
}

export interface StateRegionTarget {
  type?: "stateRegion";
  regionId: string;
}

export interface StateLocationTarget {
  type?: "stateLocation";
  locationId: string;
}

export interface StateImageTarget {
  type?: "stateImage";
  stateId: string;
  imageIds: string[];
  stateName?: string | null;
  imageNames?: string[] | null;
}

export interface CurrentPositionTarget {
  type?: "currentPosition";
}

export interface LastFindResultTarget {
  type?: "lastFindResult";
}

export interface ResultIndexTarget {
  type?: "resultIndex";
  index?: number;
}

export interface AllResultsTarget {
  type?: "allResults";
}

export interface ResultByImageTarget {
  type?: "resultByImage";
  imageId: string;
}

/**
 * Valid target type discriminators.
 * These are the literal string values used in target.type field.
 * IMPORTANT: All use lowercase (e.g., "stateImage" not "StateImage")
 */
export type TargetType =
  | "image"
  | "stateImage"
  | "region"
  | "text"
  | "coordinates"
  | "stateString"
  | "stateRegion"
  | "stateLocation"
  | "currentPosition"
  | "lastFindResult"
  | "resultIndex"
  | "allResults"
  | "resultByImage";

/**
 * Union of all valid target configurations.
 * This is the canonical definition - all frontends should use this.
 */
export type TargetConfig =
  | ImageTarget
  | RegionTarget
  | TextTarget
  | CoordinatesTarget
  | StateStringTarget
  | StateRegionTarget
  | StateLocationTarget
  | StateImageTarget
  | CurrentPositionTarget
  | LastFindResultTarget
  | ResultIndexTarget
  | AllResultsTarget
  | ResultByImageTarget;
