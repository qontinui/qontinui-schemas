//! Target configuration models for action targeting.
//!
//! Mirrors `src/qontinui_schemas/config/models/targets.py` and the
//! `search.py` helpers it references. Rust is the source of truth; TS
//! and Python bindings regenerate from the JSON Schemas emitted here.
//!
//! The `TargetConfig` discriminated union carries 14 target variants — the
//! wire format uses an internal `type` tag in camelCase ("stateImage",
//! "currentPosition", "resultByImage", etc.) to match the hand-authored
//! Python enums and the existing TS consumers.
//!
//! Field names are wire-preserved via `#[serde(rename = "...")]` where the
//! Python side declares camelCase aliases (e.g. `imageIds`, `searchOptions`,
//! `captureFirst`, `cdpHost`).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::geometry::{Coordinates, Region};

// ============================================================================
// Supporting enums
// ============================================================================

/// Strategy for selecting among multiple matches in a FIND action.
///
/// Variants serialize uppercase (`"FIRST"`, `"ALL"`, `"BEST"`, `"EACH"`) to
/// match the existing Python enum and all stored action configs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum SearchStrategy {
    #[serde(rename = "FIRST")]
    First,
    #[serde(rename = "ALL")]
    All,
    #[serde(rename = "BEST")]
    Best,
    #[serde(rename = "EACH")]
    Each,
}

// ============================================================================
// Search helpers (SearchOptions / TextSearchOptions / PatternOptions /
// MatchAdjustment / PollingConfig)
// ============================================================================

/// Polling configuration for search operations.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct PollingConfig {
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "interval")]
    pub interval: Option<i64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "maxAttempts",
        alias = "max_attempts"
    )]
    pub max_attempts: Option<i64>,
}

/// Template matching method used by OpenCV-style correlation search.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum MatchMethod {
    #[serde(rename = "CORRELATION")]
    Correlation,
    #[serde(rename = "CORRELATION_NORMED")]
    CorrelationNormed,
    #[serde(rename = "SQUARED_DIFFERENCE")]
    SquaredDifference,
    #[serde(rename = "SQUARED_DIFFERENCE_NORMED")]
    SquaredDifferenceNormed,
}

/// Advanced pattern-matching options.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct PatternOptions {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "matchMethod",
        alias = "match_method"
    )]
    pub match_method: Option<MatchMethod>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "scaleInvariant",
        alias = "scale_invariant"
    )]
    pub scale_invariant: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "rotationInvariant",
        alias = "rotation_invariant"
    )]
    pub rotation_invariant: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "minScale", alias = "min_scale")]
    pub min_scale: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "maxScale", alias = "max_scale")]
    pub max_scale: Option<f64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "scaleStep",
        alias = "scale_step"
    )]
    pub scale_step: Option<f64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "minRotation",
        alias = "min_rotation"
    )]
    pub min_rotation: Option<f64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "maxRotation",
        alias = "max_rotation"
    )]
    pub max_rotation: Option<f64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "rotationStep",
        alias = "rotation_step"
    )]
    pub rotation_step: Option<f64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "useGrayscale",
        alias = "use_grayscale"
    )]
    pub use_grayscale: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "useColorReduction",
        alias = "use_color_reduction"
    )]
    pub use_color_reduction: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "colorTolerance",
        alias = "color_tolerance"
    )]
    pub color_tolerance: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "useEdges", alias = "use_edges")]
    pub use_edges: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "edgeThreshold1",
        alias = "edge_threshold1"
    )]
    pub edge_threshold1: Option<f64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "edgeThreshold2",
        alias = "edge_threshold2"
    )]
    pub edge_threshold2: Option<f64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "nonMaxSuppression",
        alias = "non_max_suppression"
    )]
    pub non_max_suppression: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "nmsThreshold",
        alias = "nms_threshold"
    )]
    pub nms_threshold: Option<f64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "minDistanceBetweenMatches",
        alias = "min_distance_between_matches"
    )]
    pub min_distance_between_matches: Option<f64>,
}

