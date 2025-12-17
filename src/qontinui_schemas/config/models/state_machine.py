"""
State machine models for Qontinui automation.

This module provides models for state machine definitions, including states,
state images, patterns, transitions, and related types. These models are
shared across qontinui-web, qontinui-runner, and the qontinui library.
"""

from enum import Enum
from typing import Literal

from pydantic import BaseModel, Field

from .geometry import Region

# =============================================================================
# Enums
# =============================================================================


class PositionName(str, Enum):
    """Named positions within a region (matches qontinui PositionName enum)."""

    TOPLEFT = "TOPLEFT"
    TOPMIDDLE = "TOPMIDDLE"
    TOPRIGHT = "TOPRIGHT"
    MIDDLELEFT = "MIDDLELEFT"
    MIDDLEMIDDLE = "MIDDLEMIDDLE"
    MIDDLERIGHT = "MIDDLERIGHT"
    BOTTOMLEFT = "BOTTOMLEFT"
    BOTTOMMIDDLE = "BOTTOMMIDDLE"
    BOTTOMRIGHT = "BOTTOMRIGHT"


class TransitionType(str, Enum):
    """Type of transition between states."""

    OUTGOING = "OutgoingTransition"
    INCOMING = "IncomingTransition"


class SearchMode(str, Enum):
    """Search mode for state images with multiple patterns."""

    DEFAULT = "default"  # Use project default
    RAG = "rag"  # Use RAG/semantic search
    TEMPLATE = "template"  # Use template matching


class MultiPatternMode(str, Enum):
    """How to search when StateImage has multiple patterns."""

    ALL = "all"  # Search each pattern separately
    COMBINED = "combined"  # Search using combined vector


# =============================================================================
# Position and Region Types
# =============================================================================


class Position(BaseModel):
    """Position within a region using percentages (0.0-1.0)."""

    percent_w: float = Field(
        default=0.5,
        ge=0.0,
        le=1.0,
        alias="percentW",
        description="Horizontal position: 0.0 = left, 1.0 = right, 0.5 = center",
    )
    percent_h: float = Field(
        default=0.5,
        ge=0.0,
        le=1.0,
        alias="percentH",
        description="Vertical position: 0.0 = top, 1.0 = bottom, 0.5 = center",
    )
    position_name: PositionName | None = Field(
        default=None,
        alias="positionName",
        description="Optional named position for convenience",
    )

    model_config = {"populate_by_name": True}


