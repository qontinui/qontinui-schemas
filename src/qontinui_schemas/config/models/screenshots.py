"""
Screenshot models for Qontinui automation.

This module provides models for screenshots and their annotations (regions and locations).
Screenshots are used for state discovery and visual documentation of automation workflows.

All datetime fields use UTCDateTime for consistent UTC timezone handling
and ISO 8601 format strings with 'Z' suffix for JSON serialization.
"""

from enum import Enum

from pydantic import BaseModel, Field

from qontinui_schemas.common.time import UTCDateTime

# =============================================================================
# Enums
# =============================================================================


class ScreenshotAnnotationType(str, Enum):
    """Type of screenshot annotation region."""

    STATE_REGION = "StateRegion"
    SEARCH_REGION = "SearchRegion"


class ScreenshotSource(str, Enum):
    """Source of the screenshot."""

    UPLOADED = "uploaded"
    STATE_DISCOVERY = "state_discovery"
    AUTOMATION_RUN = "automation_run"
    PATTERN_OPTIMIZATION = "pattern_optimization"


# =============================================================================
# Screenshot Annotations
# =============================================================================


class ScreenshotRegionBounds(BaseModel):
    """Bounding box for a screenshot region annotation."""

    x: int = Field(..., description="X coordinate of top-left corner")
    y: int = Field(..., description="Y coordinate of top-left corner")
    width: int = Field(..., gt=0, description="Width of the region")
    height: int = Field(..., gt=0, description="Height of the region")


class ScreenshotRegionAnnotation(BaseModel):
    """
    Region annotation on a screenshot.

    Used to mark areas of interest on screenshots, typically during
    state discovery or when creating search regions for automation.
    """

    id: str = Field(..., description="Unique identifier for the annotation")
    screenshot_id: str = Field(
        ...,
        alias="screenshotId",
        description="ID of the screenshot this annotation belongs to",
    )
    state_id: str = Field(
        ...,
        alias="stateId",
        description="ID of the state this annotation is associated with",
    )
    name: str = Field(..., description="Human-readable name for the region")
    type: ScreenshotAnnotationType = Field(
        ...,
        description="Type of region (StateRegion or SearchRegion)",
    )
    bounds: ScreenshotRegionBounds = Field(
        ...,
        description="Bounding box defining the region",
    )
    linked_state_object_id: str | None = Field(
        default=None,
        alias="linkedStateObjectId",
        description="ID of linked state object (e.g., StateImage ID)",
    )
    linked_state_object_type: str | None = Field(
        default=None,
        alias="linkedStateObjectType",
        description="Type of linked state object (e.g., 'StateImage')",
    )
    reference_state_id: str | None = Field(
        default=None,
        alias="referenceStateId",
        description="ID of reference state for relative positioning",
    )
    save_to_state_image_id: str | None = Field(
        default=None,
        alias="saveToStateImageId",
        description="StateImage ID to save extracted region to",
    )
    save_to_state_image_state_id: str | None = Field(
        default=None,
        alias="saveToStateImageStateId",
        description="State ID for the StateImage to save to",
    )

    model_config = {"populate_by_name": True}


class ScreenshotLocationAnnotation(BaseModel):
    """
    Location (point) annotation on a screenshot.

    Used to mark specific click points, anchor points, or reference
    positions on screenshots during state discovery or automation setup.
    """

    id: str = Field(..., description="Unique identifier for the annotation")
    screenshot_id: str = Field(
        ...,
        alias="screenshotId",
        description="ID of the screenshot this annotation belongs to",
    )
    state_id: str = Field(
        ...,
        alias="stateId",
        description="ID of the state this annotation is associated with",
    )
    name: str = Field(..., description="Human-readable name for the location")
    x: int = Field(..., description="X coordinate of the location")
    y: int = Field(..., description="Y coordinate of the location")
    anchor: bool = Field(
        default=False,
        description="If true, this location is used as an anchor point",
    )
    anchor_type: str | None = Field(
        default=None,
        alias="anchorType",
        description="Type of anchor positioning",
    )
    fixed: bool = Field(
        default=False,
        description="If true, location uses absolute coordinates",
    )
    reference_image_id: str | None = Field(
        default=None,
        alias="referenceImageId",
        description="ID of StateImage for relative positioning",
    )
    reference_state_id: str | None = Field(
        default=None,
        alias="referenceStateId",
        description="ID of reference state for relative positioning",
    )
    offset_x: int = Field(
        default=0,
        alias="offsetX",
        description="X offset in pixels from reference position",
    )
    offset_y: int = Field(
        default=0,
        alias="offsetY",
        description="Y offset in pixels from reference position",
    )
    percent_w: float | None = Field(
        default=None,
        ge=0.0,
        le=1.0,
        alias="percentW",
        description="Width percentage (0.0-1.0) for relative positioning",
    )
    percent_h: float | None = Field(
        default=None,
        ge=0.0,
        le=1.0,
        alias="percentH",
        description="Height percentage (0.0-1.0) for relative positioning",
    )

    model_config = {"populate_by_name": True}


# =============================================================================
# Screenshot
# =============================================================================


class Screenshot(BaseModel):
    """
    A screenshot used in visual automation.

    Screenshots are used for state discovery, visual documentation,
    and creating automation workflows. They can be annotated with
    regions and locations to identify UI elements.
    """

    id: str = Field(..., description="Unique identifier for the screenshot")
    name: str = Field(..., description="Human-readable name")
    url: str = Field(..., description="URL or path to the screenshot image")
    size: int = Field(..., gt=0, description="File size in bytes")
    uploaded_at: UTCDateTime = Field(
        ...,
        alias="uploadedAt",
        description="When the screenshot was uploaded (UTC)",
    )
    timestamp: int | None = Field(
        default=None,
        description="Unix timestamp when screenshot was taken",
    )
    description: str | None = Field(
        default=None,
        description="Optional description of what the screenshot shows",
    )
    tags: list[str] = Field(
        default_factory=list,
        description="Tags for organizing screenshots",
    )
    project_name: str | None = Field(
        default=None,
        alias="projectName",
        description="Name of the project this screenshot belongs to",
    )
    regions: list[ScreenshotRegionAnnotation] = Field(
        default_factory=list,
        description="Region annotations created in Create Regions & Locations tab",
    )
    locations: list[ScreenshotLocationAnnotation] = Field(
        default_factory=list,
        description="Location annotations created in Create Regions & Locations tab",
    )
    associated_states: list[str] = Field(
        default_factory=list,
        alias="associatedStates",
        description="IDs of states associated with this screenshot",
    )
    monitors: list[int] = Field(
        default_factory=lambda: [0],
        description="Monitor indices this screenshot is from/for (default: [0] for primary)",
    )
    source: ScreenshotSource | None = Field(
        default=None,
        description="How the screenshot was created or obtained",
    )

    model_config = {"populate_by_name": True}
