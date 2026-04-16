//! Unified State Discovery models.
//!
//! Rust is the source of truth. Ported from
//! `src/qontinui_schemas/discovery/models.py`. TS and Python bindings
//! regenerate from the JSON Schemas emitted here.
//!
//! These schemas represent the output of state discovery from any source:
//! - Playwright (web extraction)
//! - UI Bridge (render log analysis)
//! - Recording (user session recording)
//! - Vision (screenshot analysis)
//! - Manual (user-defined)
//!
//! The unified format captures:
//! - **Images**: Bounding boxes on screenshots with pixel representation.
//! - **States**: Collections of images that appear together (co-occurrence).
//! - **Transitions**: Actions that change the active set of states.
//!
//! Shared across:
//! - `qontinui-web` backend (storage and API)
//! - `qontinui-web` frontend (display)
//! - `qontinui-runner` (producing results)
//! - `qontinui` library (state discovery algorithms)
//!
//! Wire-format notes:
//! - UUIDs / IDs serialize as plain strings (see crate-level docs).
//! - Dates/timestamps are ISO 8601 strings (no `chrono` dependency).
//! - Metadata uses `serde_json::Value` for free-form `dict[str, Any]` fields.
//! - Enum string values are lowercase `snake_case` to match the Python
//!   `str, Enum` base classes in `models.py`.

use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ============================================================================
// Enums
// ============================================================================

/// Source of a state-discovery result.
///
/// Identifies which discovery pathway produced the state machine. Mirrors
/// Python `DiscoverySourceType(str, Enum)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DiscoverySourceType {
    /// Playwright / web DOM extraction.
    Playwright,
    /// Runner UI Bridge (render-log analysis).
    UiBridge,
    /// User-session recording.
    Recording,
    /// Screenshot / vision analysis.
    Vision,
    /// Manually authored by the user.
    Manual,
}

/// Type of action that triggers a state transition.
///
/// Mirrors Python `TransitionTriggerType(str, Enum)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TransitionTriggerType {
    /// Mouse click or tap.
    Click,
    /// Typed text input.
    Type,
    /// Scroll gesture.
    Scroll,
    /// Hover / pointer-over.
    Hover,
    /// Custom / tool-specific trigger.
    Custom,
}

impl Default for TransitionTriggerType {
    /// Matches Python `default=TransitionTriggerType.CLICK`.
    fn default() -> Self {
        Self::Click
    }
}

// ============================================================================
// Core Components
// ============================================================================

/// Bounding box for a discovered image element.
///
/// Pixel-space rectangle on a source screenshot. `width` / `height` are `> 0`
/// on the Python side (validator not duplicated here — this is a wire-format
/// layer).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DiscoveryBoundingBox {
    /// X coordinate of the top-left corner (pixels).
    pub x: f64,
    /// Y coordinate of the top-left corner (pixels).
    pub y: f64,
    /// Width of the bounding box (pixels, positive).
    pub width: f64,
    /// Height of the bounding box (pixels, positive).
    pub height: f64,
}

/// Trigger for a discovered state transition.
///
/// Describes the action (click, type, …) that caused a transition. All
/// identifying fields are optional — different discovery sources populate
/// different subsets.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DiscoveryTransitionTrigger {
    /// Type of trigger action. Defaults to `click` when omitted on the wire.
    #[serde(default)]
    pub r#type: TransitionTriggerType,
    /// ID of the image that was clicked/interacted with.
    #[serde(
        default,
        rename = "imageId",
        skip_serializing_if = "Option::is_none"
    )]
    pub image_id: Option<String>,
    /// ID of the DOM element (for web extraction).
    #[serde(
        default,
        rename = "elementId",
        skip_serializing_if = "Option::is_none"
    )]
    pub element_id: Option<String>,
    /// CSS selector for the trigger element.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    /// Value for type actions (text input).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

// ============================================================================
// State Machine Components
// ============================================================================

/// Visual element within a discovered state.
///
/// Represents an image crop from a screenshot with its bounding box and
/// optional pixel-level identification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DiscoveredStateImage {
    /// Unique identifier for the image.
    pub id: String,
    /// ID of the source screenshot.
    #[serde(
        default,
        rename = "screenshotId",
        skip_serializing_if = "Option::is_none"
    )]
    pub screenshot_id: Option<String>,
    /// URL to the source screenshot.
    #[serde(
        default,
        rename = "screenshotUrl",
        skip_serializing_if = "Option::is_none"
    )]
    pub screenshot_url: Option<String>,
    /// Bounding box within the screenshot.
    pub bbox: DiscoveryBoundingBox,
    /// Hash of pixel data for deduplication.
    #[serde(
        default,
        rename = "pixelHash",
        skip_serializing_if = "Option::is_none"
    )]
    pub pixel_hash: Option<String>,
    /// ID of the state this image belongs to.
    #[serde(
        default,
        rename = "stateId",
        skip_serializing_if = "Option::is_none"
    )]
    pub state_id: Option<String>,
    /// Semantic type of the element (e.g. `button`, `input`).
    #[serde(
        default,
        rename = "elementType",
        skip_serializing_if = "Option::is_none"
    )]
    pub element_type: Option<String>,
    /// Human-readable label for the image.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Confidence score for this image (0.0–1.0). Defaults to `1.0`.
    #[serde(default = "default_confidence_one")]
    pub confidence: f64,
    /// Additional free-form metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, Value>>,
}

