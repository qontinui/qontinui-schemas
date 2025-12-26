"""
Qontinui action and workflow configuration models.

This package provides Pydantic models for type-safe action and workflow
configuration, organized by domain for maintainability.
"""

# Base types and enums
# Core action model
from .action import ACTION_CONFIG_MAP, Action, get_typed_config

# AI prompt action configs
from .ai_prompts import (
    AIPromptActionConfig,
    AIPromptTemplate,
    PromptParameter,
    PromptSequence,
    PromptSequenceStep,
    RunPromptSequenceActionConfig,
)
from .base_types import (
    LogLevel,
    MouseButton,
    SearchStrategy,
    VerificationMode,
    WorkflowVisibility,
)

# Code execution configs
from .code_actions import (
    CodeBlockActionConfig,
    CustomFunctionActionConfig,
    ErrorHandling,
)

# Root configuration models
from .config_root import (
    CheckMode,
    ColorSpace,
    CompatibleVersions,
    ConfigMetadata,
    ConfigSettings,
    ExecutionRecord,
    ExecutionSettings,
    FailureStrategy,
    FindActionSettings,
    ImageAsset,
    ImageFormat,
    ImageSource,
    KeyboardActionSettings,
    LoggingSettings,
)
from .config_root import LogLevel as ConfigLogLevel
from .config_root import (
    MouseActionSettings,
    PerformanceSettings,
    QontinuiConfig,
    RecognitionSettings,
    Resolution,
    Schedule,
    ScheduleType,
    SearchAlgorithm,
    TriggerType,
    WaitActionSettings,
)

# Control flow configs
from .control_flow import (
    BreakActionConfig,
    ConditionConfig,
    ContinueActionConfig,
    IfActionConfig,
    LoopActionConfig,
    LoopCollection,
    SwitchActionConfig,
    SwitchCase,
    TryCatchActionConfig,
)

# Data operation configs
from .data_operations import (
    FilterActionConfig,
    FilterCondition,
    GetVariableActionConfig,
    MapActionConfig,
    MapTransform,
    MathOperationActionConfig,
    ReduceActionConfig,
    SetVariableActionConfig,
    SortActionConfig,
    StringOperationActionConfig,
    StringOperationParameters,
    ValueSource,
)

# Execution control
from .execution import (
    ActiveStatesResult,
    AvailableTransitionsResult,
    BaseActionSettings,
    NavigationResult,
    RepetitionOptions,
    TransitionExecutionResult,
    TransitionInfo,
)

# Expectation and checkpoint configs
from .expectations import (
    ActionDefaults,
    ActionExpectations,
    AllActionsPassCriteria,
    AssertionResult,
    CheckpointDefinition,
    CheckpointPassedCriteria,
    CheckpointValidationResult,
    ClaudeReviewResult,
    CustomCriteria,
    GlobalExpectations,
    MaxFailuresCriteria,
    MinMatchesCriteria,
    NoDuplicateMatchesAssertion,
    OcrAssertion,
    RequiredStatesCriteria,
    ScreenRegion,
    SuccessCriteria,
    TextAbsentAssertion,
    TextCountAssertion,
    TextInRegionAssertion,
    TextPresentAssertion,
    WorkflowExecutionResult,
    WorkflowExpectations,
)

# Find action configs
from .find_actions import (
    ExistsActionConfig,
    FindActionConfig,
    FindStateImageActionConfig,
    VanishActionConfig,
    WaitActionConfig,
    WaitCondition,
)

# Geometry primitives
from .geometry import Coordinates, Region

# Keyboard action configs
from .keyboard_actions import (
    HotkeyActionConfig,
    KeyDownActionConfig,
    KeyPressActionConfig,
    KeyUpActionConfig,
    TextSource,
    TypeActionConfig,
)

# Logging configuration
from .logging import LoggingOptions

