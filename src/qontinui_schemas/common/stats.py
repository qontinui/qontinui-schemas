"""Execution statistics schemas.

These schemas define aggregate statistics for execution runs,
used for dashboards, analytics, and reporting.
"""

from pydantic import BaseModel, Field


class ExecutionStats(BaseModel):
    """Aggregate statistics for an execution run.

    Calculated from individual action executions and updated
    when a run is completed.
    """

    total_actions: int = Field(..., description="Total number of actions executed")
    successful_actions: int = Field(..., description="Actions that succeeded")
    failed_actions: int = Field(..., description="Actions that failed")
    timeout_actions: int = Field(..., description="Actions that timed out")
    skipped_actions: int = Field(..., description="Actions that were skipped")
    total_duration_ms: int = Field(..., description="Total execution time in ms")
    avg_action_duration_ms: int | None = Field(
        None, description="Average action duration in ms"
    )


class CoverageData(BaseModel):
    """Coverage statistics for an execution run.

    Tracks how much of the workflow's states and transitions
    were exercised during the run.
    """

    coverage_percentage: float = Field(
        ..., ge=0.0, le=100.0, description="Overall coverage percentage"
    )
    states_covered: int = Field(..., description="Number of states visited")
    total_states: int = Field(..., description="Total states in workflow")
    transitions_covered: int = Field(..., description="Number of transitions executed")
    total_transitions: int = Field(..., description="Total transitions in workflow")
    uncovered_states: list[str] = Field(
        default_factory=list, description="List of unvisited state IDs"
    )
    uncovered_transitions: list[str] = Field(
        default_factory=list, description="List of unexecuted transition IDs"
    )
    state_visit_counts: dict[str, int] = Field(
        default_factory=dict,
        description="Map of state ID to visit count",
    )
    transition_execution_counts: dict[str, int] = Field(
        default_factory=dict,
        description="Map of transition ID to execution count",
    )


class ReliabilityStats(BaseModel):
    """Reliability statistics for a transition or action.

    Used for analytics to identify flaky or problematic automations.
    """

    total_executions: int = Field(..., description="Total number of executions")
    successful_executions: int = Field(..., description="Successful executions")
    failed_executions: int = Field(..., description="Failed executions")
    success_rate: float = Field(
        ..., ge=0.0, le=100.0, description="Success rate percentage"
    )
    avg_duration_ms: int = Field(..., description="Average duration in ms")
    median_duration_ms: int = Field(..., description="Median duration in ms")
    p95_duration_ms: int = Field(..., description="95th percentile duration in ms")
    failure_modes: list[dict[str, int]] = Field(
        default_factory=list,
        description="Breakdown of failure types and counts",
    )


class TransitionReliability(ReliabilityStats):
    """Reliability statistics for a specific transition.

    Extends ReliabilityStats with transition-specific context.
    """

    transition_name: str = Field(..., description="Name of the transition")
    from_state: str = Field(..., description="Source state")
    to_state: str = Field(..., description="Destination state")


__all__ = [
    "ExecutionStats",
    "CoverageData",
    "ReliabilityStats",
    "TransitionReliability",
]
