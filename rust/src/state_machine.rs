//! State machine DTO types.
//!
//! Wire-format types for the unified state machine domain shared by
//! `qontinui-web` and `qontinui-runner`. A state machine configuration is a
//! named collection of states and transitions describing the navigable
//! structure of an application's UI.
//!
//! Architecture:
//! - Config: a named state machine (collection of states + transitions)
//! - State: a UI state defined by which elements are present
//! - Transition: a named action sequence that moves between states
//! - Action: a single UI interaction (click, type, navigate, etc.)
//!
//! Runtime navigation (pathfinding through states) is handled by the qontinui
//! Python library via `multistate`. The pathfinding types here are for the
//! graph editor's path preview only.
//!
//! Ported from `qontinui-schemas/ts/src/state-machine/index.ts`. This module
//! is wire-format only: no business logic, no `impl` blocks, no tests. Dates,
//! times, and UUIDs are `String`s — see the crate-level docs.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

// ============================================================================
// Action Types
// ============================================================================

/// All UI Bridge SDK standard actions plus workflow-level actions
/// (`wait`, `navigate`).
///
/// Parameterized actions: `click`, `doubleClick`, `rightClick`, `type`,
/// `select`, `scroll`, `drag`.
/// No-param actions: `clear`, `focus`, `blur`, `hover`, `check`, `uncheck`,
/// `toggle`, `setValue`, `submit`, `reset`.
/// Workflow-level actions: `wait`, `navigate`.
///
/// Serialized as a bare camelCase string matching the TypeScript literal
/// union (e.g., `"doubleClick"`, `"setValue"`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum StandardActionType {
    /// Single left click on the target element.
    #[serde(rename = "click")]
    Click,
    /// Double click on the target element.
    #[serde(rename = "doubleClick")]
    DoubleClick,
    /// Right click (context menu) on the target element.
    #[serde(rename = "rightClick")]
    RightClick,
    /// Type text into the target element.
    #[serde(rename = "type")]
    Type,
    /// Clear the target element's value.
    #[serde(rename = "clear")]
    Clear,
    /// Select a value or label on a `<select>` element.
    #[serde(rename = "select")]
    Select,
    /// Move keyboard focus to the target element.
    #[serde(rename = "focus")]
    Focus,
    /// Remove keyboard focus from the target element.
    #[serde(rename = "blur")]
    Blur,
    /// Hover the mouse over the target element.
    #[serde(rename = "hover")]
    Hover,
    /// Scroll the target element or page.
    #[serde(rename = "scroll")]
    Scroll,
    /// Set a checkbox/radio to checked.
    #[serde(rename = "check")]
    Check,
    /// Set a checkbox to unchecked.
    #[serde(rename = "uncheck")]
    Uncheck,
    /// Toggle a checkbox/switch.
    #[serde(rename = "toggle")]
    Toggle,
    /// Set the value of a form control directly.
    #[serde(rename = "setValue")]
    SetValue,
    /// Drag from the target element to another position/element.
    #[serde(rename = "drag")]
    Drag,
    /// Submit the form containing the target element.
    #[serde(rename = "submit")]
    Submit,
    /// Reset the form containing the target element.
    #[serde(rename = "reset")]
    Reset,
    /// Wait for a specified duration (workflow-level).
    #[serde(rename = "wait")]
    Wait,
    /// Navigate to a URL (workflow-level).
    #[serde(rename = "navigate")]
    Navigate,
}

/// An (x, y) pixel offset used by the click-position and related parameters.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Point {
    /// Horizontal offset in pixels.
    pub x: f64,
    /// Vertical offset in pixels.
    pub y: f64,
}

/// Scroll direction for the `scroll` action.
///
/// Serialized as a bare lowercase string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ScrollDirection {
    /// Scroll upward.
    Up,
    /// Scroll downward.
    Down,
    /// Scroll to the left.
    Left,
    /// Scroll to the right.
    Right,
}

/// Mouse button for click actions.
///
/// Serialized as a bare lowercase string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum MouseButton {
    /// Primary (left) mouse button.
    Left,
    /// Secondary (right) mouse button.
    Right,
    /// Middle mouse button.
    Middle,
}

