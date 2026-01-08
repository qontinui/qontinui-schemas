"""GUI Environment Discovery schemas.

Defines schemas for automatically discovered GUI environment characteristics
including color palettes, typography, layout regions, dynamic regions,
visual states, and element patterns.

These schemas enable environment-aware visual verification without requiring
extensive pre-stored template images.
"""

from enum import Enum
from typing import Any

from pydantic import BaseModel, ConfigDict, Field

from qontinui_schemas.common.time import UTCDateTime

# =============================================================================
# Enums
# =============================================================================


class ThemeType(str, Enum):
    """Detected application theme type based on luminance distribution."""

    DARK = "dark"
    LIGHT = "light"
    MIXED = "mixed"


class FontFamily(str, Enum):
    """Heuristic font family classification based on character analysis."""

    SANS_SERIF = "sans-serif"
    SERIF = "serif"
    MONOSPACE = "monospace"
    DISPLAY = "display"
    UNKNOWN = "unknown"


class FontWeight(str, Enum):
    """Font weight estimation based on stroke width analysis."""

    THIN = "thin"
    LIGHT = "light"
    NORMAL = "normal"
    MEDIUM = "medium"
    SEMIBOLD = "semibold"
    BOLD = "bold"
    EXTRA_BOLD = "extra-bold"


class ChangeFrequency(str, Enum):
    """How frequently a dynamic region changes."""

    CONTINUOUS = "continuous"  # Changes every frame
    PERIODIC = "periodic"  # Changes at intervals
    ON_ACTION = "on_action"  # Changes after user action
    RARE = "rare"  # Changes infrequently


class AnimationType(str, Enum):
    """Detected animation pattern type."""

    SPINNER = "spinner"  # Rotation animation
    PROGRESS = "progress"  # Linear progress bar
    PULSE = "pulse"  # Opacity oscillation
    SLIDE = "slide"  # Translation animation
    FADE = "fade"  # Fade in/out
    UNKNOWN = "unknown"


class ElementStateType(str, Enum):
    """Visual state types for UI elements."""

    ENABLED = "enabled"
    DISABLED = "disabled"
    HOVER = "hover"
    PRESSED = "pressed"
    FOCUSED = "focused"
    UNFOCUSED = "unfocused"
    CHECKED = "checked"
    UNCHECKED = "unchecked"
    SELECTED = "selected"
    UNSELECTED = "unselected"
    EXPANDED = "expanded"
    COLLAPSED = "collapsed"


class ElementShape(str, Enum):
    """Detected element shape classification."""

    RECTANGLE = "rectangle"
    ROUNDED_RECTANGLE = "rounded_rectangle"
    CIRCLE = "circle"
    OVAL = "oval"
    IRREGULAR = "irregular"


class SemanticRegionType(str, Enum):
    """Inferred semantic region types based on position and content."""

    HEADER = "header"
    FOOTER = "footer"
    SIDEBAR = "sidebar"
    MAIN_CONTENT = "main_content"
    NAVIGATION = "navigation"
    TOOLBAR = "toolbar"
    MODAL = "modal"
    TOAST = "toast"
    UNKNOWN = "unknown"


class StateDetectionMethod(str, Enum):
    """Method used to detect element states."""

    COLOR_SATURATION = "color_saturation"
    OPACITY = "opacity"
    INTERIOR_FILL = "interior_fill"
    BORDER_ANALYSIS = "border_analysis"
    PATTERN_MATCH = "pattern_match"
    COMBINED = "combined"


# =============================================================================
# Color Palette Models
# =============================================================================


class DisabledCharacteristics(BaseModel):
    """Learned characteristics of disabled element states."""

    saturation_reduction: float = Field(
        ..., ge=0.0, le=1.0, description="Measured saturation reduction factor"
    )
    opacity_reduction: float = Field(
        ..., ge=0.0, le=1.0, description="Measured opacity reduction factor"
    )
    brightness_change: float | None = Field(
        None, description="Brightness change (negative = darker)"
    )
    observed_samples: int = Field(
        0, ge=0, description="Number of observations used to learn this"
    )


