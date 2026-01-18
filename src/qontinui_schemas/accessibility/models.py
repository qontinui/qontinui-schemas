"""Accessibility tree models.

This module provides Pydantic models for representing accessibility trees
captured from various sources (browsers via CDP, Windows via UIA, etc.).

The ref system (e.g., @e1, @e2) provides stable, AI-friendly identifiers
for interacting with elements without re-querying the accessibility tree.
"""

from __future__ import annotations

from pydantic import BaseModel, Field

from qontinui_schemas.accessibility.enums import AccessibilityBackend, AccessibilityRole


class AccessibilityState(BaseModel):
    """Accessibility state flags for a node.

    These flags represent the current interactive state of an element.
    Boolean flags use tri-state logic: True, False, or None (not applicable).
    """

    is_focused: bool = Field(default=False, description="Element has keyboard focus")
    is_disabled: bool = Field(
        default=False, description="Element is disabled/non-interactive"
    )
    is_hidden: bool = Field(
        default=False, description="Element is hidden from accessibility tree"
    )
    is_expanded: bool | None = Field(
        default=None, description="Expandable element expansion state"
    )
    is_selected: bool | None = Field(
        default=None, description="Selectable element selection state"
    )
    is_checked: bool | None = Field(
        default=None, description="Checkable element checked state"
    )
    is_pressed: bool | None = Field(
        default=None, description="Pressable element pressed state"
    )
    is_readonly: bool = Field(default=False, description="Element is read-only")
    is_required: bool = Field(default=False, description="Element value is required")
    is_multiselectable: bool = Field(
        default=False, description="Element allows multiple selections"
    )
    is_editable: bool = Field(
        default=False, description="Element content can be edited"
    )
    is_focusable: bool = Field(default=False, description="Element can receive focus")
    is_modal: bool = Field(default=False, description="Element is a modal dialog")


class AccessibilityBounds(BaseModel):
    """Bounding rectangle for an accessibility node.

    Coordinates are in screen pixels, typically absolute screen coordinates.
    """

    x: int = Field(description="X coordinate of top-left corner")
    y: int = Field(description="Y coordinate of top-left corner")
    width: int = Field(description="Width in pixels")
    height: int = Field(description="Height in pixels")

    @property
    def center_x(self) -> int:
        """X coordinate of the center point."""
        return self.x + self.width // 2

    @property
    def center_y(self) -> int:
        """Y coordinate of the center point."""
        return self.y + self.height // 2


class AccessibilityNode(BaseModel):
    """A node in the accessibility tree.

    Each node represents an element in the accessibility hierarchy with
    its role, name, value, state, and bounds. The ref field provides a
    stable identifier for AI-driven automation (e.g., @e1, @e2).

    Example:
        A button might have:
        - ref: "@e5"
        - role: AccessibilityRole.BUTTON
        - name: "Submit"
        - is_interactive: True
    """

    ref: str = Field(description="Reference ID like @e1, @e2 for AI interaction")
    role: AccessibilityRole = Field(
        description="Accessibility role (button, textbox, etc.)"
    )
    name: str | None = Field(default=None, description="Accessible name (label)")
    value: str | None = Field(default=None, description="Current value (for inputs)")
    description: str | None = Field(
        default=None, description="Accessible description (additional context)"
    )
    bounds: AccessibilityBounds | None = Field(
        default=None, description="Bounding rectangle in screen coordinates"
    )
    state: AccessibilityState = Field(
        default_factory=AccessibilityState, description="Current state flags"
    )
    is_interactive: bool = Field(
        default=False, description="Whether element accepts user interaction"
    )
    level: int | None = Field(
        default=None, description="Hierarchical level (for headings, tree items)"
    )
    automation_id: str | None = Field(
        default=None, description="Automation ID / test ID attribute"
    )
    class_name: str | None = Field(
        default=None, description="CSS class name or control class"
    )
    html_tag: str | None = Field(
        default=None, description="HTML tag name (for web elements)"
    )
    url: str | None = Field(default=None, description="URL for link elements")
    children: list[AccessibilityNode] = Field(
        default_factory=list, description="Child nodes in the tree"
    )