/// A value for `select` / `setValue` actions: either a single string or a list
/// of strings (for multi-select).
///
/// Serialized as an untagged union matching the TS `string | string[]`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum TransitionActionValue {
    /// A single value.
    Single(String),
    /// Multiple values (for multi-select form controls).
    Multiple(Vec<String>),
}

/// A single action within a transition.
///
/// Each action targets an element and performs an interaction. Most fields are
/// only meaningful for specific action types — see the per-field docs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TransitionAction {
    /// The kind of action to perform.
    #[serde(rename = "type")]
    pub action_type: StandardActionType,
    /// Target element ID (used by most element actions).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    /// Text to type (`type` action).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Clear before typing (`type` action).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clear_first: Option<bool>,
    /// Keystroke delay in milliseconds (`type` action).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_delay: Option<f64>,
    /// Value to select or set (`select` / `setValue` actions).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<TransitionActionValue>,
    /// Select by label instead of value (`select` action).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub select_by_label: Option<bool>,
    /// URL to navigate to (`navigate` action).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Delay in milliseconds (`wait` action).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delay_ms: Option<f64>,
    /// Scroll direction (`scroll` action).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scroll_direction: Option<ScrollDirection>,
    /// Scroll amount in pixels (`scroll` action).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scroll_amount: Option<f64>,
    /// Drag target element ID or selector (`drag` action).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drag_target: Option<String>,
    /// Drag target position (e.g., `{ x, y }` stringified, or a named
    /// position) (`drag` action).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drag_target_position: Option<String>,
    /// Number of intermediate mousemove steps (`drag` action).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drag_steps: Option<f64>,
    /// Hold delay before the first move in milliseconds (`drag` action).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drag_hold_delay: Option<f64>,
    /// Dispatch HTML5 drag events alongside mouse events (`drag` action).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drag_html5: Option<bool>,
    /// Mouse button: `left`, `right`, `middle`
    /// (`click` / `doubleClick` / `rightClick`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub button: Option<MouseButton>,
    /// Click position relative to the element
    /// (`click` / `doubleClick` / `rightClick`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<Point>,
}

// ============================================================================
// Domain Knowledge
// ============================================================================

/// A piece of domain knowledge attached to a state — free-form notes that
/// help the AI reason about what a state represents.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DomainKnowledge {
    /// Unique identifier for this knowledge entry.
    pub id: String,
    /// Short title / headline.
    pub title: String,
    /// Full knowledge content (markdown or plain text).
    pub content: String,
    /// Tags for filtering and grouping.
    pub tags: Vec<String>,
}

// ============================================================================
// State Machine Config
// ============================================================================

/// A state machine configuration — a named collection of states and
/// transitions that describe the navigable structure of an application's UI.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StateMachineConfig {
    /// Unique identifier (UUID).
    pub id: String,
    /// Display name.
    pub name: String,
    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Number of DOM renders collected for this config.
    pub render_count: u64,
    /// Number of unique elements discovered in this config.
    pub element_count: u64,
    /// Whether to include HTML IDs when generating selectors.
    pub include_html_ids: bool,
    /// ISO 8601 timestamp of creation.
    pub created_at: String,
    /// ISO 8601 timestamp of last modification.
    pub updated_at: String,
}

/// Payload for creating a new state machine configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StateMachineConfigCreate {
    /// Display name.
    pub name: String,
    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Payload for updating an existing state machine configuration.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StateMachineConfigUpdate {
    /// New display name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// New description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// A config with all its states and transitions loaded.
///
/// Used when the full state machine needs to be displayed or exported. This
/// mirrors the TypeScript `StateMachineConfigFull extends StateMachineConfig`
/// by flattening the base config's fields via `#[serde(flatten)]`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StateMachineConfigFull {
    /// Inlined [`StateMachineConfig`] fields.
    #[serde(flatten)]
    pub config: StateMachineConfig,
    /// All states belonging to this config.
    pub states: Vec<StateMachineState>,
    /// All transitions belonging to this config.
    pub transitions: Vec<StateMachineTransition>,
}

// ============================================================================
// State
// ============================================================================

