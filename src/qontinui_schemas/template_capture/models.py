"""
Template Capture Pydantic schemas.

These schemas define the data structures for the click-to-template system,
shared across qontinui library, runner, and web components.
"""

from enum import Enum
from typing import Any

from pydantic import BaseModel, Field

# =============================================================================
# Enums
# =============================================================================


class ElementType(str, Enum):
    """Types of GUI elements that can be detected."""

    BUTTON = "button"
    ICON = "icon"
    TEXT = "text"
    IMAGE = "image"
    CHECKBOX = "checkbox"
    RADIO = "radio"
    INPUT_FIELD = "input_field"
    LINK = "link"
    MENU_ITEM = "menu_item"
    TAB = "tab"
    UNKNOWN = "unknown"


class DetectionStrategyType(str, Enum):
    """Strategies for detecting element boundaries."""

    EDGE_BASED = "edge_based"
    CONTOUR_BASED = "contour_based"
    COLOR_SEGMENTATION = "color_segmentation"
    FLOOD_FILL = "flood_fill"
    GRADIENT_BASED = "gradient_based"
    TEMPLATE_MATCH = "template_match"
    FIXED_SIZE = "fixed_size"


class CandidateStatus(str, Enum):
    """Review status of a template candidate."""

    PENDING = "pending"
    APPROVED = "approved"
    REJECTED = "rejected"
    MODIFIED = "modified"


# =============================================================================
# Core Components
# =============================================================================


class CandidateBoundingBox(BaseModel):
    """Bounding box for a detected element."""

    x: int = Field(..., description="X coordinate of top-left corner")
    y: int = Field(..., description="Y coordinate of top-left corner")
    width: int = Field(..., gt=0, description="Width of the bounding box")
    height: int = Field(..., gt=0, description="Height of the bounding box")
    confidence: float = Field(
        default=0.0,
        ge=0.0,
        le=1.0,
        description="Detection confidence score",
    )
    strategy_used: DetectionStrategyType = Field(
        default=DetectionStrategyType.FIXED_SIZE,
        alias="strategyUsed",
        description="Detection strategy that found this boundary",
    )
    element_type: ElementType = Field(
        default=ElementType.UNKNOWN,
        alias="elementType",
        description="Detected element type",
    )
    has_mask: bool = Field(
        default=False,
        alias="hasMask",
        description="Whether a non-rectangular mask is available",
    )
    metadata: dict[str, Any] | None = Field(
        default=None,
        description="Additional detection metadata",
    )

    model_config = {"populate_by_name": True}


# =============================================================================
# Template Candidate Schemas
# =============================================================================


class TemplateCandidateCreate(BaseModel):
    """Request to create a template candidate (from runner)."""

    id: str = Field(..., description="Unique identifier for the candidate")
    session_id: str = Field(
        ...,
        alias="sessionId",
        description="ID of the capture session",
    )
    click_x: int = Field(
        ...,
        alias="clickX",
        description="X coordinate of the original click",
    )
    click_y: int = Field(
        ...,
        alias="clickY",
        description="Y coordinate of the original click",
    )
    click_button: str = Field(
        default="left",
        alias="clickButton",
        description="Mouse button used",
    )
    timestamp: float = Field(
        ...,
        description="Time of click in seconds from session start",
    )
    frame_number: int = Field(
        ...,
        alias="frameNumber",
        description="Video frame number",
    )
    primary_boundary: CandidateBoundingBox = Field(
        ...,
        alias="primaryBoundary",
        description="Best detected bounding box",
    )
    alternative_boundaries: list[CandidateBoundingBox] = Field(
        default_factory=list,
        alias="alternativeBoundaries",
        description="Alternative bounding boxes",
    )
    detection_strategies_used: list[DetectionStrategyType] = Field(
        default_factory=list,
        alias="detectionStrategiesUsed",
        description="Strategies attempted",
    )
    pixel_data_base64: str | None = Field(
        default=None,
        alias="pixelDataBase64",
        description="Base64-encoded pixel data",
    )
    mask_base64: str | None = Field(
        default=None,
        alias="maskBase64",
        description="Base64-encoded mask data",
    )
    pixel_shape: list[int] | None = Field(
        default=None,
        alias="pixelShape",
        description="Shape of pixel data array",
    )
    mask_shape: list[int] | None = Field(
        default=None,
        alias="maskShape",
        description="Shape of mask array",
    )
    application_hint: str | None = Field(
        default=None,
        alias="applicationHint",
        description="Application name for profile lookup",
    )
    confidence_score: float = Field(
        default=0.0,
        alias="confidenceScore",
        ge=0.0,
        le=1.0,
        description="Overall detection confidence",
    )
    element_type: str = Field(
        default="unknown",
        alias="elementType",
        description="Detected element type",
    )

    model_config = {"populate_by_name": True}


