"""
Extraction schemas for web extraction results.

These models define the data structures shared between:
- qontinui library (extraction output)
- qontinui-runner (extraction orchestration)
- qontinui-web backend (storage and API)
- qontinui-web frontend (display)

The extraction process discovers states and transitions from web applications,
which can then be imported into the project's state machine configuration.
"""

from datetime import datetime
from enum import Enum
from typing import Any

from pydantic import BaseModel, Field

# =============================================================================
# Enums
# =============================================================================


class ExtractionStatus(str, Enum):
    """Status of an extraction session."""

    PENDING = "pending"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"


class StateType(str, Enum):
    """Type of extracted state (semantic region type)."""

    PAGE = "page"
    NAVIGATION = "navigation"
    HEADER = "header"
    FOOTER = "footer"
    SIDEBAR = "sidebar"
    MAIN = "main"
    MODAL = "modal"
    DIALOG = "dialog"
    FORM = "form"
    CARD = "card"
    LIST = "list"
    TABLE = "table"
    MENU = "menu"
    TOOLBAR = "toolbar"
    TAB_PANEL = "tab_panel"
    ACCORDION = "accordion"
    CAROUSEL = "carousel"
    ALERT = "alert"
    TOAST = "toast"
    TOOLTIP = "tooltip"
    POPOVER = "popover"
    DROPDOWN = "dropdown"
    UNKNOWN = "unknown"


class TriggerType(str, Enum):
    """Type of trigger that causes a transition."""

    CLICK = "click"
    HOVER = "hover"
    FOCUS = "focus"
    SCROLL = "scroll"
    NAVIGATION = "navigation"
    KEYBOARD = "keyboard"
    STATE_CHANGE = "state_change"
    UNKNOWN = "unknown"


# =============================================================================
# Bounding Box
# =============================================================================


class BoundingBox(BaseModel):
    """Bounding box for an element or state."""

    x: float = Field(..., description="X coordinate of top-left corner")
    y: float = Field(..., description="Y coordinate of top-left corner")
    width: float = Field(..., gt=0, description="Width of the bounding box")
    height: float = Field(..., gt=0, description="Height of the bounding box")

    model_config = {"populate_by_name": True}


# =============================================================================
# Extracted Element
# =============================================================================


class ExtractedElement(BaseModel):
    """An interactive element discovered during extraction."""

    id: str = Field(..., description="Unique identifier for the element")
    tag_name: str = Field(
        ...,
        alias="tagName",
        description="HTML tag name (e.g., 'button', 'a', 'input')",
    )
    element_type: str = Field(
        ...,
        alias="elementType",
        description="Semantic type (e.g., 'button', 'link', 'input')",
    )
    text: str | None = Field(
        default=None,
        description="Text content of the element",
    )
    bbox: BoundingBox = Field(..., description="Bounding box of the element")
    selector: str | None = Field(
        default=None,
        description="CSS selector for the element",
    )
    attributes: dict[str, str] = Field(
        default_factory=dict,
        description="HTML attributes of the element",
    )
    is_interactive: bool = Field(
        default=True,
        alias="isInteractive",
        description="Whether the element is interactive",
    )
    confidence: float = Field(
        default=1.0,
        ge=0.0,
        le=1.0,
        description="Confidence score for element detection",
    )

    model_config = {"populate_by_name": True}


# =============================================================================
# State Annotation (Extracted State)
# =============================================================================


class StateAnnotation(BaseModel):
    """
    A state discovered during extraction.

    States represent distinct UI regions or views identified by their
    visual and semantic properties.
    """

    id: str = Field(..., description="Unique identifier for the state")
    name: str = Field(..., description="Human-readable name for the state")
    bbox: BoundingBox = Field(..., description="Bounding box of the state region")
    state_type: StateType | str = Field(
        default=StateType.UNKNOWN,
        alias="stateType",
        description="Semantic type of the state",
    )
    element_ids: list[str] = Field(
        default_factory=list,
        alias="elementIds",
        description="IDs of elements contained in this state",
    )
    screenshot_id: str | None = Field(
        default=None,
        alias="screenshotId",
        description="ID of the screenshot showing this state",
    )
    source_url: str | None = Field(
        default=None,
        alias="sourceUrl",
        description="URL where this state was discovered",
    )
    detection_method: str | None = Field(
        default=None,
        alias="detectionMethod",
        description="How the state was detected (semantic, heuristic, etc.)",
    )
    confidence: float = Field(
        default=1.0,
        ge=0.0,
        le=1.0,
        description="Confidence score for state detection",
    )
    metadata: dict[str, Any] = Field(
        default_factory=dict,
        description="Additional metadata about the state",
    )

    model_config = {"populate_by_name": True}