/// A UI state, defined by which elements are present on screen.
///
/// States are discovered via co-occurrence analysis of DOM snapshots.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StateMachineState {
    /// Unique database identifier (UUID).
    pub id: String,
    /// Parent config ID.
    pub config_id: String,
    /// Stable logical state ID (distinct from `id`; set by the user or
    /// generator).
    pub state_id: String,
    /// Display name.
    pub name: String,
    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Element IDs that must be present for this state to be active.
    pub element_ids: Vec<String>,
    /// Render IDs that contributed to discovering this state.
    pub render_ids: Vec<String>,
    /// Confidence score (0.0–1.0) from the discovery pass.
    pub confidence: f64,
    /// Acceptance criteria for verifying the state is reached.
    pub acceptance_criteria: Vec<String>,
    /// Free-form metadata bag.
    pub extra_metadata: HashMap<String, Value>,
    /// Domain knowledge entries attached to this state.
    pub domain_knowledge: Vec<DomainKnowledge>,
    /// ISO 8601 timestamp of creation.
    pub created_at: String,
    /// ISO 8601 timestamp of last modification.
    pub updated_at: String,
}

/// Payload for creating a new state within a config.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StateMachineStateCreate {
    /// Optional stable logical state ID (generated if omitted).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_id: Option<String>,
    /// Display name.
    pub name: String,
    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Element IDs that define this state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element_ids: Option<Vec<String>>,
    /// Render IDs associated with this state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub render_ids: Option<Vec<String>>,
    /// Initial confidence score.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    /// Acceptance criteria.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acceptance_criteria: Option<Vec<String>>,
    /// Free-form metadata bag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra_metadata: Option<HashMap<String, Value>>,
    /// Domain knowledge entries.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain_knowledge: Option<Vec<DomainKnowledge>>,
}

/// Payload for updating an existing state.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StateMachineStateUpdate {
    /// New display name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// New description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// New element ID set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element_ids: Option<Vec<String>>,
    /// New render ID set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub render_ids: Option<Vec<String>>,
    /// New confidence score.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    /// New acceptance criteria.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acceptance_criteria: Option<Vec<String>>,
    /// New metadata bag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra_metadata: Option<HashMap<String, Value>>,
    /// New domain knowledge entries.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain_knowledge: Option<Vec<DomainKnowledge>>,
}

// ============================================================================
// Transition
// ============================================================================

/// A transition between states, consisting of one or more actions.
///
/// Transitions define the edges of the state machine graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StateMachineTransition {
    /// Unique database identifier (UUID).
    pub id: String,
    /// Parent config ID.
    pub config_id: String,
    /// Stable logical transition ID (distinct from `id`).
    pub transition_id: String,
    /// Display name.
    pub name: String,
    /// States this transition may be taken from.
    pub from_states: Vec<String>,
    /// States activated when this transition fires.
    pub activate_states: Vec<String>,
    /// States exited when this transition fires.
    pub exit_states: Vec<String>,
    /// Ordered list of actions executed as part of this transition.
    pub actions: Vec<TransitionAction>,
    /// Cost used by pathfinding to prefer cheaper transitions.
    pub path_cost: f64,
    /// Whether source states stay visible after activation.
    pub stays_visible: bool,
    /// Free-form metadata bag.
    pub extra_metadata: HashMap<String, Value>,
    /// ISO 8601 timestamp of creation.
    pub created_at: String,
    /// ISO 8601 timestamp of last modification.
    pub updated_at: String,
}

/// Payload for creating a new transition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StateMachineTransitionCreate {
    /// Display name.
    pub name: String,
    /// States this transition may be taken from.
    pub from_states: Vec<String>,
    /// States activated when this transition fires.
    pub activate_states: Vec<String>,
    /// States exited when this transition fires.
    pub exit_states: Vec<String>,
    /// Ordered list of actions executed as part of this transition.
    pub actions: Vec<TransitionAction>,
    /// Cost used by pathfinding.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path_cost: Option<f64>,
    /// Whether source states stay visible after activation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stays_visible: Option<bool>,
    /// Free-form metadata bag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra_metadata: Option<HashMap<String, Value>>,
}

/// Payload for updating an existing transition.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StateMachineTransitionUpdate {
    /// New display name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// New "from" states.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_states: Option<Vec<String>>,
    /// New "activate" states.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activate_states: Option<Vec<String>>,
    /// New "exit" states.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_states: Option<Vec<String>>,
    /// New action list.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actions: Option<Vec<TransitionAction>>,
    /// New path cost.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path_cost: Option<f64>,
    /// New stays-visible flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stays_visible: Option<bool>,
    /// New metadata bag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra_metadata: Option<HashMap<String, Value>>,
}