class TemplateCandidateBatchCreate(BaseModel):
    """Request to create multiple template candidates."""

    candidates: list[TemplateCandidateCreate] = Field(
        ...,
        description="List of candidates to create",
    )


class TemplateCandidateResponse(BaseModel):
    """API response for a template candidate."""

    id: str = Field(..., description="Unique identifier")
    session_id: str = Field(
        ...,
        alias="sessionId",
        description="Capture session ID",
    )
    project_id: str | None = Field(
        default=None,
        alias="projectId",
        description="Project ID",
    )
    click_x: int = Field(
        ...,
        alias="clickX",
        description="Click X coordinate",
    )
    click_y: int = Field(
        ...,
        alias="clickY",
        description="Click Y coordinate",
    )
    click_button: str = Field(
        ...,
        alias="clickButton",
        description="Mouse button",
    )
    timestamp: float = Field(..., description="Click timestamp")
    frame_number: int = Field(
        ...,
        alias="frameNumber",
        description="Frame number",
    )
    primary_boundary: CandidateBoundingBox = Field(
        ...,
        alias="primaryBoundary",
        description="Primary bounding box",
    )
    status: CandidateStatus = Field(
        default=CandidateStatus.PENDING,
        description="Review status",
    )
    confidence_score: float = Field(
        ...,
        alias="confidenceScore",
        description="Detection confidence",
    )
    element_type: str = Field(
        ...,
        alias="elementType",
        description="Element type",
    )
    application_hint: str | None = Field(
        default=None,
        alias="applicationHint",
        description="Application name",
    )
    pixel_data_url: str | None = Field(
        default=None,
        alias="pixelDataUrl",
        description="URL to pixel data image",
    )
    thumbnail_url: str | None = Field(
        default=None,
        alias="thumbnailUrl",
        description="URL to thumbnail",
    )
    created_at: str = Field(
        ...,
        alias="createdAt",
        description="ISO timestamp of creation",
    )

    model_config = {"populate_by_name": True}


class TemplateCandidateDetail(TemplateCandidateResponse):
    """Full detail view of a template candidate."""

    alternative_boundaries: list[CandidateBoundingBox] = Field(
        default_factory=list,
        alias="alternativeBoundaries",
        description="Alternative boundaries",
    )
    detection_strategies_used: list[DetectionStrategyType] = Field(
        default_factory=list,
        alias="detectionStrategiesUsed",
        description="Strategies used",
    )
    adjusted_boundary: CandidateBoundingBox | None = Field(
        default=None,
        alias="adjustedBoundary",
        description="User-adjusted boundary",
    )
    mask_url: str | None = Field(
        default=None,
        alias="maskUrl",
        description="URL to mask image",
    )
    reviewed_by: str | None = Field(
        default=None,
        alias="reviewedBy",
        description="User ID who reviewed",
    )
    reviewed_at: str | None = Field(
        default=None,
        alias="reviewedAt",
        description="Review timestamp",
    )


class TemplateCandidateSummary(BaseModel):
    """Summary of a template candidate (for listings)."""

    id: str = Field(..., description="Unique identifier")
    session_id: str = Field(
        ...,
        alias="sessionId",
        description="Session ID",
    )
    click_x: int = Field(
        ...,
        alias="clickX",
        description="Click X",
    )
    click_y: int = Field(
        ...,
        alias="clickY",
        description="Click Y",
    )
    status: CandidateStatus = Field(..., description="Review status")
    confidence_score: float = Field(
        ...,
        alias="confidenceScore",
        description="Confidence",
    )
    element_type: str = Field(
        ...,
        alias="elementType",
        description="Element type",
    )
    thumbnail_url: str | None = Field(
        default=None,
        alias="thumbnailUrl",
        description="Thumbnail URL",
    )
    created_at: str = Field(
        ...,
        alias="createdAt",
        description="Creation timestamp",
    )

    model_config = {"populate_by_name": True}


class TemplateCandidateUpdate(BaseModel):
    """Request to update a template candidate."""

    status: CandidateStatus | None = Field(
        default=None,
        description="New status",
    )
    adjusted_boundary: CandidateBoundingBox | None = Field(
        default=None,
        alias="adjustedBoundary",
        description="User-adjusted boundary",
    )

    model_config = {"populate_by_name": True}