/// Match adjustment — modify the matched region before acting.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct MatchAdjustment {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "targetPosition",
        alias = "target_position"
    )]
    pub target_position: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "targetOffset",
        alias = "target_offset"
    )]
    pub target_offset: Option<Coordinates>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "addW", alias = "add_w")]
    pub add_w: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "addH", alias = "add_h")]
    pub add_h: Option<i64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "absoluteW",
        alias = "absolute_w"
    )]
    pub absolute_w: Option<i64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "absoluteH",
        alias = "absolute_h"
    )]
    pub absolute_h: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "addX", alias = "add_x")]
    pub add_x: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "addY", alias = "add_y")]
    pub add_y: Option<i64>,
}

/// Search options for target finding.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct SearchOptions {
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "similarity")]
    pub similarity: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "timeout")]
    pub timeout: Option<i64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "searchRegions",
        alias = "search_regions"
    )]
    pub search_regions: Option<Vec<Region>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "searchStrategy",
        alias = "strategy"
    )]
    pub strategy: Option<SearchStrategy>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "useDefinedRegion",
        alias = "use_defined_region"
    )]
    pub use_defined_region: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "maxMatchesToActOn",
        alias = "max_matches_to_act_on"
    )]
    pub max_matches_to_act_on: Option<i64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "minMatches",
        alias = "min_matches"
    )]
    pub min_matches: Option<i64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "maxMatches",
        alias = "max_matches"
    )]
    pub max_matches: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "polling")]
    pub polling: Option<PollingConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "pattern")]
    pub pattern: Option<PatternOptions>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "adjustment")]
    pub adjustment: Option<MatchAdjustment>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "captureImage",
        alias = "capture_image"
    )]
    pub capture_image: Option<bool>,
}

/// OCR engine for text search.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum OcrEngine {
    #[serde(rename = "TESSERACT")]
    Tesseract,
    #[serde(rename = "EASYOCR")]
    EasyOcr,
    #[serde(rename = "PADDLEOCR")]
    PaddleOcr,
    #[serde(rename = "NATIVE")]
    Native,
}

/// Text-match comparison mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum TextMatchType {
    #[serde(rename = "EXACT")]
    Exact,
    #[serde(rename = "CONTAINS")]
    Contains,
    #[serde(rename = "STARTS_WITH")]
    StartsWith,
    #[serde(rename = "ENDS_WITH")]
    EndsWith,
    #[serde(rename = "REGEX")]
    Regex,
    #[serde(rename = "FUZZY")]
    Fuzzy,
}

/// Text search options for OCR-based finding.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct TextSearchOptions {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "ocrEngine",
        alias = "ocr_engine"
    )]
    pub ocr_engine: Option<OcrEngine>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "language")]
    pub language: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "whitelistChars",
        alias = "whitelist_chars"
    )]
    pub whitelist_chars: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "blacklistChars",
        alias = "blacklist_chars"
    )]
    pub blacklist_chars: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "matchType",
        alias = "match_type"
    )]
    pub match_type: Option<TextMatchType>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "caseSensitive",
        alias = "case_sensitive"
    )]
    pub case_sensitive: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "ignoreWhitespace",
        alias = "ignore_whitespace"
    )]
    pub ignore_whitespace: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "normalizeUnicode",
        alias = "normalize_unicode"
    )]
    pub normalize_unicode: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "fuzzyThreshold",
        alias = "fuzzy_threshold"
    )]
    pub fuzzy_threshold: Option<f64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "editDistance",
        alias = "edit_distance"
    )]
    pub edit_distance: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "preprocessing")]
    pub preprocessing: Option<Vec<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "scaleFactor",
        alias = "scale_factor"
    )]
    pub scale_factor: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "psmMode", alias = "psm_mode")]
    pub psm_mode: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "oemMode", alias = "oem_mode")]
    pub oem_mode: Option<i64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "confidenceThreshold",
        alias = "confidence_threshold"
    )]
    pub confidence_threshold: Option<f64>,
}

// ============================================================================
// Role criterion for AccessibilityTarget — single or list of role strings
// ============================================================================

/// Role selector criterion — either a single role name or a list of roles.
/// Matches the Python `str | list[str] | None` type on
/// `AccessibilityTarget.role`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum AccessibilityRoleCriterion {
    Single(String),
    Any(Vec<String>),
}