// ============================================================================
// Pathfinding (Graph Editor Visualization)
// ============================================================================

/// Request to compute a path between state sets.
///
/// The pathfinding types are for the graph editor's path-preview feature.
/// Runtime navigation uses the qontinui Python library (`multistate`)
/// directly.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PathfindingRequest {
    /// States assumed to be active at the start of the search.
    pub from_states: Vec<String>,
    /// States that must all be active at the end.
    pub target_states: Vec<String>,
}

/// A single step on a computed path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PathfindingStep {
    /// Logical transition ID taken in this step.
    pub transition_id: String,
    /// Display name of the transition.
    pub transition_name: String,
    /// States required to be active before this step.
    pub from_states: Vec<String>,
    /// States activated by this step.
    pub activate_states: Vec<String>,
    /// States exited by this step.
    pub exit_states: Vec<String>,
    /// Cost of this step.
    pub path_cost: f64,
}

/// Result of a pathfinding search.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PathfindingResult {
    /// Whether a valid path was found.
    pub found: bool,
    /// Ordered steps on the computed path (empty when `found` is `false`).
    pub steps: Vec<PathfindingStep>,
    /// Sum of `path_cost` across steps.
    pub total_cost: f64,
    /// Error message when pathfinding failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// ============================================================================
// Execution Results (Runtime)
// ============================================================================

/// Result of executing a single state transition at runtime.
///
/// Returned by the runner's Tauri commands or the qontinui Python subprocess.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TransitionExecutionResult {
    /// Whether the transition executed successfully.
    pub success: bool,
    /// Logical transition ID that was executed.
    pub transition_id: String,
    /// States that are active after the transition.
    pub active_states: Vec<String>,
    /// Error message when execution failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Result of navigating to one or more target states.
///
/// Navigation uses multistate pathfinding to determine the optimal path.
/// Nested `results` allow a single top-level navigation to fan out to several
/// targets and report per-target outcomes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct NavigationResult {
    /// Whether the overall navigation succeeded.
    pub success: bool,
    /// Ordered list of state IDs visited.
    pub path: Vec<String>,
    /// States active after the navigation completed.
    pub active_states: Vec<String>,
    /// Target state for this navigation branch (if applicable).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_state: Option<String>,
    /// Per-target sub-results for fan-out navigation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub results: Option<Vec<NavigationResult>>,
    /// Error message when navigation failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Result of querying currently active states in the state machine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ActiveStatesResult {
    /// Whether the query succeeded.
    pub success: bool,
    /// State IDs currently active.
    pub active_states: Vec<String>,
    /// The "primary" current state, when one can be singled out.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_state: Option<String>,
    /// Ordered history of states the machine has passed through.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_history: Option<Vec<String>>,
    /// Error message when the query failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Information about a single available transition from the current state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TransitionInfo {
    /// Transition ID.
    pub id: String,
    /// Source state ID.
    pub from_state: String,
    /// Destination state ID (`null` if the transition stays within the same
    /// state set — e.g. a self-transition or a guard-only action).
    #[serde(default)]
    pub to_state: Option<String>,
    /// Names of workflows attached to this transition.
    pub workflows: Vec<String>,
}

/// Result of querying available transitions from the current state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AvailableTransitionsResult {
    /// Whether the query succeeded.
    pub success: bool,
    /// Transitions available from the current state.
    pub transitions: Vec<TransitionInfo>,
    /// The state the transitions are taken from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_state: Option<String>,
    /// Human-readable message (e.g., "no transitions available").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Error message when the query failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// ============================================================================
// Initial States
// ============================================================================

/// Source of initial states configuration.
///
/// - `Defaults`: states with `is_initial=true` in the state machine definition
/// - `Workflow`: initial states defined on the workflow (`initialStateIds`)
/// - `Override`: session-only override from the runner UI
///
/// Serialized as a bare lowercase string (`"defaults"` / `"workflow"` /
/// `"override"`) to match the TS literal union.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum InitialStatesSource {
    /// States with `is_initial=true` in the state machine definition.
    Defaults,
    /// Initial states defined on the workflow.
    Workflow,
    /// Session-only override from the runner UI.
    Override,
}

