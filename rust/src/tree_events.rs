//! Tree event schemas for execution logging.
//!
//! Mirrors `src/qontinui_schemas/events/tree_events.py`. Rust is the source
//! of truth; the Python and TypeScript bindings are regenerated from the
//! JSON Schemas emitted here.
//!
//! Used across the Qontinui ecosystem:
//! - `qontinui` (Python library) — emits events during execution
//! - `qontinui-runner` (Tauri app) — receives and displays events
//! - `qontinui-web` (backend) — stores and forwards events
//! - `qontinui-web` (frontend) — displays historical events
//!
//! Schema shape: workflows contain actions; actions can nest
//! (`GO_TO_STATE`, `RUN_WORKFLOW`); each event captures a node state change
//! with rich metadata.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Enums
// ============================================================================

/// Types of nodes in the execution tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum NodeType {
    Workflow,
    Action,
    Transition,
}

/// Execution status of a tree node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum NodeStatus {
    Pending,
    Running,
    Success,
    Failed,
}

/// Types of tree events emitted during execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TreeEventType {
    // Workflow lifecycle
    WorkflowStarted,
    WorkflowCompleted,
    WorkflowFailed,
    // Action lifecycle
    ActionStarted,
    ActionCompleted,
    ActionFailed,
    // Transition lifecycle (state-machine navigation)
    TransitionStarted,
    TransitionCompleted,
    TransitionFailed,
}

/// Types of actions in the automation system.
///
/// Corresponds to the action types defined in the qontinui-schemas config
/// models. Variants use SCREAMING_SNAKE_CASE on the wire to match the Python
/// enum values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ActionType {
    // Vision actions
    #[serde(rename = "FIND")]
    Find,
    #[serde(rename = "FIND_STATE_IMAGE")]
    FindStateImage,
    #[serde(rename = "EXISTS")]
    Exists,
    #[serde(rename = "VANISH")]
    Vanish,

    // Mouse actions
    #[serde(rename = "CLICK")]
    Click,
    #[serde(rename = "DOUBLE_CLICK")]
    DoubleClick,
    #[serde(rename = "RIGHT_CLICK")]
    RightClick,
    #[serde(rename = "DRAG")]
    Drag,
    #[serde(rename = "SCROLL")]
    Scroll,
    #[serde(rename = "MOUSE_MOVE")]
    MouseMove,

    // Keyboard actions
    #[serde(rename = "TYPE")]
    Type,
    #[serde(rename = "KEY_PRESS")]
    KeyPress,
    #[serde(rename = "HOTKEY")]
    Hotkey,

    // Control flow
    #[serde(rename = "IF")]
    If,
    #[serde(rename = "LOOP")]
    Loop,
    #[serde(rename = "SWITCH")]
    Switch,
    #[serde(rename = "TRY_CATCH")]
    TryCatch,
    #[serde(rename = "BREAK")]
    Break,
    #[serde(rename = "CONTINUE")]
    Continue,

    // State machine
    #[serde(rename = "GO_TO_STATE")]
    GoToState,
    #[serde(rename = "WAIT")]
    Wait,

    // Workflow
    #[serde(rename = "RUN_WORKFLOW")]
    RunWorkflow,

    // Utility
    #[serde(rename = "SCREENSHOT")]
    Screenshot,
    #[serde(rename = "CODE_BLOCK")]
    CodeBlock,
    #[serde(rename = "SHELL")]
    Shell,
    #[serde(rename = "CUSTOM")]
    Custom,
}

// ============================================================================
// Nested Metadata Models
// ============================================================================

/// Location of a pattern match on screen.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct MatchLocation {
    #[serde(alias = "x")]
    pub x: i64,
    #[serde(alias = "y")]
    pub y: i64,
    /// Width of the matched region (optional — point matches omit it).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "w")]
    pub w: Option<i64>,
    /// Height of the matched region (optional — point matches omit it).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "h")]
    pub h: Option<i64>,
}

/// A single match result with confidence and location.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct TopMatch {
    #[serde(alias = "confidence")]
    pub confidence: f64,
    #[serde(alias = "location")]
    pub location: MatchLocation,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "dimensions")]
    pub dimensions: Option<MatchLocation>,
}

