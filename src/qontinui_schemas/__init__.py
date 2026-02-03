"""Qontinui Schemas - Shared Pydantic models for Qontinui ecosystem.

This package provides schema definitions used across qontinui-web, qontinui-api,
qontinui-runner, and other Qontinui services. It has minimal dependencies
(pydantic only) to avoid pulling in heavy ML libraries.

Modules:
- accessibility: Accessibility tree capture and interaction schemas
- discovery: Unified state discovery results (from any source)
- task_run: Unified task run model (AI tasks, automation, or both)
- execution: Execution tracking (runs, actions, screenshots, issues)
- testing: Visual regression and coverage tracking
- api: External API request/response schemas (legacy, prefer execution module)
- config: Workflow and action configuration models
- events: Execution event schemas
- common: Shared utilities (time, metadata, stats)
- descriptions: Rich descriptions for AI verification agent

Usage:
    # TaskRun - the unified task execution model
    from qontinui_schemas.task_run import (
        TaskType, TaskRunStatus, TaskRunCreate, TaskRunDetail,
    )

    # Execution tracking
    from qontinui_schemas.execution import (
        RunType, RunStatus, ExecutionRunCreate, ActionExecutionCreate,
    )

    # Visual regression testing
    from qontinui_schemas.testing import (
        VisualBaselineCreate, CoverageUpdate,
    )
"""

__version__ = "0.2.0"

# Accessibility module - accessibility tree capture and interaction
from qontinui_schemas.accessibility import (  # noqa: F401
    AccessibilityActionResult,
    AccessibilityBackend,
    AccessibilityBounds,
    AccessibilityCaptureOptions,
    AccessibilityConfig,
    AccessibilityNode,
    AccessibilityRole,
    AccessibilitySelector,
    AccessibilitySnapshot,
    AccessibilityState,
)

# Re-export common metadata and stats (unique names)
from qontinui_schemas.common.metadata import (  # noqa: F401
    MatchLocation,
    RunnerMetadata,
    ScreenshotAnnotation,
    WorkflowMetadata,
)
from qontinui_schemas.common.stats import (  # noqa: F401
    CoverageData,
    ExecutionStats,
    ReliabilityStats,
    TransitionReliability,
)

# Re-export common utilities (these are unique, no conflicts)
from qontinui_schemas.common.time import (  # noqa: F401
    UTCDateTime,
    ensure_utc,
    from_iso,
    to_iso,
    to_utc,
    utc_now,
)

# Config models - use explicit imports to avoid conflicts
from qontinui_schemas.config.models.action import Action  # noqa: F401
from qontinui_schemas.config.models.base_types import (  # noqa: F401
    LogLevel,
    MouseButton,
    SearchStrategy,
    VerificationMode,
    WorkflowVisibility,
)
from qontinui_schemas.config.models.config_root import (  # noqa: F401
    ImageAsset,
    QontinuiConfig,
)
from qontinui_schemas.config.models.geometry import (  # noqa: F401
    Coordinates,
    CoordinateSystem,
    Region,
)
from qontinui_schemas.config.models.monitors import (  # noqa: F401
    Monitor,
    VirtualDesktop,
)
from qontinui_schemas.config.models.state_machine import (  # noqa: F401
    Pattern,
    SearchRegion,
    State,
    StateImage,
    StateLocation,
    StateRegion,
    StateString,
)
from qontinui_schemas.config.models.workflow import (  # noqa: F401
    Connection,
    Variables,
    Workflow,
)
from qontinui_schemas.config.models.workflow import (  # noqa: F401
    WorkflowMetadata as ConfigWorkflowMetadata,
)

# Descriptions module - rich descriptions for AI verification agent
from qontinui_schemas.descriptions import (  # noqa: F401
    ActionDescription,
    StateDescription,
    TransitionDescription,
    WorkflowDescription,
)

# Discovery module - unified state discovery results
from qontinui_schemas.discovery import (  # noqa: F401
    SOURCE_TYPE_LABELS,
    DiscoveredState,
    DiscoveredStateImage,
    DiscoveredTransition,
    DiscoveryBoundingBox,
    DiscoverySourceType,
    DiscoveryTransitionTrigger,
    StateDiscoveryResult,
    StateDiscoveryResultCreate,
    StateDiscoveryResultListResponse,
    StateDiscoveryResultSummary,
    StateDiscoveryResultUpdate,
    StateMachineExport,
    StateMachineImport,
    TransitionTriggerType,
)

# Events module - healing events for self-healing metrics
from qontinui_schemas.events.healing_events import (  # noqa: F401
    CacheMetrics,
    HealingAttemptInfo,
    HealingEvent,
    HealingEventCreate,
    HealingEventData,
    HealingEventType,
    HealingMetrics,
    HealingStatsResponse,
    HealingStrategy,
    ReliabilityInfo,
    VisualValidationInfo,
)

