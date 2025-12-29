"""Unified Execution API schemas.

This module provides a consolidated schema for ALL automation execution data:
- QA Testing
- Integration Testing
- Live Automation
- Recording/Playback

Replaces the fragmented systems:
- software_test_runs + automation_sessions → execution_runs
- transition_executions + automation_logs → action_executions
- test_screenshots + automation_screenshots → execution_screenshots
- test_deficiencies + detected_issues → execution_issues

Used by:
- qontinui-web backend (FastAPI)
- qontinui-web frontend (TypeScript, via generated types)
- qontinui-runner (execution reporting)
"""

from enum import Enum
from typing import Any
from uuid import UUID

from pydantic import BaseModel, Field

from qontinui_schemas.common.time import UTCDateTime

# =============================================================================
# Enums
# =============================================================================


class RunType(str, Enum):
    """Type of execution run."""

    QA_TEST = "qa_test"  # Structured QA testing with coverage tracking
    INTEGRATION_TEST = "integration_test"  # Integration/mock testing
    LIVE_AUTOMATION = "live_automation"  # Live automation execution
    RECORDING = "recording"  # Recording session for playback
    DEBUG = "debug"  # Debug/development run


class RunStatus(str, Enum):
    """Status of an execution run."""

    PENDING = "pending"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"
    TIMEOUT = "timeout"
    CANCELLED = "cancelled"
    PAUSED = "paused"


class ActionStatus(str, Enum):
    """Status of an action execution."""

    SUCCESS = "success"
    FAILED = "failed"
    TIMEOUT = "timeout"
    SKIPPED = "skipped"
    ERROR = "error"
    PENDING = "pending"


class ActionType(str, Enum):
    """Type of action executed."""

    # Vision actions
    FIND = "find"
    FIND_ALL = "find_all"
    EXISTS = "exists"
    VANISH = "vanish"
    WAIT = "wait"
    # Mouse actions
    CLICK = "click"
    DOUBLE_CLICK = "double_click"
    RIGHT_CLICK = "right_click"
    DRAG = "drag"
    SCROLL = "scroll"
    MOUSE_MOVE = "mouse_move"
    # Keyboard actions
    TYPE = "type"
    KEY_PRESS = "key_press"
    HOTKEY = "hotkey"
    # State actions
    GO_TO_STATE = "go_to_state"
    TRANSITION = "transition"
    # Control flow
    IF = "if"
    LOOP = "loop"
    SWITCH = "switch"
    TRY_CATCH = "try_catch"
    # Code execution
    CODE_BLOCK = "code_block"
    SHELL = "shell"
    AI_PROMPT = "ai_prompt"
    # Other
    SCREENSHOT = "screenshot"
    LOG = "log"
    CUSTOM = "custom"


class ScreenshotType(str, Enum):
    """Type of screenshot captured."""

    STATE_VERIFICATION = "state_verification"
    BEFORE_ACTION = "before_action"
    AFTER_ACTION = "after_action"
    ON_ERROR = "on_error"
    ON_SUCCESS = "on_success"
    MANUAL = "manual"
    PERIODIC = "periodic"


class IssueSeverity(str, Enum):
    """Severity level of an issue."""

    CRITICAL = "critical"
    HIGH = "high"
    MEDIUM = "medium"
    LOW = "low"
    INFO = "info"


class IssueStatus(str, Enum):
    """Status of an issue."""

    NEW = "new"
    OPEN = "open"
    IN_PROGRESS = "in_progress"
    RESOLVED = "resolved"
    CLOSED = "closed"
    WONT_FIX = "wont_fix"


class IssueType(str, Enum):
    """Type of issue detected."""

    FUNCTIONAL = "functional"
    VISUAL = "visual"
    PERFORMANCE = "performance"
    CRASH = "crash"
    TIMEOUT = "timeout"
    ASSERTION = "assertion"
    STATE_MISMATCH = "state_mismatch"
    ELEMENT_NOT_FOUND = "element_not_found"
    AI_DETECTED = "ai_detected"
    OTHER = "other"


class IssueSource(str, Enum):
    """Source that detected the issue."""

    AUTOMATION = "automation"  # Detected during automation execution
    AI_ANALYSIS = "ai_analysis"  # Detected by AI log/screenshot analysis
    VISUAL_REGRESSION = "visual_regression"  # Detected by visual comparison
    USER_REPORTED = "user_reported"  # Manually reported by user


# =============================================================================
# Runner Metadata
# =============================================================================


