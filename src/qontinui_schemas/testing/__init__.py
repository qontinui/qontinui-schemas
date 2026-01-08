"""Testing schemas module.

Provides schemas for testing-related features:
- Visual regression testing
- Coverage tracking
- Test analytics
- GUI Environment Discovery
- Vision Verification Assertions

Usage:
    from qontinui_schemas.testing import (
        VisualBaselineCreate, VisualComparisonResponse,
        CoverageUpdate, CoverageTrendResponse,
        GUIEnvironment, ColorPalette, Typography,
        VisionAssertion, AssertionResult, VisionTest,
    )
"""

# Vision Verification Assertion schemas
from qontinui_schemas.testing.assertions import (  # Enums; Locator models; Assertion configs; Assertion definition; Results; Test definition
    AnimationAssertionConfig,
    AssertableState,
    AssertionOptions,
    AssertionResult,
    AssertionStatus,
    AssertionSuiteResult,
    AssertionType,
    AttributeAssertionConfig,
    CountAssertionConfig,
    LocatorType,
    ScreenshotAssertionConfig,
    SpatialAssertionConfig,
    SpatialRelation,
    StateAssertionConfig,
    TextAssertionConfig,
    VisibilityAssertionConfig,
    VisionAssertion,
    VisionLocatorConfig,
    VisionLocatorMatch,
    VisionTest,
    VisionTestConfig,
)

# Coverage schemas
from qontinui_schemas.testing.coverage import (
    CoverageData,
    CoverageGap,
    CoverageGapsResponse,
    CoverageHeatmapCell,
    CoverageHeatmapResponse,
    CoverageSnapshot,
    CoverageTrendDataPoint,
    CoverageTrendResponse,
    CoverageUpdate,
    CoverageUpdateResponse,
)

# GUI Environment Discovery schemas
from qontinui_schemas.testing.environment import (  # Enums; Color models; Typography models; Layout models; Dynamic region models; Visual state models; Element pattern models; Main model; API models
    AlignmentGuide,
    AnimationRegion,
    AnimationType,
    BoundingBox,
    ChangeFrequency,
    ColorPalette,
    ColorProfile,
    ConfidenceScores,
    DetectedFont,
    DisabledCharacteristics,
    DiscoveryProgress,
    DiscoveryRequest,
    DynamicRegion,
    DynamicRegions,
    ElementPattern,
    ElementPatterns,
    ElementSample,
    ElementShape,
    ElementState,
    ElementStateType,
    ElementTypeStates,
    FontFamily,
    FontWeight,
    GridConfiguration,
    GUIEnvironment,
    GUIEnvironmentCreate,
    GUIEnvironmentUpdate,
    Layout,
    LayoutRegion,
    RegionCharacteristics,
    SemanticColors,
    SemanticRegionType,
    SizeRange,
    StateDetectionMethod,
    TextRegion,
    TextSizes,
    TextStyle,
    ThemeType,
    Typography,
    VisualSignature,
    VisualStates,
)

# Visual regression schemas
from qontinui_schemas.testing.visual import (
    ComparisonReview,
    ComparisonSettings,
    ComparisonStats,
    DiffRegion,
    IgnoreRegion,
    VisualBaselineCreate,
    VisualBaselineFromScreenshot,
    VisualBaselineListResponse,
    VisualBaselineResponse,
    VisualBaselineUpdate,
    VisualComparisonCreate,
    VisualComparisonDetail,
    VisualComparisonListResponse,
    VisualComparisonResponse,
    VisualComparisonSummary,
)

__all__ = [
    # Visual regression
    "IgnoreRegion",
    "ComparisonSettings",
    "VisualBaselineCreate",
    "VisualBaselineFromScreenshot",
    "VisualBaselineUpdate",
    "VisualBaselineResponse",
    "VisualBaselineListResponse",
    "DiffRegion",
    "VisualComparisonCreate",
    "VisualComparisonResponse",
    "VisualComparisonDetail",
    "VisualComparisonListResponse",
    "VisualComparisonSummary",
    "ComparisonReview",
    "ComparisonStats",
    # Coverage
    "CoverageData",
    "CoverageSnapshot",
    "CoverageUpdate",
    "CoverageUpdateResponse",
    "CoverageTrendDataPoint",
    "CoverageTrendResponse",
    "CoverageGap",
    "CoverageGapsResponse",
    "CoverageHeatmapCell",
    "CoverageHeatmapResponse",
    # GUI Environment Discovery - Enums
    "ThemeType",
    "FontFamily",
    "FontWeight",
    "ChangeFrequency",
    "AnimationType",
    "ElementStateType",
    "ElementShape",
    "SemanticRegionType",
    "StateDetectionMethod",
    # GUI Environment Discovery - Color models
    "DisabledCharacteristics",
    "SemanticColors",
    "ColorPalette",
    # GUI Environment Discovery - Typography models
    "BoundingBox",
    "DetectedFont",
    "TextSizes",
    "TextRegion",
    "Typography",
    # GUI Environment Discovery - Layout models
    "RegionCharacteristics",
    "LayoutRegion",
    "GridConfiguration",
    "AlignmentGuide",
    "Layout",
    # GUI Environment Discovery - Dynamic region models
    "DynamicRegion",
    "AnimationRegion",
    "DynamicRegions",
    # GUI Environment Discovery - Visual state models
    "ColorProfile",
    "VisualSignature",
    "ElementState",
    "ElementTypeStates",
    "VisualStates",
    # GUI Environment Discovery - Element pattern models
    "SizeRange",
    "TextStyle",
    "ElementSample",
    "ElementPattern",
    "ElementPatterns",
    # GUI Environment Discovery - Main model
    "ConfidenceScores",
    "GUIEnvironment",
    # GUI Environment Discovery - API models
    "GUIEnvironmentCreate",
    "GUIEnvironmentUpdate",
    "DiscoveryRequest",
    "DiscoveryProgress",
    # Vision Verification Assertions - Enums
    "LocatorType",
    "AssertionType",
    "AssertionStatus",
    "SpatialRelation",
    "AssertableState",
    # Vision Verification Assertions - Locator models
    "VisionLocatorConfig",
    "VisionLocatorMatch",
    # Vision Verification Assertions - Configs
    "AssertionOptions",
    "VisibilityAssertionConfig",
    "TextAssertionConfig",
    "CountAssertionConfig",
    "StateAssertionConfig",
    "AttributeAssertionConfig",
    "SpatialAssertionConfig",
    "ScreenshotAssertionConfig",
    "AnimationAssertionConfig",
    # Vision Verification Assertions - Definition
    "VisionAssertion",
    # Vision Verification Assertions - Results
    "AssertionResult",
    "AssertionSuiteResult",
    # Vision Verification Assertions - Test
    "VisionTestConfig",
    "VisionTest",
]