# Events module - tree events for execution logging
from qontinui_schemas.events.tree_events import (  # noqa: F401
    ActionType as TreeActionType,
)
from qontinui_schemas.events.tree_events import (  # noqa: F401
    MatchLocation as TreeMatchLocation,
)
from qontinui_schemas.events.tree_events import (  # noqa: F401
    NodeStatus as TreeNodeStatusEvent,
)
from qontinui_schemas.events.tree_events import (  # noqa: F401
    NodeType as TreeNodeTypeEvent,
)
from qontinui_schemas.events.tree_events import (  # noqa: F401
    Outcome,
    RuntimeData,
    StateContext,
    TimingInfo,
    TopMatch,
    TreeEvent,
    TreeEventCreate,
    TreeEventListResponse,
    TreeEventResponse,
)
from qontinui_schemas.events.tree_events import (  # noqa: F401
    TreeEventType as TreeEventTypeEvent,
)

# Execution module - the unified execution tracking schemas (NEW)
from qontinui_schemas.execution import (  # noqa: F401
    ActionExecutionBatchCreate,
    ActionExecutionCreate,
    ActionExecutionDetail,
    ActionExecutionListResponse,
    ActionExecutionResponse,
    ActionStatus,
    ActionType,
    DisplayNode,
    ErrorType,
    ExecutionIssueBatchCreate,
    ExecutionIssueCreate,
    ExecutionIssueDetail,
    ExecutionIssueListResponse,
    ExecutionIssueResponse,
    ExecutionIssueSummary,
    ExecutionIssueUpdate,
    ExecutionRunComplete,
    ExecutionRunCompleteResponse,
    ExecutionRunCreate,
    ExecutionRunDetail,
    ExecutionRunListResponse,
    ExecutionRunResponse,
    ExecutionScreenshotCreate,
    ExecutionScreenshotDetail,
    ExecutionScreenshotListResponse,
    ExecutionScreenshotResponse,
    ExecutionTreeEventBatchCreate,
    ExecutionTreeEventCreate,
    ExecutionTreeEventListResponse,
    ExecutionTreeEventResponse,
    ExecutionTreeResponse,
    IssueSeverity,
    IssueSource,
    IssueStatus,
    IssueType,
    NodeMetadata,
    NodeStatus,
    NodeType,
    PathElement,
    RunStatus,
    RunType,
    ScreenshotType,
    TreeEventType,
    TreeNode,
    TreeNodeStatus,
    TreeNodeType,
)

# TaskRun module - unified task execution model
from qontinui_schemas.task_run import (  # noqa: F401
    AutomationStatus,
    TaskRunAutomationComplete,
    TaskRunAutomationCreate,
    TaskRunAutomationDetail,
    TaskRunAutomationListResponse,
    TaskRunAutomationResponse,
    TaskRunComplete,
    TaskRunCreate,
    TaskRunDetail,
    TaskRunListResponse,
    TaskRunReopen,
    TaskRunResponse,
    TaskRunStatus,
    TaskRunSyncPayload,
    TaskRunUpdate,
    TaskType,
)

# Template Capture module - click-to-template system
from qontinui_schemas.template_capture import (  # noqa: F401
    ApplicationProfile,
    ApplicationProfileCreate,
    ApplicationProfileListResponse,
    ApplicationProfileResponse,
    ApplicationProfileUpdate,
    CandidateBoundingBox,
    CandidateStatus,
    DetectionStrategyType,
    ElementType,
    TemplateCandidateBatchCreate,
    TemplateCandidateCreate,
    TemplateCandidateDetail,
    TemplateCandidateListResponse,
    TemplateCandidateResponse,
    TemplateCandidateSummary,
    TemplateCandidateUpdate,
    TuningMetrics,
    TuningRequest,
    TuningResult,
)

# Testing module - visual regression and coverage
from qontinui_schemas.testing import (  # noqa: F401
    ComparisonReview,
    ComparisonSettings,
    ComparisonStats,
    CoverageGap,
    CoverageGapsResponse,
    CoverageHeatmapCell,
    CoverageHeatmapResponse,
    CoverageSnapshot,
    CoverageTrendDataPoint,
    CoverageTrendResponse,
    CoverageUpdate,
    CoverageUpdateResponse,
    DiffRegion,
    IgnoreRegion,
    VisualBaselineCreate,
    VisualBaselineFromScreenshot,
    VisualBaselineListResponse,
    VisualBaselineResponse,
    VisualBaselineUpdate,
    VisualComparisonCreate,
    VisualComparisonDetail,
    VisualComparisonListResponse,
    VisualComparisonResponse,
    VisualComparisonSummary,
)

