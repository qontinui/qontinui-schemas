"""Testing API schemas for software testing results.

These schemas define request/response models for testing endpoints,
used by:
- qontinui-web backend (FastAPI)
- qontinui-web frontend (TypeScript, via generated types)
- qontinui-runner (test result submission)

All datetime fields use UTCDateTime for consistent UTC timezone handling
and ISO 8601 format strings with 'Z' suffix for JSON serialization.
"""

from enum import Enum
from typing import Any, Generic, TypeVar
from uuid import UUID

from pydantic import BaseModel, Field, field_validator

from qontinui_schemas.common.time import UTCDateTime

# =============================================================================
# Enums
# =============================================================================


class TestRunStatus(str, Enum):
    """Status of a test run."""

    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"
    TIMEOUT = "timeout"
    CANCELLED = "cancelled"


class TransitionStatus(str, Enum):
    """Status of a transition execution."""

    SUCCESS = "success"
    FAILED = "failed"
    TIMEOUT = "timeout"
    SKIPPED = "skipped"
    ERROR = "error"


class DeficiencySeverity(str, Enum):
    """Severity level of a deficiency."""

    CRITICAL = "critical"
    HIGH = "high"
    MEDIUM = "medium"
    LOW = "low"
    INFO = "informational"


class DeficiencyStatus(str, Enum):
    """Status of a deficiency."""

    NEW = "new"
    OPEN = "open"
    IN_PROGRESS = "in_progress"
    RESOLVED = "resolved"
    CLOSED = "closed"
    WONT_FIX = "wont_fix"


class DeficiencyType(str, Enum):
    """Type of deficiency."""

    FUNCTIONAL = "functional_bug"
    VISUAL = "ui_issue"
    PERFORMANCE = "performance"
    CRASH = "crash"
    SECURITY = "security"
    ACCESSIBILITY = "accessibility"
    DATA = "data"
    OTHER = "other"


class ScreenshotType(str, Enum):
    """Type of screenshot."""

    ERROR = "error"
    SUCCESS = "success"
    MANUAL = "manual"
    PERIODIC = "periodic"
    STATE_VERIFICATION = "state_verification"
    ACTION_RESULT = "action_result"
    BEFORE_ACTION = "before_action"
    AFTER_ACTION = "after_action"


# =============================================================================
# Pagination
# =============================================================================


T = TypeVar("T")


