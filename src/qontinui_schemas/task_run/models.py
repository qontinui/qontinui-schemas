"""TaskRun Pydantic models.

TaskRun is THE unified run concept for all execution in Qontinui.
GUI automation is one aspect of a task, not a separate system.

Key design decisions:
1. Always create TaskRun, even for ad-hoc GUI automation
2. One model, one source of output, one storage location
3. task_run_automation is a child table for automation-specific metrics
"""

from typing import Any

from pydantic import BaseModel, ConfigDict, Field

from qontinui_schemas.common.time import UTCDateTime
from qontinui_schemas.task_run.enums import AutomationStatus, TaskRunStatus, TaskType

# =============================================================================
# TaskRun Models (Primary)
# =============================================================================


class TaskRunBase(BaseModel):
    """Base fields for TaskRun shared across create/response schemas."""

    task_name: str = Field(..., description="Human-readable name for this task")
    prompt: str | None = Field(
        None, description="Task prompt (NULL for pure automation)"
    )
    task_type: TaskType = Field(
        default=TaskType.TASK,
        description="Type of task: 'task', 'automation', or 'scheduled'",
    )
    auto_continue: bool = Field(
        default=True, description="Whether to auto-continue on session completion"
    )
    max_sessions: int | None = Field(
        None, description="Maximum number of AI sessions allowed"
    )
    execution_steps_json: str | None = Field(
        None, description="JSON-encoded execution steps to run before each AI session"
    )
    log_sources_json: str | None = Field(
        None, description="JSON-encoded log sources to capture during execution"
    )
    config_id: str | None = Field(
        None, description="Config ID for automation-enabled tasks"
    )
    workflow_name: str | None = Field(None, description="Workflow name being executed")


class TaskRunCreate(TaskRunBase):
    """Request schema for creating a new task run.

    Used by qontinui-runner when starting any type of task.
    """

    pass


class TaskRunResponse(BaseModel):
    """Response schema for task run creation and listing.

    Contains essential fields for display in lists.
    """

    model_config = ConfigDict(from_attributes=True)

    id: str = Field(..., description="Unique task run identifier")
    task_name: str = Field(..., description="Task name")
    task_type: TaskType = Field(..., description="Type of task")
    status: TaskRunStatus = Field(..., description="Current status")
    sessions_count: int = Field(default=0, description="Number of AI sessions run")
    max_sessions: int | None = Field(None, description="Maximum sessions allowed")
    auto_continue: bool = Field(default=True, description="Auto-continue enabled")
    config_id: str | None = Field(None, description="Config ID")
    workflow_name: str | None = Field(None, description="Workflow name")
    created_at: UTCDateTime = Field(..., description="When the task was created (UTC)")
    completed_at: UTCDateTime | None = Field(
        None, description="When the task completed (UTC)"
    )


class TaskRunDetail(TaskRunResponse):
    """Detailed task run information.

    Includes full output, summary, and all metadata.
    """

    prompt: str | None = Field(None, description="Task prompt")
    output_log: str = Field(default="", description="Full task output log")
    error_message: str | None = Field(None, description="Error message if failed")
    execution_steps_json: str | None = Field(None, description="Execution steps JSON")
    log_sources_json: str | None = Field(None, description="Log sources JSON")

    # Summary fields (populated after completion)
    summary: str | None = Field(
        None,
        description="AI-generated paragraph summary of the task run",
        json_schema_extra={"alias": "ai_summary"},  # For backward compat with runner
    )
    goal_achieved: bool | None = Field(
        None, description="Whether the stated goal was achieved"
    )
    remaining_work: str | None = Field(
        None, description="What remains to be done if goal not achieved"
    )
    summary_generated_at: UTCDateTime | None = Field(
        None, description="When the summary was generated"
    )

    updated_at: UTCDateTime = Field(..., description="Last update time (UTC)")


class TaskRunUpdate(BaseModel):
    """Request schema for updating a task run."""

    status: TaskRunStatus | None = Field(None, description="New status")
    output_log: str | None = Field(None, description="Output to append")
    error_message: str | None = Field(None, description="Error message")
    sessions_count: int | None = Field(None, description="Update session count")
    summary: str | None = Field(None, description="AI summary")
    goal_achieved: bool | None = Field(None, description="Goal achieved flag")
    remaining_work: str | None = Field(None, description="Remaining work description")


class TaskRunComplete(BaseModel):
    """Request schema for completing a task run.

    Sent when task execution finishes (success or failure).
    """

    status: TaskRunStatus = Field(
        ..., description="Final status (complete, failed, stopped)"
    )
    error_message: str | None = Field(None, description="Error message if failed")
    summary: str | None = Field(None, description="Optional execution summary")
    goal_achieved: bool | None = Field(None, description="Whether goal was achieved")
    remaining_work: str | None = Field(
        None, description="What remains if goal not achieved"
    )


