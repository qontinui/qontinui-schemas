"""Tree event schemas for execution logging.

This module defines the TreeEvent schema used across the Qontinui ecosystem
for execution logging and visualization. These types are used by:
- qontinui (Python library) - emits events during execution
- qontinui-runner (Tauri app) - receives and displays events
- qontinui-web (backend) - stores and forwards events
- qontinui-web (frontend) - displays historical events

The schema follows the tree-based execution architecture where:
- Workflows contain actions
- Actions can be nested (e.g., GO_TO_STATE, RUN_WORKFLOW)
- Each event captures node state changes with rich metadata
"""

from enum import Enum
from typing import Any
from uuid import UUID

from pydantic import BaseModel, Field

# =============================================================================
# Enums
# =============================================================================


class NodeType(str, Enum):
    """Types of nodes in the execution tree."""

    WORKFLOW = "workflow"
    ACTION = "action"
    TRANSITION = "transition"


class NodeStatus(str, Enum):
    """Execution status of a tree node."""

    PENDING = "pending"
    RUNNING = "running"
    SUCCESS = "success"
    FAILED = "failed"


class TreeEventType(str, Enum):
    """Types of tree events emitted during execution."""

    # Workflow lifecycle
    WORKFLOW_STARTED = "workflow_started"
    WORKFLOW_COMPLETED = "workflow_completed"
    WORKFLOW_FAILED = "workflow_failed"

    # Action lifecycle
    ACTION_STARTED = "action_started"
    ACTION_COMPLETED = "action_completed"
    ACTION_FAILED = "action_failed"

    # Transition lifecycle (for state machine navigation)
    TRANSITION_STARTED = "transition_started"
    TRANSITION_COMPLETED = "transition_completed"
    TRANSITION_FAILED = "transition_failed"


class ActionType(str, Enum):
    """Types of actions in the automation system.

    These correspond to the action types defined in qontinui-schemas config models.
    """

    # Vision actions
    FIND = "FIND"
    FIND_STATE_IMAGE = "FIND_STATE_IMAGE"
    EXISTS = "EXISTS"
    VANISH = "VANISH"

    # Mouse actions
    CLICK = "CLICK"
    DOUBLE_CLICK = "DOUBLE_CLICK"
    RIGHT_CLICK = "RIGHT_CLICK"
    DRAG = "DRAG"
    SCROLL = "SCROLL"
    MOUSE_MOVE = "MOUSE_MOVE"

    # Keyboard actions
    TYPE = "TYPE"
    KEY_PRESS = "KEY_PRESS"
    HOTKEY = "HOTKEY"

    # Control flow
    IF = "IF"
    LOOP = "LOOP"
    SWITCH = "SWITCH"
    TRY_CATCH = "TRY_CATCH"
    BREAK = "BREAK"
    CONTINUE = "CONTINUE"

    # State machine
    GO_TO_STATE = "GO_TO_STATE"
    WAIT = "WAIT"

    # Workflow
    RUN_WORKFLOW = "RUN_WORKFLOW"

    # Utility
    SCREENSHOT = "SCREENSHOT"
    CODE_BLOCK = "CODE_BLOCK"
    SHELL = "SHELL"
    CUSTOM = "CUSTOM"


# =============================================================================
# Nested Metadata Models
# =============================================================================


class MatchLocation(BaseModel):
    """Location of a pattern match on screen."""

    x: int
    y: int
    w: int | None = Field(default=None, description="Width of matched region")
    h: int | None = Field(default=None, description="Height of matched region")

    class Config:
        populate_by_name = True


class TopMatch(BaseModel):
    """A single match result with confidence and location."""

    confidence: float
    location: MatchLocation
    dimensions: MatchLocation | None = None


class RuntimeData(BaseModel):
    """Runtime execution data captured during action execution.

    Different fields are populated depending on the action type.
    """

    # TYPE actions
    typed_text: str | None = None
    character_count: int | None = None

    # FIND/IF actions - pattern matching
    image_id: str | None = None
    found: bool | None = None
    confidence: float | None = None
    location: MatchLocation | None = None
    dimensions: MatchLocation | None = None
    match_method: str | None = None
    top_matches: list[TopMatch] | None = None

    # CLICK actions
    clicked_at: MatchLocation | None = None
    button: str | None = None
    target_type: str | None = None

    # GO_TO_STATE actions
    source_states: list[str] | None = None
    target_states: list[str] | None = None
    targets_reached: list[str] | None = None
    transitions_executed: list[str] | None = None
    already_at_target: bool | None = None

    # IF actions
    condition_passed: bool | None = None
    branch_taken: str | None = None

    # RUN_WORKFLOW actions
    workflow_name: str | None = None
    workflow_status: str | None = None

    class Config:
        extra = "allow"  # Allow additional runtime fields


class StateContext(BaseModel):
    """State machine context for an event.

    Captures the state of the system before and after an action.
    """

    active_before: list[str] = Field(default_factory=list)
    active_after: list[str] = Field(default_factory=list)
    changed: bool = False
    activated: list[str] = Field(default_factory=list)
    deactivated: list[str] = Field(default_factory=list)


class TimingInfo(BaseModel):
    """Precise timing information for an event."""

    start_time: str = Field(description="ISO 8601 timestamp")
    end_time: str | None = Field(default=None, description="ISO 8601 timestamp")
    duration_ms: float | None = None