class TemplateCandidateListResponse(BaseModel):
    """API response for listing template candidates."""

    items: list[TemplateCandidateSummary] = Field(
        ...,
        description="List of candidates",
    )
    total: int = Field(..., description="Total count")


# =============================================================================
# Tuning Schemas
# =============================================================================


class TuningMetrics(BaseModel):
    """Metrics from profile tuning."""

    sample_count: int = Field(
        default=0,
        alias="sampleCount",
        description="Number of samples used",
    )
    edge_score: float = Field(
        default=0.0,
        alias="edgeScore",
        description="Edge detection score",
    )
    contour_score: float = Field(
        default=0.0,
        alias="contourScore",
        description="Contour detection score",
    )
    color_score: float = Field(
        default=0.0,
        alias="colorScore",
        description="Color segmentation score",
    )
    flood_fill_score: float = Field(
        default=0.0,
        alias="floodFillScore",
        description="Flood fill score",
    )
    gradient_score: float = Field(
        default=0.0,
        alias="gradientScore",
        description="Gradient detection score",
    )
    avg_detection_time_ms: float = Field(
        default=0.0,
        alias="avgDetectionTimeMs",
        description="Average detection time",
    )
    avg_confidence: float = Field(
        default=0.0,
        alias="avgConfidence",
        description="Average confidence",
    )
    tuning_iterations: int = Field(
        default=0,
        alias="tuningIterations",
        description="Number of tuning iterations",
    )
    last_tuned_at: str | None = Field(
        default=None,
        alias="lastTunedAt",
        description="Last tuning timestamp",
    )

    model_config = {"populate_by_name": True}


class TuningRequest(BaseModel):
    """Request to tune an application profile."""

    screenshot_urls: list[str] = Field(
        default_factory=list,
        alias="screenshotUrls",
        description="URLs to sample screenshots",
    )
    known_elements: list[CandidateBoundingBox] | None = Field(
        default=None,
        alias="knownElements",
        description="Optional ground truth elements",
    )

    model_config = {"populate_by_name": True}


class TuningResult(BaseModel):
    """Result of profile tuning."""

    success: bool = Field(..., description="Whether tuning succeeded")
    metrics: TuningMetrics | None = Field(
        default=None,
        description="Tuning metrics",
    )
    strategy_rankings: list[tuple[str, float]] = Field(
        default_factory=list,
        alias="strategyRankings",
        description="Strategies ranked by score",
    )
    error_message: str | None = Field(
        default=None,
        alias="errorMessage",
        description="Error message if failed",
    )

    model_config = {"populate_by_name": True}


# =============================================================================
# Application Profile Schemas
# =============================================================================


class InferenceConfigSchema(BaseModel):
    """Configuration for boundary detection."""

    search_radius: int = Field(
        default=100,
        alias="searchRadius",
        description="Max distance from click to search",
    )
    min_element_size: list[int] = Field(
        default=[10, 10],
        alias="minElementSize",
        description="Minimum element dimensions [w, h]",
    )
    max_element_size: list[int] = Field(
        default=[500, 500],
        alias="maxElementSize",
        description="Maximum element dimensions [w, h]",
    )
    edge_threshold_low: int = Field(
        default=50,
        alias="edgeThresholdLow",
        description="Low Canny threshold",
    )
    edge_threshold_high: int = Field(
        default=150,
        alias="edgeThresholdHigh",
        description="High Canny threshold",
    )
    color_tolerance: int = Field(
        default=30,
        alias="colorTolerance",
        description="Color segmentation tolerance",
    )
    contour_area_min: int = Field(
        default=100,
        alias="contourAreaMin",
        description="Minimum contour area",
    )
    fallback_box_size: int = Field(
        default=50,
        alias="fallbackBoxSize",
        description="Fallback box size",
    )
    use_fallback: bool = Field(
        default=True,
        alias="useFallback",
        description="Use fallback if no detection",
    )
    preferred_strategies: list[DetectionStrategyType] = Field(
        default_factory=lambda: [
            DetectionStrategyType.CONTOUR_BASED,
            DetectionStrategyType.EDGE_BASED,
            DetectionStrategyType.COLOR_SEGMENTATION,
        ],
        alias="preferredStrategies",
        description="Preferred detection strategies",
    )
    enable_mask_generation: bool = Field(
        default=True,
        alias="enableMaskGeneration",
        description="Generate masks for non-rect elements",
    )
    enable_element_classification: bool = Field(
        default=True,
        alias="enableElementClassification",
        description="Classify element types",
    )

    model_config = {"populate_by_name": True}


