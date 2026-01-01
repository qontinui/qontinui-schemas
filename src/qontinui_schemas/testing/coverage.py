"""Coverage tracking schemas.

Defines schemas for tracking workflow coverage during testing.
Coverage measures how much of the workflow has been exercised.
"""

from typing import Any
from uuid import UUID

from pydantic import BaseModel, ConfigDict, Field

from qontinui_schemas.common.stats import CoverageData
from qontinui_schemas.common.time import UTCDateTime


class CoverageSnapshot(BaseModel):
    """Point-in-time coverage snapshot.

    Captured periodically during a test run to track coverage growth.
    """

    model_config = ConfigDict(from_attributes=True)

    id: UUID = Field(..., description="Snapshot ID")
    run_id: UUID = Field(..., description="Test run ID")
    sequence_number: int = Field(..., description="Snapshot sequence number")
    coverage_percentage: float = Field(..., description="Coverage percentage")
    states_covered: int = Field(..., description="States covered")
    total_states: int = Field(..., description="Total states")
    transitions_covered: int = Field(..., description="Transitions covered")
    total_transitions: int = Field(..., description="Total transitions")
    timestamp: UTCDateTime = Field(..., description="Snapshot time (UTC)")


class CoverageUpdate(BaseModel):
    """Request schema for updating coverage metrics.

    Sent by the runner when coverage changes.
    """

    total_transitions_executed: int = Field(
        ..., ge=0, description="Total transitions executed"
    )
    unique_transitions_covered: int = Field(
        ..., ge=0, description="Unique transitions covered"
    )
    coverage_percentage: float = Field(
        ..., ge=0.0, le=100.0, description="Coverage percentage"
    )
    transition_coverage_map: dict[str, int] = Field(
        default_factory=dict, description="Transition to execution count"
    )
    state_coverage_map: dict[str, int] = Field(
        default_factory=dict, description="State to visit count"
    )
    uncovered_transitions: list[str] = Field(
        default_factory=list, description="Uncovered transition IDs"
    )


class CoverageUpdateResponse(BaseModel):
    """Response schema for coverage update."""

    run_id: UUID = Field(..., description="Test run ID")
    coverage_updated: bool = Field(..., description="Whether update succeeded")
    coverage_percentage: float = Field(..., description="Current coverage")
    unique_transitions_covered: int = Field(..., description="Unique transitions")


class CoverageTrendDataPoint(BaseModel):
    """Single data point in coverage trend."""

    date: str = Field(..., description="Date (YYYY-MM-DD)")
    runs_count: int = Field(..., description="Number of runs")
    avg_coverage_percentage: float = Field(..., description="Average coverage")
    max_coverage_percentage: float = Field(..., description="Maximum coverage")
    min_coverage_percentage: float = Field(..., description="Minimum coverage")
    total_transitions_executed: int = Field(..., description="Total transitions")
    unique_transitions_covered: int = Field(..., description="Unique transitions")


class CoverageTrendResponse(BaseModel):
    """Response schema for coverage trend analysis."""

    project_id: UUID = Field(..., description="Project ID")
    workflow_id: str | None = Field(None, description="Workflow ID if filtered")
    start_date: str = Field(..., description="Start date")
    end_date: str = Field(..., description="End date")
    granularity: str = Field(..., description="Granularity: daily, weekly, monthly")
    data_points: list[CoverageTrendDataPoint] = Field(..., description="Trend data")
    overall_stats: dict[str, Any] = Field(..., description="Overall statistics")


class CoverageGap(BaseModel):
    """A gap in coverage (uncovered state or transition)."""

    id: str = Field(..., description="State or transition ID")
    name: str = Field(..., description="Display name")
    type: str = Field(..., description="Type: 'state' or 'transition'")
    from_state: str | None = Field(None, description="Source state (for transitions)")
    to_state: str | None = Field(None, description="Target state (for transitions)")
    last_covered_at: UTCDateTime | None = Field(
        None, description="Last time covered (UTC)"
    )
    priority: str = Field(
        "medium", description="Priority: 'critical', 'high', 'medium', 'low'"
    )


class CoverageGapsResponse(BaseModel):
    """Response schema for coverage gaps analysis."""

    project_id: UUID = Field(..., description="Project ID")
    workflow_id: str = Field(..., description="Workflow ID")
    current_coverage: float = Field(..., description="Current coverage percentage")
    gaps: list[CoverageGap] = Field(..., description="Coverage gaps")
    total_gaps: int = Field(..., description="Total gap count")
    critical_gaps: int = Field(..., description="Critical gaps count")
    recommended_next: list[str] = Field(
        default_factory=list, description="Recommended transitions to cover"
    )


class CoverageHeatmapCell(BaseModel):
    """Single cell in coverage heatmap."""

    state_id: str = Field(..., description="State ID")
    state_name: str = Field(..., description="State name")
    visit_count: int = Field(..., description="Number of visits")
    last_visited_at: UTCDateTime | None = Field(
        None, description="Last visit time (UTC)"
    )
    coverage_intensity: float = Field(
        ..., ge=0.0, le=1.0, description="Coverage intensity (0-1)"
    )


class CoverageHeatmapResponse(BaseModel):
    """Response schema for coverage heatmap."""

    project_id: UUID = Field(..., description="Project ID")
    workflow_id: str = Field(..., description="Workflow ID")
    run_id: UUID | None = Field(None, description="Run ID if single-run view")
    cells: list[CoverageHeatmapCell] = Field(..., description="Heatmap cells")
    max_visits: int = Field(..., description="Maximum visit count")


# Re-export CoverageData from common
__all__ = [
    "CoverageData",
    "CoverageSnapshot",
    "CoverageUpdate",
    "CoverageUpdateResponse",
    "CoverageTrendDataPoint",
    "CoverageTrendResponse",
    "CoverageGap",
    "CoverageGapsResponse",
    "CoverageHeatmapCell",
    "CoverageHeatmapResponse",
]