class Outcome(BaseModel):
    """Execution outcome of an action."""

    success: bool
    error: str | None = None
    retry_count: int = 0


# =============================================================================
# Main Models
# =============================================================================


class NodeMetadata(BaseModel):
    """Metadata for a tree node.

    Contains action configuration, runtime data, state context, and timing.
    All fields are optional as different node types populate different fields.
    """

    # Action configuration
    config: dict[str, Any] | None = None

    # Node behavior flags
    is_expandable: bool = Field(
        default=False, description="Whether this action can have child nodes"
    )
    is_inline: bool = Field(
        default=False, description="Whether this action is displayed inline"
    )

    # Runtime execution data
    runtime: RuntimeData | None = None

    # State machine context
    state_context: StateContext | None = None

    # Timing information
    timing: TimingInfo | None = None

    # Execution outcome
    outcome: Outcome | None = None

    # Screenshot references (paths or URLs)
    screenshot_reference: str | None = None
    visual_debug_reference: str | None = None

    class Config:
        extra = "allow"  # Allow additional metadata fields
        populate_by_name = True


class TreeNode(BaseModel):
    """A node in the execution tree.

    Represents a single workflow, action, or transition in the execution hierarchy.
    """

    id: str = Field(description="Unique identifier for this node")
    node_type: NodeType = Field(
        description="Type of node (workflow, action, transition)"
    )
    name: str = Field(description="Display name for this node")

    # Timestamps (Unix epoch in seconds)
    timestamp: float = Field(description="When this node was created")
    end_timestamp: float | None = Field(
        default=None, description="When this node completed"
    )
    duration: float | None = Field(default=None, description="Duration in seconds")

    # Hierarchy
    parent_id: str | None = Field(
        default=None, description="ID of parent node, null for root"
    )

    # Status
    status: NodeStatus = Field(description="Current execution status")

    # Metadata and error
    metadata: NodeMetadata = Field(default_factory=NodeMetadata)
    error: str | None = Field(default=None, description="Error message if failed")

    class Config:
        populate_by_name = True


class PathElement(BaseModel):
    """Element in a tree path (for breadcrumb navigation)."""

    id: str = Field(description="Unique identifier for this path element")
    name: str = Field(description="Display name of this element")
    node_type: NodeType = Field(description="Type of this element")

    class Config:
        populate_by_name = True


class TreeEvent(BaseModel):
    """A tree event emitted during execution.

    This is the primary event type for execution logging, containing:
    - The event type (started, completed, failed)
    - The affected node with full metadata
    - The path from root to this node
    - Sequence number for ordering
    """

    type: str = Field(default="tree_event", description="Event type identifier")
    event_type: TreeEventType = Field(description="Specific tree event type")
    node: TreeNode = Field(description="The node this event is about")
    path: list[PathElement] = Field(
        default_factory=list, description="Path from root to this node"
    )
    timestamp: float = Field(description="When this event was emitted (Unix epoch)")
    sequence: int = Field(default=0, description="Sequence number for ordering")

    class Config:
        populate_by_name = True


# =============================================================================
# Display Models (Frontend-specific)
# =============================================================================


class DisplayNode(BaseModel):
    """Display node structure used by the frontend.

    Extended version of TreeNode with tree rendering properties.
    This is NOT persisted - it's constructed from TreeNode data for UI display.
    """

    id: str
    node_type: NodeType
    name: str
    timestamp: float
    end_timestamp: float | None = None
    duration: float | None = None
    status: NodeStatus
    metadata: NodeMetadata = Field(default_factory=NodeMetadata)
    error: str | None = None

    # Tree rendering properties (not in TreeNode)
    children: list["DisplayNode"] = Field(default_factory=list)
    is_expanded: bool = Field(default=True)
    level: int = Field(default=0, description="Nesting level (0 for root)")

    class Config:
        populate_by_name = True


# Rebuild model for forward reference
DisplayNode.model_rebuild()


# =============================================================================
# API Request/Response Models
# =============================================================================


class TreeEventCreate(BaseModel):
    """Request to store a tree event."""

    event_type: TreeEventType
    node: TreeNode
    path: list[PathElement] = Field(default_factory=list)
    timestamp: float
    sequence: int = 0


class TreeEventResponse(BaseModel):
    """Response for a stored tree event."""

    id: UUID
    run_id: UUID
    event_type: TreeEventType
    node_id: str
    node_type: NodeType
    node_name: str
    parent_node_id: str | None
    path: list[PathElement]
    sequence: int
    event_timestamp: float
    status: NodeStatus
    error_message: str | None
    metadata: NodeMetadata | None
    created_at: str  # ISO 8601


class TreeEventListResponse(BaseModel):
    """Paginated list of tree events."""

    events: list[TreeEventResponse]
    total: int
    limit: int
    offset: int
    has_more: bool


class ExecutionTreeResponse(BaseModel):
    """Full execution tree structure reconstructed from events."""

    run_id: UUID
    root_nodes: list[DisplayNode]
    total_events: int
    workflow_name: str | None
    status: NodeStatus
    duration_ms: float | None
    initial_state_ids: list[str] = Field(
        default_factory=list,
        description="Initial active states when workflow started",
    )
