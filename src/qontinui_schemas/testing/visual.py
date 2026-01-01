"""Visual regression testing schemas.

Defines schemas for visual comparison baselines and results.
Used for detecting visual regressions in UI automation.
"""

from typing import Any
from uuid import UUID

from pydantic import BaseModel, ConfigDict, Field

from qontinui_schemas.common.time import UTCDateTime


class IgnoreRegion(BaseModel):
    """A region to ignore during visual comparison.

    Used to exclude dynamic areas (timestamps, ads, etc.).
    """

    x: int = Field(..., ge=0, description="X coordinate")
    y: int = Field(..., ge=0, description="Y coordinate")
    width: int = Field(..., gt=0, description="Width in pixels")
    height: int = Field(..., gt=0, description="Height in pixels")
    name: str | None = Field(None, max_length=100, description="Optional region name")


class ComparisonSettings(BaseModel):
    """Settings for visual comparison algorithm.

    Controls how two images are compared for similarity.
    """

    algorithm: str = Field(
        "ssim",
        description="Comparison algorithm: 'ssim', 'pixel_diff', 'perceptual_hash'",
    )
    threshold: float = Field(
        0.95, ge=0.0, le=1.0, description="Similarity threshold (0-1)"
    )
    ignore_regions: list[IgnoreRegion] = Field(
        default_factory=list, description="Regions to ignore"
    )


class VisualBaselineCreate(BaseModel):
    """Request schema for creating a visual baseline.

    Baselines are reference images used for comparison.
    """

    state_name: str = Field(
        ..., max_length=500, description="State this baseline is for"
    )
    workflow_id: str | None = Field(
        None, max_length=500, description="Optional workflow ID"
    )
    comparison_settings: ComparisonSettings | None = Field(
        None, description="Comparison settings (defaults to SSIM with 0.95 threshold)"
    )
    approval_notes: str | None = Field(None, description="Notes about this baseline")


class VisualBaselineFromScreenshot(BaseModel):
    """Request to create baseline from existing screenshot."""

    screenshot_id: UUID = Field(..., description="Source screenshot ID")
    state_name: str = Field(..., max_length=500, description="State name")
    workflow_id: str | None = Field(None, max_length=500, description="Workflow ID")
    comparison_settings: ComparisonSettings | None = Field(
        None, description="Comparison settings (defaults to SSIM with 0.95 threshold)"
    )
    approval_notes: str | None = Field(None, description="Approval notes")


class VisualBaselineUpdate(BaseModel):
    """Request schema for updating a baseline."""

    comparison_settings: ComparisonSettings | None = Field(
        None, description="New comparison settings"
    )
    approval_notes: str | None = Field(None, description="New approval notes")


class VisualBaselineResponse(BaseModel):
    """Response schema for a visual baseline."""

    model_config = ConfigDict(from_attributes=True)

    id: UUID = Field(..., description="Baseline ID")
    project_id: UUID = Field(..., description="Project ID")
    state_name: str = Field(..., description="State name")
    workflow_id: str | None = Field(None, description="Workflow ID")
    width: int = Field(..., description="Image width")
    height: int = Field(..., description="Image height")
    file_size_bytes: int | None = Field(None, description="File size")
    perceptual_hash: str | None = Field(None, description="Perceptual hash")
    version: int = Field(..., description="Baseline version")
    is_active: bool = Field(..., description="Whether this is the active version")
    approved_by_user_id: UUID | None = Field(None, description="Approved by user")
    approved_at: UTCDateTime | None = Field(None, description="Approval time (UTC)")
    approval_notes: str | None = Field(None, description="Approval notes")
    comparison_settings: dict[str, Any] = Field(..., description="Comparison settings")
    source_test_run_id: UUID | None = Field(None, description="Source run ID")
    source_screenshot_id: UUID | None = Field(None, description="Source screenshot ID")
    created_at: UTCDateTime = Field(..., description="Creation time (UTC)")
    updated_at: UTCDateTime = Field(..., description="Last update time (UTC)")
    image_url: str | None = Field(None, description="Image URL")
    thumbnail_url: str | None = Field(None, description="Thumbnail URL")


class VisualBaselineListResponse(BaseModel):
    """Response schema for paginated baseline list."""

    items: list[VisualBaselineResponse] = Field(..., description="Baselines")
    total: int = Field(..., description="Total count")
    skip: int = Field(..., description="Items skipped")
    limit: int = Field(..., description="Items per page")