class ApplicationProfileCreate(BaseModel):
    """Request to create an application profile."""

    name: str = Field(..., description="Application name (e.g., 'Civilization 6')")
    inference_config: InferenceConfigSchema | None = Field(
        default=None,
        alias="inferenceConfig",
        description="Detection configuration",
    )

    model_config = {"populate_by_name": True}


class ApplicationProfileResponse(BaseModel):
    """API response for an application profile."""

    id: str = Field(..., description="Unique identifier")
    name: str = Field(..., description="Application name")
    inference_config: InferenceConfigSchema = Field(
        ...,
        alias="inferenceConfig",
        description="Detection configuration",
    )
    preferred_strategies: list[DetectionStrategyType] = Field(
        default_factory=list,
        alias="preferredStrategies",
        description="Preferred strategies",
    )
    avg_element_size: list[int] = Field(
        default=[60, 30],
        alias="avgElementSize",
        description="Average element size [w, h]",
    )
    tuning_metrics: TuningMetrics | None = Field(
        default=None,
        alias="tuningMetrics",
        description="Tuning metrics",
    )
    success_rate: float = Field(
        default=0.0,
        alias="successRate",
        description="Detection success rate",
    )
    sample_count: int = Field(
        default=0,
        alias="sampleCount",
        description="Number of samples used",
    )
    created_at: str = Field(
        ...,
        alias="createdAt",
        description="Creation timestamp",
    )
    updated_at: str = Field(
        ...,
        alias="updatedAt",
        description="Last update timestamp",
    )

    model_config = {"populate_by_name": True}


class ApplicationProfileUpdate(BaseModel):
    """Request to update an application profile."""

    name: str | None = Field(default=None, description="New name")
    inference_config: InferenceConfigSchema | None = Field(
        default=None,
        alias="inferenceConfig",
        description="Updated config",
    )
    preferred_strategies: list[DetectionStrategyType] | None = Field(
        default=None,
        alias="preferredStrategies",
        description="Updated strategies",
    )

    model_config = {"populate_by_name": True}


class ApplicationProfile(ApplicationProfileResponse):
    """Full application profile model (alias for response)."""

    pass


class ApplicationProfileListResponse(BaseModel):
    """API response for listing application profiles."""

    items: list[ApplicationProfileResponse] = Field(
        ...,
        description="List of profiles",
    )
    total: int = Field(..., description="Total count")


# =============================================================================
# State Machine Generation Schemas
# =============================================================================


class GroupingMethod(str, Enum):
    """Methods for grouping templates into states."""

    STATE_HINTS = "state_hints"
    USER_ASSIGNMENTS = "user_assignments"
    CO_OCCURRENCE = "co_occurrence"
    SINGLE_STATE = "single_state"
    ONE_PER_TEMPLATE = "one_per_template"


class ApprovedTemplateData(BaseModel):
    """Approved template data for state machine generation.

    This represents a user-approved template from the web UI,
    ready for state machine generation.
    """

    id: str = Field(..., description="Unique identifier for the template")
    session_id: str = Field(
        ...,
        alias="sessionId",
        description="ID of the capture session",
    )
    click_x: int = Field(
        ...,
        alias="clickX",
        description="X coordinate of the original click",
    )
    click_y: int = Field(
        ...,
        alias="clickY",
        description="Y coordinate of the original click",
    )
    click_timestamp: float = Field(
        ...,
        alias="clickTimestamp",
        description="Unix timestamp of when click occurred",
    )
    frame_number: int = Field(
        ...,
        alias="frameNumber",
        description="Frame number in video",
    )
    boundary: CandidateBoundingBox = Field(
        ...,
        description="Approved bounding box",
    )
    name: str | None = Field(
        default=None,
        description="User-assigned name",
    )
    state_hint: str | None = Field(
        default=None,
        alias="stateHint",
        description="User-assigned state grouping hint",
    )
    element_type: str = Field(
        default="unknown",
        alias="elementType",
        description="Element type",
    )
    confidence: float = Field(
        default=0.0,
        description="Detection confidence",
    )
    approved_at: str | None = Field(
        default=None,
        alias="approvedAt",
        description="ISO timestamp of approval",
    )
    metadata: dict[str, Any] | None = Field(
        default=None,
        description="Additional metadata",
    )

    model_config = {"populate_by_name": True}