class RunnerMetadata(BaseModel):
    """Metadata about the runner environment."""

    runner_version: str = Field(..., description="Runner version")
    os: str = Field(..., description="Operating system")
    hostname: str = Field(..., description="Machine hostname")
    screen_resolution: str | None = Field(None, description="Screen resolution")
    python_version: str | None = Field(None, description="Python version if applicable")
    extra: dict[str, Any] = Field(
        default_factory=dict, description="Additional metadata"
    )


class ExecutionWorkflowMetadata(BaseModel):
    """Metadata about the workflow being executed."""

    workflow_id: str = Field(..., description="Workflow identifier")
    workflow_name: str = Field(..., description="Workflow name")
    workflow_version: str | None = Field(None, description="Workflow version")
    total_states: int = Field(0, description="Total states in workflow")
    total_transitions: int = Field(0, description="Total transitions in workflow")
    tags: list[str] = Field(default_factory=list, description="Workflow tags")
    initial_state_ids: list[str] = Field(
        default_factory=list,
        description="Initial active states when workflow starts (resolved from config)",
    )


# =============================================================================
# Execution Run Schemas
# =============================================================================


class ExecutionRunCreate(BaseModel):
    """Request to create a new execution run."""

    project_id: UUID = Field(..., description="Project ID")
    run_type: RunType = Field(..., description="Type of execution run")
    run_name: str = Field(..., description="Name of the run", max_length=255)
    description: str | None = Field(None, description="Optional description")
    runner_metadata: RunnerMetadata = Field(..., description="Runner environment info")
    workflow_metadata: ExecutionWorkflowMetadata | None = Field(
        None, description="Workflow info if applicable"
    )
    configuration: dict[str, Any] = Field(
        default_factory=dict, description="Execution configuration snapshot"
    )
    max_duration_seconds: int | None = Field(
        None, description="Maximum run duration in seconds"
    )


class ExecutionRunResponse(BaseModel):
    """Response for execution run creation/retrieval."""

    id: UUID = Field(..., description="Execution run ID")
    project_id: UUID = Field(..., description="Project ID")
    run_type: RunType = Field(..., description="Type of execution run")
    run_name: str = Field(..., description="Name of the run")
    status: RunStatus = Field(..., description="Current status")
    started_at: UTCDateTime = Field(..., description="Start time (UTC)")
    ended_at: UTCDateTime | None = Field(None, description="End time (UTC)")
    duration_seconds: int | None = Field(None, description="Duration in seconds")
    runner_metadata: RunnerMetadata = Field(..., description="Runner environment info")
    workflow_metadata: ExecutionWorkflowMetadata | None = Field(
        None, description="Workflow info"
    )
    created_at: UTCDateTime = Field(..., description="Record creation time (UTC)")


class ExecutionRunDetail(ExecutionRunResponse):
    """Detailed execution run information."""

    description: str | None = Field(None, description="Run description")
    configuration: dict[str, Any] = Field(..., description="Configuration snapshot")
    stats: "ExecutionStats" = Field(..., description="Execution statistics")
    coverage: "CoverageData | None" = Field(
        None, description="Coverage data if applicable"
    )
    updated_at: UTCDateTime | None = Field(None, description="Last update time (UTC)")


class ExecutionStats(BaseModel):
    """Aggregate statistics for an execution run."""

    total_actions: int = Field(0, description="Total actions executed")
    successful_actions: int = Field(0, description="Successful actions")
    failed_actions: int = Field(0, description="Failed actions")
    skipped_actions: int = Field(0, description="Skipped actions")
    timeout_actions: int = Field(0, description="Timed out actions")
    total_screenshots: int = Field(0, description="Screenshots captured")
    total_issues: int = Field(0, description="Issues detected")
    unique_states_visited: int = Field(0, description="Unique states visited")
    unique_actions_executed: int = Field(0, description="Unique action types executed")


class CoverageData(BaseModel):
    """Coverage metrics for an execution run."""

    coverage_percentage: float = Field(0.0, description="Overall coverage %")
    states_covered: int = Field(0, description="States covered")
    states_total: int = Field(0, description="Total states")
    transitions_covered: int = Field(0, description="Transitions covered")
    transitions_total: int = Field(0, description="Total transitions")
    state_coverage_map: dict[str, int] = Field(
        default_factory=dict, description="State visit counts"
    )
    transition_coverage_map: dict[str, int] = Field(
        default_factory=dict, description="Transition execution counts"
    )
    uncovered_transitions: list[str] = Field(
        default_factory=list, description="Uncovered transitions"
    )


