"""
Scheduling and execution models for Qontinui automation.

This module provides models for state check results and scheduler statistics.
For base scheduling types (Schedule, ExecutionRecord, TriggerType, etc.),
see config_root.py which defines the core scheduling models for export configs.

All datetime fields use UTCDateTime for consistent UTC timezone handling
and ISO 8601 format strings with 'Z' suffix for JSON serialization.
"""

from enum import Enum

from pydantic import BaseModel, Field

from qontinui_schemas.common.time import UTCDateTime

# =============================================================================
# Enums
# =============================================================================


class StateCheckAction(str, Enum):
    """Action to take based on state check results."""

    NONE = "NONE"  # No action required
    WAIT = "WAIT"  # Wait before next check
    REBUILD = "REBUILD"  # Rebuild/recover the workflow


# =============================================================================
# State Check Result
# =============================================================================


class StateCheckResult(BaseModel):
    """
    Result of checking whether required states are present.

    Used by the scheduler to monitor workflow state and determine
    when recovery actions (rebuild) are needed.
    """

    schedule_id: str = Field(
        ...,
        alias="scheduleId",
        description="ID of the schedule being checked",
    )
    schedule_name: str = Field(
        ...,
        alias="scheduleName",
        description="Name of the schedule being checked",
    )
    check_time: UTCDateTime = Field(
        ...,
        alias="checkTime",
        description="When the check was performed (UTC)",
    )
    all_states_present: bool = Field(
        ...,
        alias="allStatesPresent",
        description="If true, all required states were found",
    )
    missing_states: list[str] = Field(
        default_factory=list,
        alias="missingStates",
        description="List of state IDs that were not found",
    )
    failure_streak: int = Field(
        ...,
        alias="failureStreak",
        description="Number of consecutive failed checks",
    )
    exceeded_threshold: bool = Field(
        ...,
        alias="exceededThreshold",
        description="If true, failure streak has exceeded the configured threshold",
    )
    action: StateCheckAction = Field(
        ...,
        description="Recommended action based on check result",
    )

    model_config = {"populate_by_name": True}


# =============================================================================
# Scheduler Statistics
# =============================================================================


class SchedulerStatistics(BaseModel):
    """
    Aggregate statistics about scheduler activity.

    Provides overview metrics for monitoring scheduler health
    and execution patterns.
    """

    total_schedules: int = Field(
        ...,
        alias="totalSchedules",
        description="Total number of schedules (enabled + disabled)",
    )
    active_schedules: int = Field(
        ...,
        alias="activeSchedules",
        description="Number of currently enabled schedules",
    )
    total_executions: int = Field(
        ...,
        alias="totalExecutions",
        description="Total number of executions across all schedules",
    )
    successful_executions: int = Field(
        ...,
        alias="successfulExecutions",
        description="Number of successful executions",
    )
    failed_executions: int = Field(
        ...,
        alias="failedExecutions",
        description="Number of failed executions",
    )
    average_iteration_count: float = Field(
        ...,
        alias="averageIterationCount",
        description="Average number of iterations per execution",
    )

    model_config = {"populate_by_name": True}
