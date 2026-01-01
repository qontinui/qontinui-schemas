"""Findings-related enumerations.

These enums are shared between:
- qontinui-runner (TypeScript, via generated types)
- qontinui-web backend (Python/Pydantic)
- qontinui-web frontend (TypeScript, via generated types)
- Claude Code analysis tools

IMPORTANT: Changes to these enums require updates in all consumers.
"""

from enum import Enum


class FindingCategory(str, Enum):
    """Category of a detected finding.

    Determines the type of issue or observation detected during analysis.
    """

    CODE_BUG = "code_bug"
    SECURITY = "security"
    PERFORMANCE = "performance"
    TODO = "todo"
    ENHANCEMENT = "enhancement"
    CONFIG_ISSUE = "config_issue"
    TEST_ISSUE = "test_issue"
    DOCUMENTATION = "documentation"
    RUNTIME_ISSUE = "runtime_issue"
    ALREADY_FIXED = "already_fixed"
    EXPECTED_BEHAVIOR = "expected_behavior"


class FindingSeverity(str, Enum):
    """Severity level of a finding.

    CRITICAL: Blocks functionality, immediate attention required.
    HIGH: Major issue, should be fixed soon.
    MEDIUM: Moderate issue, should be fixed.
    LOW: Minor issue, fix when convenient.
    INFO: Informational, no action required.
    """

    CRITICAL = "critical"
    HIGH = "high"
    MEDIUM = "medium"
    LOW = "low"
    INFO = "info"


class FindingStatus(str, Enum):
    """Status of a finding.

    Lifecycle: DETECTED -> IN_PROGRESS -> (RESOLVED | WONT_FIX | DEFERRED)
    NEEDS_INPUT is a special state requiring user decision.
    """

    DETECTED = "detected"
    IN_PROGRESS = "in_progress"
    NEEDS_INPUT = "needs_input"
    RESOLVED = "resolved"
    WONT_FIX = "wont_fix"
    DEFERRED = "deferred"


class FindingActionType(str, Enum):
    """Type of action recommended for a finding.

    AUTO_FIX: Can be automatically fixed without user intervention.
    NEEDS_USER_INPUT: Requires user decision or input to resolve.
    INFORMATIONAL: No action needed, for awareness only.
    """

    AUTO_FIX = "auto_fix"
    NEEDS_USER_INPUT = "needs_user_input"
    INFORMATIONAL = "informational"


# Re-export for convenience
__all__ = [
    "FindingCategory",
    "FindingSeverity",
    "FindingStatus",
    "FindingActionType",
]
