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

use crate::ir::IrEffect;
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
    /// Whether the element is enabled: the derived fold
    /// `!disabled && !ariaDisabled`.
    ///
    /// Kept as the single-signal convenience view of the two fields below.
    /// It is the ONLY interactivity signal an SDK build predating the
    /// `disabled`/`ariaDisabled` split emits, so it stays required.
    pub enabled: bool,
    /// Whether the element carries the native `disabled` attribute/property.
    ///
    /// Distinguished from `aria_disabled` because the two differ in effect:
    /// a natively disabled control cannot receive events at all, whereas an
    /// `aria-disabled` one is still focusable and clickable and merely
    /// announces itself as disabled.
    ///
    /// `#[serde(default)]`: absent in snapshots from SDK builds predating
    /// the split, where `enabled` is the only trustworthy signal. Absent
    /// therefore reads as `false`, which is NOT the same as "observed
    /// enabled": cross-check `enabled` before trusting it.
    #[serde(default)]
    pub disabled: bool,
    /// Whether the element carries `aria-disabled="true"`.
    ///
    /// See `disabled` for why this is tracked separately, and for the
    /// absent-reads-as-`false` caveat that applies identically here.
    #[serde(default)]
    pub aria_disabled: bool,
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

/// Information about a single custom (application-defined) action exposed by a
/// UI Bridge **element**.
///
/// The element-level twin of [`ComponentActionInfo`], and deliberately the same
/// field vocabulary so a consumer that walks both action planes reads one shape,
/// not two. One field is NOT carried over: `path`. A component action has a
/// resolved per-action invocation path because the SDK annotates one
/// (`/control/component/<componentId>/action/<actionId>`); an element action has
/// no analogue — every element action is invoked through the single
/// `/control/element/<elementId>/action` route with the action name in the body.
/// A field nothing would ever populate is dead schema surface, so it is absent
/// rather than perpetually `null`.
///
/// ## Why this is an object rather than a name
///
/// It exists so `effect` has somewhere to live. A bare list of action NAMES is
/// unannotatable: there is no slot on a name for a safety class, so an
/// element-level custom action could not declare itself `destructive` however
/// much its author wanted to. That is not hypothetical — the runner exposes
/// element custom actions that write raw bytes into a live agent PTY, and some
/// of them act on a pane with no visible view.
///
/// The wire projection is widened FIRST and the annotation lands on top of it.
/// The reverse order ships an UNREACHABLE annotation, which is worse than an
/// absent one: the author believes the delete button is marked, and the walker
/// walks it anyway.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct ElementActionInfo {
    /// Unique action identifier within the element — the key the action is
    /// registered under, and the name `/control/element/<id>/action` is called
    /// with.
    pub id: String,
    /// Human-readable label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Longer description of what the action does.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Free-form declaration of the action's parameters, author-supplied on the
    /// SDK side and surfaced verbatim. Conventionally a small JSON Schema
    /// subset, but the SDK does not constrain the shape, so it is carried
    /// through untyped — the same treatment `ElementActionRequest::params` and
    /// `ComponentActionInfo::param_schema` get.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub param_schema: Option<serde_json::Value>,
    /// Safety class of this action: `read`, `write` or `destructive`.
    ///
    /// **A safety annotation, not a hint.** An autonomous walk MUST NOT fire an
    /// action annotated `destructive`; that is the same exclusion the IR already
    /// applies to `destructive` transitions when generating auto-regressions.
    /// Author-declared, because only the app author knows that a particular
    /// custom action sends the keystrokes that end a session.
    ///
    /// Absent means **unclassified, not safe** — an action nobody has judged
    /// must be treated as unknown rather than as `read`. `skip_serializing_if`
    /// keeps an un-annotated action ABSENT on the wire rather than defaulting to
    /// a class nobody chose.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effect: Option<IrEffect>,
}

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
    ///
    /// **Objects, not names.** These used to be a bare `Vec<String>` of action
    /// names, which left an element-level custom action with nowhere to carry a
    /// safety class — so the walker had no way to tell a `refresh` from a
    /// `sendKeys` that writes raw bytes into a live agent PTY. Widening this to
    /// [`ElementActionInfo`] is what makes `effect` REACHABLE from an element;
    /// see that type's docs.
    ///
    /// Absent (`None`) means the element declares none. That is deliberately
    /// distinct from an empty list, which would mean "declared, and empty".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_actions: Option<Vec<ElementActionInfo>>,
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

    /// ARIA role of the element (explicit `role=` or implicit per W3C ARIA-in-HTML).
    /// Populated by the SDK's element walker. Source of truth for IrElementCriteria.role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,

    /// HTML tag name in lowercase. Source of truth for IrElementCriteria.tag_name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag_name: Option<String>,

    /// Computed aria-label (explicit attribute, falling back to aria-labelledby
    /// reference resolution). Source of truth for IrElementCriteria.aria_label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aria_label: Option<String>,

    /// W3C "accessible name" per the accessible-name algorithm. Distinct from
    /// aria_label because the algorithm may consult aria-labelledby, associated
    /// label elements, title, or visible content. Source of truth for
    /// IrElementCriteria.accessible_name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accessible_name: Option<String>,

    /// Visible text content with whitespace collapsed (DOM innerText-equivalent
    /// on web; accessibilityLabel/text equivalent on native). Source of truth for
    /// IrElementCriteria.text and text_contains. Distinct from state.text_content
    /// which is a snapshot of the form-control value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
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
    /// Free-form declaration of the action's parameters, author-supplied on the
    /// SDK side as `ComponentAction.paramSchema` and surfaced verbatim on
    /// `/control/components`. Conventionally a small JSON Schema subset, but the
    /// SDK does not constrain the shape, so it is carried through untyped — the
    /// same treatment `ElementActionRequest::params` gets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub param_schema: Option<serde_json::Value>,
    /// Safety class of this action: `read`, `write` or `destructive`.
    ///
    /// **A safety annotation, not a hint.** An autonomous walk MUST NOT fire an
    /// action annotated `destructive`; that is the same exclusion the IR already
    /// applies to `destructive` transitions when generating auto-regressions.
    /// Author-declared, because only the app author knows that a particular
    /// `click` is a delete button.
    ///
    /// Absent means **unclassified, not safe** — an action nobody has judged
    /// must be treated as unknown rather than as `read`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effect: Option<IrEffect>,
    /// Fully-resolved invocation path for this action:
    /// `/control/component/<componentId>/action/<actionId>`.
    ///
    /// **Server-annotated, not author-declared**: the SDK's
    /// `annotateComponentWithInvocationPaths` computes it when serving the
    /// component listing, so it is present on the wire but absent from anything
    /// an app author writes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
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
    /// Invocation-path template for this component's actions, emitted with a
    /// literal `{actionId}` placeholder:
    /// `/control/component/<componentId>/action/{actionId}`.
    ///
    /// **Server-annotated, not author-declared**: emitted by the SDK's
    /// `annotateComponentWithInvocationPaths` alongside the per-action
    /// [`ComponentActionInfo::path`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_invocation_path: Option<String>,
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
    /// D3 effect-calculus verification: the predicted-vs-observed outcome for
    /// this action, present only when a handler effect signature resolved for
    /// the `(action, element)` (opt-in; absent otherwise).
    ///
    /// Carried as an opaque JSON object on the wire, deliberately matching the
    /// established lean-wire / rich-SDK split (the SDK's `EffectVerification`
    /// has a richer shape than any consumer needs to type). The runner
    /// deserializes the sub-shape it asserts on (`outcome` / `cause` /
    /// `containment` / `durationMs`) with its own local struct in the
    /// `effect_check` step handler, and relays the rest into `result_json`. See
    /// `ui-bridge/.../control/effect-types.ts` for the SDK-side producer. Kept
    /// opaque here so the nested effect types need no top-level codegen
    /// registration (which would couple this crate's bindings to a runner
    /// `schema_export.rs` change).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effect_verification: Option<serde_json::Value>,
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