class SearchRegion(BaseModel):
    """A region where to search for patterns."""

    id: str = Field(..., description="Unique identifier for the search region")
    name: str = Field(..., description="Human-readable name")
    x: int = Field(..., description="X coordinate of top-left corner")
    y: int = Field(..., description="Y coordinate of top-left corner")
    width: int = Field(..., gt=0, description="Width of the region")
    height: int = Field(..., gt=0, description="Height of the region")
    reference_image_id: str | None = Field(
        default=None,
        alias="referenceImageId",
        description="ID of StateImage for relative positioning",
    )
    position: Position | None = Field(
        default=None,
        description="Position within referenced image region",
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

    model_config = {"populate_by_name": True}


# =============================================================================
# Pattern and StateImage
# =============================================================================


class Pattern(BaseModel):
    """
    A single image variation with its search configuration.

    Patterns represent different visual appearances of the same UI element
    (e.g., normal, hover, clicked states). Images are referenced by ID
    from the Library - the Library is the source of truth for image data.
    """

    id: str = Field(..., description="Unique identifier for the pattern")
    name: str | None = Field(default=None, description="Optional name for the pattern")
    image_id: str | None = Field(
        default=None,
        alias="imageId",
        description="ID of ImageAsset in library (library is source of truth)",
    )
    search_regions: list[SearchRegion] = Field(
        default_factory=list,
        alias="searchRegions",
        description="Pattern-level search regions (precedence level 2)",
    )
    fixed: bool = Field(
        default=False,
        description="If true, pattern position is fixed on screen",
    )
    similarity: float | None = Field(
        default=None,
        ge=0.0,
        le=1.0,
        description="Similarity threshold (0.0-1.0)",
    )
    target_position: Position | None = Field(
        default=None,
        alias="targetPosition",
        description="Click position within pattern (default: center 0.5, 0.5)",
    )
    offset_x: int | None = Field(
        default=None,
        alias="offsetX",
        description="Pixel offset for click position X",
    )
    offset_y: int | None = Field(
        default=None,
        alias="offsetY",
        description="Pixel offset for click position Y",
    )

    model_config = {"populate_by_name": True}


class StateImage(BaseModel):
    """
    An image used to identify a state in visual automation.

    StateImages can have multiple patterns representing visual variations
    (e.g., button normal vs hover state). The image data itself is stored
    in the ImageAsset library and referenced by ID.
    """

    id: str = Field(..., description="Unique identifier for the state image")
    name: str = Field(..., description="Human-readable name")
    patterns: list[Pattern] = Field(
        default_factory=list,
        description="Multiple patterns for visual variations (e.g., normal, hover, clicked)",
    )
    shared: bool = Field(
        default=False,
        description="If true, this image appears in multiple states",
    )
    source: str | None = Field(
        default=None,
        description="How the image was created (upload, pattern-optimization, image-extraction)",
    )
    probability: float | None = Field(
        default=None,
        ge=0.0,
        le=1.0,
        description="Mock testing: probability image appears (0.0-1.0)",
    )
    search_regions: list[SearchRegion] | None = Field(
        default=None,
        alias="searchRegions",
        description="StateImage-level search regions (precedence level 3)",
    )
    monitors: list[int] | None = Field(
        default=None,
        description="Monitor indices where this image should be searched",
    )
    rag_multi_pattern_mode: MultiPatternMode | None = Field(
        default=None,
        alias="ragMultiPatternMode",
        description="How to search when StateImage has >1 pattern",
    )
    search_mode: SearchMode | None = Field(
        default=None,
        alias="searchMode",
        description="Search mode for this image (default, rag, template)",
    )
    # RAG embeddings (optional, computed by RAG setup)
    image_embedding: list[float] | None = Field(
        default=None,
        alias="imageEmbedding",
        description="RAG image embedding vector",
    )
    text_embedding: list[float] | None = Field(
        default=None,
        alias="textEmbedding",
        description="RAG text embedding vector",
    )
    ocr_text: str | None = Field(
        default=None,
        alias="ocrText",
        description="OCR extracted text from image",
    )
    ocr_confidence: float | None = Field(
        default=None,
        alias="ocrConfidence",
        description="Confidence score of OCR extraction",
    )

    model_config = {"populate_by_name": True}


# =============================================================================
# State Components (Regions, Locations, Strings)
# =============================================================================


class StateRegion(BaseModel):
    """
    A region associated with a state.

    Regions can be used as search regions, interaction regions, or
    general areas of interest within a state.
    """

    id: str = Field(..., description="Unique identifier for the region")
    name: str = Field(..., description="Human-readable name")
    x: int = Field(..., description="X coordinate of top-left corner")
    y: int = Field(..., description="Y coordinate of top-left corner")
    width: int = Field(..., gt=0, description="Width of the region")
    height: int = Field(..., gt=0, description="Height of the region")
    # Alternative bounds representation
    bounds: Region | None = Field(
        default=None,
        description="Bounding box (alternative to x, y, width, height)",
    )
    # Relative positioning
    reference_image_id: str | None = Field(
        default=None,
        alias="referenceImageId",
        description="ID of StateImage for relative positioning",
    )
    position: Position | None = Field(
        default=None,
        description="Position within referenced image region",
    )
    offset_x: int = Field(
        default=0,
        alias="offsetX",
        description="X offset in pixels",
    )
    offset_y: int = Field(
        default=0,
        alias="offsetY",
        description="Y offset in pixels",
    )
    # Flags
    is_search_region: bool = Field(
        default=False,
        alias="isSearchRegion",
        description="If true, used as a search region for StateImages",
    )
    monitors: list[int] | None = Field(
        default=None,
        description="Monitor indices where this region should be checked",
    )

    model_config = {"populate_by_name": True}


class StateLocation(BaseModel):
    """
    A location (point) associated with a state.

    Locations can be used as click targets, anchor points, or
    reference positions within a state.
    """

    id: str = Field(..., description="Unique identifier for the location")
    name: str = Field(..., description="Human-readable name")
    x: int = Field(..., description="X coordinate")
    y: int = Field(..., description="Y coordinate")
    # StateLocation specific properties
    fixed: bool = Field(
        default=False,
        description="If true, location uses absolute coordinates",
    )
    anchor: bool = Field(
        default=False,
        description="If true, used as anchor point for relative positioning",
    )
    # Relative positioning
    reference_image_id: str | None = Field(
        default=None,
        alias="referenceImageId",
        description="ID of StateImage for relative positioning",
    )
    position: Position | None = Field(
        default=None,
        description="Position within referenced image region",
    )
    offset_x: int = Field(
        default=0,
        alias="offsetX",
        description="X offset in pixels",
    )
    offset_y: int = Field(
        default=0,
        alias="offsetY",
        description="Y offset in pixels",
    )
    # Percentage-based positioning (alternative)
    percent_w: float | None = Field(
        default=None,
        ge=0.0,
        le=1.0,
        alias="percentW",
        description="Width percentage (0.0-1.0)",
    )
    percent_h: float | None = Field(
        default=None,
        ge=0.0,
        le=1.0,
        alias="percentH",
        description="Height percentage (0.0-1.0)",
    )
    anchor_type: str | None = Field(
        default=None,
        alias="anchorType",
        description="Position anchor type",
    )
    monitors: list[int] | None = Field(
        default=None,
        description="Monitor indices where this location should be checked",
    )
    metadata: dict[str, object] | None = Field(
        default=None,
        description="Additional metadata",
    )

    model_config = {"populate_by_name": True}


class StateString(BaseModel):
    """
    A string associated with a state for OCR, input, or verification.

    Strings can be used for OCR verification, text input, or
    expected text validation.
    """

    id: str = Field(..., description="Unique identifier for the string")
    name: str = Field(..., description="Human-readable name")
    value: str = Field(..., description="The string value")
    # Type flags - define how the string is used
    identifier: bool = Field(
        default=False,
        description="If true, used for OCR verification",
    )
    input_text: bool = Field(
        default=True,
        alias="inputText",
        description="If true, used as text to be typed",
    )
    expected_text: bool = Field(
        default=False,
        alias="expectedText",
        description="If true, used for validation/expected text",
    )
    regex_pattern: bool = Field(
        default=False,
        alias="regexPattern",
        description="If true, value is a regex pattern",
    )
    monitors: list[int] | None = Field(
        default=None,
        description="Monitor indices where this string should be checked",
    )

    model_config = {"populate_by_name": True}


# =============================================================================
# State
# =============================================================================


class StatePosition(BaseModel):
    """Position of a state node in the state machine graph."""

    x: int = Field(..., description="X coordinate in graph")
    y: int = Field(..., description="Y coordinate in graph")


class State(BaseModel):
    """
    A state in the visual automation state machine.

    States represent distinct UI screens or views that can be identified
    by their visual elements (StateImages) and contain associated data
    (regions, locations, strings).
    """

    id: str = Field(..., description="Unique identifier for the state")
    name: str = Field(..., description="Human-readable name")
    description: str = Field(default="", description="Description of the state")
    state_images: list[StateImage] = Field(
        default_factory=list,
        alias="stateImages",
        description="Images that identify this state",
    )
    regions: list[StateRegion] = Field(
        default_factory=list,
        description="Regions associated with this state",
    )
    locations: list[StateLocation] = Field(
        default_factory=list,
        description="Locations associated with this state",
    )
    strings: list[StateString] = Field(
        default_factory=list,
        description="Strings associated with this state",
    )
    position: StatePosition = Field(
        ...,
        description="Position in the state machine graph",
    )
    is_initial: bool = Field(
        default=False,
        alias="isInitial",
        description="If true, this is an initial state",
    )
    is_final: bool = Field(
        default=False,
        alias="isFinal",
        description="If true, this is a final state",
    )
    entry_actions: list[str] | None = Field(
        default=None,
        alias="entryActions",
        description="Workflow IDs to run on state entry",
    )
    exit_actions: list[str] | None = Field(
        default=None,
        alias="exitActions",
        description="Workflow IDs to run on state exit",
    )
    timeout: int | None = Field(
        default=None,
        description="Timeout for state detection in milliseconds",
    )

    model_config = {"populate_by_name": True}


# =============================================================================
# Transitions
# =============================================================================


class TransitionCondition(BaseModel):
    """Condition that must be met for a transition to occur."""

    type: Literal["always", "image", "time", "custom"] = Field(
        default="always",
        description="Type of condition",
    )
    image_id: str | None = Field(
        default=None,
        alias="imageId",
        description="Image ID for image-based conditions",
    )
    threshold: float | None = Field(
        default=None,
        ge=0.0,
        le=1.0,
        description="Similarity threshold for image matching",
    )
    time_delay: int | None = Field(
        default=None,
        alias="timeDelay",
        description="Time delay in milliseconds",
    )
    custom_script: str | None = Field(
        default=None,
        alias="customScript",
        description="Custom condition script",
    )

    model_config = {"populate_by_name": True}


class BaseTransition(BaseModel):
    """Base class for transitions between states."""

    id: str = Field(..., description="Unique identifier for the transition")
    type: TransitionType = Field(..., description="Type of transition")
    workflows: list[str] = Field(
        default_factory=list,
        description="Workflow IDs to execute during transition",
    )
    timeout: int = Field(
        default=10000,
        description="Transition timeout in milliseconds",
    )
    retry_count: int = Field(
        default=3,
        alias="retryCount",
        description="Number of retry attempts",
    )
    name: str | None = Field(default=None, description="Optional name")
    description: str | None = Field(default=None, description="Optional description")
    priority: int | None = Field(
        default=None,
        description="Priority for handling multiple valid transitions",
    )

    model_config = {"populate_by_name": True}


class OutgoingTransition(BaseTransition):
    """Transition from one state to another."""

    type: Literal[TransitionType.OUTGOING] = Field(
        default=TransitionType.OUTGOING,
        description="Always 'OutgoingTransition'",
    )
    from_state: str = Field(
        ...,
        alias="fromState",
        description="Source state ID",
    )
    to_state: str | None = Field(
        default=None,
        alias="toState",
        description="Target state ID",
    )
    stays_visible: bool = Field(
        default=False,
        alias="staysVisible",
        description="If true, source state remains visible after transition",
    )
    activate_states: list[str] = Field(
        default_factory=list,
        alias="activateStates",
        description="State IDs to activate",
    )
    deactivate_states: list[str] = Field(
        default_factory=list,
        alias="deactivateStates",
        description="State IDs to deactivate",
    )
    condition: TransitionCondition | None = Field(
        default=None,
        description="Condition for this transition",
    )

    model_config = {"populate_by_name": True}


class IncomingTransition(BaseTransition):
    """Transition into a state (entry transition)."""

    type: Literal[TransitionType.INCOMING] = Field(
        default=TransitionType.INCOMING,
        description="Always 'IncomingTransition'",
    )
    to_state: str = Field(
        ...,
        alias="toState",
        description="Target state ID",
    )
    execute_after: list[str] | None = Field(
        default=None,
        alias="executeAfter",
        description="OutgoingTransition IDs that trigger this",
    )

    model_config = {"populate_by_name": True}


# Union type for any transition
Transition = OutgoingTransition | IncomingTransition
