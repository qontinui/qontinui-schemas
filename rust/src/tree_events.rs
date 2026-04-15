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
pub struct MatchLocation {
    pub x: i64,
    pub y: i64,
    /// Width of the matched region (optional — point matches omit it).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub w: Option<i64>,
    /// Height of the matched region (optional — point matches omit it).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub h: Option<i64>,
}

/// A single match result with confidence and location.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TopMatch {
    pub confidence: f64,
    pub location: MatchLocation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<MatchLocation>,
}

/// Runtime execution data captured during action execution.
///
/// Different fields are populated depending on the action type. All fields
/// are optional; the wire schema allows additional runtime fields via a
/// `flatten`-d map.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct RuntimeData {
    // TYPE actions
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub typed_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub character_count: Option<i64>,

    // FIND / IF actions — pattern matching
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub found: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<MatchLocation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<MatchLocation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub match_method: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_matches: Option<Vec<TopMatch>>,

    // CLICK actions
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clicked_at: Option<MatchLocation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub button: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_type: Option<String>,

    // GO_TO_STATE actions
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_states: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_states: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub targets_reached: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transitions_executed: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub already_at_target: Option<bool>,

    // IF actions
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition_passed: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch_taken: Option<String>,

    // RUN_WORKFLOW actions
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_status: Option<String>,
}

/// State-machine context captured before/after an action.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct StateContext {
    #[serde(default)]
    pub active_before: Vec<String>,
    #[serde(default)]
    pub active_after: Vec<String>,
    #[serde(default)]
    pub changed: bool,
    #[serde(default)]
    pub activated: Vec<String>,
    #[serde(default)]
    pub deactivated: Vec<String>,
}

/// Precise timing information for an event. Times are ISO 8601 strings (see
/// crate-level docs for the rationale — the types crate is wire-only and
/// doesn't depend on a chrono version).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TimingInfo {
    /// ISO 8601 timestamp.
    pub start_time: String,
    /// ISO 8601 timestamp. `None` while the event is still in flight.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<f64>,
}

/// Execution outcome of an action.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Outcome {
    pub success: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default)]
    pub retry_count: i64,
}

// ============================================================================
// Main Models
// ============================================================================

/// Metadata for a tree node — action configuration, runtime data, state
/// context, and timing. All fields are optional because different node
/// types populate different fields.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct NodeMetadata {
    /// Action configuration (JSON object — shape varies by `ActionType`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config: Option<HashMap<String, serde_json::Value>>,

    /// Whether this action can have child nodes.
    #[serde(default)]
    pub is_expandable: bool,
    /// Whether this action is displayed inline.
    #[serde(default)]
    pub is_inline: bool,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime: Option<RuntimeData>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_context: Option<StateContext>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timing: Option<TimingInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<Outcome>,

    /// Screenshot reference (path or URL).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub screenshot_reference: Option<String>,
    /// Visual-debug image reference (path or URL).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visual_debug_reference: Option<String>,
}

/// A node in the execution tree — a single workflow, action, or transition
/// in the execution hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TreeNode {
    /// Unique identifier for this node.
    pub id: String,
    /// Type of node (workflow / action / transition).
    pub node_type: NodeType,
    /// Display name for this node.
    pub name: String,

    /// When this node was created (Unix epoch seconds).
    pub timestamp: f64,
    /// When this node completed (Unix epoch seconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_timestamp: Option<f64>,
    /// Duration in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,

    /// ID of parent node, `None` for root.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,

    /// Current execution status.
    pub status: NodeStatus,

    #[serde(default)]
    pub metadata: NodeMetadata,
    /// Error message if `status == Failed`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Element in a tree path (for breadcrumb navigation).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PathElement {
    pub id: String,
    pub name: String,
    pub node_type: NodeType,
}

/// A tree event emitted during execution.
///
/// Primary event type for execution logging. Carries the event type, the
/// affected node with full metadata, the path from root to this node, and
/// a sequence number for ordering.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TreeEvent {
    /// Event type identifier — always `"tree_event"` on the wire.
    #[serde(default = "default_tree_event_type")]
    pub r#type: String,
    /// Specific tree-event variant.
    pub event_type: TreeEventType,
    /// The node this event is about.
    pub node: TreeNode,
    /// Path from root to this node (breadcrumb).
    #[serde(default)]
    pub path: Vec<PathElement>,
    /// When this event was emitted (Unix epoch seconds).
    pub timestamp: f64,
    /// Sequence number for ordering.
    #[serde(default)]
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
pub struct DisplayNode {
    pub id: String,
    pub node_type: NodeType,
    pub name: String,
    pub timestamp: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_timestamp: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,
    pub status: NodeStatus,
    #[serde(default)]
    pub metadata: NodeMetadata,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Child nodes in the tree.
    #[serde(default)]
    pub children: Vec<DisplayNode>,
    /// Whether this node should be expanded in the UI (default: true).
    #[serde(default = "default_true")]
    pub is_expanded: bool,
    /// Nesting level in the tree (0 for root, 1 for first-level children).
    #[serde(default)]
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
pub struct TreeEventCreate {
    pub event_type: TreeEventType,
    pub node: TreeNode,
    #[serde(default)]
    pub path: Vec<PathElement>,
    pub timestamp: f64,
    #[serde(default)]
    pub sequence: i64,
}

/// Response for a stored tree event.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TreeEventResponse {
    /// UUID as string (wire-format — see crate-level docs).
    pub id: String,
    /// Run UUID as string.
    pub run_id: String,
    pub event_type: TreeEventType,
    pub node_id: String,
    pub node_type: NodeType,
    pub node_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_node_id: Option<String>,
    pub path: Vec<PathElement>,
    pub sequence: i64,
    pub event_timestamp: f64,
    pub status: NodeStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<NodeMetadata>,
    /// ISO 8601 timestamp.
    pub created_at: String,
}

/// Paginated list of tree events.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TreeEventListResponse {
    pub events: Vec<TreeEventResponse>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
    pub has_more: bool,
}

/// Full execution tree reconstructed from events.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionTreeResponse {
    /// Run UUID as string.
    pub run_id: String,
    pub root_nodes: Vec<DisplayNode>,
    pub total_events: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_name: Option<String>,
    pub status: NodeStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<f64>,
    /// Initial active states when the workflow started.
    #[serde(default)]
    pub initial_state_ids: Vec<String>,
    /// Mapping of state IDs to display names.
    #[serde(default)]
    pub state_name_map: HashMap<String, String>,
}
