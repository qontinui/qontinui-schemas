"""Execution-related enumerations.

These enums are shared between:
- qontinui-runner (TypeScript, via generated types)
- qontinui-web backend (Python/Pydantic)
- qontinui-web frontend (TypeScript, via generated types)

IMPORTANT: Changes to these enums require updates in all consumers.
"""

from enum import Enum


class RunType(str, Enum):
    """Type of execution run.

    Determines the behavior and reporting expectations for a run.
    """

    QA_TEST = "qa_test"
    INTEGRATION_TEST = "integration_test"
    LIVE_AUTOMATION = "live_automation"
    RECORDING = "recording"
    DEBUG = "debug"


class RunStatus(str, Enum):
    """Status of an execution run.

    Lifecycle: PENDING -> RUNNING -> (COMPLETED | FAILED | TIMEOUT | CANCELLED)
    PAUSED is a special state that can transition back to RUNNING.
    """

    PENDING = "pending"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"
    TIMEOUT = "timeout"
    CANCELLED = "cancelled"
    PAUSED = "paused"


class ActionType(str, Enum):
    """Type of action executed during automation.

    These correspond to the action types in the Qontinui automation system.
    """

    # Vision/Pattern matching
    FIND = "find"
    FIND_ALL = "find_all"
    WAIT_FOR = "wait_for"
    WAIT_UNTIL_GONE = "wait_until_gone"

    # Mouse actions
    CLICK = "click"
    DOUBLE_CLICK = "double_click"
    RIGHT_CLICK = "right_click"
    SCROLL = "scroll"
    DRAG = "drag"

    # Keyboard actions
    TYPE = "type"
    PRESS_KEY = "press_key"
    HOTKEY = "hotkey"

    # State machine
    GO_TO_STATE = "go_to_state"
    TRANSITION = "transition"
    VERIFY_STATE = "verify_state"

    # Control flow
    CONDITIONAL = "conditional"
    LOOP = "loop"
    PARALLEL = "parallel"
    SEQUENCE = "sequence"

    # Utility
    WAIT = "wait"
    SCREENSHOT = "screenshot"
    LOG = "log"
    ASSERT = "assert"
    CUSTOM = "custom"


class ActionStatus(str, Enum):
    """Status of an individual action execution.

    SUCCESS/FAILED/TIMEOUT are terminal states.
    PENDING is initial, RUNNING is transient (for long actions).
    SKIPPED means action was intentionally not executed.
    ERROR indicates an unexpected error during execution.
    """

    SUCCESS = "success"
    FAILED = "failed"
    TIMEOUT = "timeout"
    SKIPPED = "skipped"
    ERROR = "error"
    PENDING = "pending"


class ErrorType(str, Enum):
    """Type of error that occurred during execution.

    Used for categorizing failures for analytics and debugging.
    """

    ELEMENT_NOT_FOUND = "element_not_found"
    TIMEOUT = "timeout"
    ASSERTION_FAILED = "assertion_failed"
    CRASH = "crash"
    NETWORK_ERROR = "network_error"
    VALIDATION_ERROR = "validation_error"
    STATE_MISMATCH = "state_mismatch"
    NAVIGATION_ERROR = "navigation_error"
    SCRIPT_ERROR = "script_error"
    OTHER = "other"


class IssueSeverity(str, Enum):
    """Severity level of an execution issue.

    CRITICAL: Blocks functionality, immediate attention required.
    HIGH: Major issue, should be fixed soon.
    MEDIUM: Moderate issue, should be fixed.
    LOW: Minor issue, fix when convenient.
    INFORMATIONAL: FYI, no action required.
    """

    CRITICAL = "critical"
    HIGH = "high"
    MEDIUM = "medium"
    LOW = "low"
    INFORMATIONAL = "informational"


class IssueType(str, Enum):
    """Type of issue detected during execution."""

    VISUAL_REGRESSION = "visual_regression"
    ELEMENT_NOT_FOUND = "element_not_found"
    STATE_MISMATCH = "state_mismatch"
    TIMEOUT = "timeout"
    ASSERTION_FAILED = "assertion_failed"
    NAVIGATION_ERROR = "navigation_error"
    SCRIPT_ERROR = "script_error"
    PERFORMANCE = "performance"
    ACCESSIBILITY = "accessibility"
    OTHER = "other"


class IssueStatus(str, Enum):
    """Status of an execution issue.

    Lifecycle: OPEN -> IN_PROGRESS -> (RESOLVED | WONT_FIX | DUPLICATE | CANNOT_REPRODUCE)
    """

    OPEN = "open"
    IN_PROGRESS = "in_progress"
    RESOLVED = "resolved"
    WONT_FIX = "wont_fix"
    DUPLICATE = "duplicate"
    CANNOT_REPRODUCE = "cannot_reproduce"


class IssueSource(str, Enum):
    """Source that detected the issue."""

    AUTOMATION = "automation"
    AI_ANALYSIS = "ai_analysis"
    VISUAL_REGRESSION = "visual_regression"
    USER_REPORTED = "user_reported"


class ScreenshotType(str, Enum):
    """Type of screenshot captured during execution.

    Used to categorize screenshots for filtering and display.
    """

    ERROR = "error"
    SUCCESS = "success"
    MANUAL = "manual"
    PERIODIC = "periodic"
    ACTION_RESULT = "action_result"
    STATE_VERIFICATION = "state_verification"
    BEFORE_ACTION = "before_action"
    AFTER_ACTION = "after_action"
    DIFF_BASELINE = "diff_baseline"
    DIFF_COMPARISON = "diff_comparison"


class TreeNodeType(str, Enum):
    """Type of node in the execution tree."""

    WORKFLOW = "workflow"
    ACTION = "action"
    TRANSITION = "transition"


class TreeEventType(str, Enum):
    """Type of tree event emitted during execution."""

    # Workflow lifecycle
    WORKFLOW_STARTED = "workflow_started"
    WORKFLOW_COMPLETED = "workflow_completed"
    WORKFLOW_FAILED = "workflow_failed"

    # Action lifecycle
    ACTION_STARTED = "action_started"
    ACTION_COMPLETED = "action_completed"
    ACTION_FAILED = "action_failed"

    # Transition lifecycle
    TRANSITION_STARTED = "transition_started"
    TRANSITION_COMPLETED = "transition_completed"
    TRANSITION_FAILED = "transition_failed"


class TreeNodeStatus(str, Enum):
    """Status of a node in the execution tree."""

    PENDING = "pending"
    RUNNING = "running"
    SUCCESS = "success"
    FAILED = "failed"


# Re-export for convenience
__all__ = [
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
]