/// A discovered UI state (collection of co-occurring elements).
///
/// States represent distinct UI screens or views identified by the set of
/// images that consistently appear together.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DiscoveredState {
    /// Unique identifier for the state.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// IDs of images in this state.
    #[serde(
        default,
        rename = "imageIds",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub image_ids: Vec<String>,
    /// IDs of renders where this state appears.
    #[serde(
        default,
        rename = "renderIds",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub render_ids: Vec<String>,
    /// IDs of DOM elements (for web extraction).
    #[serde(
        default,
        rename = "elementIds",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub element_ids: Vec<String>,
    /// Confidence score for state detection (0.0–1.0). Defaults to `1.0`.
    #[serde(default = "default_confidence_one")]
    pub confidence: f64,
    /// Description of what this state represents.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Additional free-form metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, Value>>,
}

/// A transition between discovered states.
///
/// Transitions represent actions that change the active set of states on the
/// screen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DiscoveredTransition {
    /// Unique identifier for the transition.
    pub id: String,
    /// ID of the source state.
    #[serde(rename = "fromStateId")]
    pub from_state_id: String,
    /// ID of the target state.
    #[serde(rename = "toStateId")]
    pub to_state_id: String,
    /// What triggers this transition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger: Option<DiscoveryTransitionTrigger>,
    /// Confidence score for transition detection (0.0–1.0). Defaults to `1.0`.
    #[serde(default = "default_confidence_one")]
    pub confidence: f64,
    /// Additional free-form metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, Value>>,
}

// ============================================================================
// Complete Result
// ============================================================================

/// Complete state-machine result from discovery.
///
/// Unified output format regardless of the source (Playwright, UI Bridge,
/// Recording, Vision, Manual).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StateDiscoveryResult {
    /// Unique identifier for the result.
    pub id: String,
    /// ID of the project this belongs to.
    #[serde(rename = "projectId")]
    pub project_id: String,
    /// Human-readable name.
    pub name: String,
    /// Description of this state machine.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// How this state machine was discovered.
    #[serde(rename = "sourceType")]
    pub source_type: DiscoverySourceType,
    /// ID of the source session (extraction, recording, …).
    #[serde(
        default,
        rename = "sourceSessionId",
        skip_serializing_if = "Option::is_none"
    )]
    pub source_session_id: Option<String>,
    /// Strategy used for discovery (`auto`, `fingerprint`, `legacy`, …).
    #[serde(
        default,
        rename = "discoveryStrategy",
        skip_serializing_if = "Option::is_none"
    )]
    pub discovery_strategy: Option<String>,
    /// All discovered images.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub images: Vec<DiscoveredStateImage>,
    /// All discovered states.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub states: Vec<DiscoveredState>,
    /// All discovered transitions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transitions: Vec<DiscoveredTransition>,
    /// Mapping of element IDs to render IDs where they appear.
    #[serde(
        default,
        rename = "elementToRenders",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub element_to_renders: HashMap<String, Vec<String>>,
    /// Number of images (statistic).
    #[serde(default, rename = "imageCount")]
    pub image_count: i64,
    /// Number of states (statistic).
    #[serde(default, rename = "stateCount")]
    pub state_count: i64,
    /// Number of transitions (statistic).
    #[serde(default, rename = "transitionCount")]
    pub transition_count: i64,
    /// Number of renders analyzed (statistic).
    #[serde(default, rename = "renderCount")]
    pub render_count: i64,
    /// Number of unique elements (statistic).
    #[serde(default, rename = "uniqueElementCount")]
    pub unique_element_count: i64,
    /// Overall confidence score (0.0–1.0).
    #[serde(default)]
    pub confidence: f64,
    /// Additional discovery metadata.
    #[serde(
        default,
        rename = "discoveryMetadata",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub discovery_metadata: HashMap<String, Value>,
    /// ISO 8601 timestamp of creation.
    #[serde(rename = "createdAt")]
    pub created_at: String,
    /// ISO 8601 timestamp of last update.
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

