"""Execution Issue schemas.

Defines schemas for issues detected during execution.
Issues represent problems found during automation that need attention.
"""

from typing import Any
from uuid import UUID

from pydantic import BaseModel, ConfigDict, Field

from qontinui_schemas.common.time import UTCDateTime
from qontinui_schemas.execution.enums import (
    IssueSeverity,
    IssueSource,
    IssueStatus,
    IssueType,
)


class ExecutionIssueCreate(BaseModel):
    """Schema for reporting an execution issue.

    Sent by the runner when an issue is detected during execution.
    """

    title: str = Field(..., max_length=500, description="Brief issue title")
    description: str = Field(..., description="Detailed issue description")
    severity: IssueSeverity = Field(..., description="Issue severity")
    issue_type: IssueType = Field(..., description="Type of issue")

    # Context
    action_sequence_number: int | None = Field(
        None, description="Related action sequence number"
    )
    state: str | None = Field(
        None, max_length=255, description="State where issue occurred"
    )
    screenshot_ids: list[UUID] = Field(
        default_factory=list, description="Associated screenshot IDs"
    )

    # Reproduction information
    reproduction_steps: list[str] = Field(
        default_factory=list, description="Steps to reproduce"
    )
    expected_behavior: str | None = Field(None, description="Expected behavior")
    actual_behavior: str | None = Field(None, description="Actual behavior observed")

    # Additional data
    metadata: dict[str, Any] = Field(
        default_factory=dict, description="Additional issue metadata"
    )


class ExecutionIssueBatchCreate(BaseModel):
    """Request schema for batch issue reporting."""

    issues: list[ExecutionIssueCreate] = Field(
        ...,
        min_length=1,
        max_length=20,
        description="List of issues to report",
    )


class ExecutionIssueResponse(BaseModel):
    """Response schema for issue batch creation."""

    model_config = ConfigDict(from_attributes=True)

    recorded: int = Field(..., description="Number of issues recorded")
    run_id: UUID = Field(..., description="Run ID")
    issue_ids: list[UUID] = Field(
        default_factory=list, description="IDs of created issues"
    )


class ExecutionIssueDetail(BaseModel):
    """Detailed issue information.

    Used when retrieving individual issue details.
    """

    model_config = ConfigDict(from_attributes=True)

    id: UUID = Field(..., description="Issue ID")
    run_id: UUID = Field(..., description="Run ID")
    action_execution_id: UUID | None = Field(None, description="Associated action ID")
    assigned_to_user_id: UUID | None = Field(None, description="Assigned user ID")
    issue_type: IssueType = Field(..., description="Issue type")
    severity: IssueSeverity = Field(..., description="Severity")
    status: IssueStatus = Field(..., description="Current status")
    source: IssueSource = Field(..., description="How issue was detected")
    title: str = Field(..., description="Issue title")
    description: str = Field(..., description="Issue description")
    state_name: str | None = Field(None, description="State name")
    screenshot_ids: list[UUID] = Field(
        default_factory=list, description="Screenshot IDs"
    )
    reproduction_steps: list[str] = Field(
        default_factory=list, description="Reproduction steps"
    )
    error_details: dict[str, Any] = Field(
        default_factory=dict, description="Error details"
    )
    metadata: dict[str, Any] = Field(default_factory=dict, description="Metadata")
    resolution_notes: str | None = Field(None, description="Resolution notes")
    created_at: UTCDateTime = Field(..., description="Creation time (UTC)")
    updated_at: UTCDateTime = Field(..., description="Last update time (UTC)")


class ExecutionIssueUpdate(BaseModel):
    """Request schema for updating an issue."""

    status: IssueStatus | None = Field(None, description="New status")
    severity: IssueSeverity | None = Field(None, description="New severity")
    assigned_to_user_id: UUID | None = Field(None, description="Assign to user")
    resolution_notes: str | None = Field(None, description="Resolution notes")


class ExecutionIssueListResponse(BaseModel):
    """Response schema for paginated issue list."""

    issues: list[ExecutionIssueDetail] = Field(..., description="List of issues")
    total: int = Field(..., description="Total count")
    limit: int = Field(..., description="Items per page")
    offset: int = Field(..., description="Items skipped")
    has_more: bool = Field(..., description="Whether more items exist")


class ExecutionIssueSummary(BaseModel):
    """Summary statistics for issues in a run."""

    total: int = Field(0, description="Total issues")
    by_severity: dict[str, int] = Field(
        default_factory=dict, description="Count by severity"
    )
    by_type: dict[str, int] = Field(default_factory=dict, description="Count by type")
    by_status: dict[str, int] = Field(
        default_factory=dict, description="Count by status"
    )
    critical_count: int = Field(0, description="Critical issues")
    high_count: int = Field(0, description="High priority issues")
    open_count: int = Field(0, description="Open issues")
    resolved_count: int = Field(0, description="Resolved issues")


__all__ = [
    "ExecutionIssueCreate",
    "ExecutionIssueBatchCreate",
    "ExecutionIssueResponse",
    "ExecutionIssueDetail",
    "ExecutionIssueUpdate",
    "ExecutionIssueListResponse",
    "ExecutionIssueSummary",
]
