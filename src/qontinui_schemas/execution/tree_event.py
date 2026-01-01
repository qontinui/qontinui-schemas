"""Execution Tree Event schemas.

Defines schemas for hierarchical execution events.
These events capture the tree structure of workflow execution
(workflows contain actions, actions can be nested).

Note: Core TreeEvent types are defined in qontinui_schemas.events.tree_events.
This module provides additional API schemas for tree event persistence.
"""

from uuid import UUID

from pydantic import BaseModel, ConfigDict, Field

from qontinui_schemas.common.time import UTCDateTime
from qontinui_schemas.events.tree_events import (
    DisplayNode,
    NodeMetadata,
    NodeStatus,
    NodeType,
    PathElement,
    TreeEventType,
    TreeNode,
)


class ExecutionTreeEventCreate(BaseModel):
    """Request schema for storing a tree event.

    Sent by the runner during execution to record tree structure.
    """

    event_type: TreeEventType = Field(..., description="Type of tree event")
    node: TreeNode = Field(..., description="The tree node this event is about")
    path: list[PathElement] = Field(
        default_factory=list, description="Path from root to this node"
    )
    timestamp: float = Field(..., description="When event occurred (Unix epoch)")
    sequence: int = Field(0, description="Sequence number for ordering")


class ExecutionTreeEventBatchCreate(BaseModel):
    """Request schema for batch tree event storage."""

    events: list[ExecutionTreeEventCreate] = Field(
        ...,
        min_length=1,
        max_length=100,
        description="List of tree events to store",
    )


class ExecutionTreeEventResponse(BaseModel):
    """Response schema for a stored tree event.

    Returned when querying stored events.
    """

    model_config = ConfigDict(from_attributes=True)

    id: UUID = Field(..., description="Event ID")
    run_id: UUID = Field(..., description="Run ID")
    event_type: TreeEventType = Field(..., description="Event type")
    node_id: str = Field(..., description="Node ID")
    node_type: NodeType = Field(..., description="Node type")
    node_name: str = Field(..., description="Node name")
    parent_node_id: str | None = Field(None, description="Parent node ID")
    path: list[PathElement] = Field(default_factory=list, description="Path to node")
    sequence: int = Field(..., description="Sequence number")
    event_timestamp: float = Field(..., description="Event timestamp (Unix epoch)")
    node_start_timestamp: float | None = Field(None, description="Node start timestamp")
    node_end_timestamp: float | None = Field(None, description="Node end timestamp")
    duration_ms: float | None = Field(None, description="Duration in milliseconds")
    status: NodeStatus = Field(..., description="Node status")
    error_message: str | None = Field(None, description="Error message if failed")
    active_states_before: list[str] = Field(
        default_factory=list, description="Active states before event"
    )
    active_states_after: list[str] = Field(
        default_factory=list, description="Active states after event"
    )
    states_changed: bool = Field(False, description="Whether states changed")
    metadata: NodeMetadata | None = Field(None, description="Node metadata")
    created_at: UTCDateTime = Field(..., description="Record creation time (UTC)")


class ExecutionTreeEventListResponse(BaseModel):
    """Response schema for paginated tree event list."""

    events: list[ExecutionTreeEventResponse] = Field(
        ..., description="List of tree events"
    )
    total: int = Field(..., description="Total count")
    limit: int = Field(..., description="Items per page")
    offset: int = Field(..., description="Items skipped")
    has_more: bool = Field(..., description="Whether more items exist")


class ExecutionTreeResponse(BaseModel):
    """Full execution tree structure reconstructed from events.

    Used for displaying the execution tree in the UI.
    """

    run_id: UUID = Field(..., description="Run ID")
    root_nodes: list[DisplayNode] = Field(
        default_factory=list, description="Root nodes of the tree"
    )
    total_events: int = Field(..., description="Total number of events")
    workflow_name: str | None = Field(None, description="Workflow name")
    status: NodeStatus = Field(..., description="Overall execution status")
    duration_ms: float | None = Field(None, description="Total duration in ms")
    initial_state_ids: list[str] = Field(
        default_factory=list, description="Initial active states"
    )
    state_name_map: dict[str, str] = Field(
        default_factory=dict, description="State ID to name mapping"
    )


# Re-export core tree event types for convenience
__all__ = [
    # Create/Response schemas
    "ExecutionTreeEventCreate",
    "ExecutionTreeEventBatchCreate",
    "ExecutionTreeEventResponse",
    "ExecutionTreeEventListResponse",
    "ExecutionTreeResponse",
    # Core types (from events module)
    "TreeNode",
    "TreeEventType",
    "NodeType",
    "NodeStatus",
    "NodeMetadata",
    "PathElement",
    "DisplayNode",
]