class ExecutionRunComplete(BaseModel):
    """Request to complete an execution run."""

    status: RunStatus = Field(..., description="Final status")
    ended_at: UTCDateTime = Field(..., description="End time (UTC)")
    stats: ExecutionStats = Field(..., description="Final statistics")
    coverage: CoverageData | None = Field(None, description="Final coverage data")
    summary: str | None = Field(None, description="Optional summary text")
    error_message: str | None = Field(None, description="Error message if failed")


class ExecutionRunCompleteResponse(BaseModel):
    """Response for execution run completion."""

    id: UUID = Field(..., description="Execution run ID")
    status: RunStatus = Field(..., description="Final status")
    started_at: UTCDateTime = Field(..., description="Start time (UTC)")
    ended_at: UTCDateTime = Field(..., description="End time (UTC)")
    duration_seconds: int = Field(..., description="Duration in seconds")
    stats: ExecutionStats = Field(..., description="Final statistics")


# =============================================================================
# Action Execution Schemas
# =============================================================================


class ActionExecutionCreate(BaseModel):
    """Report a single action execution."""

    sequence_number: int = Field(..., description="Sequence within run", ge=1)
    action_type: ActionType = Field(..., description="Type of action")
    action_name: str = Field(..., description="Action name/identifier", max_length=255)
    status: ActionStatus = Field(..., description="Execution status")
    started_at: UTCDateTime = Field(..., description="Start time (UTC)")
    completed_at: UTCDateTime = Field(..., description="Completion time (UTC)")
    duration_ms: int = Field(..., description="Duration in milliseconds", ge=0)
    from_state: str | None = Field(None, description="Source state", max_length=255)
    to_state: str | None = Field(None, description="Target state", max_length=255)
    actual_state: str | None = Field(None, description="Actual state after action")
    input_data: dict[str, Any] = Field(
        default_factory=dict, description="Action input parameters"
    )
    output_data: dict[str, Any] = Field(
        default_factory=dict, description="Action output/result data"
    )
    error_message: str | None = Field(None, description="Error message if failed")
    error_type: str | None = Field(None, description="Error type if failed")
    screenshot_id: UUID | None = Field(None, description="Associated screenshot ID")
    metadata: dict[str, Any] = Field(
        default_factory=dict, description="Additional metadata"
    )


class ActionExecutionBatch(BaseModel):
    """Batch of action executions to report."""

    actions: list[ActionExecutionCreate] = Field(
        ..., description="List of actions", min_length=1, max_length=100
    )


class ActionExecutionResponse(BaseModel):
    """Response for a single action execution."""

    id: UUID = Field(..., description="Action execution ID")
    run_id: UUID = Field(..., description="Execution run ID")
    sequence_number: int = Field(..., description="Sequence number")
    action_type: ActionType = Field(..., description="Action type")
    action_name: str = Field(..., description="Action name")
    status: ActionStatus = Field(..., description="Status")
    started_at: UTCDateTime = Field(..., description="Start time (UTC)")
    completed_at: UTCDateTime = Field(..., description="Completion time (UTC)")
    duration_ms: int = Field(..., description="Duration in ms")
    from_state: str | None = Field(None, description="Source state")
    to_state: str | None = Field(None, description="Target state")
    error_message: str | None = Field(None, description="Error message")


class ActionExecutionBatchResponse(BaseModel):
    """Response for batch action execution creation."""

    run_id: UUID = Field(..., description="Execution run ID")
    actions_recorded: int = Field(..., description="Number of actions recorded")
    action_ids: list[UUID] = Field(..., description="IDs of created action records")


# =============================================================================
# Execution Screenshot Schemas
# =============================================================================


class ExecutionScreenshotCreate(BaseModel):
    """Metadata for screenshot upload."""

    screenshot_id: UUID = Field(..., description="Client-generated screenshot ID")
    sequence_number: int = Field(..., description="Sequence within run", ge=1)
    screenshot_type: ScreenshotType = Field(..., description="Screenshot type")
    action_sequence_number: int | None = Field(
        None, description="Associated action sequence number"
    )
    state_name: str | None = Field(
        None, description="State when captured", max_length=255
    )
    captured_at: UTCDateTime = Field(..., description="Capture timestamp (UTC)")
    width: int = Field(..., description="Image width in pixels", ge=1)
    height: int = Field(..., description="Image height in pixels", ge=1)
    metadata: dict[str, Any] = Field(
        default_factory=dict, description="Additional metadata"
    )