// ============================================================================
// TargetConfig variants — internally tagged by `type` (camelCase)
// ============================================================================

/// The discriminated union over all 14 target configurations. Wire format is
/// a flat object tagged by the `type` field (`"image"`, `"stateImage"`, …).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum TargetConfig {
    /// Image target configuration supporting multiple images with search
    /// strategies. `imageIds` is required with at least one element — single
    /// image targets use a one-element list.
    #[serde(rename = "image")]
    Image {
        #[serde(rename = "imageIds")]
        image_ids: Vec<String>,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "searchOptions"
        )]
        search_options: Option<SearchOptions>,
    },

    /// Region target — a rectangular area of the screen.
    #[serde(rename = "region")]
    Region { region: Region },

    /// Text target — OCR-based targeting by visible text.
    #[serde(rename = "text")]
    Text {
        text: String,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "searchOptions"
        )]
        search_options: Option<SearchOptions>,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "textOptions"
        )]
        text_options: Option<TextSearchOptions>,
    },

    /// Absolute coordinates target.
    #[serde(rename = "coordinates")]
    Coordinates { coordinates: Coordinates },

    /// State-string target — references one or more strings in a state.
    #[serde(rename = "stateString")]
    StateString {
        #[serde(rename = "stateId")]
        state_id: String,
        #[serde(rename = "stringIds")]
        string_ids: Vec<String>,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "useAll"
        )]
        use_all: Option<bool>,
    },

    /// Target a StateRegion by ID. Preserves the region's monitor
    /// association instead of using a global default.
    #[serde(rename = "stateRegion")]
    StateRegion {
        #[serde(rename = "regionId")]
        region_id: String,
    },

    /// Target a StateLocation by ID. Preserves the location's monitor
    /// association.
    #[serde(rename = "stateLocation")]
    StateLocation {
        #[serde(rename = "locationId")]
        location_id: String,
    },

    /// Target a StateImage by state ID + image IDs. Used by navigation to
    /// verify state via FIND on state-associated images.
    #[serde(rename = "stateImage")]
    StateImage {
        #[serde(rename = "stateId")]
        state_id: String,
        #[serde(rename = "imageIds")]
        image_ids: Vec<String>,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "stateName"
        )]
        state_name: Option<String>,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "imageNames"
        )]
        image_names: Option<Vec<String>>,
    },

    /// Act at the current mouse position (pure action — no localization).
    #[serde(rename = "currentPosition")]
    CurrentPosition,

    /// Use the location from the most recent FIND action.
    #[serde(rename = "lastFindResult")]
    LastFindResult,

    /// Target a specific match (by zero-based index) from the last action
    /// result.
    #[serde(rename = "resultIndex")]
    ResultIndex {
        #[serde(default)]
        index: i64,
    },

    /// Target all matches from the last action result.
    #[serde(rename = "allResults")]
    AllResults,

    /// Target the match that came from a specific image ID in a multi-image
    /// FIND result (requires the previous FIND used `EACH`).
    #[serde(rename = "resultByImage")]
    ResultByImage {
        #[serde(rename = "imageId")]
        image_id: String,
    },

    /// Target an element by accessibility ref or selector — AI-optimized
    /// element selection via `@e1` refs or role/name matching. Requires a
    /// captured accessibility tree.
    #[serde(rename = "accessibility")]
    Accessibility {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        r#ref: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        role: Option<AccessibilityRoleCriterion>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "nameContains"
        )]
        name_contains: Option<String>,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "isInteractive"
        )]
        is_interactive: Option<bool>,
        #[serde(default = "default_true", rename = "captureFirst")]
        capture_first: bool,
        #[serde(default = "default_cdp_host", rename = "cdpHost")]
        cdp_host: String,
        #[serde(default = "default_cdp_port", rename = "cdpPort")]
        cdp_port: i64,
    },
}

fn default_true() -> bool {
    true
}
fn default_cdp_host() -> String {
    "localhost".to_string()
}
fn default_cdp_port() -> i64 {
    9222
}