# =============================================================================
# Element Annotation (for backend storage)
# =============================================================================


class ElementAnnotation(BaseModel):
    """Element annotation for backend storage."""

    id: str = Field(..., description="Unique identifier")
    name: str | None = Field(
        default=None,
        description="Human-readable name for the element (OCR-based or derived)",
    )
    element_type: str = Field(
        ...,
        alias="elementType",
        description="Type of element",
    )
    bbox: BoundingBox = Field(..., description="Bounding box")
    text: str | None = Field(default=None, description="Text content")
    selector: str | None = Field(default=None, description="CSS selector")
    confidence: float = Field(default=1.0, ge=0.0, le=1.0)

    model_config = {"populate_by_name": True}


# =============================================================================
# Inferred Transition
# =============================================================================


class InferredTransition(BaseModel):
    """
    A transition discovered during extraction.

    Transitions represent navigation or state changes between states,
    triggered by user interactions (clicks, hovers, etc.).
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
    trigger_type: TriggerType | str = Field(
        default=TriggerType.CLICK,
        alias="triggerType",
        description="Type of trigger that causes the transition",
    )
    trigger_selector: str | None = Field(
        default=None,
        alias="triggerSelector",
        description="CSS selector of the trigger element",
    )
    trigger_text: str | None = Field(
        default=None,
        alias="triggerText",
        description="Text of the trigger element (for non-image triggers)",
    )
    trigger_image: str | None = Field(
        default=None,
        alias="triggerImage",
        description="Image URL of the trigger element (for image triggers)",
    )
    has_image: bool = Field(
        default=False,
        alias="hasImage",
        description="Whether the trigger is an image link",
    )
    source_url: str | None = Field(
        default=None,
        alias="sourceUrl",
        description="URL of the source page",
    )
    target_url: str | None = Field(
        default=None,
        alias="targetUrl",
        description="URL of the target page",
    )
    confidence: float = Field(
        default=1.0,
        ge=0.0,
        le=1.0,
        description="Confidence score for transition detection",
    )
    metadata: dict[str, Any] = Field(
        default_factory=dict,
        description="Additional metadata about the transition",
    )

    model_config = {"populate_by_name": True}


# =============================================================================
# Extraction Stats
# =============================================================================


class ExtractionStats(BaseModel):
    """Statistics from an extraction session."""

    pages_extracted: int = Field(
        default=0,
        alias="pagesExtracted",
        description="Number of pages crawled",
    )
    elements_found: int = Field(
        default=0,
        alias="elementsFound",
        description="Number of elements discovered",
    )
    states_found: int = Field(
        default=0,
        alias="statesFound",
        description="Number of states discovered",
    )
    transitions_found: int = Field(
        default=0,
        alias="transitionsFound",
        description="Number of transitions discovered",
    )
    screenshot_extraction_id: str | None = Field(
        default=None,
        alias="screenshotExtractionId",
        description="Extraction ID where screenshots are stored",
    )

    model_config = {"populate_by_name": True}


# =============================================================================
# Extraction Annotation (Page-level)
# =============================================================================


class ExtractionAnnotation(BaseModel):
    """
    Annotation for a single page/screenshot in an extraction.

    Groups states and elements discovered on a specific page at a specific viewport.
    """

    id: str = Field(..., description="Unique identifier")
    session_id: str = Field(
        ...,
        alias="sessionId",
        description="ID of the parent extraction session",
    )
    screenshot_id: str = Field(
        ...,
        alias="screenshotId",
        description="ID of the screenshot for this annotation",
    )
    source_url: str = Field(
        ...,
        alias="sourceUrl",
        description="URL of the page",
    )
    viewport_width: int = Field(
        ...,
        alias="viewportWidth",
        description="Viewport width when screenshot was taken",
    )
    viewport_height: int = Field(
        ...,
        alias="viewportHeight",
        description="Viewport height when screenshot was taken",
    )
    elements: list[ElementAnnotation] = Field(
        default_factory=list,
        description="Elements discovered on this page",
    )
    states: list[StateAnnotation] = Field(
        default_factory=list,
        description="States discovered on this page",
    )
    created_at: datetime | None = Field(
        default=None,
        alias="createdAt",
        description="When this annotation was created",
    )
    updated_at: datetime | None = Field(
        default=None,
        alias="updatedAt",
        description="When this annotation was last updated",
    )

    model_config = {"populate_by_name": True}


# =============================================================================
# Extraction Session
# =============================================================================


class ExtractionSessionConfig(BaseModel):
    """Configuration for an extraction session."""

    viewports: list[tuple[int, int]] = Field(
        default_factory=lambda: [(1920, 1080)],
        description="Viewport sizes to use for extraction",
    )
    capture_hover_states: bool = Field(
        default=True,
        alias="captureHoverStates",
        description="Whether to capture hover states",
    )
    capture_focus_states: bool = Field(
        default=True,
        alias="captureFocusStates",
        description="Whether to capture focus states",
    )
    max_depth: int = Field(
        default=5,
        alias="maxDepth",
        description="Maximum crawl depth",
    )
    max_pages: int = Field(
        default=100,
        alias="maxPages",
        description="Maximum number of pages to crawl",
    )
    auth_cookies: dict[str, str] | None = Field(
        default=None,
        alias="authCookies",
        description="Authentication cookies for the target site",
    )

    model_config = {"populate_by_name": True}


class ExtractionSession(BaseModel):
    """
    An extraction session representing a web extraction job.

    Sessions track the configuration, status, and results of extracting
    states and transitions from web applications.
    """

    id: str = Field(..., description="Unique identifier for the session")
    project_id: str = Field(
        ...,
        alias="projectId",
        description="ID of the project this extraction belongs to",
    )
    source_urls: list[str] = Field(
        ...,
        alias="sourceUrls",
        description="URLs to extract from",
    )
    config: ExtractionSessionConfig | dict[str, Any] = Field(
        ...,
        description="Extraction configuration",
    )
    status: ExtractionStatus | str = Field(
        ...,
        description="Current status of the extraction",
    )
    stats: ExtractionStats | dict[str, Any] = Field(
        ...,
        description="Extraction statistics",
    )
    error_message: str | None = Field(
        default=None,
        alias="errorMessage",
        description="Error message if extraction failed",
    )
    created_at: datetime | str = Field(
        ...,
        alias="createdAt",
        description="When the session was created",
    )
    started_at: datetime | str | None = Field(
        default=None,
        alias="startedAt",
        description="When extraction started",
    )
    completed_at: datetime | str | None = Field(
        default=None,
        alias="completedAt",
        description="When extraction completed",
    )
    created_by: str | None = Field(
        default=None,
        alias="createdBy",
        description="User who created the session",
    )

    model_config = {"populate_by_name": True}


class ExtractionSessionDetail(ExtractionSession):
    """Extraction session with full annotation details."""

    annotations: list[ExtractionAnnotation] = Field(
        default_factory=list,
        description="Page annotations with states and elements",
    )
    transitions: list[InferredTransition] = Field(
        default_factory=list,
        description="Transitions discovered during extraction",
    )


# =============================================================================
# Import Request/Result
# =============================================================================


class StateImportRequest(BaseModel):
    """Request to import extracted states into a project."""

    state_ids: list[str] | None = Field(
        default=None,
        alias="stateIds",
        description="Specific state IDs to import (None = all)",
    )
    target_workflow_id: str | None = Field(
        default=None,
        alias="targetWorkflowId",
        description="Workflow to add states to",
    )

    model_config = {"populate_by_name": True}


class ImportResult(BaseModel):
    """Result of importing states into a project."""

    imported_states: int = Field(
        ...,
        alias="importedStates",
        description="Number of states imported",
    )
    imported_transitions: int = Field(
        ...,
        alias="importedTransitions",
        description="Number of transitions imported",
    )
    workflow_id: str | None = Field(
        default=None,
        alias="workflowId",
        description="ID of the workflow states were added to",
    )

    model_config = {"populate_by_name": True}