class GenerateStateMachineRequest(BaseModel):
    """Request to generate a state machine from approved templates."""

    approved_templates: list[ApprovedTemplateData] = Field(
        ...,
        alias="approvedTemplates",
        description="List of approved templates to process",
    )
    grouping_method: GroupingMethod = Field(
        default=GroupingMethod.STATE_HINTS,
        alias="groupingMethod",
        description="How to group templates into states",
    )
    state_assignments: dict[str, list[str]] | None = Field(
        default=None,
        alias="stateAssignments",
        description="Explicit state-to-template mappings (required if using user_assignments)",
    )
    session_id: str | None = Field(
        default=None,
        alias="sessionId",
        description="Session ID for metadata",
    )
    video_path: str | None = Field(
        default=None,
        alias="videoPath",
        description="Path to video file used for capture",
    )

    model_config = {"populate_by_name": True}


class StateImageDefResponse(BaseModel):
    """A StateImage definition in the generated state machine."""

    id: str = Field(..., description="Unique identifier for the state image")
    name: str | None = Field(default=None, description="Human-readable name")
    template_path: str | None = Field(
        default=None,
        alias="templatePath",
        description="Path to template image file",
    )
    similarity_threshold: float = Field(
        default=0.8,
        alias="similarityThreshold",
        description="Matching threshold",
    )
    click_offset: tuple[int, int] | None = Field(
        default=None,
        alias="clickOffset",
        description="Click offset from template center",
    )
    source_template_id: str | None = Field(
        default=None,
        alias="sourceTemplateId",
        description="ID of the original approved template",
    )
    metadata: dict[str, Any] | None = Field(
        default=None,
        description="Additional metadata",
    )

    model_config = {"populate_by_name": True}


class StateDefResponse(BaseModel):
    """A state definition in the generated state machine."""

    state_id: str = Field(
        ...,
        alias="stateId",
        description="Unique identifier for the state",
    )
    state_name: str = Field(
        ...,
        alias="stateName",
        description="Human-readable state name",
    )
    state_images: list[StateImageDefResponse] = Field(
        default_factory=list,
        alias="stateImages",
        description="StateImages that identify this state",
    )
    description: str = Field(
        default="",
        description="Description of the state",
    )
    is_initial: bool = Field(
        default=False,
        alias="isInitial",
        description="Whether this is an initial state",
    )
    metadata: dict[str, Any] | None = Field(
        default=None,
        description="Additional metadata",
    )

    model_config = {"populate_by_name": True}


class TransitionDefResponse(BaseModel):
    """A transition definition in the generated state machine."""

    transition_id: str = Field(
        ...,
        alias="transitionId",
        description="Unique identifier",
    )
    from_state: str = Field(
        ...,
        alias="fromState",
        description="Source state ID",
    )
    to_state: str = Field(
        ...,
        alias="toState",
        description="Target state ID",
    )
    trigger_image_id: str | None = Field(
        default=None,
        alias="triggerImageId",
        description="StateImage ID that triggers the transition",
    )
    action_type: str = Field(
        default="click",
        alias="actionType",
        description="Type of action to perform",
    )
    description: str = Field(
        default="",
        description="Human-readable description",
    )

    model_config = {"populate_by_name": True}


class StateMachineConfigResponse(BaseModel):
    """The generated state machine configuration."""

    name: str = Field(
        default="Generated State Machine",
        description="State machine name",
    )
    states: list[StateDefResponse] = Field(
        default_factory=list,
        description="State definitions",
    )
    transitions: list[TransitionDefResponse] = Field(
        default_factory=list,
        description="Transition definitions",
    )
    initial_state_id: str | None = Field(
        default=None,
        alias="initialStateId",
        description="ID of the initial state",
    )
    metadata: dict[str, Any] | None = Field(
        default=None,
        description="Additional metadata",
    )
    version: str = Field(
        default="1.0.0",
        description="Config version",
    )

    model_config = {"populate_by_name": True}


class GenerateStateMachineResponse(BaseModel):
    """Response from state machine generation."""

    success: bool = Field(..., description="Whether generation succeeded")
    state_machine: StateMachineConfigResponse | None = Field(
        default=None,
        alias="stateMachine",
        description="Generated state machine configuration",
    )
    states_count: int = Field(
        default=0,
        alias="statesCount",
        description="Number of states",
    )
    state_images_count: int = Field(
        default=0,
        alias="stateImagesCount",
        description="Total number of state images",
    )
    transitions_count: int = Field(
        default=0,
        alias="transitionsCount",
        description="Number of transitions",
    )
    ungrouped_count: int = Field(
        default=0,
        alias="ungroupedCount",
        description="Templates not assigned to any state",
    )
    processing_time_ms: float = Field(
        default=0.0,
        alias="processingTimeMs",
        description="Processing time in milliseconds",
    )
    error_message: str | None = Field(
        default=None,
        alias="errorMessage",
        description="Error message if failed",
    )

    model_config = {"populate_by_name": True}