/// Runtime execution data captured during action execution.
///
/// Different fields are populated depending on the action type. All fields
/// are optional; the wire schema allows additional runtime fields via a
/// `flatten`-d map.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct RuntimeData {
    // TYPE actions
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "typed_text")]
    pub typed_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "character_count")]
    pub character_count: Option<i64>,

    // FIND / IF actions — pattern matching
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "image_id")]
    pub image_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "found")]
    pub found: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "confidence")]
    pub confidence: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "location")]
    pub location: Option<MatchLocation>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "dimensions")]
    pub dimensions: Option<MatchLocation>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "match_method")]
    pub match_method: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "top_matches")]
    pub top_matches: Option<Vec<TopMatch>>,

    // CLICK actions
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "clicked_at")]
    pub clicked_at: Option<MatchLocation>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "button")]
    pub button: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "target_type")]
    pub target_type: Option<String>,

    // GO_TO_STATE actions
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "source_states")]
    pub source_states: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "target_states")]
    pub target_states: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "targets_reached")]
    pub targets_reached: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "transitions_executed")]
    pub transitions_executed: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "already_at_target")]
    pub already_at_target: Option<bool>,

    // IF actions
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "condition_passed")]
    pub condition_passed: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "branch_taken")]
    pub branch_taken: Option<String>,

    // RUN_WORKFLOW actions
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "workflow_name")]
    pub workflow_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "workflow_status")]
    pub workflow_status: Option<String>,
}

/// State-machine context captured before/after an action.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct StateContext {
    #[serde(default, alias = "active_before")]
    pub active_before: Vec<String>,
    #[serde(default, alias = "active_after")]
    pub active_after: Vec<String>,
    #[serde(default, alias = "changed")]
    pub changed: bool,
    #[serde(default, alias = "activated")]
    pub activated: Vec<String>,
    #[serde(default, alias = "deactivated")]
    pub deactivated: Vec<String>,
}

/// Precise timing information for an event. Times are ISO 8601 strings (see
/// crate-level docs for the rationale — the types crate is wire-only and
/// doesn't depend on a chrono version).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct TimingInfo {
    /// ISO 8601 timestamp.
    #[serde(alias = "start_time")]
    pub start_time: String,
    /// ISO 8601 timestamp. `None` while the event is still in flight.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "end_time")]
    pub end_time: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "duration_ms")]
    pub duration_ms: Option<f64>,
}

/// Execution outcome of an action.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct Outcome {
    #[serde(alias = "success")]
    pub success: bool,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "error")]
    pub error: Option<String>,
    #[serde(default, alias = "retry_count")]
    pub retry_count: i64,
}

// ============================================================================
// Main Models
// ============================================================================

/// Metadata for a tree node — action configuration, runtime data, state
/// context, and timing. All fields are optional because different node
/// types populate different fields.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct NodeMetadata {
    /// Action configuration (JSON object — shape varies by `ActionType`).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "config")]
    pub config: Option<HashMap<String, serde_json::Value>>,

    /// Whether this action can have child nodes.
    #[serde(default, alias = "is_expandable")]
    pub is_expandable: bool,
    /// Whether this action is displayed inline.
    #[serde(default, alias = "is_inline")]
    pub is_inline: bool,

    #[serde(default, skip_serializing_if = "Option::is_none", alias = "runtime")]
    pub runtime: Option<RuntimeData>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "state_context")]
    pub state_context: Option<StateContext>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "timing")]
    pub timing: Option<TimingInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "outcome")]
    pub outcome: Option<Outcome>,

    /// Screenshot reference (path or URL).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "screenshot_reference")]
    pub screenshot_reference: Option<String>,
    /// Visual-debug image reference (path or URL).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "visual_debug_reference")]
    pub visual_debug_reference: Option<String>,
}

/// A node in the execution tree — a single workflow, action, or transition
/// in the execution hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct TreeNode {
    /// Unique identifier for this node.
    #[serde(alias = "id")]
    pub id: String,
    /// Type of node (workflow / action / transition).
    #[serde(alias = "node_type")]
    pub node_type: NodeType,
    /// Display name for this node.
    #[serde(alias = "name")]
    pub name: String,

    /// When this node was created (Unix epoch seconds).
    #[serde(alias = "timestamp")]
    pub timestamp: f64,
    /// When this node completed (Unix epoch seconds).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "end_timestamp")]
    pub end_timestamp: Option<f64>,
    /// Duration in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "duration")]
    pub duration: Option<f64>,

    /// ID of parent node, `None` for root.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "parent_id")]
    pub parent_id: Option<String>,

    /// Current execution status.
    #[serde(alias = "status")]
    pub status: NodeStatus,

    #[serde(default, alias = "metadata")]
    pub metadata: NodeMetadata,
    /// Error message if `status == Failed`.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "error")]
    pub error: Option<String>,
}

/// Element in a tree path (for breadcrumb navigation).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct PathElement {
    #[serde(alias = "id")]
    pub id: String,
    #[serde(alias = "name")]
    pub name: String,
    #[serde(alias = "node_type")]
    pub node_type: NodeType,
}