class AccessibilitySnapshot(BaseModel):
    """Complete accessibility tree snapshot.

    Represents the full accessibility tree at a point in time, with
    metadata about the capture source and statistics.

    Example AI context output:
        ## Accessibility Tree
        URL: https://example.com/login
        Title: Login Page

        ### Interactive Elements
        - @e1: button "Sign In"
        - @e2: textbox "Email"
        - @e3: textbox "Password"
    """

    root: AccessibilityNode = Field(description="Root node of the accessibility tree")
    timestamp: float = Field(description="Unix timestamp of capture")
    backend: AccessibilityBackend = Field(description="Backend used for capture")
    url: str | None = Field(default=None, description="Page URL (for web targets)")
    title: str | None = Field(default=None, description="Page/window title")
    total_nodes: int = Field(default=0, description="Total number of nodes in tree")
    interactive_nodes: int = Field(default=0, description="Number of interactive nodes")

    def get_node_by_ref(self, ref: str) -> AccessibilityNode | None:
        """Find a node by its ref ID.

        Args:
            ref: The reference ID (e.g., "@e1")

        Returns:
            The node if found, None otherwise.
        """

        def search(node: AccessibilityNode) -> AccessibilityNode | None:
            if node.ref == ref:
                return node
            for child in node.children:
                result = search(child)
                if result:
                    return result
            return None

        return search(self.root)

    def get_interactive_nodes(self) -> list[AccessibilityNode]:
        """Get all interactive nodes in the tree.

        Returns:
            List of nodes where is_interactive is True.
        """
        results: list[AccessibilityNode] = []

        def collect(node: AccessibilityNode) -> None:
            if node.is_interactive:
                results.append(node)
            for child in node.children:
                collect(child)

        collect(self.root)
        return results


class AccessibilitySelector(BaseModel):
    """Selector for finding nodes in an accessibility tree.

    Provides flexible matching criteria for locating elements by role,
    name, automation ID, or other attributes. Multiple criteria are
    combined with AND logic.

    Example:
        # Find a button named "Submit"
        selector = AccessibilitySelector(
            role=AccessibilityRole.BUTTON,
            name="Submit"
        )

        # Find any textbox containing "email" (case-insensitive)
        selector = AccessibilitySelector(
            role=AccessibilityRole.TEXTBOX,
            name_contains="email",
            case_sensitive=False
        )
    """

    role: AccessibilityRole | list[AccessibilityRole] | None = Field(
        default=None, description="Match by role (single or list)"
    )
    name: str | None = Field(default=None, description="Exact name match")
    name_contains: str | None = Field(
        default=None, description="Partial name match (contains)"
    )
    name_pattern: str | None = Field(
        default=None, description="Regex pattern for name matching"
    )
    value: str | None = Field(default=None, description="Exact value match")
    value_contains: str | None = Field(
        default=None, description="Partial value match (contains)"
    )
    automation_id: str | None = Field(
        default=None, description="Match by automation/test ID"
    )
    class_name: str | None = Field(
        default=None, description="Match by CSS/control class name"
    )
    html_tag: str | None = Field(default=None, description="Match by HTML tag name")
    state: AccessibilityState | None = Field(
        default=None, description="Required state flags (partial match)"
    )
    is_interactive: bool | None = Field(
        default=None, description="Filter by interactivity"
    )
    ancestor: AccessibilitySelector | None = Field(
        default=None, description="Required ancestor selector"
    )
    max_depth: int | None = Field(
        default=None, description="Maximum tree depth to search"
    )
    case_sensitive: bool = Field(
        default=True, description="Whether string matching is case-sensitive"
    )


# Forward reference update for self-referential models
AccessibilityNode.model_rebuild()
AccessibilitySelector.model_rebuild()


__all__ = [
    "AccessibilityState",
    "AccessibilityBounds",
    "AccessibilityNode",
    "AccessibilitySnapshot",
    "AccessibilitySelector",
]
