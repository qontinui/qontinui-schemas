"""Event schemas for Qontinui execution logging.

This module contains event schemas used for real-time execution logging
and historical playback across the Qontinui ecosystem.
"""

from qontinui_schemas.events.tree_events import (  # Enums; Nested models; Main models; API models
    ActionType,
    DisplayNode,
    ExecutionTreeResponse,
    MatchLocation,
    NodeMetadata,
    NodeStatus,
    NodeType,
    Outcome,
    PathElement,
    RuntimeData,
    StateContext,
    TimingInfo,
    TopMatch,
    TreeEvent,
    TreeEventCreate,
    TreeEventListResponse,
    TreeEventResponse,
    TreeEventType,
    TreeNode,
)

__all__ = [
    # ==========================================================================
    # Enums
    # ==========================================================================
    "NodeType",
    "NodeStatus",
    "TreeEventType",
    "ActionType",
    # ==========================================================================
    # Nested Metadata Models
    # ==========================================================================
    "MatchLocation",
    "TopMatch",
    "RuntimeData",
    "StateContext",
    "TimingInfo",
    "Outcome",
    # ==========================================================================
    # Main Tree Event Models
    # ==========================================================================
    "NodeMetadata",
    "TreeNode",
    "PathElement",
    "TreeEvent",
    # ==========================================================================
    # Display Models (Frontend)
    # ==========================================================================
    "DisplayNode",
    # ==========================================================================
    # API Request/Response Models
    # ==========================================================================
    "TreeEventCreate",
    "TreeEventResponse",
    "TreeEventListResponse",
    "ExecutionTreeResponse",
]
