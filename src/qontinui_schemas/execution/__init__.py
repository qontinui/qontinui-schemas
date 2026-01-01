"""Execution schemas module.

Provides unified schemas for execution tracking used by:
- qontinui-runner (TypeScript, via generated types)
- qontinui-web backend (Python/Pydantic)
- qontinui-web frontend (TypeScript, via generated types)

Usage:
    from qontinui_schemas.execution import (
        RunType, RunStatus, ExecutionRunCreate,
        ActionType, ActionExecutionCreate,
        ScreenshotType, ExecutionScreenshotCreate,
        ExecutionIssueCreate,
    )
"""

# Action schemas
from qontinui_schemas.execution.action import (
    ActionExecutionBatchCreate,
    ActionExecutionCreate,
    ActionExecutionDetail,
    ActionExecutionListResponse,
    ActionExecutionResponse,
)

# Enums
from qontinui_schemas.execution.enums import (
    ActionStatus,
    ActionType,
    ErrorType,
    IssueSeverity,
    IssueSource,
    IssueStatus,
    IssueType,
    RunStatus,
    RunType,
    ScreenshotType,
    TreeEventType,
    TreeNodeStatus,
    TreeNodeType,
)

# Issue schemas
from qontinui_schemas.execution.issue import (
    ExecutionIssueBatchCreate,
    ExecutionIssueCreate,
    ExecutionIssueDetail,
    ExecutionIssueListResponse,
    ExecutionIssueResponse,
    ExecutionIssueSummary,
    ExecutionIssueUpdate,
)

# Run schemas
from qontinui_schemas.execution.run import (
    ExecutionRunComplete,
    ExecutionRunCompleteResponse,
    ExecutionRunCreate,
    ExecutionRunDetail,
    ExecutionRunListResponse,
    ExecutionRunResponse,
)

# Screenshot schemas
from qontinui_schemas.execution.screenshot import (
    ExecutionScreenshotCreate,
    ExecutionScreenshotDetail,
    ExecutionScreenshotListResponse,
    ExecutionScreenshotResponse,
)

# Tree event schemas
from qontinui_schemas.execution.tree_event import (
    DisplayNode,
    ExecutionTreeEventBatchCreate,
    ExecutionTreeEventCreate,
    ExecutionTreeEventListResponse,
    ExecutionTreeEventResponse,
    ExecutionTreeResponse,
    NodeMetadata,
    NodeStatus,
    NodeType,
    PathElement,
    TreeNode,
)

__all__ = [
    # Enums
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
    # Run schemas
    "ExecutionRunCreate",
    "ExecutionRunResponse",
    "ExecutionRunDetail",
    "ExecutionRunComplete",
    "ExecutionRunCompleteResponse",
    "ExecutionRunListResponse",
    # Action schemas
    "ActionExecutionCreate",
    "ActionExecutionBatchCreate",
    "ActionExecutionResponse",
    "ActionExecutionDetail",
    "ActionExecutionListResponse",
    # Screenshot schemas
    "ExecutionScreenshotCreate",
    "ExecutionScreenshotResponse",
    "ExecutionScreenshotDetail",
    "ExecutionScreenshotListResponse",
    # Issue schemas
    "ExecutionIssueCreate",
    "ExecutionIssueBatchCreate",
    "ExecutionIssueResponse",
    "ExecutionIssueDetail",
    "ExecutionIssueUpdate",
    "ExecutionIssueListResponse",
    "ExecutionIssueSummary",
    # Tree event schemas
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
]
