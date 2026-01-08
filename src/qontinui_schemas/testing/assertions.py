"""Vision verification assertion schemas.

Defines schemas for DOM-independent visual assertions using machine vision.
These assertions provide Playwright-like capabilities without DOM access.
"""

from enum import Enum
from typing import Any

from pydantic import BaseModel, ConfigDict, Field

from qontinui_schemas.common.time import UTCDateTime

# =============================================================================
# Enums
# =============================================================================


class LocatorType(str, Enum):
    """Types of vision locators for targeting elements."""

    IMAGE = "image"  # Template image matching
    TEXT = "text"  # OCR text detection
    REGION = "region"  # Coordinate-based region
    STATE = "state"  # Qontinui state machine state
    SEMANTIC = "semantic"  # ML-based semantic detection
    ENVIRONMENT = "environment"  # Environment-aware (uses discovered data)


class AssertionType(str, Enum):
    """Types of visual assertions."""

    VISIBILITY = "visibility"  # toBeVisible, toBeHidden
    TEXT = "text"  # toHaveText, toContainText
    COUNT = "count"  # toHaveCount
    STATE = "state"  # toBeEnabled, toBeChecked, toBeFocused
    ATTRIBUTE = "attribute"  # toHaveColor, toHaveSize
    SPATIAL = "spatial"  # toBeAbove, toBeLeftOf, toBeInside
    SCREENSHOT = "screenshot"  # toMatchScreenshot
    ANIMATION = "animation"  # toStopAnimating, toBeStable


class AssertionStatus(str, Enum):
    """Status of an assertion result."""

    PASSED = "passed"
    FAILED = "failed"
    ERROR = "error"
    SKIPPED = "skipped"
    PENDING = "pending"


class SpatialRelation(str, Enum):
    """Spatial relationship types."""

    ABOVE = "above"
    BELOW = "below"
    LEFT_OF = "left_of"
    RIGHT_OF = "right_of"
    INSIDE = "inside"
    OUTSIDE = "outside"
    NEAR = "near"
    ALIGNED_HORIZONTAL = "aligned_horizontal"
    ALIGNED_VERTICAL = "aligned_vertical"
    OVERLAPPING = "overlapping"


class AssertableState(str, Enum):
    """Visual states that can be asserted on elements."""

    ENABLED = "enabled"
    DISABLED = "disabled"
    CHECKED = "checked"
    UNCHECKED = "unchecked"
    FOCUSED = "focused"
    UNFOCUSED = "unfocused"
    SELECTED = "selected"
    EXPANDED = "expanded"
    COLLAPSED = "collapsed"
    VISIBLE = "visible"
    HIDDEN = "hidden"


# =============================================================================
# Locator Models
# =============================================================================


class BoundingBox(BaseModel):
    """A rectangular region on screen."""

    x: int = Field(..., ge=0, description="X coordinate (left)")
    y: int = Field(..., ge=0, description="Y coordinate (top)")
    width: int = Field(..., gt=0, description="Width in pixels")
    height: int = Field(..., gt=0, description="Height in pixels")


class VisionLocatorConfig(BaseModel):
    """Configuration for a vision locator."""

    type: LocatorType = Field(..., description="Locator type")
    value: str = Field(
        ..., description="Locator value (image path, text, state name, etc.)"
    )
    options: dict[str, Any] = Field(
        default_factory=dict, description="Type-specific options"
    )

    # Image locator options
    threshold: float | None = Field(None, ge=0.0, le=1.0, description="Match threshold")
    scale_invariant: bool = Field(False, description="Allow scale-invariant matching")

    # Text locator options
    exact_match: bool = Field(True, description="Require exact text match")
    case_sensitive: bool = Field(True, description="Case-sensitive matching")
    regex: bool = Field(False, description="Treat value as regex pattern")

    # Region options
    region: BoundingBox | None = Field(None, description="Search within this region")

    # Filtering
    nth: int | None = Field(None, ge=0, description="Select nth match (0-indexed)")
    first: bool = Field(False, description="Select first match only")
    last: bool = Field(False, description="Select last match only")


class VisionLocatorMatch(BaseModel):
    """A matched element location."""

    bounds: BoundingBox = Field(..., description="Element bounding box")
    confidence: float = Field(..., ge=0.0, le=1.0, description="Match confidence")
    center: tuple[int, int] = Field(..., description="Center point (x, y)")
    text: str | None = Field(None, description="Extracted text if applicable")
    locator_type: LocatorType = Field(..., description="Locator type used")
    match_index: int = Field(0, ge=0, description="Index among multiple matches")


# =============================================================================
# Assertion Configuration
# =============================================================================


class AssertionOptions(BaseModel):
    """Common options for all assertions."""

    timeout: int = Field(5000, gt=0, description="Timeout in milliseconds")
    polling_interval: int = Field(100, gt=0, description="Polling interval in ms")
    soft: bool = Field(False, description="Soft assertion (continue on failure)")
    negate: bool = Field(False, description="Negate the assertion")
    message: str | None = Field(None, description="Custom failure message")


