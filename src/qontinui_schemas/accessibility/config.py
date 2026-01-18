"""Accessibility capture configuration.

This module provides configuration models for accessibility tree capture
operations, including backend selection and capture options.
"""

from pydantic import BaseModel, Field

from qontinui_schemas.accessibility.enums import AccessibilityBackend


class AccessibilityConfig(BaseModel):
    """Configuration for accessibility capture.

    Controls how accessibility trees are captured, including which backend
    to use and filtering options.
    """

    backend: AccessibilityBackend = Field(
        default=AccessibilityBackend.AUTO,
        description="Accessibility backend to use (auto, cdp, uia, etc.)",
    )
    interactive_only: bool = Field(
        default=False,
        description="Only capture interactive elements (reduces tree size)",
    )
    include_hidden: bool = Field(
        default=False,
        description="Include hidden/offscreen elements in capture",
    )
    max_depth: int | None = Field(
        default=None,
        description="Maximum tree depth to capture (None for unlimited)",
    )
    cdp_host: str = Field(
        default="localhost",
        description="CDP WebSocket host for browser connections",
    )
    cdp_port: int = Field(
        default=9222,
        description="CDP WebSocket port for browser connections",
    )
    cdp_timeout: float = Field(
        default=30.0,
        description="Timeout in seconds for CDP operations",
    )
    include_bounds: bool = Field(
        default=True,
        description="Include bounding rectangles for each node",
    )
    include_value: bool = Field(
        default=True,
        description="Include current values for input elements",
    )


class AccessibilityCaptureOptions(BaseModel):
    """Options for a single accessibility capture operation.

    Used when invoking a capture to specify target and overrides.
    """

    target: str = Field(
        default="auto",
        description="Capture target: 'auto', 'web', 'native', 'tauri', or a window title/handle",
    )
    include_screenshot: bool = Field(
        default=True,
        description="Also capture a screenshot alongside the tree",
    )
    config: AccessibilityConfig = Field(
        default_factory=AccessibilityConfig,
        description="Capture configuration overrides",
    )


class AccessibilityActionResult(BaseModel):
    """Result of an accessibility-based action.

    Returned when performing actions like click_by_ref or type_by_ref.
    """

    success: bool = Field(description="Whether the action succeeded")
    ref: str = Field(description="The ref that was acted upon")
    action: str = Field(description="The action performed (click, type, focus)")
    error: str | None = Field(
        default=None, description="Error message if action failed"
    )
    element_name: str | None = Field(
        default=None, description="Name of the element acted upon"
    )
    element_role: str | None = Field(
        default=None, description="Role of the element acted upon"
    )


__all__ = [
    "AccessibilityConfig",
    "AccessibilityCaptureOptions",
    "AccessibilityActionResult",
]
