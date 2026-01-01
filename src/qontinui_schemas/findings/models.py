"""Findings models for AI analysis results.

Defines schemas for findings detected during AI-powered code analysis.
Findings represent issues, observations, or recommendations from AI sessions.
"""

from uuid import UUID

from pydantic import BaseModel, ConfigDict, Field

from qontinui_schemas.common.time import UTCDateTime

from .enums import (
    FindingActionType,
    FindingCategory,
    FindingSeverity,
    FindingStatus,
)


class FindingCodeContext(BaseModel):
    """Code context for a finding.

    Provides location and snippet information for findings related to specific code.
    """

    file: str | None = Field(
        None,
        description="File path where the finding was detected",
    )
    line: int | None = Field(
        None,
        description="Line number where the finding was detected",
    )
    column: int | None = Field(
        None,
        description="Column number where the finding was detected",
    )
    snippet: str | None = Field(
        None,
        max_length=1000,
        description="Code snippet related to the finding (max 1000 chars)",
    )


class FindingUserInput(BaseModel):
    """User input request for a finding.

    Defines the question and input format when a finding requires user decision.
    """

    question: str = Field(
        ...,
        description="Question to present to the user",
    )
    input_type: str = Field(
        default="text",
        description="Type of input expected: 'text' or 'choice'",
    )
    options: list[str] | None = Field(
        None,
        description="Options for choice-type input",
    )


class FindingCreate(BaseModel):
    """Schema for creating a finding.

    Sent by the runner when an AI analysis session detects issues or observations.
    Runner -> Backend.
    """

    task_run_id: str = Field(
        ...,
        description="Parent task run ID",
    )
    session_num: int = Field(
        ...,
        description="Session number where the finding was detected",
    )
    category: FindingCategory = Field(
        ...,
        description="Category of the finding",
    )
    severity: FindingSeverity = Field(
        ...,
        description="Severity level of the finding",
    )
    title: str = Field(
        ...,
        max_length=500,
        description="Brief title describing the finding (max 500 chars)",
    )
    description: str = Field(
        ...,
        description="Detailed description of the finding",
    )
    code_context: FindingCodeContext | None = Field(
        None,
        description="Code context if the finding relates to specific code",
    )
    signature_hash: str | None = Field(
        None,
        description="Hash for deduplication across sessions",
    )
    action_type: FindingActionType = Field(
        ...,
        description="Type of action for this finding",
    )
    user_input: FindingUserInput | None = Field(
        None,
        description="User input request if action_type requires user decision",
    )


class FindingBatchCreate(BaseModel):
    """Request schema for batch finding creation.

    Allows creating multiple findings in a single request.
    """

    findings: list[FindingCreate] = Field(
        ...,
        min_length=1,
        max_length=50,
        description="List of findings to create (1-50 items)",
    )


class FindingUpdate(BaseModel):
    """Schema for updating a finding.

    Used to update status, resolution, or provide user response.
    """

    status: FindingStatus | None = Field(
        None,
        description="New status for the finding",
    )
    resolution: str | None = Field(
        None,
        description="Resolution description",
    )
    resolved_in_session: int | None = Field(
        None,
        description="Session number where the finding was resolved",
    )
    user_response: str | None = Field(
        None,
        description="User's response to a finding requiring input",
    )


class FindingDetail(BaseModel):
    """Detailed finding information.

    Used when retrieving individual finding details.
    Backend -> Frontend.
    """

    model_config = ConfigDict(from_attributes=True)

    id: UUID = Field(
        ...,
        description="Finding ID",
    )
    task_run_id: str = Field(
        ...,
        description="Parent task run ID",
    )
    session_num: int = Field(
        ...,
        description="Session number where the finding was detected",
    )
    category: FindingCategory = Field(
        ...,
        description="Category of the finding",
    )
    severity: FindingSeverity = Field(
        ...,
        description="Severity level of the finding",
    )
    status: FindingStatus = Field(
        ...,
        description="Current status of the finding",
    )
    title: str = Field(
        ...,
        description="Brief title describing the finding",
    )
    description: str = Field(
        ...,
        description="Detailed description of the finding",
    )
    resolution: str | None = Field(
        None,
        description="Resolution description if resolved",
    )
    code_context: FindingCodeContext | None = Field(
        None,
        description="Code context if the finding relates to specific code",
    )
    signature_hash: str | None = Field(
        None,
        description="Hash for deduplication across sessions",
    )
    action_type: FindingActionType = Field(
        ...,
        description="Type of action for this finding",
    )
    user_input: FindingUserInput | None = Field(
        None,
        description="User input request if action_type requires user decision",
    )
    user_response: str | None = Field(
        None,
        description="User's response if input was requested",
    )
    detected_at: UTCDateTime = Field(
        ...,
        description="When the finding was detected (UTC)",
    )
    resolved_at: UTCDateTime | None = Field(
        None,
        description="When the finding was resolved (UTC)",
    )
    resolved_in_session: int | None = Field(
        None,
        description="Session number where the finding was resolved",
    )


class FindingListResponse(BaseModel):
    """Response schema for paginated finding list."""

    findings: list[FindingDetail] = Field(
        ...,
        description="List of findings",
    )
    total: int = Field(
        ...,
        description="Total count of findings matching the query",
    )
    limit: int = Field(
        ...,
        description="Maximum items per page",
    )
    offset: int = Field(
        ...,
        description="Number of items skipped",
    )
    has_more: bool = Field(
        ...,
        description="Whether more items exist beyond this page",
    )


class FindingSummary(BaseModel):
    """Summary statistics for findings in a task run."""

    task_run_id: str = Field(
        ...,
        description="Task run ID",
    )
    total: int = Field(
        default=0,
        description="Total number of findings",
    )
    by_category: dict[str, int] = Field(
        default_factory=dict,
        description="Count of findings by category",
    )
    by_severity: dict[str, int] = Field(
        default_factory=dict,
        description="Count of findings by severity",
    )
    by_status: dict[str, int] = Field(
        default_factory=dict,
        description="Count of findings by status",
    )
    needs_input_count: int = Field(
        default=0,
        description="Number of findings awaiting user input",
    )
    resolved_count: int = Field(
        default=0,
        description="Number of resolved findings",
    )
    outstanding_count: int = Field(
        default=0,
        description="Number of unresolved findings",
    )


__all__ = [
    "FindingCodeContext",
    "FindingUserInput",
    "FindingCreate",
    "FindingBatchCreate",
    "FindingUpdate",
    "FindingDetail",
    "FindingListResponse",
    "FindingSummary",
]
