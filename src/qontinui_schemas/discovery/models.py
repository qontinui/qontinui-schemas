"""
Unified State Discovery Result schemas.

These schemas represent the output of state discovery from any source:
- Playwright (web extraction)
- UI Bridge (render log analysis)
- Recording (user session recording)
- Vision (screenshot analysis)
- Manual (user-defined)

The unified format captures:
- Images: Bounding boxes on screenshots with pixel representation
- States: Collections of images that appear together (co-occurrence)
- Transitions: Actions that change the active set of states

This module is shared across:
- qontinui-web backend (storage and API)
- qontinui-web frontend (display)
- qontinui-runner (producing results)
- qontinui library (state discovery algorithms)
"""

from enum import Enum
from typing import Any

from pydantic import BaseModel, Field


# =============================================================================
# Enums
# =============================================================================


class DiscoverySourceType(str, Enum):
    """Source of state discovery."""

    PLAYWRIGHT = "playwright"
    UI_BRIDGE = "ui_bridge"
    RECORDING = "recording"
    VISION = "vision"
    MANUAL = "manual"


class TransitionTriggerType(str, Enum):
    """Type of action that triggers a transition."""

    CLICK = "click"
    TYPE = "type"
    SCROLL = "scroll"
    HOVER = "hover"
    CUSTOM = "custom"


# =============================================================================
# Core Components
# =============================================================================


class DiscoveryBoundingBox(BaseModel):
    """Bounding box for an image element."""

    x: float = Field(..., description="X coordinate of top-left corner")
    y: float = Field(..., description="Y coordinate of top-left corner")
    width: float = Field(..., gt=0, description="Width of the bounding box")
    height: float = Field(..., gt=0, description="Height of the bounding box")

    model_config = {"populate_by_name": True}


class DiscoveryTransitionTrigger(BaseModel):
    """Trigger for a state transition."""

    type: TransitionTriggerType = Field(
        default=TransitionTriggerType.CLICK,
        description="Type of trigger action",
    )
    image_id: str | None = Field(
        default=None,
        alias="imageId",
        description="ID of the image that was clicked/interacted with",
    )
    element_id: str | None = Field(
        default=None,
        alias="elementId",
        description="ID of the DOM element (for web extraction)",
    )
    selector: str | None = Field(
        default=None,
        description="CSS selector for the trigger element",
    )
    value: str | None = Field(
        default=None,
        description="Value for type actions (text input)",
    )

    model_config = {"populate_by_name": True}


# =============================================================================
# State Machine Components
# =============================================================================


class DiscoveredStateImage(BaseModel):
    """
    Visual element within a discovered state.

    Represents an image crop from a screenshot with its bounding box
    and optional pixel-level identification.
    """

    id: str = Field(..., description="Unique identifier for the image")
    screenshot_id: str | None = Field(
        default=None,
        alias="screenshotId",
        description="ID of the source screenshot",
    )
    screenshot_url: str | None = Field(
        default=None,
        alias="screenshotUrl",
        description="URL to the source screenshot",
    )
    bbox: DiscoveryBoundingBox = Field(
        ...,
        description="Bounding box within the screenshot",
    )
    pixel_hash: str | None = Field(
        default=None,
        alias="pixelHash",
        description="Hash of pixel data for deduplication",
    )
    state_id: str | None = Field(
        default=None,
        alias="stateId",
        description="ID of the state this image belongs to",
    )
    element_type: str | None = Field(
        default=None,
        alias="elementType",
        description="Semantic type of the element (button, input, etc.)",
    )
    label: str | None = Field(
        default=None,
        description="Human-readable label for the image",
    )
    confidence: float = Field(
        default=1.0,
        ge=0.0,
        le=1.0,
        description="Confidence score for this image",
    )
    metadata: dict[str, Any] | None = Field(
        default=None,
        description="Additional metadata",
    )

    model_config = {"populate_by_name": True}


class DiscoveredState(BaseModel):
    """
    A discovered UI state (collection of co-occurring elements).

    States represent distinct UI screens or views identified by the
    set of images that consistently appear together.
    """

    id: str = Field(..., description="Unique identifier for the state")
    name: str = Field(..., description="Human-readable name")
    image_ids: list[str] = Field(
        default_factory=list,
        alias="imageIds",
        description="IDs of images in this state",
    )
    render_ids: list[str] = Field(
        default_factory=list,
        alias="renderIds",
        description="IDs of renders where this state appears",
    )
    element_ids: list[str] = Field(
        default_factory=list,
        alias="elementIds",
        description="IDs of DOM elements (for web extraction)",
    )
    confidence: float = Field(
        default=1.0,
        ge=0.0,
        le=1.0,
        description="Confidence score for state detection",
    )
    description: str | None = Field(
        default=None,
        description="Description of what this state represents",
    )
    metadata: dict[str, Any] | None = Field(
        default=None,
        description="Additional metadata",
    )

    model_config = {"populate_by_name": True}