class VisibilityAssertionConfig(BaseModel):
    """Configuration for visibility assertions."""

    visible: bool = Field(True, description="Assert visible (True) or hidden (False)")


class TextAssertionConfig(BaseModel):
    """Configuration for text assertions."""

    expected_text: str = Field(..., description="Expected text content")
    exact: bool = Field(True, description="Require exact match vs contains")
    case_sensitive: bool = Field(True, description="Case-sensitive comparison")
    normalize_whitespace: bool = Field(True, description="Normalize whitespace")
    use_regex: bool = Field(False, description="Treat expected_text as regex")


class CountAssertionConfig(BaseModel):
    """Configuration for count assertions."""

    count: int | None = Field(None, ge=0, description="Exact expected count")
    min_count: int | None = Field(None, ge=0, description="Minimum count")
    max_count: int | None = Field(None, ge=0, description="Maximum count")


class StateAssertionConfig(BaseModel):
    """Configuration for element state assertions."""

    expected_state: AssertableState = Field(..., description="Expected element state")
    use_environment: bool = Field(True, description="Use environment-learned states")


class AttributeAssertionConfig(BaseModel):
    """Configuration for attribute assertions."""

    attribute: str = Field(
        ..., description="Attribute to check: color, size, opacity, etc."
    )
    expected_value: Any = Field(..., description="Expected attribute value")
    tolerance: float = Field(0.0, ge=0.0, description="Allowed tolerance")


class SpatialAssertionConfig(BaseModel):
    """Configuration for spatial relationship assertions."""

    relation: SpatialRelation = Field(..., description="Spatial relationship type")
    reference_locator: VisionLocatorConfig = Field(
        ..., description="Reference element locator"
    )
    max_distance: int | None = Field(
        None, ge=0, description="Max distance for 'near' relation"
    )
    tolerance: int = Field(5, ge=0, description="Alignment tolerance in pixels")


class ScreenshotAssertionConfig(BaseModel):
    """Configuration for screenshot comparison assertions."""

    baseline_path: str = Field(..., description="Path to baseline image")
    threshold: float = Field(0.95, ge=0.0, le=1.0, description="Similarity threshold")
    algorithm: str = Field("ssim", description="Comparison algorithm")
    auto_mask_dynamic: bool = Field(
        True, description="Auto-mask discovered dynamic regions"
    )
    ignore_regions: list[BoundingBox] = Field(
        default_factory=list, description="Additional regions to ignore"
    )


class AnimationAssertionConfig(BaseModel):
    """Configuration for animation assertions."""

    stable_duration: int = Field(
        500, gt=0, description="Required stable duration in ms"
    )
    check_region: BoundingBox | None = Field(
        None, description="Region to check for stability"
    )


# =============================================================================
# Assertion Definition
# =============================================================================


class VisionAssertion(BaseModel):
    """A complete vision assertion definition."""

    model_config = ConfigDict(from_attributes=True)

    id: str = Field(..., description="Unique assertion ID")
    name: str | None = Field(None, description="Human-readable name")
    description: str | None = Field(None, description="Assertion description")

    # Target
    locator: VisionLocatorConfig = Field(..., description="Target element locator")

    # Assertion type and config
    assertion_type: AssertionType = Field(..., description="Type of assertion")
    assertion_method: str = Field(
        ..., description="Method name: to_be_visible, to_have_text, etc."
    )

    # Type-specific configuration (one of these)
    visibility_config: VisibilityAssertionConfig | None = Field(None)
    text_config: TextAssertionConfig | None = Field(None)
    count_config: CountAssertionConfig | None = Field(None)
    state_config: StateAssertionConfig | None = Field(None)
    attribute_config: AttributeAssertionConfig | None = Field(None)
    spatial_config: SpatialAssertionConfig | None = Field(None)
    screenshot_config: ScreenshotAssertionConfig | None = Field(None)
    animation_config: AnimationAssertionConfig | None = Field(None)

    # Common options
    options: AssertionOptions = Field(
        default_factory=AssertionOptions, description="Assertion options"
    )

    # Metadata
    created_at: UTCDateTime | None = Field(None, description="Creation timestamp (UTC)")
    updated_at: UTCDateTime | None = Field(
        None, description="Last update timestamp (UTC)"
    )


# =============================================================================
# Assertion Results
# =============================================================================