/// Summary of a state-discovery result (for listings).
///
/// Lightweight projection of `StateDiscoveryResult` used by list endpoints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StateDiscoveryResultSummary {
    /// Unique identifier.
    pub id: String,
    /// ID of the project.
    #[serde(rename = "projectId")]
    pub project_id: String,
    /// Human-readable name.
    pub name: String,
    /// Description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Discovery source.
    #[serde(rename = "sourceType")]
    pub source_type: DiscoverySourceType,
    /// Strategy used.
    #[serde(
        default,
        rename = "discoveryStrategy",
        skip_serializing_if = "Option::is_none"
    )]
    pub discovery_strategy: Option<String>,
    /// Number of images.
    #[serde(default, rename = "imageCount")]
    pub image_count: i64,
    /// Number of states.
    #[serde(default, rename = "stateCount")]
    pub state_count: i64,
    /// Number of transitions.
    #[serde(default, rename = "transitionCount")]
    pub transition_count: i64,
    /// Confidence score (0.0–1.0).
    #[serde(default)]
    pub confidence: f64,
    /// ISO 8601 timestamp of creation.
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

/// API response for listing discovery results.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StateDiscoveryResultListResponse {
    /// List of result summaries.
    pub items: Vec<StateDiscoveryResultSummary>,
    /// Total count of results.
    pub total: i64,
}

// ============================================================================
// API Schemas
// ============================================================================

/// Request payload to create a state-discovery result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StateDiscoveryResultCreate {
    /// Human-readable name.
    pub name: String,
    /// Description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Discovery source.
    #[serde(rename = "sourceType")]
    pub source_type: DiscoverySourceType,
    /// ID of the source session.
    #[serde(
        default,
        rename = "sourceSessionId",
        skip_serializing_if = "Option::is_none"
    )]
    pub source_session_id: Option<String>,
    /// Strategy used.
    #[serde(
        default,
        rename = "discoveryStrategy",
        skip_serializing_if = "Option::is_none"
    )]
    pub discovery_strategy: Option<String>,
    /// Discovered images.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub images: Vec<DiscoveredStateImage>,
    /// Discovered states.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub states: Vec<DiscoveredState>,
    /// Discovered transitions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transitions: Vec<DiscoveredTransition>,
    /// Element to renders mapping.
    #[serde(
        default,
        rename = "elementToRenders",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub element_to_renders: HashMap<String, Vec<String>>,
    /// Confidence score (0.0–1.0).
    #[serde(default)]
    pub confidence: f64,
    /// Additional metadata.
    #[serde(
        default,
        rename = "discoveryMetadata",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub discovery_metadata: HashMap<String, Value>,
}

/// Request payload to update a state-discovery result.
///
/// All fields optional; only supplied fields are applied.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StateDiscoveryResultUpdate {
    /// Human-readable name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Updated images.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<DiscoveredStateImage>>,
    /// Updated states.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub states: Option<Vec<DiscoveredState>>,
    /// Updated transitions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transitions: Option<Vec<DiscoveredTransition>>,
    /// Updated metadata.
    #[serde(
        default,
        rename = "discoveryMetadata",
        skip_serializing_if = "Option::is_none"
    )]
    pub discovery_metadata: Option<HashMap<String, Value>>,
}

// ============================================================================
// Export / Import
// ============================================================================

/// Portable export format for state machines.
///
/// Used when exporting a discovery result to a shareable artifact.
/// `source_type` is kept as a free-form `String` to match Python's
/// `DiscoverySourceType | str` union (enabling imports that predate the enum).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StateMachineExport {
    /// Export format version. Defaults to `"1.0.0"`.
    #[serde(default = "default_export_version")]
    pub version: String,
    /// State machine name.
    pub name: String,
    /// Description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Original discovery source (string for forward compatibility — Python
    /// accepts `DiscoverySourceType | str`).
    #[serde(rename = "sourceType")]
    pub source_type: String,
    /// State images.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub images: Vec<DiscoveredStateImage>,
    /// States.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub states: Vec<DiscoveredState>,
    /// Transitions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transitions: Vec<DiscoveredTransition>,
    /// Element to renders mapping.
    #[serde(
        default,
        rename = "elementToRenders",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub element_to_renders: HashMap<String, Vec<String>>,
    /// Export metadata (original ID, export timestamp, …).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, Value>,
}

/// Request payload to import a state machine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StateMachineImport {
    /// The state machine to import.
    #[serde(rename = "stateMachine")]
    pub state_machine: StateMachineExport,
    /// Override name (uses export name when omitted).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

// ============================================================================
// Serde default helpers
// ============================================================================

fn default_confidence_one() -> f64 {
    1.0
}

fn default_export_version() -> String {
    "1.0.0".to_string()
}
