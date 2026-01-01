"""Execution Run schemas.

Defines the primary execution run models used for:
- Creating new execution runs
- Reporting run completion
- Retrieving run details

These schemas are used by both the runner (to submit data) and
the backend (to store and serve data).
"""

from typing import Any
from uuid import UUID

from pydantic import BaseModel, ConfigDict, Field

from qontinui_schemas.common.metadata import RunnerMetadata, WorkflowMetadata
from qontinui_schemas.common.stats import CoverageData, ExecutionStats
from qontinui_schemas.common.time import UTCDateTime
from qontinui_schemas.execution.enums import RunStatus, RunType


class ExecutionRunCreate(BaseModel):
    """Request schema for creating a new execution run.

    Sent by the runner at the start of an execution session.
    """

    project_id: UUID = Field(..., description="Project this run belongs to")
    run_type: RunType = Field(..., description="Type of execution run")
    run_name: str = Field(
        ..., max_length=255, description="Human-readable name for this run"
    )
    description: str | None = Field(None, description="Optional description")
    runner_metadata: RunnerMetadata = Field(
        ..., description="Information about the runner environment"
    )
    workflow_metadata: WorkflowMetadata | None = Field(
        None, description="Information about the workflow being executed"
    )
    configuration: dict[str, Any] = Field(
        default_factory=dict,
        description="Snapshot of execution configuration",
    )


class ExecutionRunResponse(BaseModel):
    """Response schema for execution run creation and retrieval.

    Returned after creating a run or when listing runs.
    """

    model_config = ConfigDict(from_attributes=True)

    run_id: UUID = Field(..., description="Unique run identifier")
    project_id: UUID = Field(..., description="Project ID")
    run_type: RunType = Field(..., description="Type of run")
    run_name: str = Field(..., description="Run name")
    status: RunStatus = Field(..., description="Current status")
    started_at: UTCDateTime = Field(..., description="When the run started (UTC)")
    ended_at: UTCDateTime | None = Field(None, description="When the run ended (UTC)")
    duration_seconds: int | None = Field(None, description="Total duration in seconds")


class ExecutionRunDetail(ExecutionRunResponse):
    """Detailed execution run information.

    Includes all metadata, stats, and configuration.
    """

    description: str | None = Field(None, description="Run description")
    runner_metadata: dict[str, Any] = Field(..., description="Runner metadata")
    workflow_metadata: dict[str, Any] | None = Field(
        None, description="Workflow metadata"
    )
    configuration: dict[str, Any] = Field(..., description="Configuration snapshot")
    stats: ExecutionStats | None = Field(None, description="Execution statistics")
    coverage_data: CoverageData | None = Field(None, description="Coverage data")
    error_message: str | None = Field(None, description="Error message if failed")
    created_at: UTCDateTime = Field(..., description="Record creation time (UTC)")
    updated_at: UTCDateTime | None = Field(None, description="Last update time (UTC)")
    created_by: dict[str, Any] | None = Field(None, description="User who created")


class ExecutionRunComplete(BaseModel):
    """Request schema for completing an execution run.

    Sent by the runner when execution finishes (success or failure).
    """

    status: RunStatus = Field(
        ..., description="Final status (completed, failed, timeout, cancelled)"
    )
    ended_at: UTCDateTime = Field(..., description="When the run ended (UTC)")
    stats: ExecutionStats = Field(..., description="Final execution statistics")
    coverage: CoverageData | None = Field(None, description="Final coverage data")
    summary: str | None = Field(None, description="Optional execution summary")
    error_message: str | None = Field(None, description="Error message if failed")


class ExecutionRunCompleteResponse(BaseModel):
    """Response schema for run completion.

    Confirms the run was successfully marked as complete.
    """

    model_config = ConfigDict(from_attributes=True)

    run_id: UUID = Field(..., description="Run ID")
    status: RunStatus = Field(..., description="Final status")
    started_at: UTCDateTime = Field(..., description="Start time (UTC)")
    ended_at: UTCDateTime = Field(..., description="End time (UTC)")
    duration_seconds: int = Field(..., description="Total duration in seconds")
    stats: ExecutionStats = Field(..., description="Final statistics")
    coverage: CoverageData | None = Field(None, description="Coverage data")


class ExecutionRunListResponse(BaseModel):
    """Response schema for paginated run list."""

    runs: list[ExecutionRunResponse] = Field(..., description="List of runs")
    total: int = Field(..., description="Total number of runs")
    limit: int = Field(..., description="Items per page")
    offset: int = Field(..., description="Items skipped")
    has_more: bool = Field(..., description="Whether more items exist")


__all__ = [
    "ExecutionRunCreate",
    "ExecutionRunResponse",
    "ExecutionRunDetail",
    "ExecutionRunComplete",
    "ExecutionRunCompleteResponse",
    "ExecutionRunListResponse",
]