class SemanticColors(BaseModel):
    """Semantically meaningful colors extracted from the application."""

    background: str | None = Field(
        None, pattern=r"^#[0-9A-Fa-f]{6}$", description="Primary background color"
    )
    surface: str | None = Field(
        None, pattern=r"^#[0-9A-Fa-f]{6}$", description="Surface/card background"
    )
    text_primary: str | None = Field(
        None, pattern=r"^#[0-9A-Fa-f]{6}$", description="Primary text color"
    )
    text_secondary: str | None = Field(
        None, pattern=r"^#[0-9A-Fa-f]{6}$", description="Secondary text color"
    )
    accent: str | None = Field(
        None, pattern=r"^#[0-9A-Fa-f]{6}$", description="Accent/brand color"
    )
    error: str | None = Field(
        None,
        pattern=r"^#[0-9A-Fa-f]{6}$",
        description="Error color (near 'error', 'failed' text)",
    )
    success: str | None = Field(
        None,
        pattern=r"^#[0-9A-Fa-f]{6}$",
        description="Success color (near 'success', 'done' text)",
    )
    warning: str | None = Field(
        None, pattern=r"^#[0-9A-Fa-f]{6}$", description="Warning color"
    )
    info: str | None = Field(
        None, pattern=r"^#[0-9A-Fa-f]{6}$", description="Info/notice color"
    )
    border: str | None = Field(
        None, pattern=r"^#[0-9A-Fa-f]{6}$", description="Common border color"
    )


class ColorPalette(BaseModel):
    """Complete color palette discovered from the application."""

    dominant_colors: list[str] = Field(
        default_factory=list,
        description="List of dominant colors (hex) from k-means clustering",
    )
    semantic_colors: SemanticColors = Field(
        default_factory=lambda: SemanticColors(), description="Semantically associated colors"
    )
    theme_type: ThemeType = Field(
        ThemeType.DARK, description="Detected theme type based on luminance"
    )
    disabled_characteristics: DisabledCharacteristics | None = Field(
        None, description="Learned disabled state visual characteristics"
    )
    screenshots_analyzed: int = Field(
        0, ge=0, description="Number of screenshots used for analysis"
    )
    confidence: float = Field(
        0.0, ge=0.0, le=1.0, description="Confidence score for extraction"
    )


# =============================================================================
# Typography Models
# =============================================================================


class BoundingBox(BaseModel):
    """A rectangular region defined by position and size."""

    x: int = Field(..., ge=0, description="X coordinate (left)")
    y: int = Field(..., ge=0, description="Y coordinate (top)")
    width: int = Field(..., gt=0, description="Width in pixels")
    height: int = Field(..., gt=0, description="Height in pixels")


class DetectedFont(BaseModel):
    """A detected font sample from OCR analysis."""

    sample_region: BoundingBox = Field(
        ..., description="Region where font was detected"
    )
    estimated_family: FontFamily = Field(
        FontFamily.UNKNOWN, description="Heuristic font family classification"
    )
    size_px: int = Field(..., gt=0, description="Estimated font size in pixels")
    weight: FontWeight = Field(FontWeight.NORMAL, description="Estimated font weight")
    line_height: float | None = Field(
        None, gt=0, description="Calculated line height ratio"
    )
    letter_spacing: float | None = Field(
        None, description="Estimated letter spacing in pixels"
    )
    sample_text: str | None = Field(None, description="Sample text from this detection")


class TextSizes(BaseModel):
    """Semantic text size mappings discovered from the application."""

    heading_large: int | None = Field(None, gt=0, description="Largest heading size")
    heading: int | None = Field(None, gt=0, description="Standard heading size")
    heading_small: int | None = Field(None, gt=0, description="Small heading size")
    body: int | None = Field(None, gt=0, description="Body text size (most common)")
    small: int | None = Field(None, gt=0, description="Small text size")
    tiny: int | None = Field(None, gt=0, description="Smallest text size")


class TextRegion(BaseModel):
    """A commonly occurring text region in the application."""

    name: str = Field(..., description="Descriptive name for this region")
    bounds: BoundingBox = Field(..., description="Region bounds")
    avg_size: int = Field(..., gt=0, description="Average text size in this region")
    typical_content: list[str] | None = Field(
        None, description="Sample text content found in this region"
    )