class DiscoveredTransition(BaseModel):
    """
    A transition between discovered states.

    Transitions represent actions that change the active set of states
    on the screen.
    """

    id: str = Field(..., description="Unique identifier for the transition")
    from_state_id: str = Field(
        ...,
        alias="fromStateId",
        description="ID of the source state",
    )
    to_state_id: str = Field(
        ...,
        alias="toStateId",
        description="ID of the target state",
    )
    trigger: DiscoveryTransitionTrigger | None = Field(
        default=None,
        description="What triggers this transition",
    )
    confidence: float = Field(
        default=1.0,
        ge=0.0,
        le=1.0,
        description="Confidence score for transition detection",
    )
    metadata: dict[str, Any] | None = Field(
        default=None,
        description="Additional metadata",
    )

    model_config = {"populate_by_name": True}


# =============================================================================
# Complete Result
# =============================================================================


class StateDiscoveryResult(BaseModel):
    """
    Complete state machine result from discovery.

    This is the unified output format for state discovery regardless
    of the source (Playwright, UI Bridge, Recording, Vision, Manual).
    """

    id: str = Field(..., description="Unique identifier for the result")
    project_id: str = Field(
        ...,
        alias="projectId",
        description="ID of the project this belongs to",
    )
    name: str = Field(..., description="Human-readable name")
    description: str | None = Field(
        default=None,
        description="Description of this state machine",
    )
    source_type: DiscoverySourceType = Field(
        ...,
        alias="sourceType",
        description="How this state machine was discovered",
    )
    source_session_id: str | None = Field(
        default=None,
        alias="sourceSessionId",
        description="ID of the source session (extraction, recording, etc.)",
    )
    discovery_strategy: str | None = Field(
        default=None,
        alias="discoveryStrategy",
        description="Strategy used for discovery (auto, fingerprint, legacy, etc.)",
    )
    images: list[DiscoveredStateImage] = Field(
        default_factory=list,
        description="All discovered images",
    )
    states: list[DiscoveredState] = Field(
        default_factory=list,
        description="All discovered states",
    )
    transitions: list[DiscoveredTransition] = Field(
        default_factory=list,
        description="All discovered transitions",
    )
    element_to_renders: dict[str, list[str]] = Field(
        default_factory=dict,
        alias="elementToRenders",
        description="Mapping of element IDs to render IDs where they appear",
    )
    # Statistics
    image_count: int = Field(
        default=0,
        alias="imageCount",
        description="Number of images",
    )
    state_count: int = Field(
        default=0,
        alias="stateCount",
        description="Number of states",
    )
    transition_count: int = Field(
        default=0,
        alias="transitionCount",
        description="Number of transitions",
    )
    render_count: int = Field(
        default=0,
        alias="renderCount",
        description="Number of renders analyzed",
    )
    unique_element_count: int = Field(
        default=0,
        alias="uniqueElementCount",
        description="Number of unique elements",
    )
    confidence: float = Field(
        default=0.0,
        ge=0.0,
        le=1.0,
        description="Overall confidence score",
    )
    discovery_metadata: dict[str, Any] = Field(
        default_factory=dict,
        alias="discoveryMetadata",
        description="Additional discovery metadata",
    )
    # Timestamps
    created_at: str = Field(
        ...,
        alias="createdAt",
        description="ISO timestamp of creation",
    )
    updated_at: str = Field(
        ...,
        alias="updatedAt",
        description="ISO timestamp of last update",
    )

    model_config = {"populate_by_name": True}


class StateDiscoveryResultSummary(BaseModel):
    """Summary of a state discovery result (for listings)."""

    id: str = Field(..., description="Unique identifier")
    project_id: str = Field(
        ...,
        alias="projectId",
        description="ID of the project",
    )
    name: str = Field(..., description="Human-readable name")
    description: str | None = Field(
        default=None,
        description="Description",
    )
    source_type: DiscoverySourceType = Field(
        ...,
        alias="sourceType",
        description="Discovery source",
    )
    discovery_strategy: str | None = Field(
        default=None,
        alias="discoveryStrategy",
        description="Strategy used",
    )
    image_count: int = Field(
        default=0,
        alias="imageCount",
        description="Number of images",
    )
    state_count: int = Field(
        default=0,
        alias="stateCount",
        description="Number of states",
    )
    transition_count: int = Field(
        default=0,
        alias="transitionCount",
        description="Number of transitions",
    )
    confidence: float = Field(
        default=0.0,
        ge=0.0,
        le=1.0,
        description="Confidence score",
    )
    created_at: str = Field(
        ...,
        alias="createdAt",
        description="ISO timestamp of creation",
    )

    model_config = {"populate_by_name": True}