# Mouse action configs
from .mouse_actions import (
    ClickActionConfig,
    DragActionConfig,
    HighlightActionConfig,
    MouseDownActionConfig,
    MouseMoveActionConfig,
    MouseUpActionConfig,
    ScrollActionConfig,
)

# Scheduling models (runtime)
from .scheduling import (
    SchedulerStatistics,
    StateCheckAction,
    StateCheckResult,
)

# Screenshot models
from .screenshots import (
    Screenshot,
    ScreenshotAnnotationType,
    ScreenshotLocationAnnotation,
    ScreenshotRegionAnnotation,
    ScreenshotRegionBounds,
    ScreenshotSource,
)

# Search and pattern matching
from .search import (
    MatchAdjustment,
    PatternOptions,
    PollingConfig,
    SearchOptions,
    TextSearchOptions,
)

# Shell action configs
from .shell_actions import ShellActionConfig, ShellScriptActionConfig

# State and workflow action configs
from .state_actions import (
    GoToStateActionConfig,
    RunWorkflowActionConfig,
    ScreenshotActionConfig,
    ScreenshotSaveConfig,
    WorkflowRepetition,
)

# State machine models
from .state_machine import (
    BaseTransition,
    IncomingTransition,
    MultiPatternMode,
    OutgoingTransition,
    Pattern,
    Position,
    PositionName,
    SearchMode,
    SearchRegion,
    State,
    StateImage,
    StateLocation,
    StatePosition,
    StateRegion,
    StateString,
    Transition,
    TransitionCondition,
    TransitionType,
)

# Target configurations
from .targets import (
    AllResultsTarget,
    CoordinatesTarget,
    CurrentPositionTarget,
    ImageTarget,
    LastFindResultTarget,
    RegionTarget,
    ResultByImageTarget,
    ResultIndexTarget,
    StateImageTarget,
    StateLocationTarget,
    StateRegionTarget,
    StateStringTarget,
    TargetConfig,
    TextTarget,
)

# Verification
from .verification import VerificationConfig

# Workflow models
from .workflow import (
    Connection,
    Connections,
    Variables,
    Workflow,
    WorkflowMetadata,
    WorkflowSettings,
)