class Typography(BaseModel):
    """Complete typography information discovered from the application."""

    detected_fonts: list[DetectedFont] = Field(
        default_factory=list, description="Font samples detected via OCR"
    )
    text_sizes: TextSizes = Field(
        default_factory=lambda: TextSizes(), description="Semantic text size mappings"
    )
    languages_detected: list[str] = Field(
        default_factory=list, description="Detected languages (ISO 639-1 codes)"
    )
    common_text_regions: list[TextRegion] = Field(
        default_factory=list, description="Commonly occurring text regions"
    )
    screenshots_analyzed: int = Field(
        0, ge=0, description="Number of screenshots analyzed"
    )
    confidence: float = Field(
        0.0, ge=0.0, le=1.0, description="Confidence score for extraction"
    )


# =============================================================================
# Layout Models
# =============================================================================


class RegionCharacteristics(BaseModel):
    """Visual characteristics of a detected layout region."""

    background_color: str | None = Field(
        None, pattern=r"^#[0-9A-Fa-f]{6}$", description="Dominant background color"
    )
    contains_logo: bool = Field(
        False, description="Whether region contains a logo/image"
    )
    text_content: list[str] = Field(
        default_factory=list, description="Text strings found in region"
    )
    element_count: int = Field(0, ge=0, description="Number of detected elements")
    has_vertical_list: bool = Field(
        False, description="Whether region contains a vertical list"
    )
    has_horizontal_list: bool = Field(
        False, description="Whether region contains a horizontal list"
    )
    is_scrollable: bool = Field(
        False, description="Whether region is scrollable (detected via scroll action)"
    )
    content_varies: bool = Field(
        False, description="Whether content varies significantly across screenshots"
    )


class LayoutRegion(BaseModel):
    """A detected layout region in the application."""

    id: str = Field(..., description="Unique identifier for this region")
    bounds: BoundingBox = Field(..., description="Region bounds")
    characteristics: RegionCharacteristics = Field(
        default_factory=lambda: RegionCharacteristics(), description="Region characteristics"
    )
    semantic_label: SemanticRegionType = Field(
        SemanticRegionType.UNKNOWN, description="Inferred semantic type"
    )
    stability: float = Field(
        0.0, ge=0.0, le=1.0, description="How stable this region is across screenshots"
    )


class GridConfiguration(BaseModel):
    """Detected grid system configuration."""

    detected: bool = Field(False, description="Whether a grid system was detected")
    columns: int | None = Field(None, gt=0, description="Number of columns")
    gutter: int | None = Field(None, ge=0, description="Gutter width in pixels")
    margin: int | None = Field(None, ge=0, description="Margin width in pixels")
    row_height: int | None = Field(
        None, gt=0, description="Base row height if detected"
    )


class AlignmentGuide(BaseModel):
    """A detected alignment guide (vertical or horizontal line)."""

    type: str = Field(..., pattern=r"^(vertical|horizontal)$", description="Guide type")
    position: int = Field(
        ..., ge=0, description="X position (vertical) or Y position (horizontal)"
    )
    confidence: float = Field(0.0, ge=0.0, le=1.0, description="Detection confidence")


class Layout(BaseModel):
    """Complete layout information discovered from the application."""

    regions: dict[str, LayoutRegion] = Field(
        default_factory=dict, description="Named layout regions"
    )
    grid: GridConfiguration = Field(
        default_factory=lambda: GridConfiguration(), description="Detected grid system"
    )
    alignment_guides: list[AlignmentGuide] = Field(
        default_factory=list, description="Detected alignment guides"
    )
    screen_resolution: tuple[int, int] | None = Field(
        None, description="Screen resolution (width, height)"
    )
    screenshots_analyzed: int = Field(
        0, ge=0, description="Number of screenshots analyzed"
    )
    confidence: float = Field(
        0.0, ge=0.0, le=1.0, description="Confidence score for extraction"
    )


# =============================================================================
# Dynamic Region Models
# =============================================================================


class DynamicRegion(BaseModel):
    """A region that changes dynamically."""

    bounds: BoundingBox = Field(..., description="Region bounds")
    change_frequency: ChangeFrequency = Field(
        ..., description="How frequently the region changes"
    )
    pattern: str | None = Field(
        None, description="Detected pattern (e.g., 'timestamp', 'counter')"
    )
    regex_pattern: str | None = Field(
        None, description="Regex pattern that matches content"
    )
    auto_mask: bool = Field(True, description="Whether to auto-mask during comparisons")


