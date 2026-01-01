"""Action Execution schemas.

Defines schemas for individual action executions within a run.
Actions are the atomic units of automation (click, type, find, etc.).
"""

from typing import Any
from uuid import UUID

from pydantic import BaseModel, ConfigDict, Field

from qontinui_schemas.common.metadata import MatchLocation
from qontinui_schemas.common.time import UTCDateTime
from qontinui_schemas.execution.enums import ActionStatus, ActionType, ErrorType


class ActionExecutionCreate(BaseModel):
    """Schema for reporting a single action execution.

    Sent by the runner during or after action execution.
    """

    sequence_number: int = Field(..., ge=0, description="Order within the run")
    action_type: ActionType = Field(..., description="Type of action")
    action_name: str = Field(
        ..., max_length=255, description="Human-readable action name"
    )
    status: ActionStatus = Field(..., description="Execution status")
    started_at: UTCDateTime = Field(..., description="When action started (UTC)")
    completed_at: UTCDateTime = Field(..., description="When action completed (UTC)")
    duration_ms: int = Field(..., ge=0, description="Duration in milliseconds")

    # State context
    from_state: str | None = Field(None, description="State before action")
    to_state: str | None = Field(None, description="Expected state after action")
    active_states: list[str] | None = Field(None, description="Currently active states")

    # Pattern matching context (for vision actions)
    pattern_id: str | None = Field(None, description="Pattern ID used")
    pattern_name: str | None = Field(None, description="Pattern name")
    confidence_score: float | None = Field(
        None, ge=0.0, le=1.0, description="Match confidence (0-1)"
    )
    match_location: MatchLocation | None = Field(
        None, description="Where the element was found"
    )

    # Error information
    error_message: str | None = Field(None, description="Error message if failed")
    error_type: ErrorType | None = Field(None, description="Type of error")
    error_stack: str | None = Field(None, description="Stack trace if available")

    # References
    screenshot_id: UUID | None = Field(None, description="Associated screenshot ID")
    parent_action_id: UUID | None = Field(
        None, description="Parent action ID (for nested actions)"
    )

    # Additional data
    input_data: dict[str, Any] = Field(
        default_factory=dict, description="Action input parameters"
    )
    output_data: dict[str, Any] = Field(
        default_factory=dict, description="Action output/results"
    )
    metadata: dict[str, Any] = Field(
        default_factory=dict, description="Additional metadata"
    )


class ActionExecutionBatchCreate(BaseModel):
    """Request schema for batch action reporting.

    Allows reporting multiple actions in a single request for efficiency.
    """

    actions: list[ActionExecutionCreate] = Field(
        ...,
        min_length=1,
        max_length=50,
        description="List of actions to report",
    )


class ActionExecutionResponse(BaseModel):
    """Response schema for action batch creation."""

    model_config = ConfigDict(from_attributes=True)

    recorded: int = Field(..., description="Number of actions recorded")
    run_id: UUID = Field(..., description="Run ID")
    action_ids: list[UUID] = Field(
        default_factory=list, description="IDs of created actions"
    )


class ActionExecutionDetail(BaseModel):
    """Detailed action execution information.

    Used when retrieving individual action details.
    """

    model_config = ConfigDict(from_attributes=True)

    id: UUID = Field(..., description="Action execution ID")
    run_id: UUID = Field(..., description="Run ID")
    sequence_number: int = Field(..., description="Sequence number")
    action_type: ActionType = Field(..., description="Action type")
    action_name: str = Field(..., description="Action name")
    status: ActionStatus = Field(..., description="Status")
    started_at: UTCDateTime = Field(..., description="Start time (UTC)")
    completed_at: UTCDateTime | None = Field(None, description="Completion time (UTC)")
    duration_ms: int | None = Field(None, description="Duration in ms")
    from_state: str | None = Field(None, description="Source state")
    to_state: str | None = Field(None, description="Target state")
    actual_state: str | None = Field(None, description="Actual state reached")
    error_message: str | None = Field(None, description="Error message")
    error_type: str | None = Field(None, description="Error type")
    input_data: dict[str, Any] = Field(default_factory=dict, description="Input data")
    output_data: dict[str, Any] = Field(default_factory=dict, description="Output data")
    metadata: dict[str, Any] = Field(default_factory=dict, description="Metadata")
    created_at: UTCDateTime = Field(..., description="Record creation time (UTC)")


class ActionExecutionListResponse(BaseModel):
    """Response schema for paginated action list."""

    actions: list[ActionExecutionDetail] = Field(..., description="List of actions")
    total: int = Field(..., description="Total count")
    limit: int = Field(..., description="Items per page")
    offset: int = Field(..., description="Items skipped")
    has_more: bool = Field(..., description="Whether more items exist")


__all__ = [
    "ActionExecutionCreate",
    "ActionExecutionBatchCreate",
    "ActionExecutionResponse",
    "ActionExecutionDetail",
    "ActionExecutionListResponse",
]