/// Modal/dialog entry in the active modal stack.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct UIBridgeModalInfo {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Modal kind. Web: dialog/alertdialog/modal/drawer/popover/sheet.
    /// Native (RN): modal/sheet/drawer/popover/alertdialog/dialog.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    /// Whether this modal blocks interaction with content behind it.
    pub blocking: bool,
    /// Whether the modal is dismissible (RN-specific). Optional so the
    /// web shape (which doesn't carry this bit) round-trips cleanly.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dismissible: Option<bool>,
    /// Web-only: computed z-index of the modal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub z_index: Option<i64>,
    /// Web-only: whether a backdrop/overlay is present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_backdrop: Option<bool>,
    /// Timestamp when the modal was detected (epoch ms).
    pub detected_at: i64,
}

/// Modal stack context attached to a snapshot when the SDK has a
/// `ModalDetector` enricher configured.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct UIBridgeModalStack {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modals: Vec<UIBridgeModalInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_modal: Option<UIBridgeModalInfo>,
    pub has_blocking_modal: bool,
    pub count: usize,
}

/// A captured toast/notification entry.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct UIBridgeCapturedToast {
    pub id: String,
    pub message: String,
    /// Severity level. One of: info|success|warning|error|loading|unknown.
    pub level: String,
    pub appeared_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dismissed_at: Option<i64>,
    pub visible: bool,
    pub duration_ms: i64,
}

