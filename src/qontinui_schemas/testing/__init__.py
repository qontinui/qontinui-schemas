"""Testing schemas module.

Provides schemas for testing-related features:
- Visual regression testing
- Coverage tracking
- Test analytics

Usage:
    from qontinui_schemas.testing import (
        VisualBaselineCreate, VisualComparisonResponse,
        CoverageUpdate, CoverageTrendResponse,
    )
"""

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
]