class Pagination(BaseModel):
    """Pagination metadata."""

    total: int = Field(..., description="Total number of items")
    limit: int = Field(..., description="Items per page")
    offset: int = Field(..., description="Number of items skipped")
    has_more: bool = Field(..., description="Whether more items exist")

    @property
    def page(self) -> int:
        """Current page number (1-indexed)."""
        return (self.offset // self.limit) + 1 if self.limit > 0 else 1

    @property
    def total_pages(self) -> int:
        """Total number of pages."""
        return (self.total + self.limit - 1) // self.limit if self.limit > 0 else 1


class PaginatedResponse(BaseModel, Generic[T]):
    """Generic paginated response."""

    items: list[Any] = Field(..., description="List of items")
    pagination: Pagination = Field(..., description="Pagination metadata")


# =============================================================================
# Test Run Schemas
# =============================================================================


class TestRunCreate(BaseModel):
    """Request schema for creating a new test run."""

    project_id: UUID = Field(..., description="Project ID")
    run_name: str = Field(
        ...,
        description="Name of the test run",
        max_length=255,
    )
    description: str | None = Field(None, description="Optional description")
    runner_metadata: dict[str, Any] = Field(
        ...,
        description="Metadata about the runner environment",
    )
    workflow_metadata: dict[str, Any] = Field(
        ...,
        description="Metadata about the workflow being tested",
    )
    configuration_snapshot: dict[str, Any] = Field(
        ...,
        description="Snapshot of the test configuration",
    )


class TestRunResponse(BaseModel):
    """Response schema for test run creation and retrieval."""

    run_id: UUID = Field(..., description="Unique test run identifier")
    project_id: UUID = Field(..., description="Project ID")
    run_name: str = Field(..., description="Name of the test run")
    status: TestRunStatus = Field(..., description="Test run status")
    started_at: UTCDateTime = Field(..., description="Test run start time (UTC)")
    ended_at: UTCDateTime | None = Field(None, description="Test run end time (UTC)")
    duration_seconds: int | None = Field(None, description="Duration in seconds")
    runner_metadata: dict[str, Any] = Field(..., description="Runner metadata")
    created_at: UTCDateTime = Field(..., description="Record creation time (UTC)")


class TestRunDetail(TestRunResponse):
    """Detailed test run information."""

    description: str | None = Field(None, description="Test run description")
    workflow_metadata: dict[str, Any] = Field(..., description="Workflow metadata")
    configuration_snapshot: dict[str, Any] = Field(
        ..., description="Configuration snapshot"
    )
    final_metrics: dict[str, Any] | None = Field(None, description="Final metrics")
    coverage_data: dict[str, Any] | None = Field(None, description="Coverage data")
    updated_at: UTCDateTime | None = Field(None, description="Last update time (UTC)")
    created_by: dict[str, Any] | None = Field(None, description="User who created")
    transitions: list[dict[str, Any]] | None = Field(None, description="Transitions")
    deficiencies: list[dict[str, Any]] | None = Field(None, description="Deficiencies")
    screenshots: list[dict[str, Any]] | None = Field(None, description="Screenshots")


class TestRunListResponse(BaseModel):
    """Response schema for paginated test run list."""

    runs: list[TestRunResponse] = Field(..., description="List of test runs")
    pagination: Pagination = Field(..., description="Pagination metadata")


class TestRunComplete(BaseModel):
    """Request schema for completing a test run."""

    status: str = Field(..., description="Final status")
    ended_at: UTCDateTime = Field(..., description="End time (UTC)")
    final_metrics: dict[str, Any] = Field(..., description="Final test metrics")
    summary: str | None = Field(None, description="Optional summary text")

    @field_validator("status")
    @classmethod
    def validate_status(cls, v: str) -> str:
        allowed = ["completed", "failed", "timeout", "aborted", "crashed"]
        if v not in allowed:
            raise ValueError(f"Status must be one of: {', '.join(allowed)}")
        return v


class TestRunCompleteResponse(BaseModel):
    """Response schema for test run completion."""

    run_id: UUID = Field(..., description="Test run ID")
    status: TestRunStatus = Field(..., description="Final status")
    started_at: UTCDateTime = Field(..., description="Start time (UTC)")
    ended_at: UTCDateTime = Field(..., description="End time (UTC)")
    duration_seconds: int = Field(..., description="Duration in seconds")
    final_metrics: dict[str, Any] = Field(..., description="Final metrics")


# =============================================================================
# Transition Schemas
# =============================================================================


class TransitionCreate(BaseModel):
    """Schema for a single transition report."""

    sequence_number: int = Field(..., description="Order within test run", ge=1)
    from_state: str = Field(..., description="Source state", max_length=255)
    to_state: str = Field(..., description="Destination state", max_length=255)
    transition_name: str = Field(..., description="Transition name", max_length=255)
    status: TransitionStatus = Field(..., description="Transition status")
    started_at: UTCDateTime = Field(..., description="Transition start time (UTC)")
    completed_at: UTCDateTime = Field(
        ..., description="Transition completion time (UTC)"
    )
    duration_ms: int = Field(..., description="Duration in milliseconds", ge=0)
    error_message: str | None = Field(None, description="Error message if failed")
    error_type: str | None = Field(None, description="Error type if failed")
    screenshot_id: UUID | None = Field(None, description="Associated screenshot ID")
    metadata: dict[str, Any] = Field(
        default_factory=dict,
        description="Additional transition metadata",
    )


class TransitionBatchCreate(BaseModel):
    """Request schema for batch transition reporting."""

    transitions: list[TransitionCreate] = Field(
        ..., description="List of transitions", min_length=1, max_length=50
    )


class TransitionResponse(BaseModel):
    """Response schema for a single transition."""

    transition_id: UUID = Field(..., description="Transition ID")
    sequence_number: int = Field(..., description="Sequence number")
    from_state: str = Field(..., description="Source state")
    to_state: str = Field(..., description="Destination state")
    transition_name: str = Field(..., description="Transition name")
    status: TransitionStatus = Field(..., description="Transition status")
    duration_ms: int = Field(..., description="Duration in milliseconds")
    started_at: UTCDateTime = Field(..., description="Start time (UTC)")
    completed_at: UTCDateTime = Field(..., description="Completion time (UTC)")
    error_message: str | None = Field(None, description="Error message")
    error_type: str | None = Field(None, description="Error type")


class TransitionBatchResponse(BaseModel):
    """Response schema for batch transition creation."""

    run_id: UUID = Field(..., description="Test run ID")
    transitions_recorded: int = Field(..., description="Number recorded")
    transition_ids: list[UUID] = Field(..., description="IDs of created transitions")
    coverage_updated: dict[str, Any] = Field(..., description="Updated coverage")


# =============================================================================
# Deficiency Schemas
# =============================================================================


class DeficiencyCreate(BaseModel):
    """Schema for a single deficiency report."""

    title: str = Field(..., description="Deficiency title", max_length=500)
    description: str = Field(..., description="Detailed description")
    severity: DeficiencySeverity = Field(..., description="Severity level")
    deficiency_type: DeficiencyType = Field(..., description="Type of deficiency")
    transition_sequence_number: int | None = Field(
        None, description="Related transition sequence number"
    )
    state: str | None = Field(None, description="State where occurred", max_length=255)
    screenshot_ids: list[UUID] = Field(
        default_factory=list, description="Associated screenshot IDs"
    )
    reproduction_steps: list[str] = Field(
        default_factory=list, description="Steps to reproduce"
    )
    metadata: dict[str, Any] = Field(
        default_factory=dict, description="Additional metadata"
    )


class DeficiencyBatchCreate(BaseModel):
    """Request schema for batch deficiency reporting."""

    deficiencies: list[DeficiencyCreate] = Field(
        ..., description="List of deficiencies", min_length=1, max_length=20
    )


class DeficiencyResponse(BaseModel):
    """Response schema for a single deficiency."""

    deficiency_id: UUID = Field(..., description="Deficiency ID")
    run_id: UUID = Field(..., description="Test run ID")
    title: str = Field(..., description="Deficiency title")
    description: str = Field(..., description="Deficiency description")
    severity: DeficiencySeverity = Field(..., description="Severity level")
    status: DeficiencyStatus = Field(..., description="Deficiency status")
    deficiency_type: DeficiencyType = Field(..., description="Deficiency type")
    state: str | None = Field(None, description="State where occurred")
    transition_sequence_number: int | None = Field(
        None, description="Related transition"
    )
    screenshot_count: int | None = Field(None, description="Number of screenshots")
    created_at: UTCDateTime = Field(..., description="Creation time (UTC)")
    updated_at: UTCDateTime = Field(..., description="Last update time (UTC)")
    run_info: dict[str, Any] | None = Field(None, description="Related run info")


class DeficiencyDetail(DeficiencyResponse):
    """Detailed deficiency information."""

    reproduction_steps: list[str] = Field(
        default_factory=list, description="Reproduction steps"
    )
    screenshots: list[Any] = Field(
        default_factory=list, description="Associated screenshots"
    )
    metadata: dict[str, Any] = Field(
        default_factory=dict, description="Additional metadata"
    )
    assigned_to: dict[str, Any] | None = Field(None, description="Assigned user")
    resolution_notes: str | None = Field(None, description="Resolution notes")
    comments: list[Any] = Field(default_factory=list, description="Comments")


class DeficiencyUpdate(BaseModel):
    """Request schema for updating a deficiency."""

    status: DeficiencyStatus | None = Field(None, description="New status")
    severity: DeficiencySeverity | None = Field(None, description="New severity")
    assigned_to_user_id: UUID | None = Field(None, description="Assign to user")
    resolution_notes: str | None = Field(None, description="Resolution notes")


class DeficiencyListResponse(BaseModel):
    """Response schema for paginated deficiency list."""

    deficiencies: list[DeficiencyResponse] = Field(
        ..., description="List of deficiencies"
    )
    pagination: Pagination = Field(..., description="Pagination metadata")
    summary: dict[str, Any] = Field(..., description="Summary statistics")


class DeficiencyBatchResponse(BaseModel):
    """Response schema for batch deficiency creation."""

    run_id: UUID = Field(..., description="Test run ID")
    deficiencies_recorded: int = Field(..., description="Number recorded")
    deficiency_ids: list[UUID] = Field(..., description="IDs of created deficiencies")


# =============================================================================
# Coverage Schemas
# =============================================================================


class CoverageUpdate(BaseModel):
    """Request schema for updating coverage metrics."""

    total_transitions_executed: int = Field(..., description="Total executed", ge=0)
    unique_transitions_covered: int = Field(..., description="Unique covered", ge=0)
    coverage_percentage: float = Field(..., description="Coverage %", ge=0.0, le=100.0)
    transition_coverage_map: dict[str, int] = Field(
        default_factory=dict,
        description="Map of transition names to execution counts",
    )
    state_coverage_map: dict[str, int] = Field(
        default_factory=dict,
        description="Map of state names to visit counts",
    )
    uncovered_transitions: list[str] = Field(
        default_factory=list,
        description="List of uncovered transitions",
    )


class CoverageUpdateResponse(BaseModel):
    """Response schema for coverage update."""

    run_id: UUID = Field(..., description="Test run ID")
    coverage_updated: bool = Field(..., description="Whether update succeeded")
    coverage_percentage: float = Field(..., description="Current coverage %")
    unique_transitions_covered: int = Field(
        ..., description="Unique transitions covered"
    )


# =============================================================================
# Screenshot Schemas
# =============================================================================


class ScreenshotMetadata(BaseModel):
    """Metadata for screenshot upload."""

    screenshot_id: UUID = Field(..., description="Screenshot ID (client-generated)")
    sequence_number: int = Field(..., description="Screenshot sequence number", ge=1)
    transition_sequence_number: int | None = Field(
        None, description="Associated transition sequence number"
    )
    state: str | None = Field(None, description="State when taken", max_length=255)
    screenshot_type: ScreenshotType = Field(..., description="Screenshot type")
    timestamp: UTCDateTime = Field(..., description="Screenshot timestamp (UTC)")
    width: int = Field(..., description="Image width", ge=1)
    height: int = Field(..., description="Image height", ge=1)
    metadata: dict[str, Any] = Field(
        default_factory=dict, description="Additional metadata"
    )


class VisualComparisonSummary(BaseModel):
    """Summary of visual comparison result."""

    comparison_id: UUID = Field(..., description="Visual comparison result ID")
    baseline_id: UUID | None = Field(None, description="Baseline ID compared against")
    similarity_score: float = Field(..., description="Similarity score (0.0-1.0)")
    threshold: float = Field(..., description="Threshold used")
    passed: bool = Field(..., description="Whether comparison passed")
    status: str = Field(..., description="Comparison status")
    diff_image_url: str | None = Field(None, description="Diff image URL")
    diff_region_count: int = Field(0, description="Number of diff regions")


class ScreenshotUploadResponse(BaseModel):
    """Response schema for screenshot upload."""

    screenshot_id: UUID = Field(..., description="Screenshot ID")
    run_id: UUID = Field(..., description="Test run ID")
    image_url: str = Field(..., description="Full image URL")
    thumbnail_url: str | None = Field(None, description="Thumbnail URL")
    uploaded_at: UTCDateTime = Field(..., description="Upload time (UTC)")
    file_size_bytes: int = Field(..., description="File size in bytes")
    state_name: str | None = Field(None, description="State name")
    visual_comparison: VisualComparisonSummary | None = Field(
        None, description="Visual comparison result"
    )


# =============================================================================
# Analytics Schemas
# =============================================================================


class CoverageTrendDataPoint(BaseModel):
    """Single data point in coverage trend."""

    date: str = Field(..., description="Date (YYYY-MM-DD)")
    runs_count: int = Field(..., description="Number of runs on this date")
    avg_coverage_percentage: float = Field(..., description="Average coverage %")
    max_coverage_percentage: float = Field(..., description="Maximum coverage %")
    min_coverage_percentage: float = Field(..., description="Minimum coverage %")
    total_transitions_executed: int = Field(..., description="Total transitions")
    unique_transitions_covered: int = Field(..., description="Unique transitions")


class CoverageTrendResponse(BaseModel):
    """Response schema for coverage trends."""

    project_id: UUID = Field(..., description="Project ID")
    start_date: str = Field(..., description="Start date")
    end_date: str = Field(..., description="End date")
    granularity: str = Field(..., description="Granularity")
    data_points: list[CoverageTrendDataPoint] = Field(..., description="Trend data")
    overall_stats: dict[str, Any] = Field(..., description="Overall statistics")


class TransitionReliabilityStats(BaseModel):
    """Statistics for a single transition."""

    transition_name: str = Field(..., description="Transition name")
    from_state: str = Field(..., description="Source state")
    to_state: str = Field(..., description="Destination state")
    total_executions: int = Field(..., description="Total executions")
    successful_executions: int = Field(..., description="Successful executions")
    failed_executions: int = Field(..., description="Failed executions")
    success_rate: float = Field(..., description="Success rate %")
    avg_duration_ms: int = Field(..., description="Average duration ms")
    median_duration_ms: int = Field(..., description="Median duration ms")
    p95_duration_ms: int = Field(..., description="95th percentile duration")
    failure_modes: list[dict[str, Any]] = Field(
        default_factory=list, description="Failure mode breakdown"
    )


class ReliabilityResponse(BaseModel):
    """Response schema for transition reliability statistics."""

    workflow_id: str = Field(..., description="Workflow ID")
    workflow_name: str | None = Field(None, description="Workflow name")
    project_id: UUID = Field(..., description="Project ID")
    date_range: dict[str, str] = Field(..., description="Date range")
    transition_stats: list[TransitionReliabilityStats] = Field(
        ..., description="Transition statistics"
    )
    overall_reliability: dict[str, Any] = Field(..., description="Overall metrics")


# =============================================================================
# Historical Data Schemas (Config Testing / Mock Mode)
# =============================================================================


class HistoricalResultRequest(BaseModel):
    """Request for random historical result (integration testing)."""

    pattern_id: str | None = Field(None, description="Filter by pattern ID")
    action_type: str | None = Field(
        None, description="Filter by action type (FIND, CLICK, etc.)"
    )
    active_states: list[str] | None = Field(
        None, description="Filter by active states (any match)"
    )
    success_only: bool = Field(True, description="Only return successful results")
    workflow_id: int | None = Field(None, description="Filter by workflow ID")
    project_id: UUID | None = Field(None, description="Filter by project ID")


class HistoricalResultResponse(BaseModel):
    """Response for a historical result."""

    id: int = Field(..., description="Historical result ID")
    pattern_id: str | None = Field(None, description="Pattern ID")
    pattern_name: str | None = Field(None, description="Pattern name")
    action_type: str = Field(..., description="Action type")
    active_states: list[str] | None = Field(None, description="Active states")
    success: bool = Field(..., description="Whether action succeeded")
    match_count: int | None = Field(None, description="Number of matches")
    best_match_score: float | None = Field(None, description="Best match score")
    match_x: int | None = Field(None, description="Match X coordinate")
    match_y: int | None = Field(None, description="Match Y coordinate")
    match_width: int | None = Field(None, description="Match width")
    match_height: int | None = Field(None, description="Match height")
    frame_timestamp_ms: int | None = Field(None, description="Frame timestamp")
    has_frame: bool = Field(False, description="Whether frame is available")


class ActionDataCreate(BaseModel):
    """Action data submitted by runner for historical indexing."""

    action_id: str = Field(..., description="Unique action ID")
    action_type: str = Field(..., description="Action type (FIND, CLICK, TYPE, etc.)")
    success: bool = Field(..., description="Whether action succeeded")
    pattern_id: str | None = Field(None, description="Pattern ID if applicable")
    pattern_name: str | None = Field(None, description="Pattern name if applicable")
    active_states: list[str] = Field(
        default_factory=list, description="Active states during action"
    )
    match_count: int | None = Field(None, description="Number of matches found")
    best_match_score: float | None = Field(None, description="Best match confidence")
    match_x: int | None = Field(None, description="Match X coordinate")
    match_y: int | None = Field(None, description="Match Y coordinate")
    match_width: int | None = Field(None, description="Match width")
    match_height: int | None = Field(None, description="Match height")
    duration_ms: int | None = Field(None, description="Action duration in ms")
    result_data: dict[str, Any] = Field(
        default_factory=dict, description="Additional result data"
    )


class ActionDataBatch(BaseModel):
    """Batch of action data from runner."""

    run_id: UUID = Field(..., description="Test run or execution ID")
    project_id: UUID = Field(..., description="Project ID")
    workflow_id: int | None = Field(None, description="Workflow ID if applicable")
    actions: list[ActionDataCreate] = Field(
        ..., description="List of action data", min_length=1, max_length=100
    )


class ActionDataBatchResponse(BaseModel):
    """Response for batch action data submission."""

    indexed: int = Field(..., description="Number of actions indexed")
    run_id: UUID = Field(..., description="Test run ID")


class HistoricalFrameResponse(BaseModel):
    """Response with frame data for playback."""

    historical_result_id: int = Field(..., description="Historical result ID")
    action_type: str = Field(..., description="Action type")
    pattern_id: str | None = Field(None, description="Pattern ID")
    pattern_name: str | None = Field(None, description="Pattern name")
    success: bool = Field(..., description="Whether action succeeded")
    match_x: int | None = Field(None, description="Match X coordinate")
    match_y: int | None = Field(None, description="Match Y coordinate")
    match_width: int | None = Field(None, description="Match width")
    match_height: int | None = Field(None, description="Match height")
    timestamp_ms: int | None = Field(None, description="Timestamp in ms")
    frame_base64: str | None = Field(None, description="Base64 encoded JPEG frame")
    has_frame: bool = Field(False, description="Whether frame is available")


class PlaybackRequest(BaseModel):
    """Request for integration test playback frames."""

    historical_result_ids: list[int] = Field(
        ..., description="List of historical result IDs in order"
    )
