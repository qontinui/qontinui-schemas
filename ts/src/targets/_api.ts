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

import type { TargetConfig } from "../generated/TargetConfig";

export type { TargetConfig } from "../generated/TargetConfig";
export type { SearchStrategy } from "../generated/SearchStrategy";
export type { MatchMethod } from "../generated/MatchMethod";
export type { PollingConfig } from "../generated/PollingConfig";
export type { PatternOptions } from "../generated/PatternOptions";
export type { MatchAdjustment } from "../generated/MatchAdjustment";
export type { SearchOptions } from "../generated/SearchOptions";
export type { OcrEngine } from "../generated/OcrEngine";
export type { TextMatchType } from "../generated/TextMatchType";
export type { TextSearchOptions } from "../generated/TextSearchOptions";

// Per-variant narrowed aliases for consumers that want to Omit/Extend a
// single target shape. `TargetConfig` itself is the discriminated union;
// each alias is `TargetConfig` narrowed on the `type` discriminator.
export type ImageTarget = Extract<TargetConfig, { type: "image" }>;
export type RegionTarget = Extract<TargetConfig, { type: "region" }>;
export type TextTarget = Extract<TargetConfig, { type: "text" }>;
export type CoordinatesTarget = Extract<TargetConfig, { type: "coordinates" }>;
export type StateStringTarget = Extract<TargetConfig, { type: "stateString" }>;
export type StateRegionTarget = Extract<TargetConfig, { type: "stateRegion" }>;
export type StateLocationTarget = Extract<TargetConfig, { type: "stateLocation" }>;
export type StateImageTarget = Extract<TargetConfig, { type: "stateImage" }>;
export type CurrentPositionTarget = Extract<TargetConfig, { type: "currentPosition" }>;
export type LastFindResultTarget = Extract<TargetConfig, { type: "lastFindResult" }>;
export type ResultIndexTarget = Extract<TargetConfig, { type: "resultIndex" }>;
export type AllResultsTarget = Extract<TargetConfig, { type: "allResults" }>;
export type ResultByImageTarget = Extract<TargetConfig, { type: "resultByImage" }>;
export type AccessibilityTarget = Extract<TargetConfig, { type: "accessibility" }>;

/** Wire-format type discriminator string for `TargetConfig`. */
export type TargetType = TargetConfig["type"];