class AssertionResult(BaseModel):
    """Result of executing a vision assertion."""

    model_config = ConfigDict(from_attributes=True)

    assertion_id: str = Field(..., description="ID of the assertion")
    assertion_method: str = Field(..., description="Method called")
    status: AssertionStatus = Field(..., description="Result status")

    # Timing
    started_at: UTCDateTime = Field(..., description="Start timestamp (UTC)")
    completed_at: UTCDateTime = Field(..., description="Completion timestamp (UTC)")
    duration_ms: int = Field(..., ge=0, description="Execution duration in ms")
    retry_count: int = Field(0, ge=0, description="Number of retries")

    # Match information
    matches_found: int = Field(0, ge=0, description="Number of matches found")
    best_match: VisionLocatorMatch | None = Field(
        None, description="Best match details"
    )
    all_matches: list[VisionLocatorMatch] = Field(
        default_factory=list, description="All matches found"
    )

    # Expected vs actual
    expected_value: Any | None = Field(None, description="Expected value")
    actual_value: Any | None = Field(None, description="Actual value")

    # Failure details
    error_message: str | None = Field(None, description="Error message if failed")
    error_details: dict[str, Any] = Field(
        default_factory=dict, description="Additional error details"
    )
    suggestion: str | None = Field(None, description="Suggestion for fixing the issue")

    # Screenshots
    screenshot_path: str | None = Field(
        None, description="Screenshot at assertion time"
    )
    annotated_screenshot_path: str | None = Field(
        None, description="Annotated screenshot"
    )
    diff_screenshot_path: str | None = Field(
        None, description="Diff image for comparisons"
    )


class AssertionSuiteResult(BaseModel):
    """Result of executing multiple assertions."""

    model_config = ConfigDict(from_attributes=True)

    suite_id: str = Field(..., description="Suite execution ID")
    suite_name: str | None = Field(None, description="Suite name")

    # Timing
    started_at: UTCDateTime = Field(..., description="Start timestamp (UTC)")
    completed_at: UTCDateTime = Field(..., description="Completion timestamp (UTC)")
    total_duration_ms: int = Field(..., ge=0, description="Total execution time")

    # Results
    results: list[AssertionResult] = Field(
        ..., description="Individual assertion results"
    )

    # Summary
    total_assertions: int = Field(..., ge=0, description="Total assertions")
    passed: int = Field(0, ge=0, description="Passed count")
    failed: int = Field(0, ge=0, description="Failed count")
    errors: int = Field(0, ge=0, description="Error count")
    skipped: int = Field(0, ge=0, description="Skipped count")
    pass_rate: float = Field(0.0, ge=0.0, le=1.0, description="Pass rate (0-1)")

    # Environment used
    environment_id: str | None = Field(None, description="GUI environment ID if used")


# =============================================================================
# Vision Test Definition
# =============================================================================


class VisionTestConfig(BaseModel):
    """Configuration for a vision verification test."""

    # Detection settings
    template_threshold: float = Field(
        0.8, ge=0.0, le=1.0, description="Default template match threshold"
    )
    ocr_confidence_threshold: float = Field(
        0.7, ge=0.0, le=1.0, description="OCR confidence threshold"
    )
    ocr_language: str = Field("eng", description="OCR language code")

    # Wait settings
    default_timeout: int = Field(5000, gt=0, description="Default timeout in ms")
    polling_interval: int = Field(100, gt=0, description="Default polling interval")

    # Screenshot settings
    screenshot_on_failure: bool = Field(
        True, description="Capture screenshot on failure"
    )
    annotate_failures: bool = Field(True, description="Annotate failure screenshots")

    # Comparison settings
    ssim_threshold: float = Field(0.95, ge=0.0, le=1.0, description="SSIM threshold")
    color_tolerance: int = Field(10, ge=0, description="Color tolerance (0-255)")
    position_tolerance: int = Field(5, ge=0, description="Position tolerance in pixels")

    # Environment
    use_environment: bool = Field(True, description="Use discovered GUI environment")
    environment_path: str | None = Field(None, description="Path to environment JSON")


class VisionTest(BaseModel):
    """A complete vision verification test definition."""

    model_config = ConfigDict(from_attributes=True)

    id: str = Field(..., description="Test ID")
    name: str = Field(..., description="Test name")
    description: str | None = Field(None, description="Test description")

    # Assertions
    assertions: list[VisionAssertion] = Field(..., description="Assertions to execute")

    # Configuration
    config: VisionTestConfig = Field(
        default_factory=VisionTestConfig, description="Test configuration"
    )

    # Baselines
    baseline_screenshots: dict[str, str] = Field(
        default_factory=dict, description="Baseline screenshot paths"
    )

    # Metadata
    tags: list[str] = Field(default_factory=list, description="Test tags")
    created_at: UTCDateTime | None = Field(None)
    updated_at: UTCDateTime | None = Field(None)


__all__ = [
    # Enums
    "LocatorType",
    "AssertionType",
    "AssertionStatus",
    "SpatialRelation",
    "AssertableState",
    # Locator models
    "BoundingBox",
    "VisionLocatorConfig",
    "VisionLocatorMatch",
    # Assertion configs
    "AssertionOptions",
    "VisibilityAssertionConfig",
    "TextAssertionConfig",
    "CountAssertionConfig",
    "StateAssertionConfig",
    "AttributeAssertionConfig",
    "SpatialAssertionConfig",
    "ScreenshotAssertionConfig",
    "AnimationAssertionConfig",
    # Assertion definition
    "VisionAssertion",
    # Results
    "AssertionResult",
    "AssertionSuiteResult",
    # Test definition
    "VisionTestConfig",
    "VisionTest",
]