class AnimationRegion(BaseModel):
    """A region containing an animation."""

    bounds: BoundingBox = Field(..., description="Region bounds")
    animation_type: AnimationType = Field(
        AnimationType.UNKNOWN, description="Detected animation type"
    )
    duration_ms: int | None = Field(
        None, ge=0, description="Animation duration if finite"
    )
    is_continuous: bool = Field(
        False, description="Whether animation runs continuously"
    )


class DynamicRegions(BaseModel):
    """All detected dynamic regions in the application."""

    always_changing: list[DynamicRegion] = Field(
        default_factory=list,
        description="Regions that always change (timestamps, etc.)",
    )
    conditionally_changing: list[DynamicRegion] = Field(
        default_factory=list, description="Regions that change based on actions"
    )
    animation_regions: list[AnimationRegion] = Field(
        default_factory=list, description="Regions containing animations"
    )
    idle_frames_analyzed: int = Field(
        0, ge=0, description="Number of idle frames analyzed"
    )
    action_pairs_analyzed: int = Field(
        0, ge=0, description="Number of action before/after pairs analyzed"
    )
    confidence: float = Field(
        0.0, ge=0.0, le=1.0, description="Confidence score for detection"
    )


# =============================================================================
# Visual State Models
# =============================================================================


class ColorProfile(BaseModel):
    """Color profile for a visual state."""

    background: str | None = Field(
        None, pattern=r"^#[0-9A-Fa-f]{6}$", description="Background color"
    )
    foreground: str | None = Field(
        None, pattern=r"^#[0-9A-Fa-f]{6}$", description="Foreground/text color"
    )
    border: str | None = Field(
        None, pattern=r"^#[0-9A-Fa-f]{6}$", description="Border color"
    )
    saturation: float | None = Field(
        None, ge=0.0, le=1.0, description="Relative saturation"
    )
    brightness: float | None = Field(
        None, ge=0.0, le=2.0, description="Relative brightness (1.0 = normal)"
    )


class VisualSignature(BaseModel):
    """Visual signature for element state detection."""

    color_profile: ColorProfile | None = Field(
        None, description="Color characteristics"
    )
    has_checkmark: bool | None = Field(None, description="For checkboxes")
    fill_percentage: float | None = Field(
        None, ge=0.0, le=1.0, description="Interior fill percentage"
    )
    border_width: int | None = Field(None, ge=0, description="Border width in pixels")
    has_glow: bool | None = Field(None, description="Whether element has glow effect")
    glow_color: str | None = Field(None, description="Glow color if present")
    opacity: float | None = Field(None, ge=0.0, le=1.0, description="Element opacity")


class ElementState(BaseModel):
    """A learned visual state for an element type."""

    state_type: ElementStateType = Field(..., description="Type of state")
    observed_samples: int = Field(0, ge=0, description="Number of observations")
    visual_signature: VisualSignature = Field(
        default_factory=lambda: VisualSignature(), description="Visual characteristics"
    )
    confidence: float = Field(0.0, ge=0.0, le=1.0, description="Detection confidence")


class ElementTypeStates(BaseModel):
    """All learned states for an element type (e.g., button, checkbox)."""

    element_type: str = Field(..., description="Element type name")
    states: dict[str, ElementState] = Field(
        default_factory=dict, description="State name to ElementState mapping"
    )
    detection_method: StateDetectionMethod = Field(
        StateDetectionMethod.COMBINED,
        description="Best detection method for this element",
    )
    total_observations: int = Field(
        0, ge=0, description="Total observations across all states"
    )


class VisualStates(BaseModel):
    """All learned visual states for the application."""

    element_states: dict[str, ElementTypeStates] = Field(
        default_factory=dict,
        description="Element type to states mapping (e.g., 'button', 'checkbox')",
    )
    actions_observed: int = Field(
        0, ge=0, description="Number of actions observed for learning"
    )
    confidence: float = Field(
        0.0, ge=0.0, le=1.0, description="Overall confidence score"
    )