/// Toast snapshot context.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct UIBridgeToastContext {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub active: Vec<UIBridgeCapturedToast>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recent: Vec<UIBridgeCapturedToast>,
    pub total_captured: usize,
}

/// Undo/redo availability snapshot context.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct UIBridgeUndoContext {
    pub can_undo: bool,
    pub can_redo: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub undo_description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redo_description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub undo_depth: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redo_depth: Option<usize>,
    pub summary: String,
}

/// Full snapshot of the UI Bridge state.
///
/// Captures all registered elements, components, and active workflows
/// at a single point in time. The optional `modalStack` / `toasts` /
/// `undoRedo` fields are populated by the SDK's enricher slot when
/// configured (see `setEnrichers` on web and native registries).
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
    /// Modal/sheet stack (populated when ModalDetector enricher is set).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modal_stack: Option<UIBridgeModalStack>,
    /// Active and recently dismissed toasts (populated by ToastCapture).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub toasts: Option<UIBridgeToastContext>,
    /// Undo/redo availability (populated by UndoTracker).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub undo_redo: Option<UIBridgeUndoContext>,
    /// Native-only: current navigation route (Expo Router pathname).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_route: Option<String>,
    /// Native-only: current route segments.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub segments: Vec<String>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// The SDK's `/control/components` payload carries `paramSchema` and a
    /// server-annotated per-action `path`. Before those fields existed here the
    /// published JSON Schema (`deny_unknown_fields`) rejected the payload the
    /// SDK actually sends, so this pins BOTH directions against literal JSON.
    #[test]
    fn component_action_info_round_trips_param_schema_and_path() {
        // Literal wire bytes, as emitted by
        // `annotateComponentWithInvocationPaths` in the UI Bridge SDK.
        let wire = r#"{"id":"search","label":"Search","description":"Run a search","paramSchema":{"properties":{"query":{"type":"string"}},"type":"object"},"path":"/control/component/search-box/action/search"}"#;

        let action: ComponentActionInfo = serde_json::from_str(wire).expect("deserializes");

        assert_eq!(action.id, "search");
        assert_eq!(action.label.as_deref(), Some("Search"));
        assert_eq!(action.description.as_deref(), Some("Run a search"));
        assert_eq!(
            action.path.as_deref(),
            Some("/control/component/search-box/action/search")
        );
        // The schema is carried through untyped and unmodified.
        let param_schema = action.param_schema.as_ref().expect("paramSchema present");
        assert_eq!(param_schema["type"], "object");
        assert_eq!(param_schema["properties"]["query"]["type"], "string");

        // Re-serializing reproduces the same literal bytes: field order is
        // declaration order, and serde_json orders map keys lexicographically.
        let reserialized = serde_json::to_string(&action).expect("serializes");
        assert_eq!(reserialized, wire);
    }

    /// `skip_serializing_if = "Option::is_none"` must OMIT the keys entirely
    /// rather than emit `null` — a `null` would be a different wire shape and
    /// consumers reading `action.paramSchema` truthily would see it change.
    #[test]
    fn component_action_info_omits_absent_param_schema_and_path() {
        let action = ComponentActionInfo {
            id: "reset".to_string(),
            label: None,
            description: None,
            param_schema: None,
            effect: None,
            path: None,
        };

        let json = serde_json::to_string(&action).expect("serializes");
        assert_eq!(json, r#"{"id":"reset"}"#);
        assert!(!json.contains("paramSchema"));
        assert!(!json.contains("\"path\""));
        assert!(!json.contains("effect"));
    }

    /// The effect annotation gates automatic walks, so it has to survive the
    /// wire intact — pinned against literal JSON in both directions.
    #[test]
    fn component_action_info_round_trips_destructive_effect() {
        let wire = r#"{"id":"delete","label":"Delete row","effect":"destructive","path":"/control/component/row/action/delete"}"#;

        let action: ComponentActionInfo = serde_json::from_str(wire).expect("deserializes");

        assert_eq!(action.effect, Some(IrEffect::Destructive));
        assert_eq!(serde_json::to_string(&action).expect("serializes"), wire);
    }

    /// All three vocabulary members are accepted, and each renders lowercase.
    #[test]
    fn component_action_info_effect_accepts_the_whole_vocabulary() {
        for (wire, expected) in [
            (r#"{"id":"a","effect":"read"}"#, IrEffect::Read),
            (r#"{"id":"a","effect":"write"}"#, IrEffect::Write),
            (
                r#"{"id":"a","effect":"destructive"}"#,
                IrEffect::Destructive,
            ),
        ] {
            let action: ComponentActionInfo = serde_json::from_str(wire).expect("deserializes");
            assert_eq!(action.effect, Some(expected));
            assert_eq!(serde_json::to_string(&action).expect("serializes"), wire);
        }
    }

    /// `effect` is a CLOSED set. A plausible-but-wrong verb must be REJECTED,
    /// not silently carried — an unrecognised value accepted as data would let
    /// a destructive action masquerade as unclassified and get walked.
    #[test]
    fn component_action_info_rejects_an_out_of_vocabulary_effect() {
        let wire = r#"{"id":"delete","effect":"delete"}"#;

        let err = serde_json::from_str::<ComponentActionInfo>(wire)
            .expect_err("an out-of-vocabulary effect must not deserialize");
        let msg = err.to_string();
        assert!(
            msg.contains("unknown variant") && msg.contains("delete"),
            "expected an unknown-variant error naming the bad value, got: {msg}"
        );

        // Capitalisation is not a synonym either — the wire form is lowercase.
        assert!(serde_json::from_str::<ComponentActionInfo>(
            r#"{"id":"a","effect":"Destructive"}"#
        )
        .is_err());
    }

    // ── ElementActionInfo — the element-level effect annotation ──────────
    //
    // These are the reachability tests for the widened `customActions`
    // projection. Every one pins hand-written wire bytes rather than a
    // round-trip of a Rust value, because the property under test is the
    // SERDE CONTRACT with the SDK, not internal consistency: a round-trip of
    // whatever Rust happens to emit stays green through a rename.

    /// All three vocabulary members survive an element action intact, each
    /// rendering lowercase, in both directions.
    #[test]
    fn element_action_info_round_trips_each_effect() {
        for (wire, expected) in [
            (
                r#"{"id":"readBuffer","label":"Read buffer","effect":"read"}"#,
                IrEffect::Read,
            ),
            (
                r#"{"id":"setTitle","label":"Set title","effect":"write"}"#,
                IrEffect::Write,
            ),
            (
                r#"{"id":"sendKeys","label":"Send keys","effect":"destructive"}"#,
                IrEffect::Destructive,
            ),
        ] {
            let action: ElementActionInfo = serde_json::from_str(wire).expect("deserializes");

            assert_eq!(action.effect, Some(expected));
            assert_eq!(serde_json::to_string(&action).expect("serializes"), wire);
        }
    }

    /// An UN-ANNOTATED action must serialize with `effect` **absent** — not
    /// `null`, and above all not defaulted to `read`.
    ///
    /// Absence is the encoding of "nobody has judged this action". A default
    /// would silently claim a safety class no author chose, and a walker
    /// reading it would fire the action believing it had been cleared.
    #[test]
    fn element_action_info_omits_an_unannotated_effect() {
        let action = ElementActionInfo {
            id: "scrollToTop".to_string(),
            label: None,
            description: None,
            param_schema: None,
            effect: None,
        };

        let json = serde_json::to_string(&action).expect("serializes");
        assert_eq!(json, r#"{"id":"scrollToTop"}"#);
        assert!(!json.contains("effect"));
        assert!(!json.contains("null"));

        // And the absence survives the read back as `None`, rather than
        // resolving to any member of the vocabulary.
        let back: ElementActionInfo = serde_json::from_str(&json).expect("deserializes");
        assert_eq!(back.effect, None);
    }

    /// `effect` is a CLOSED set here too. A plausible-but-wrong verb must be
    /// REJECTED, not carried as data — an unrecognised value accepted would let
    /// a destructive element action masquerade as unclassified and get walked.
    #[test]
    fn element_action_info_rejects_an_out_of_vocabulary_effect() {
        let err = serde_json::from_str::<ElementActionInfo>(r#"{"id":"sendKeys","effect":"send"}"#)
            .expect_err("an out-of-vocabulary effect must not deserialize");
        let msg = err.to_string();
        assert!(
            msg.contains("unknown variant") && msg.contains("send"),
            "expected an unknown-variant error naming the bad value, got: {msg}"
        );

        // Capitalisation is not a synonym either — the wire form is lowercase.
        assert!(
            serde_json::from_str::<ElementActionInfo>(r#"{"id":"a","effect":"Destructive"}"#)
                .is_err()
        );
    }

    /// The whole point of the widening: an ELEMENT's custom actions arrive as
    /// objects that carry `effect`, mixed freely with un-annotated ones.
    ///
    /// Pinned against literal element bytes, because the old shape here was
    /// `["sendKeys","readBuffer"]` and nothing but a wire assertion would
    /// notice a regression back to a list of names.
    #[test]
    fn ui_bridge_element_round_trips_annotated_custom_actions() {
        let wire = r##"{"id":"term-pane-3","type":"terminal","actions":["click"],"customActions":[{"id":"sendKeys","label":"Send keys","effect":"destructive"},{"id":"readBuffer","effect":"read"},{"id":"scrollToTop"}],"identifier":{"xpath":"/div[3]","selector":"#term-3"},"state":{"visible":true,"enabled":true,"disabled":false,"ariaDisabled":false,"focused":false,"rect":{"x":0.0,"y":0.0,"width":640.0,"height":480.0,"top":0.0,"right":640.0,"bottom":480.0,"left":0.0}},"registeredAt":1755800000000,"mounted":true}"##;

        let element: UIBridgeElement = serde_json::from_str(wire).expect("deserializes");

        let custom = element
            .custom_actions
            .as_ref()
            .expect("customActions present");
        assert_eq!(custom.len(), 3);

        assert_eq!(custom[0].id, "sendKeys");
        assert_eq!(custom[0].label.as_deref(), Some("Send keys"));
        assert_eq!(custom[0].effect, Some(IrEffect::Destructive));

        assert_eq!(custom[1].id, "readBuffer");
        assert_eq!(custom[1].effect, Some(IrEffect::Read));

        // Unclassified, and it stays unclassified — not promoted to `read`.
        assert_eq!(custom[2].id, "scrollToTop");
        assert_eq!(custom[2].effect, None);

        assert_eq!(serde_json::to_string(&element).expect("serializes"), wire);
    }

    /// `None` and `Some(vec![])` are different wire shapes and must stay so:
    /// "declares no custom actions" is not "declares an empty set".
    #[test]
    fn ui_bridge_element_omits_absent_custom_actions() {
        let wire = r#"{"id":"plain","type":"button","identifier":{"xpath":"/button[1]","selector":"button"},"state":{"visible":true,"enabled":true,"disabled":false,"ariaDisabled":false,"focused":false,"rect":{"x":0.0,"y":0.0,"width":10.0,"height":10.0,"top":0.0,"right":10.0,"bottom":10.0,"left":0.0}},"registeredAt":1755800000000,"mounted":true}"#;

        let element: UIBridgeElement = serde_json::from_str(wire).expect("deserializes");
        assert!(element.custom_actions.is_none());

        let json = serde_json::to_string(&element).expect("serializes");
        assert_eq!(json, wire);
        assert!(!json.contains("customActions"));
    }

    /// The component-level `actionInvocationPath` is a TEMPLATE carrying a
    /// literal `{actionId}` placeholder — not a resolved path.
    #[test]
    fn ui_bridge_component_round_trips_action_invocation_path() {
        let wire = r#"{"id":"search-box","name":"SearchBox","actions":[{"id":"search","paramSchema":{"type":"object"},"path":"/control/component/search-box/action/search"}],"actionInvocationPath":"/control/component/search-box/action/{actionId}","registeredAt":1755800000000,"mounted":true}"#;

        let component: UIBridgeComponent = serde_json::from_str(wire).expect("deserializes");

        assert_eq!(component.id, "search-box");
        assert_eq!(
            component.action_invocation_path.as_deref(),
            Some("/control/component/search-box/action/{actionId}")
        );
        assert_eq!(component.actions.len(), 1);
        assert_eq!(
            component.actions[0]
                .param_schema
                .as_ref()
                .expect("paramSchema present")["type"],
            "object"
        );

        let reserialized = serde_json::to_string(&component).expect("serializes");
        assert_eq!(reserialized, wire);
    }
}