class VisualComparisonResult(BaseModel):
    """Result of visual comparison against baseline."""

    comparison_id: UUID = Field(..., description="Comparison result ID")
    baseline_id: UUID | None = Field(None, description="Baseline screenshot ID")
    similarity_score: float = Field(..., description="Similarity score (0.0-1.0)")
    threshold: float = Field(..., description="Threshold used")
    passed: bool = Field(..., description="Whether comparison passed")
    diff_image_url: str | None = Field(None, description="Diff image URL")
    diff_region_count: int = Field(0, description="Number of diff regions")


class ExecutionScreenshotResponse(BaseModel):
    """Response for screenshot upload."""

    id: UUID = Field(..., description="Screenshot ID")
    run_id: UUID = Field(..., description="Execution run ID")
    sequence_number: int = Field(..., description="Sequence number")
    screenshot_type: ScreenshotType = Field(..., description="Screenshot type")
    image_url: str = Field(..., description="Full image URL")
    thumbnail_url: str | None = Field(None, description="Thumbnail URL")
    state_name: str | None = Field(None, description="State name")
    captured_at: UTCDateTime = Field(..., description="Capture time (UTC)")
    file_size_bytes: int = Field(..., description="File size in bytes")
    visual_comparison: VisualComparisonResult | None = Field(
        None, description="Visual comparison result if baseline exists"
    )


# =============================================================================
# Execution Issue Schemas
# =============================================================================


class ExecutionIssueCreate(BaseModel):
    """Report an issue detected during execution."""

    issue_type: IssueType = Field(..., description="Type of issue")
    severity: IssueSeverity = Field(..., description="Severity level")
    source: IssueSource = Field(..., description="Source that detected the issue")
    title: str = Field(..., description="Issue title", max_length=500)
    description: str = Field(..., description="Detailed description")
    action_sequence_number: int | None = Field(
        None, description="Related action sequence number"
    )
    state_name: str | None = Field(None, description="State where issue occurred")
    screenshot_ids: list[UUID] = Field(
        default_factory=list, description="Associated screenshot IDs"
    )
    reproduction_steps: list[str] = Field(
        default_factory=list, description="Steps to reproduce"
    )
    error_details: dict[str, Any] = Field(
        default_factory=dict, description="Error details (stack trace, etc.)"
    )
    metadata: dict[str, Any] = Field(
        default_factory=dict, description="Additional metadata"
    )


class ExecutionIssueBatch(BaseModel):
    """Batch of issues to report."""

    issues: list[ExecutionIssueCreate] = Field(
        ..., description="List of issues", min_length=1, max_length=50
    )


class ExecutionIssueResponse(BaseModel):
    """Response for a single issue."""

    id: UUID = Field(..., description="Issue ID")
    run_id: UUID = Field(..., description="Execution run ID")
    issue_type: IssueType = Field(..., description="Issue type")
    severity: IssueSeverity = Field(..., description="Severity")
    status: IssueStatus = Field(..., description="Current status")
    source: IssueSource = Field(..., description="Detection source")
    title: str = Field(..., description="Issue title")
    description: str = Field(..., description="Description")
    state_name: str | None = Field(None, description="State where occurred")
    screenshot_count: int = Field(0, description="Number of screenshots")
    created_at: UTCDateTime = Field(..., description="Creation time (UTC)")
    updated_at: UTCDateTime = Field(..., description="Last update time (UTC)")


class ExecutionIssueDetail(ExecutionIssueResponse):
    """Detailed issue information."""

    action_sequence_number: int | None = Field(None, description="Related action")
    reproduction_steps: list[str] = Field(
        default_factory=list, description="Reproduction steps"
    )
    screenshots: list[ExecutionScreenshotResponse] = Field(
        default_factory=list, description="Associated screenshots"
    )
    error_details: dict[str, Any] = Field(
        default_factory=dict, description="Error details"
    )
    metadata: dict[str, Any] = Field(
        default_factory=dict, description="Additional metadata"
    )
    assigned_to: dict[str, Any] | None = Field(None, description="Assigned user")
    resolution_notes: str | None = Field(None, description="Resolution notes")


class ExecutionIssueUpdate(BaseModel):
    """Update an issue."""

    status: IssueStatus | None = Field(None, description="New status")
    severity: IssueSeverity | None = Field(None, description="New severity")
    assigned_to_user_id: UUID | None = Field(None, description="Assign to user")
    resolution_notes: str | None = Field(None, description="Resolution notes")


class ExecutionIssueBatchResponse(BaseModel):
    """Response for batch issue creation."""

    run_id: UUID = Field(..., description="Execution run ID")
    issues_recorded: int = Field(..., description="Number of issues recorded")
    issue_ids: list[UUID] = Field(..., description="IDs of created issues")