# =============================================================================
# Element Pattern Models
# =============================================================================


class SizeRange(BaseModel):
    """A range of sizes for an element dimension."""

    min: int = Field(..., ge=0, description="Minimum size")
    max: int = Field(..., ge=0, description="Maximum size")


class TextStyle(BaseModel):
    """Text styling within an element."""

    case: str | None = Field(
        None,
        pattern=r"^(uppercase|lowercase|mixed|capitalize)$",
        description="Text case pattern",
    )
    alignment: str | None = Field(
        None,
        pattern=r"^(left|center|right)$",
        description="Text alignment",
    )
    truncated: bool = Field(False, description="Whether text appears truncated")


class ElementSample(BaseModel):
    """A captured sample of an element for pattern matching."""

    image_path: str | None = Field(None, description="Path to saved image sample")
    image_hash: str | None = Field(None, description="Perceptual hash of sample")
    text: str | None = Field(None, description="Text content if any")
    bounds: BoundingBox | None = Field(None, description="Original bounds")


class ElementPattern(BaseModel):
    """A detected UI element pattern."""

    element_type: str = Field(
        ..., description="Element type name (button, input, etc.)"
    )
    typical_width: SizeRange | None = Field(None, description="Typical width range")
    typical_height: SizeRange | None = Field(None, description="Typical height range")
    typical_colors: list[str] = Field(
        default_factory=list, description="Common colors (hex)"
    )
    shape: ElementShape = Field(ElementShape.RECTANGLE, description="Detected shape")
    corner_radius: int | None = Field(
        None, ge=0, description="Corner radius if rounded"
    )
    has_shadow: bool = Field(False, description="Whether element has shadow")
    has_border: bool = Field(False, description="Whether element has visible border")
    border_width: int | None = Field(None, ge=0, description="Border width if present")
    padding: int | None = Field(None, ge=0, description="Internal padding")
    text_style: TextStyle | None = Field(None, description="Text styling")
    examples: list[ElementSample] = Field(
        default_factory=list, description="Sample images of this element"
    )
    detection_count: int = Field(0, ge=0, description="Number of times detected")
    confidence: float = Field(0.0, ge=0.0, le=1.0, description="Pattern confidence")


class ElementPatterns(BaseModel):
    """All detected element patterns in the application."""

    patterns: dict[str, ElementPattern] = Field(
        default_factory=dict, description="Element type to pattern mapping"
    )
    screenshots_analyzed: int = Field(
        0, ge=0, description="Number of screenshots analyzed"
    )
    elements_detected: int = Field(0, ge=0, description="Total elements detected")
    confidence: float = Field(
        0.0, ge=0.0, le=1.0, description="Overall confidence score"
    )


# =============================================================================
# Main GUIEnvironment Model
# =============================================================================


class ConfidenceScores(BaseModel):
    """Confidence scores for each discovery component."""

    color_extraction: float = Field(0.0, ge=0.0, le=1.0)
    typography_detection: float = Field(0.0, ge=0.0, le=1.0)
    layout_analysis: float = Field(0.0, ge=0.0, le=1.0)
    dynamic_detection: float = Field(0.0, ge=0.0, le=1.0)
    state_learning: float = Field(0.0, ge=0.0, le=1.0)
    element_detection: float = Field(0.0, ge=0.0, le=1.0)