class StateDiscoveryResultListResponse(BaseModel):
    """API response for listing discovery results."""

    items: list[StateDiscoveryResultSummary] = Field(
        ...,
        description="List of result summaries",
    )
    total: int = Field(
        ...,
        description="Total count of results",
    )


# =============================================================================
# API Schemas
# =============================================================================


class StateDiscoveryResultCreate(BaseModel):
    """Request to create a state discovery result."""

    name: str = Field(..., description="Human-readable name")
    description: str | None = Field(
        default=None,
        description="Description",
    )
    source_type: DiscoverySourceType = Field(
        ...,
        alias="sourceType",
        description="Discovery source",
    )
    source_session_id: str | None = Field(
        default=None,
        alias="sourceSessionId",
        description="ID of the source session",
    )
    discovery_strategy: str | None = Field(
        default=None,
        alias="discoveryStrategy",
        description="Strategy used",
    )
    images: list[DiscoveredStateImage] = Field(
        default_factory=list,
        description="Discovered images",
    )
    states: list[DiscoveredState] = Field(
        default_factory=list,
        description="Discovered states",
    )
    transitions: list[DiscoveredTransition] = Field(
        default_factory=list,
        description="Discovered transitions",
    )
    element_to_renders: dict[str, list[str]] = Field(
        default_factory=dict,
        alias="elementToRenders",
        description="Element to renders mapping",
    )
    confidence: float = Field(
        default=0.0,
        ge=0.0,
        le=1.0,
        description="Confidence score",
    )
    discovery_metadata: dict[str, Any] = Field(
        default_factory=dict,
        alias="discoveryMetadata",
        description="Additional metadata",
    )

    model_config = {"populate_by_name": True}


class StateDiscoveryResultUpdate(BaseModel):
    """Request to update a state discovery result."""

    name: str | None = Field(
        default=None,
        description="Human-readable name",
    )
    description: str | None = Field(
        default=None,
        description="Description",
    )
    images: list[DiscoveredStateImage] | None = Field(
        default=None,
        description="Updated images",
    )
    states: list[DiscoveredState] | None = Field(
        default=None,
        description="Updated states",
    )
    transitions: list[DiscoveredTransition] | None = Field(
        default=None,
        description="Updated transitions",
    )
    discovery_metadata: dict[str, Any] | None = Field(
        default=None,
        alias="discoveryMetadata",
        description="Updated metadata",
    )

    model_config = {"populate_by_name": True}


# =============================================================================
# Export/Import
# =============================================================================


class StateMachineExport(BaseModel):
    """Portable export format for state machines."""

    version: str = Field(
        default="1.0.0",
        description="Export format version",
    )
    name: str = Field(..., description="State machine name")
    description: str | None = Field(
        default=None,
        description="Description",
    )
    source_type: DiscoverySourceType | str = Field(
        ...,
        alias="sourceType",
        description="Original discovery source",
    )
    images: list[DiscoveredStateImage] = Field(
        default_factory=list,
        description="State images",
    )
    states: list[DiscoveredState] = Field(
        default_factory=list,
        description="States",
    )
    transitions: list[DiscoveredTransition] = Field(
        default_factory=list,
        description="Transitions",
    )
    element_to_renders: dict[str, list[str]] = Field(
        default_factory=dict,
        alias="elementToRenders",
        description="Element to renders mapping",
    )
    metadata: dict[str, Any] = Field(
        default_factory=dict,
        description="Export metadata (original ID, export timestamp, etc.)",
    )

    model_config = {"populate_by_name": True}


class StateMachineImport(BaseModel):
    """Request to import a state machine."""

    state_machine: StateMachineExport = Field(
        ...,
        alias="stateMachine",
        description="The state machine to import",
    )
    name: str | None = Field(
        default=None,
        description="Override name (uses export name if not provided)",
    )

    model_config = {"populate_by_name": True}


# =============================================================================
# Display Helpers
# =============================================================================


# Source type display names
SOURCE_TYPE_LABELS: dict[DiscoverySourceType, str] = {
    DiscoverySourceType.PLAYWRIGHT: "Web Extraction",
    DiscoverySourceType.UI_BRIDGE: "UI Bridge",
    DiscoverySourceType.RECORDING: "Recording",
    DiscoverySourceType.VISION: "Vision",
    DiscoverySourceType.MANUAL: "Manual",
}