class TaskRunReopen(BaseModel):
    """Request schema for reopening a finished task run.

    Allows continuing a task that didn't achieve its goal.
    """

    additional_sessions: int = Field(
        ..., ge=1, le=100, description="Number of additional sessions to add"
    )


class TaskRunListResponse(BaseModel):
    """Response schema for paginated task run list."""

    runs: list[TaskRunResponse] = Field(..., description="List of task runs")
    total: int = Field(..., description="Total number of runs")
    limit: int = Field(..., description="Items per page")
    offset: int = Field(..., description="Items skipped")
    has_more: bool = Field(..., description="Whether more items exist")


# =============================================================================
# TaskRunAutomation Models (Child table for automation metrics)
# =============================================================================


class TaskRunAutomationBase(BaseModel):
    """Base fields for automation execution within a task run."""

    task_run_id: str = Field(..., description="Parent task run ID")
    workflow_name: str | None = Field(None, description="Workflow being executed")
    iteration_number: int = Field(
        default=1, description="Iteration number within the task"
    )


class TaskRunAutomationCreate(TaskRunAutomationBase):
    """Request schema for creating an automation record."""

    pass


class TaskRunAutomationResponse(BaseModel):
    """Response schema for automation execution."""

    model_config = ConfigDict(from_attributes=True)

    id: str = Field(..., description="Unique automation record ID")
    task_run_id: str = Field(..., description="Parent task run ID")
    workflow_name: str | None = Field(None, description="Workflow name")
    automation_status: AutomationStatus = Field(..., description="Automation status")
    iteration_number: int = Field(default=1, description="Iteration number")
    started_at: UTCDateTime = Field(..., description="When automation started (UTC)")
    ended_at: UTCDateTime | None = Field(
        None, description="When automation ended (UTC)"
    )
    duration_ms: int | None = Field(None, description="Duration in milliseconds")


class TaskRunAutomationDetail(TaskRunAutomationResponse):
    """Detailed automation execution information.

    Includes all metrics from the run.
    """

    success: bool | None = Field(None, description="Whether automation succeeded")
    error_type: str | None = Field(None, description="Type of error if failed")
    error_message: str | None = Field(None, description="Error message if failed")

    # Metrics (stored as JSON strings, decoded on read)
    actions_summary: dict[str, Any] | None = Field(
        None, description="Summary of actions executed"
    )
    states_visited: list[str] | None = Field(None, description="States visited")
    transitions_executed: list[dict[str, Any]] | None = Field(
        None, description="Transitions executed"
    )
    template_matches: list[dict[str, Any]] | None = Field(
        None, description="Template matching results"
    )
    anomalies: list[dict[str, Any]] | None = Field(
        None, description="Anomalies detected"
    )


class TaskRunAutomationComplete(BaseModel):
    """Request schema for completing an automation execution."""

    automation_status: AutomationStatus = Field(..., description="Final status")
    success: bool = Field(..., description="Whether automation succeeded")
    error_type: str | None = Field(None, description="Error type if failed")
    error_message: str | None = Field(None, description="Error message if failed")
    actions_summary: dict[str, Any] | None = Field(None, description="Actions summary")
    states_visited: list[str] | None = Field(None, description="States visited")
    transitions_executed: list[dict[str, Any]] | None = Field(
        None, description="Transitions executed"
    )
    template_matches: list[dict[str, Any]] | None = Field(
        None, description="Template matches"
    )
    anomalies: list[dict[str, Any]] | None = Field(None, description="Anomalies")


class TaskRunAutomationListResponse(BaseModel):
    """Response schema for automation list within a task run."""

    automations: list[TaskRunAutomationResponse] = Field(
        ..., description="List of automation executions"
    )
    total: int = Field(..., description="Total count")


# =============================================================================
# Sync Payload (for unified sync to qontinui-web)
# =============================================================================


class TaskRunSyncPayload(BaseModel):
    """Unified payload for syncing task runs to qontinui-web.

    Combines task run data with automation records and findings.
    """

    task_run: TaskRunDetail = Field(..., description="Task run details")
    automations: list[TaskRunAutomationDetail] | None = Field(
        None, description="Automation execution records"
    )
    findings: list[dict[str, Any]] | None = Field(
        None, description="Code/automation findings"
    )
    discoveries: list[dict[str, Any]] | None = Field(
        None, description="Discoveries from automation"
    )


__all__ = [
    # TaskRun models
    "TaskRunBase",
    "TaskRunCreate",
    "TaskRunResponse",
    "TaskRunDetail",
    "TaskRunUpdate",
    "TaskRunComplete",
    "TaskRunReopen",
    "TaskRunListResponse",
    # TaskRunAutomation models
    "TaskRunAutomationBase",
    "TaskRunAutomationCreate",
    "TaskRunAutomationResponse",
    "TaskRunAutomationDetail",
    "TaskRunAutomationComplete",
    "TaskRunAutomationListResponse",
    # Sync
    "TaskRunSyncPayload",
]