__all__ = [
    # Version
    "__version__",
    # Time utilities
    "UTCDateTime",
    "utc_now",
    "to_utc",
    "from_iso",
    "to_iso",
    "ensure_utc",
    # Accessibility
    "AccessibilityRole",
    "AccessibilityBackend",
    "AccessibilityState",
    "AccessibilityBounds",
    "AccessibilityNode",
    "AccessibilitySnapshot",
    "AccessibilitySelector",
    "AccessibilityConfig",
    "AccessibilityCaptureOptions",
    "AccessibilityActionResult",
    # Discovery - unified state discovery results
    "DiscoverySourceType",
    "TransitionTriggerType",
    "DiscoveryBoundingBox",
    "DiscoveryTransitionTrigger",
    "DiscoveredStateImage",
    "DiscoveredState",
    "DiscoveredTransition",
    "StateDiscoveryResult",
    "StateDiscoveryResultSummary",
    "StateDiscoveryResultListResponse",
    "StateDiscoveryResultCreate",
    "StateDiscoveryResultUpdate",
    "StateMachineExport",
    "StateMachineImport",
    "SOURCE_TYPE_LABELS",
    # Common metadata
    "RunnerMetadata",
    "WorkflowMetadata",
    "MatchLocation",
    "ScreenshotAnnotation",
    # Common stats
    "ExecutionStats",
    "CoverageData",
    "ReliabilityStats",
    "TransitionReliability",
    # Execution enums
    "RunType",
    "RunStatus",
    "ActionType",
    "ActionStatus",
    "ErrorType",
    "IssueSeverity",
    "IssueType",
    "IssueStatus",
    "IssueSource",
    "ScreenshotType",
    "TreeNodeType",
    "TreeEventType",
    "TreeNodeStatus",
    # Execution run schemas
    "ExecutionRunCreate",
    "ExecutionRunResponse",
    "ExecutionRunDetail",
    "ExecutionRunComplete",
    "ExecutionRunCompleteResponse",
    "ExecutionRunListResponse",
    # Execution action schemas
    "ActionExecutionCreate",
    "ActionExecutionBatchCreate",
    "ActionExecutionResponse",
    "ActionExecutionDetail",
    "ActionExecutionListResponse",
    # Execution screenshot schemas
    "ExecutionScreenshotCreate",
    "ExecutionScreenshotResponse",
    "ExecutionScreenshotDetail",
    "ExecutionScreenshotListResponse",
    # Execution issue schemas
    "ExecutionIssueCreate",
    "ExecutionIssueBatchCreate",
    "ExecutionIssueResponse",
    "ExecutionIssueDetail",
    "ExecutionIssueUpdate",
    "ExecutionIssueListResponse",
    "ExecutionIssueSummary",
    # Execution tree event schemas
    "ExecutionTreeEventCreate",
    "ExecutionTreeEventBatchCreate",
    "ExecutionTreeEventResponse",
    "ExecutionTreeEventListResponse",
    "ExecutionTreeResponse",
    "TreeNode",
    "NodeType",
    "NodeStatus",
    "NodeMetadata",
    "PathElement",
    "DisplayNode",
    # Visual testing
    "IgnoreRegion",
    "ComparisonSettings",
    "VisualBaselineCreate",
    "VisualBaselineFromScreenshot",
    "VisualBaselineUpdate",
    "VisualBaselineResponse",
    "VisualBaselineListResponse",
    "DiffRegion",
    "VisualComparisonCreate",
    "VisualComparisonResponse",
    "VisualComparisonDetail",
    "VisualComparisonListResponse",
    "VisualComparisonSummary",
    "ComparisonReview",
    "ComparisonStats",
    # Coverage
    "CoverageSnapshot",
    "CoverageUpdate",
    "CoverageUpdateResponse",
    "CoverageTrendDataPoint",
    "CoverageTrendResponse",
    "CoverageGap",
    "CoverageGapsResponse",
    "CoverageHeatmapCell",
    "CoverageHeatmapResponse",
    # Descriptions - AI verification agent
    "StateDescription",
    "ActionDescription",
    "TransitionDescription",
    "WorkflowDescription",
    # TaskRun - unified task execution model
    "TaskType",
    "TaskRunStatus",
    "AutomationStatus",
    "TaskRunCreate",
    "TaskRunResponse",
    "TaskRunDetail",
    "TaskRunUpdate",
    "TaskRunComplete",
    "TaskRunReopen",
    "TaskRunListResponse",
    "TaskRunAutomationCreate",
    "TaskRunAutomationResponse",
    "TaskRunAutomationDetail",
    "TaskRunAutomationComplete",
    "TaskRunAutomationListResponse",
    "TaskRunSyncPayload",
    # Healing events - self-healing metrics
    "HealingEventType",
    "HealingStrategy",
    "CacheMetrics",
    "HealingAttemptInfo",
    "HealingMetrics",
    "ReliabilityInfo",
    "VisualValidationInfo",
    "HealingEvent",
    "HealingEventData",
    "HealingEventCreate",
    "HealingStatsResponse",
    # Template Capture - click-to-template system
    "ElementType",
    "DetectionStrategyType",
    "CandidateStatus",
    "CandidateBoundingBox",
    "TemplateCandidateCreate",
    "TemplateCandidateBatchCreate",
    "TemplateCandidateResponse",
    "TemplateCandidateDetail",
    "TemplateCandidateSummary",
    "TemplateCandidateUpdate",
    "TemplateCandidateListResponse",
    "TuningMetrics",
    "ApplicationProfileCreate",
    "ApplicationProfileResponse",
    "ApplicationProfileUpdate",
    "ApplicationProfile",
    "ApplicationProfileListResponse",
    "TuningRequest",
    "TuningResult",
]