/// A lightweight reference to a state by id and display name.
///
/// Used in [`ResolvedInitialStates`] and [`ResolvedInitialStatesResult`] to
/// let the UI render human-readable lists without a separate lookup.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct InitialStateRef {
    /// State ID.
    pub id: String,
    /// Display name.
    pub name: String,
}

/// The resolved set of initial states for a run, along with the source the
/// resolution came from.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedInitialStates {
    /// Resolved initial state IDs.
    pub state_ids: Vec<String>,
    /// Where the resolution came from.
    pub source: InitialStatesSource,
    /// Display-ready references for each state (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub states: Option<Vec<InitialStateRef>>,
    /// Workflow ID when `source == Workflow`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_id: Option<String>,
}

/// Result envelope for the "resolve initial states" operation.
///
/// Unlike [`ResolvedInitialStates`], this shape is non-optional: `states` and
/// `workflowId` are always present (possibly empty), and a `success` / `error`
/// pair is provided.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedInitialStatesResult {
    /// Whether resolution succeeded.
    pub success: bool,
    /// Resolved initial state IDs.
    pub state_ids: Vec<String>,
    /// Where the resolution came from.
    pub source: InitialStatesSource,
    /// Display-ready references for each state.
    pub states: Vec<InitialStateRef>,
    /// Workflow ID (may be empty).
    pub workflow_id: String,
    /// Error message when resolution failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// ============================================================================
// Discovery
// ============================================================================

/// Strategy used by the state-discovery pass.
///
/// Serialized as a bare lowercase string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum DiscoveryStrategy {
    /// Automatic strategy selection.
    Auto,
    /// Fingerprint-based discovery.
    Fingerprint,
}

// ============================================================================
// Graph Display Types (ReactFlow)
// ============================================================================

/// Data passed to a state node in the ReactFlow graph editor.
///
/// Mirrors the TypeScript shape with camelCase field names. The
/// `onStartElementDrag` callback from the TS type is a UI-layer concern and is
/// intentionally omitted from the DTO.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StateNodeData {
    /// State ID this node represents.
    pub state_id: String,
    /// Display name.
    pub name: String,
    /// Number of elements defining this state.
    pub element_count: u64,
    /// Discovery confidence.
    pub confidence: f64,
    /// Element IDs defining this state.
    pub element_ids: Vec<String>,
    /// Optional description (may be `null`).
    pub description: Option<String>,
    /// Whether the state blocks navigation.
    pub is_blocking: bool,
    /// Whether the node is currently selected in the editor.
    pub is_selected: bool,
    /// Whether this is an initial state.
    pub is_initial: bool,
    /// Outgoing transition count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outgoing_count: Option<u64>,
    /// Incoming transition count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub incoming_count: Option<u64>,
    /// Whether this node is the current drop target (drag-and-drop).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_drop_target: Option<bool>,
    /// Optional map of element ID → base64 thumbnail image (data URL or raw
    /// base64 PNG).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element_thumbnails: Option<HashMap<String, String>>,
}

/// Data passed to a transition edge in the ReactFlow graph editor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TransitionEdgeData {
    /// Transition ID this edge represents.
    pub transition_id: String,
    /// Display name.
    pub name: String,
    /// Path cost.
    pub path_cost: f64,
    /// Number of actions in this transition.
    pub action_count: u64,
    /// Distinct action types used.
    pub action_types: Vec<StandardActionType>,
    /// Whether the edge is highlighted (e.g., part of a path preview).
    pub is_highlighted: bool,
    /// Whether source states stay visible after activation.
    pub stays_visible: bool,
    /// Target of the first action, for label rendering.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_action_target: Option<String>,
}

// ============================================================================
// Export/Import
// ============================================================================

/// Format for exporting a state machine config to JSON.
///
/// Compatible with `UIBridgeRuntime.from_dict()` in the qontinui library. The
/// nested maps hold opaque per-state / per-transition / per-config dictionaries
/// because the exporter serializes implementation-specific fields not captured
/// by the DTO types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StateMachineExportFormat {
    /// State ID → state payload.
    pub states: HashMap<String, HashMap<String, Value>>,
    /// Transition ID → transition payload.
    pub transitions: HashMap<String, HashMap<String, Value>>,
    /// Config-level payload.
    pub config: HashMap<String, Value>,
}
