"""
Unified State Discovery schemas.

This module provides schemas for state discovery results from any source:
- Playwright (web extraction)
- UI Bridge (render log analysis)
- Recording (user session recording)
- Vision (screenshot analysis)
- Manual (user-defined)
"""

from qontinui_schemas.discovery.models import (
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

__all__ = [
    # Enums
    "DiscoverySourceType",
    "TransitionTriggerType",
    # Core components
    "DiscoveryBoundingBox",
    "DiscoveryTransitionTrigger",
    "DiscoveredStateImage",
    "DiscoveredState",
    "DiscoveredTransition",
    # Complete result
    "StateDiscoveryResult",
    "StateDiscoveryResultSummary",
    "StateDiscoveryResultListResponse",
    # API schemas
    "StateDiscoveryResultCreate",
    "StateDiscoveryResultUpdate",
    # Export/Import
    "StateMachineExport",
    "StateMachineImport",
    # Helpers
    "SOURCE_TYPE_LABELS",
]