class DiffRegion(BaseModel):
    """A region where images differ."""

    x: int = Field(..., description="X coordinate")
    y: int = Field(..., description="Y coordinate")
    width: int = Field(..., description="Width")
    height: int = Field(..., description="Height")
    change_percentage: float = Field(..., description="Percentage of change in region")
    pixel_count: int | None = Field(None, description="Changed pixel count")


class VisualComparisonCreate(BaseModel):
    """Request schema for creating a visual comparison."""

    screenshot_id: UUID = Field(..., description="Screenshot to compare")
    baseline_id: UUID | None = Field(None, description="Baseline to compare against")
    algorithm: str | None = Field(None, description="Override algorithm")
    threshold: float | None = Field(
        None, ge=0.0, le=1.0, description="Override threshold"
    )


class VisualComparisonResponse(BaseModel):
    """Response schema for visual comparison result."""

    model_config = ConfigDict(from_attributes=True)

    id: UUID = Field(..., description="Comparison ID")
    test_run_id: UUID = Field(..., description="Test run ID")
    baseline_id: UUID | None = Field(None, description="Baseline ID")
    screenshot_id: UUID = Field(..., description="Screenshot ID")
    transition_execution_id: UUID | None = Field(
        None, description="Transition execution ID"
    )
    state_name: str = Field(..., description="State name")
    comparison_algorithm: str = Field(..., description="Algorithm used")
    similarity_score: float = Field(..., description="Similarity score (0-1)")
    threshold_used: float = Field(..., description="Threshold used")
    status: str = Field(..., description="Comparison status: passed, failed, pending")
    diff_region_count: int = Field(..., description="Number of diff regions")
    execution_time_ms: int | None = Field(None, description="Execution time in ms")
    reviewed_by_user_id: UUID | None = Field(None, description="Reviewer user ID")
    reviewed_at: UTCDateTime | None = Field(None, description="Review time (UTC)")
    review_decision: str | None = Field(
        None, description="Decision: approved, rejected, new_baseline"
    )
    review_notes: str | None = Field(None, description="Review notes")
    deficiency_id: UUID | None = Field(None, description="Created deficiency ID")
    error_message: str | None = Field(None, description="Error message if failed")
    created_at: UTCDateTime = Field(..., description="Creation time (UTC)")
    diff_image_url: str | None = Field(None, description="Diff image URL")
    screenshot_url: str | None = Field(None, description="Screenshot URL")
    baseline_url: str | None = Field(None, description="Baseline URL")


class VisualComparisonDetail(VisualComparisonResponse):
    """Detailed comparison with diff regions."""

    diff_regions: list[DiffRegion] = Field(
        default_factory=list, description="Diff regions"
    )


class VisualComparisonListResponse(BaseModel):
    """Response schema for paginated comparison list."""

    items: list[VisualComparisonResponse] = Field(..., description="Comparisons")
    total: int = Field(..., description="Total count")
    skip: int = Field(..., description="Items skipped")
    limit: int = Field(..., description="Items per page")


class VisualComparisonSummary(BaseModel):
    """Summary of visual comparison result."""

    comparison_id: UUID = Field(..., description="Comparison ID")
    baseline_id: UUID | None = Field(None, description="Baseline ID")
    similarity_score: float = Field(..., description="Similarity score")
    threshold: float = Field(..., description="Threshold used")
    passed: bool = Field(..., description="Whether comparison passed")
    status: str = Field(..., description="Status")
    diff_image_url: str | None = Field(None, description="Diff image URL")
    diff_region_count: int = Field(0, description="Number of diff regions")


class ComparisonReview(BaseModel):
    """Request schema for reviewing a comparison."""

    decision: str = Field(
        ..., description="Decision: 'approved', 'rejected', 'new_baseline'"
    )
    notes: str | None = Field(None, description="Review notes")


class ComparisonStats(BaseModel):
    """Statistics for visual comparisons."""

    total: int = Field(..., description="Total comparisons")
    passed: int = Field(..., description="Passed comparisons")
    failed: int = Field(..., description="Failed comparisons")
    pending_review: int = Field(..., description="Pending review")
    approved_as_new: int = Field(..., description="Approved as new baseline")
    no_baseline: int = Field(..., description="No baseline available")
    pass_rate: float = Field(..., description="Pass rate percentage")


__all__ = [
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
]