__all__ = [
    # Base types
    "LogLevel",
    "MouseButton",
    "SearchStrategy",
    "VerificationMode",
    "WorkflowVisibility",
    # Geometry
    "Coordinates",
    "Region",
    # Logging
    "LoggingOptions",
    # Execution
    "ActiveStatesResult",
    "AvailableTransitionsResult",
    "BaseActionSettings",
    "ExecutionSettings",
    "NavigationResult",
    "RepetitionOptions",
    "TransitionExecutionResult",
    "TransitionInfo",
    # Search
    "MatchAdjustment",
    "PatternOptions",
    "PollingConfig",
    "SearchOptions",
    "TextSearchOptions",
    # Targets
    "AllResultsTarget",
    "CoordinatesTarget",
    "CurrentPositionTarget",
    "ImageTarget",
    "LastFindResultTarget",
    "RegionTarget",
    "ResultByImageTarget",
    "ResultIndexTarget",
    "StateImageTarget",
    "StateLocationTarget",
    "StateRegionTarget",
    "StateStringTarget",
    "TargetConfig",
    "TextTarget",
    # Verification
    "VerificationConfig",
    # Mouse actions
    "ClickActionConfig",
    "DragActionConfig",
    "HighlightActionConfig",
    "MouseDownActionConfig",
    "MouseMoveActionConfig",
    "MouseUpActionConfig",
    "ScrollActionConfig",
    # Keyboard actions
    "HotkeyActionConfig",
    "KeyDownActionConfig",
    "KeyPressActionConfig",
    "KeyUpActionConfig",
    "TextSource",
    "TypeActionConfig",
    # Find actions
    "ExistsActionConfig",
    "FindActionConfig",
    "FindStateImageActionConfig",
    "VanishActionConfig",
    "WaitActionConfig",
    "WaitCondition",
    # Control flow
    "BreakActionConfig",
    "ConditionConfig",
    "ContinueActionConfig",
    "IfActionConfig",
    "LoopActionConfig",
    "LoopCollection",
    "SwitchActionConfig",
    "SwitchCase",
    "TryCatchActionConfig",
    # Data operations
    "FilterActionConfig",
    "FilterCondition",
    "GetVariableActionConfig",
    "MapActionConfig",
    "MapTransform",
    "MathOperationActionConfig",
    "ReduceActionConfig",
    "SetVariableActionConfig",
    "SortActionConfig",
    "StringOperationActionConfig",
    "StringOperationParameters",
    "ValueSource",
    # Expectations and checkpoints
    "ActionDefaults",
    "ActionExpectations",
    "AllActionsPassCriteria",
    "AssertionResult",
    "CheckpointDefinition",
    "CheckpointPassedCriteria",
    "CheckpointValidationResult",
    "ClaudeReviewResult",
    "CustomCriteria",
    "GlobalExpectations",
    "MaxFailuresCriteria",
    "MinMatchesCriteria",
    "NoDuplicateMatchesAssertion",
    "OcrAssertion",
    "RequiredStatesCriteria",
    "ScreenRegion",
    "SuccessCriteria",
    "TextAbsentAssertion",
    "TextCountAssertion",
    "TextInRegionAssertion",
    "TextPresentAssertion",
    "WorkflowExecutionResult",
    "WorkflowExpectations",
    # Code execution
    "CodeBlockActionConfig",
    "CustomFunctionActionConfig",
    "ErrorHandling",
    # Shell actions
    "ShellActionConfig",
    "ShellScriptActionConfig",
    # AI prompt actions
    "AIPromptActionConfig",
    "AIPromptTemplate",
    "PromptParameter",
    "PromptSequence",
    "PromptSequenceStep",
    "RunPromptSequenceActionConfig",
    # State actions
    "GoToStateActionConfig",
    "RunWorkflowActionConfig",
    "ScreenshotActionConfig",
    "ScreenshotSaveConfig",
    "WorkflowRepetition",
    # Core action
    "ACTION_CONFIG_MAP",
    "Action",
    "get_typed_config",
    # Workflow
    "Connection",
    "Connections",
    "Variables",
    "Workflow",
    "WorkflowMetadata",
    "WorkflowSettings",
    # State machine models
    "BaseTransition",
    "IncomingTransition",
    "MultiPatternMode",
    "OutgoingTransition",
    "Pattern",
    "Position",
    "PositionName",
    "SearchMode",
    "SearchRegion",
    "State",
    "StateImage",
    "StateLocation",
    "StatePosition",
    "StateRegion",
    "StateString",
    "Transition",
    "TransitionCondition",
    "TransitionType",
    # Root configuration models
    "CheckMode",
    "ColorSpace",
    "CompatibleVersions",
    "ConfigLogLevel",
    "ConfigMetadata",
    "ConfigSettings",
    "ExecutionRecord",
    "FailureStrategy",
    "FindActionSettings",
    "ImageAsset",
    "ImageFormat",
    "ImageSource",
    "KeyboardActionSettings",
    "LoggingSettings",
    "MouseActionSettings",
    "PerformanceSettings",
    "QontinuiConfig",
    "RecognitionSettings",
    "Resolution",
    "Schedule",
    "ScheduleType",
    "SearchAlgorithm",
    "TriggerType",
    "WaitActionSettings",
    # Scheduling models (runtime)
    "SchedulerStatistics",
    "StateCheckAction",
    "StateCheckResult",
    # Screenshot models
    "Screenshot",
    "ScreenshotAnnotationType",
    "ScreenshotLocationAnnotation",
    "ScreenshotRegionAnnotation",
    "ScreenshotRegionBounds",
    "ScreenshotSource",
]
