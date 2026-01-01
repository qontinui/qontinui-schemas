"""Descriptions schemas module.

Provides rich description schemas for AI verification agent feature.
These descriptions enable AI agents to:
- Understand what states, actions, transitions, and workflows represent
- Verify that application reality matches described expectations
- Detect unexpected elements or error states
- Understand business context and user goals

Usage:
    from qontinui_schemas.descriptions import (
        StateDescription,
        ActionDescription,
        TransitionDescription,
        WorkflowDescription,
    )
"""

from qontinui_schemas.descriptions.models import (
    ActionDescription,
    StateDescription,
    TransitionDescription,
    WorkflowDescription,
)

__all__ = [
    "StateDescription",
    "ActionDescription",
    "TransitionDescription",
    "WorkflowDescription",
]
