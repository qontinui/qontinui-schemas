"""Accessibility tree schemas for Qontinui.

This module provides schemas for accessibility tree capture and interaction,
enabling AI-optimized automation through ref-based element selection.

The accessibility tree provides a semantic representation of UI elements
that is significantly more token-efficient than raw HTML or screenshots,
making it ideal for AI-driven automation.

Key concepts:
- AccessibilityNode: A node in the accessibility tree with role, name, state
- AccessibilitySnapshot: Complete tree capture with metadata
- AccessibilitySelector: Criteria for finding nodes in the tree
- Ref system: Stable identifiers (@e1, @e2) for element interaction

Example usage:
    from qontinui_schemas.accessibility import (
        AccessibilityNode,
        AccessibilitySnapshot,
        AccessibilitySelector,
        AccessibilityRole,
        AccessibilityBackend,
        AccessibilityConfig,
    )

    # Create a selector for a submit button
    selector = AccessibilitySelector(
        role=AccessibilityRole.BUTTON,
        name="Submit"
    )

    # Configure accessibility capture
    config = AccessibilityConfig(
        backend=AccessibilityBackend.CDP,
        interactive_only=True,
    )
"""

from qontinui_schemas.accessibility.config import (
    AccessibilityActionResult,
    AccessibilityCaptureOptions,
    AccessibilityConfig,
)
from qontinui_schemas.accessibility.enums import (
    AccessibilityBackend,
    AccessibilityRole,
)
from qontinui_schemas.accessibility.models import (
    AccessibilityBounds,
    AccessibilityNode,
    AccessibilitySelector,
    AccessibilitySnapshot,
    AccessibilityState,
)

__all__ = [
    # Enums
    "AccessibilityRole",
    "AccessibilityBackend",
    # Models
    "AccessibilityState",
    "AccessibilityBounds",
    "AccessibilityNode",
    "AccessibilitySnapshot",
    "AccessibilitySelector",
    # Config
    "AccessibilityConfig",
    "AccessibilityCaptureOptions",
    "AccessibilityActionResult",
]