class GUIEnvironment(BaseModel):
    """Complete GUI environment model discovered from an application.

    This model aggregates all discovered visual characteristics enabling
    environment-aware visual verification without requiring extensive
    pre-stored template images.
    """

    model_config = ConfigDict(from_attributes=True)

    # Metadata
    version: str = Field("1.0.0", description="Schema version")
    app_identifier: str | None = Field(None, description="Application identifier")
    discovery_timestamp: UTCDateTime | None = Field(
        None, description="When discovery was performed (UTC)"
    )
    screen_resolution: tuple[int, int] | None = Field(
        None, description="Screen resolution (width, height)"
    )
    screenshots_analyzed: int = Field(0, ge=0, description="Total screenshots analyzed")
    actions_observed: int = Field(0, ge=0, description="Total actions observed")

    # Discovered data
    colors: ColorPalette = Field(
        default_factory=lambda: ColorPalette(), description="Discovered color palette"
    )
    typography: Typography = Field(
        default_factory=lambda: Typography(), description="Discovered typography"
    )
    layout: Layout = Field(default_factory=lambda: Layout(), description="Discovered layout")
    dynamic_regions: DynamicRegions = Field(
        default_factory=lambda: DynamicRegions(), description="Discovered dynamic regions"
    )
    visual_states: VisualStates = Field(
        default_factory=lambda: VisualStates(), description="Learned visual states"
    )
    element_patterns: ElementPatterns = Field(
        default_factory=lambda: ElementPatterns(), description="Detected element patterns"
    )

    # Quality metrics
    confidence_scores: ConfidenceScores = Field(
        default_factory=lambda: ConfidenceScores(), description="Per-component confidence scores"
    )

    # Continuous learning
    continuous_learning_enabled: bool = Field(
        False, description="Whether continuous learning is enabled"
    )
    last_updated: UTCDateTime | None = Field(
        None, description="Last update timestamp (UTC)"
    )
    update_count: int = Field(0, ge=0, description="Number of updates applied")


# =============================================================================
# API Models
# =============================================================================


class GUIEnvironmentCreate(BaseModel):
    """Request schema for creating/initializing a GUI environment."""

    app_identifier: str = Field(
        ..., max_length=200, description="Application identifier"
    )
    screen_resolution: tuple[int, int] | None = Field(
        None, description="Screen resolution (width, height)"
    )
    enable_continuous_learning: bool = Field(
        False, description="Enable continuous learning"
    )


class GUIEnvironmentUpdate(BaseModel):
    """Request schema for updating GUI environment settings."""

    continuous_learning_enabled: bool | None = Field(
        None, description="Enable/disable continuous learning"
    )
    app_identifier: str | None = Field(
        None, max_length=200, description="Update app identifier"
    )


class DiscoveryRequest(BaseModel):
    """Request schema for running environment discovery."""

    screenshots: list[str] = Field(
        ..., min_length=1, description="List of screenshot paths or base64 data"
    )
    include_colors: bool = Field(True, description="Analyze colors")
    include_typography: bool = Field(True, description="Analyze typography")
    include_layout: bool = Field(True, description="Analyze layout")
    include_dynamic: bool = Field(True, description="Detect dynamic regions")
    include_states: bool = Field(
        False, description="Learn visual states (requires actions)"
    )
    include_elements: bool = Field(True, description="Detect element patterns")
    actions: list[dict[str, Any]] | None = Field(
        None, description="Actions performed (for state learning)"
    )


class DiscoveryProgress(BaseModel):
    """Progress update during environment discovery."""

    phase: str = Field(..., description="Current phase")
    progress: float = Field(..., ge=0.0, le=1.0, description="Progress (0-1)")
    message: str | None = Field(None, description="Status message")
    screenshots_processed: int = Field(0, description="Screenshots processed")
    total_screenshots: int = Field(0, description="Total screenshots")


__all__ = [
    # Enums
    "ThemeType",
    "FontFamily",
    "FontWeight",
    "ChangeFrequency",
    "AnimationType",
    "ElementStateType",
    "ElementShape",
    "SemanticRegionType",
    "StateDetectionMethod",
    # Color models
    "DisabledCharacteristics",
    "SemanticColors",
    "ColorPalette",
    # Typography models
    "BoundingBox",
    "DetectedFont",
    "TextSizes",
    "TextRegion",
    "Typography",
    # Layout models
    "RegionCharacteristics",
    "LayoutRegion",
    "GridConfiguration",
    "AlignmentGuide",
    "Layout",
    # Dynamic region models
    "DynamicRegion",
    "AnimationRegion",
    "DynamicRegions",
    # Visual state models
    "ColorProfile",
    "VisualSignature",
    "ElementState",
    "ElementTypeStates",
    "VisualStates",
    # Element pattern models
    "SizeRange",
    "TextStyle",
    "ElementSample",
    "ElementPattern",
    "ElementPatterns",
    # Main model
    "ConfidenceScores",
    "GUIEnvironment",
    # API models
    "GUIEnvironmentCreate",
    "GUIEnvironmentUpdate",
    "DiscoveryRequest",
    "DiscoveryProgress",
]
