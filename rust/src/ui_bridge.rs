//! UI Bridge element-state DTOs.
//!
//! Wire-format types for the React UI Bridge registry: the element state,
//! bounding rectangle, element identifier, the registered-element and
//! registered-component shapes, action requests/responses, discovery
//! request/response, and the full snapshot envelope.
//!
//! These are ports of the shape-bearing portion of
//! `qontinui-runner/src-tauri/src/commands/ui_bridge.rs`. Runtime state
//! (Tauri `AppHandle`, IPC event listeners, `CommandResponse` wrappers,
//! DOM capture engines, WebView handles) stays in the runner. This module
//! is data-only.
//!
//! ## Wire-format notes
//!
//! - All structs serialize with `camelCase` field names to match the
//!   JavaScript/TypeScript wire contract consumed by `qontinui-web` and
//!   the Python SDK.
//! - `ElementRect` intentionally carries both `x`/`y`/`width`/`height`
//!   **and** `top`/`right`/`bottom`/`left`; the React `getBoundingClientRect`
//!   shape includes all eight.
//! - Dates/times are ISO 8601 strings or Unix-epoch millisecond `i64`s
//!   (see crate-level docs).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ============================================================================
// Element geometry
// ============================================================================

/// Viewport-relative bounding box in CSS pixels.
///
/// This is the live on-screen geometry captured at snapshot time via
/// `Element.getBoundingClientRect()`. Present only when the SDK has a
/// live DOM ref for the element; absent when the element is registered
/// without a ref or when the snapshot is served from the DOM-fallback
/// scanner.
///
/// Click target for a hit is `(x + width/2, y + height/2)`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct ElementBbox {
    /// X coordinate of the bbox origin in viewport CSS pixels.
    pub x: f64,
    /// Y coordinate of the bbox origin in viewport CSS pixels.
    pub y: f64,
    /// Width of the bbox in CSS pixels.
    pub width: f64,
    /// Height of the bbox in CSS pixels.
    pub height: f64,
}

/// Bounding rectangle of a DOM element, mirroring the output of
/// `Element.getBoundingClientRect()`.
///
/// Contains both the origin+size pair (`x`, `y`, `width`, `height`) and the
/// edge offsets (`top`, `right`, `bottom`, `left`) for consumer convenience.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct ElementRect {
    /// X coordinate of the element's origin (same as `left`).
    pub x: f64,
    /// Y coordinate of the element's origin (same as `top`).
    pub y: f64,
    /// Width of the element in CSS pixels.
    pub width: f64,
    /// Height of the element in CSS pixels.
    pub height: f64,
    /// Distance from the top of the viewport.
    pub top: f64,
    /// Distance from the left of the viewport plus `width`.
    pub right: f64,
    /// Distance from the top of the viewport plus `height`.
    pub bottom: f64,
    /// Distance from the left of the viewport.
    pub left: f64,
}

// ============================================================================
// Element state
// ============================================================================

/// Observable state of a UI Bridge element as returned from the React
/// registry.
///
/// Every element returned by the bridge includes a snapshot of its current
/// visibility, interactivity, and form-control value.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct ElementState {
    /// Whether the element is currently visible in the viewport.
    pub visible: bool,
    /// Whether the element is enabled (not disabled).
    pub enabled: bool,
    /// Whether the element currently has keyboard focus.
    pub focused: bool,
    /// Bounding rectangle of the element.
    pub rect: ElementRect,
    /// Current value for input/textarea elements.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Current checked state for checkbox/radio elements.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checked: Option<bool>,
    /// Currently selected options for `<select>` elements.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_options: Option<Vec<String>>,
    /// Text content of the element (innerText).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_content: Option<String>,
}

// ============================================================================
// Element identifier
// ============================================================================

/// Identifier bundle for locating a UI Bridge element.
///
/// Elements can be addressed by any combination of UI-Bridge ID, test ID,
/// AWAS ID, HTML ID, XPath, or CSS selector. The `xpath` and `selector`
/// fields are always present; the named IDs are optional.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct ElementIdentifier {
    /// Application-assigned UI Bridge ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ui_id: Option<String>,
    /// `data-testid` attribute value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test_id: Option<String>,
    /// AWAS-assigned action identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub awas_id: Option<String>,
    /// Native HTML `id` attribute.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html_id: Option<String>,
    /// Full XPath to the element.
    pub xpath: String,
    /// CSS selector that uniquely identifies the element.
    pub selector: String,
}

// ============================================================================
// Registered element / component
// ============================================================================

/// A registered element in the UI Bridge registry.
///
/// This is the serializable subset of the React `RegisteredElement`; it
/// includes identity, available actions, current state, and lifecycle info.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct UIBridgeElement {
    /// Unique element ID within the registry.
    pub id: String,
    /// Element type (e.g. `"button"`, `"input"`, `"select"`).
    #[serde(rename = "type")]
    pub element_type: String,
    /// Human-readable label for the element.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Standard actions available on this element.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<String>,
    /// Custom (application-defined) actions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_actions: Option<Vec<String>>,
    /// Identifier bundle for locating the element.
    pub identifier: ElementIdentifier,
    /// Current observable state.
    pub state: ElementState,
    /// Unix-epoch millisecond timestamp when the element was registered.
    pub registered_at: i64,
    /// Whether the element's React component is currently mounted.
    pub mounted: bool,
    /// Viewport-relative bounding box in CSS pixels, when the SDK has a
    /// live DOM ref. Absent for elements registered without a ref or when
    /// the snapshot is served from the DOM-fallback scanner.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bbox: Option<ElementBbox>,
    /// Cheap viewport-visibility signal derived by the SDK as
    /// `bbox.width > 0 && bbox.height > 0`. Use `state.visible` for the
    /// richer occlusion check.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible: Option<bool>,
}

