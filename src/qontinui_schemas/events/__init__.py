"""Event schemas for Qontinui execution logging.

This module contains event schemas used for real-time execution logging
and historical playback across the Qontinui ecosystem.
"""

from qontinui_schemas.events.healing_events import (  # Enums; Nested models; Main models; API models
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
    # Tree Event Enums
    # ==========================================================================
    "NodeType",
    "NodeStatus",
    "TreeEventType",
    "ActionType",
    # ==========================================================================
    # Healing Event Enums
    # ==========================================================================
    "HealingEventType",
    "HealingStrategy",
    # ==========================================================================
    # Nested Metadata Models (Tree Events)
    # ==========================================================================
    "MatchLocation",
    "TopMatch",
    "RuntimeData",
    "StateContext",
    "TimingInfo",
    "Outcome",
    # ==========================================================================
    # Nested Metadata Models (Healing Events)
    # ==========================================================================
    "CacheMetrics",
    "HealingAttemptInfo",
    "HealingMetrics",
    "ReliabilityInfo",
    "VisualValidationInfo",
    # ==========================================================================
    # Main Tree Event Models
    # ==========================================================================
    "NodeMetadata",
    "TreeNode",
    "PathElement",
    "TreeEvent",
    # ==========================================================================
    # Main Healing Event Models
    # ==========================================================================
    "HealingEvent",
    "HealingEventData",
    # ==========================================================================
    # Display Models (Frontend)
    # ==========================================================================
    "DisplayNode",
    # ==========================================================================
    # API Request/Response Models (Tree Events)
    # ==========================================================================
    "TreeEventCreate",
    "TreeEventResponse",
    "TreeEventListResponse",
    "ExecutionTreeResponse",
    # ==========================================================================
    # API Request/Response Models (Healing Events)
    # ==========================================================================
    "HealingEventCreate",
    "HealingStatsResponse",
]