/// A tree event emitted during execution.
///
/// Primary event type for execution logging. Carries the event type, the
/// affected node with full metadata, the path from root to this node, and
/// a sequence number for ordering.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct TreeEvent {
    /// Event type identifier — always `"tree_event"` on the wire.
    #[serde(default = "default_tree_event_type", alias = "type")]
    pub r#type: String,
    /// Specific tree-event variant.
    #[serde(alias = "event_type")]
    pub event_type: TreeEventType,
    /// The node this event is about.
    #[serde(alias = "node")]
    pub node: TreeNode,
    /// Path from root to this node (breadcrumb).
    #[serde(default, alias = "path")]
    pub path: Vec<PathElement>,
    /// When this event was emitted (Unix epoch seconds).
    #[serde(alias = "timestamp")]
    pub timestamp: f64,
    /// Sequence number for ordering.
    #[serde(default, alias = "sequence")]
    pub sequence: i64,
}

fn default_tree_event_type() -> String {
    "tree_event".to_string()
}

// ============================================================================
// Display Models (frontend-specific)
// ============================================================================

/// Display node structure used by the frontend — extended version of
/// `TreeNode` with tree-rendering properties. NOT persisted; constructed
/// from `TreeNode` data for UI display.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct DisplayNode {
    #[serde(alias = "id")]
    pub id: String,
    #[serde(alias = "node_type")]
    pub node_type: NodeType,
    #[serde(alias = "name")]
    pub name: String,
    #[serde(alias = "timestamp")]
    pub timestamp: f64,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "end_timestamp")]
    pub end_timestamp: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "duration")]
    pub duration: Option<f64>,
    #[serde(alias = "status")]
    pub status: NodeStatus,
    #[serde(default, alias = "metadata")]
    pub metadata: NodeMetadata,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "error")]
    pub error: Option<String>,

    /// Child nodes in the tree.
    #[serde(default, alias = "children")]
    pub children: Vec<DisplayNode>,
    /// Whether this node should be expanded in the UI (default: true).
    #[serde(default = "default_true", alias = "is_expanded")]
    pub is_expanded: bool,
    /// Nesting level in the tree (0 for root, 1 for first-level children).
    #[serde(default, alias = "level")]
    pub level: i64,
}

fn default_true() -> bool {
    true
}

// ============================================================================
// API Request/Response Models
// ============================================================================

/// Request to store a tree event.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct TreeEventCreate {
    #[serde(alias = "event_type")]
    pub event_type: TreeEventType,
    #[serde(alias = "node")]
    pub node: TreeNode,
    #[serde(default, alias = "path")]
    pub path: Vec<PathElement>,
    #[serde(alias = "timestamp")]
    pub timestamp: f64,
    #[serde(default, alias = "sequence")]
    pub sequence: i64,
}

/// Response for a stored tree event.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct TreeEventResponse {
    /// UUID as string (wire-format — see crate-level docs).
    #[serde(alias = "id")]
    pub id: String,
    /// Run UUID as string.
    #[serde(alias = "run_id")]
    pub run_id: String,
    #[serde(alias = "event_type")]
    pub event_type: TreeEventType,
    #[serde(alias = "node_id")]
    pub node_id: String,
    #[serde(alias = "node_type")]
    pub node_type: NodeType,
    #[serde(alias = "node_name")]
    pub node_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "parent_node_id")]
    pub parent_node_id: Option<String>,
    #[serde(alias = "path")]
    pub path: Vec<PathElement>,
    #[serde(alias = "sequence")]
    pub sequence: i64,
    #[serde(alias = "event_timestamp")]
    pub event_timestamp: f64,
    #[serde(alias = "status")]
    pub status: NodeStatus,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "error_message")]
    pub error_message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "metadata")]
    pub metadata: Option<NodeMetadata>,
    /// ISO 8601 timestamp.
    #[serde(alias = "created_at")]
    pub created_at: String,
}

/// Paginated list of tree events.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct TreeEventListResponse {
    #[serde(alias = "events")]
    pub events: Vec<TreeEventResponse>,
    #[serde(alias = "total")]
    pub total: i64,
    #[serde(alias = "limit")]
    pub limit: i64,
    #[serde(alias = "offset")]
    pub offset: i64,
    #[serde(alias = "has_more")]
    pub has_more: bool,
}

/// Full execution tree reconstructed from events.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct ExecutionTreeResponse {
    /// Run UUID as string.
    #[serde(alias = "run_id")]
    pub run_id: String,
    #[serde(alias = "root_nodes")]
    pub root_nodes: Vec<DisplayNode>,
    #[serde(alias = "total_events")]
    pub total_events: i64,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "workflow_name")]
    pub workflow_name: Option<String>,
    #[serde(alias = "status")]
    pub status: NodeStatus,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "duration_ms")]
    pub duration_ms: Option<f64>,
    /// Initial active states when the workflow started.
    #[serde(default, alias = "initial_state_ids")]
    pub initial_state_ids: Vec<String>,
    /// Mapping of state IDs to display names.
    #[serde(default, alias = "state_name_map")]
    pub state_name_map: HashMap<String, String>,
}