# =============================================================================
# List/Query Response Schemas
# =============================================================================


class Pagination(BaseModel):
    """Pagination metadata."""

    total: int = Field(..., description="Total items")
    limit: int = Field(..., description="Items per page")
    offset: int = Field(..., description="Items skipped")
    has_more: bool = Field(..., description="More items exist")


class ExecutionRunListResponse(BaseModel):
    """Paginated list of execution runs."""

    runs: list[ExecutionRunResponse] = Field(..., description="List of runs")
    pagination: Pagination = Field(..., description="Pagination info")


class ActionExecutionListResponse(BaseModel):
    """Paginated list of action executions."""

    actions: list[ActionExecutionResponse] = Field(..., description="List of actions")
    pagination: Pagination = Field(..., description="Pagination info")


class ExecutionIssueListResponse(BaseModel):
    """Paginated list of issues."""

    issues: list[ExecutionIssueResponse] = Field(..., description="List of issues")
    pagination: Pagination = Field(..., description="Pagination info")
    summary: dict[str, Any] = Field(
        default_factory=dict, description="Summary by severity/status"
    )


# =============================================================================
# Analytics Schemas
# =============================================================================


class ActionReliabilityStats(BaseModel):
    """Reliability statistics for an action type."""

    action_name: str = Field(..., description="Action name")
    action_type: ActionType = Field(..., description="Action type")
    total_executions: int = Field(..., description="Total executions")
    successful_executions: int = Field(..., description="Successful")
    failed_executions: int = Field(..., description="Failed")
    success_rate: float = Field(..., description="Success rate %")
    avg_duration_ms: int = Field(..., description="Average duration")
    p50_duration_ms: int = Field(..., description="Median duration")
    p95_duration_ms: int = Field(..., description="95th percentile duration")
    common_errors: list[dict[str, Any]] = Field(
        default_factory=list, description="Common error types"
    )


class ExecutionTrendDataPoint(BaseModel):
    """Single data point in execution trend."""

    date: str = Field(..., description="Date (YYYY-MM-DD)")
    runs_count: int = Field(..., description="Number of runs")
    success_rate: float = Field(..., description="Success rate %")
    avg_duration_seconds: int = Field(..., description="Average duration")
    total_actions: int = Field(..., description="Total actions executed")
    issues_count: int = Field(..., description="Issues detected")


class ExecutionTrendResponse(BaseModel):
    """Response for execution trends."""

    project_id: UUID = Field(..., description="Project ID")
    run_type: RunType | None = Field(None, description="Filtered run type")
    start_date: str = Field(..., description="Start date")
    end_date: str = Field(..., description="End date")
    granularity: str = Field(..., description="Granularity (daily/weekly/monthly)")
    data_points: list[ExecutionTrendDataPoint] = Field(..., description="Trend data")
    overall_stats: dict[str, Any] = Field(..., description="Overall statistics")


# =============================================================================
# Historical Playback Schemas (for integration testing)
# =============================================================================


class HistoricalActionQuery(BaseModel):
    """Query for historical action results."""

    action_type: ActionType | None = Field(None, description="Filter by action type")
    action_name: str | None = Field(None, description="Filter by action name")
    state_name: str | None = Field(None, description="Filter by state")
    success_only: bool = Field(True, description="Only successful actions")
    project_id: UUID | None = Field(None, description="Filter by project")
    workflow_id: str | None = Field(None, description="Filter by workflow")
    limit: int = Field(10, description="Maximum results", ge=1, le=100)


class HistoricalActionResult(BaseModel):
    """Historical action execution for playback."""

    id: UUID = Field(..., description="Action execution ID")
    action_type: ActionType = Field(..., description="Action type")
    action_name: str = Field(..., description="Action name")
    status: ActionStatus = Field(..., description="Status")
    from_state: str | None = Field(None, description="Source state")
    to_state: str | None = Field(None, description="Target state")
    input_data: dict[str, Any] = Field(..., description="Input data")
    output_data: dict[str, Any] = Field(..., description="Output data")
    duration_ms: int = Field(..., description="Duration")
    screenshot_url: str | None = Field(None, description="Screenshot URL if available")
    has_screenshot: bool = Field(False, description="Whether screenshot exists")


class PlaybackFrameRequest(BaseModel):
    """Request frames for playback."""

    action_ids: list[UUID] = Field(
        ..., description="Action execution IDs in order", min_length=1, max_length=100
    )
    include_screenshots: bool = Field(True, description="Include screenshot URLs")