/// Information about a single action exposed by a UI Bridge component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct ComponentActionInfo {
    /// Unique action identifier within the component.
    pub id: String,
    /// Human-readable label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Longer description of what the action does.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// A registered component in the UI Bridge registry.
///
/// Components group related elements and expose higher-level actions
/// (e.g. "submit form", "reset filters").
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct UIBridgeComponent {
    /// Unique component ID within the registry.
    pub id: String,
    /// Component name.
    pub name: String,
    /// Human-readable description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Actions exposed by this component.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<ComponentActionInfo>,
    /// IDs of elements that belong to this component.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element_ids: Option<Vec<String>>,
    /// Unix-epoch millisecond timestamp when the component was registered.
    pub registered_at: i64,
    /// Whether the component's React component is currently mounted.
    pub mounted: bool,
}

// ============================================================================
// Action requests / responses
// ============================================================================

/// Wait-condition options attached to an element action request.
///
/// Before executing the action the bridge can optionally wait until the
/// target element reaches a specified visibility/enabled/focused state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct WaitOptions {
    /// Wait until the element is visible (or hidden if `false`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible: Option<bool>,
    /// Wait until the element is enabled (or disabled if `false`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// Wait until the element has focus (or loses focus if `false`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focused: Option<bool>,
    /// Maximum time to wait in milliseconds before timing out.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u32>,
    /// Polling interval in milliseconds for condition checks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interval: Option<u32>,
}

/// Request to execute an action on a UI Bridge element.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct ElementActionRequest {
    /// Action name (e.g. `"click"`, `"type"`, `"select"`).
    pub action: String,
    /// Optional action-specific parameters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    /// Optional wait conditions to satisfy before executing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wait_options: Option<WaitOptions>,
}

/// Request to execute an action on a UI Bridge component.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct ComponentActionRequest {
    /// Action name.
    pub action: String,
    /// Optional action-specific parameters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// Response from executing an action on an element or component.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct ActionResponse {
    /// Whether the action completed successfully.
    pub success: bool,
    /// Updated element state after the action (if applicable).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element_state: Option<ElementState>,
    /// Action-specific return value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// Error message if the action failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Stack trace if the action threw an exception.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stack: Option<String>,
    /// Time taken to execute the action in milliseconds.
    pub duration_ms: u64,
    /// Unix-epoch millisecond timestamp when the action completed.
    pub timestamp: i64,
}

// ============================================================================
// Discovery
// ============================================================================

/// Options for a UI Bridge element-discovery scan.
///
/// Discovery crawls the live DOM and returns elements that match the
/// provided filters, regardless of whether they are registered in the
/// bridge registry.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct DiscoveryRequest {
    /// CSS selector for the root element to start scanning from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root: Option<String>,
    /// If `true`, only return interactive elements (buttons, inputs, etc.).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interactive_only: Option<bool>,
    /// If `true`, include hidden/off-screen elements.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_hidden: Option<bool>,
    /// Maximum number of elements to return.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Filter by element types (e.g. `["button", "input"]`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub types: Option<Vec<String>>,
    /// CSS selector filter (only elements matching this selector).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
}

/// An element found during a discovery scan.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct DiscoveredElement {
    /// Unique element ID.
    pub id: String,
    /// Element type (e.g. `"button"`, `"input"`).
    #[serde(rename = "type")]
    pub element_type: String,
    /// Human-readable label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// HTML tag name (e.g. `"BUTTON"`, `"INPUT"`).
    pub tag_name: String,
    /// ARIA role attribute.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// Computed accessible name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accessible_name: Option<String>,
    /// Available actions for this element.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<String>,
    /// Current observable state.
    pub state: ElementState,
    /// Whether the element is already registered in the bridge registry.
    pub registered: bool,
}

/// Response from a UI Bridge discovery scan.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct DiscoveryResponse {
    /// Discovered elements matching the request filters.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub elements: Vec<DiscoveredElement>,
    /// Total number of elements found (before any limit).
    pub total: usize,
    /// Time taken for the discovery scan in milliseconds.
    pub duration_ms: u64,
    /// Unix-epoch millisecond timestamp of the scan.
    pub timestamp: i64,
}

// ============================================================================
// Snapshot
// ============================================================================

/// Workflow metadata included in a UI Bridge snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct WorkflowInfo {
    /// Workflow ID.
    pub id: String,
    /// Workflow name.
    pub name: String,
    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Number of steps in the workflow.
    pub step_count: usize,
}

/// Full snapshot of the UI Bridge state.
///
/// Captures all registered elements, components, and active workflows
/// at a single point in time.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct UIBridgeSnapshot {
    /// Unix-epoch millisecond timestamp of the snapshot.
    pub timestamp: i64,
    /// All registered elements.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub elements: Vec<UIBridgeElement>,
    /// All registered components.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub components: Vec<UIBridgeComponent>,
    /// Active workflows.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub workflows: Vec<WorkflowInfo>,
}
