"""Execution Screenshot schemas.

Defines schemas for screenshots captured during execution.
Screenshots provide visual evidence and context for debugging.
"""

from typing import Any
from uuid import UUID

from pydantic import BaseModel, ConfigDict, Field

from qontinui_schemas.common.metadata import ScreenshotAnnotation
from qontinui_schemas.common.time import UTCDateTime
from qontinui_schemas.execution.enums import ScreenshotType


class ExecutionScreenshotCreate(BaseModel):
    """Metadata for screenshot upload.

    Sent along with the binary image data when uploading a screenshot.
    """

    screenshot_id: UUID = Field(
        ..., description="Screenshot ID (client-generated UUID)"
    )
    sequence_number: int = Field(..., ge=0, description="Order within the run")
    screenshot_type: ScreenshotType = Field(..., description="Type of screenshot")
    timestamp: UTCDateTime = Field(..., description="When screenshot was taken (UTC)")
    width: int = Field(..., ge=1, description="Image width in pixels")
    height: int = Field(..., ge=1, description="Image height in pixels")

    # Context
    action_sequence_number: int | None = Field(
        None, description="Associated action sequence number"
    )
    state: str | None = Field(
        None, max_length=255, description="State when screenshot was taken"
    )
    active_states: list[str] | None = Field(
        None, description="Active states at capture time"
    )

    # Annotations (visual overlays)
    annotations: list[ScreenshotAnnotation] = Field(
        default_factory=list, description="Visual annotations on the screenshot"
    )

    # Additional metadata
    metadata: dict[str, Any] = Field(
        default_factory=dict, description="Additional screenshot metadata"
    )


class ExecutionScreenshotResponse(BaseModel):
    """Response schema for screenshot upload.

    Returned after successfully uploading a screenshot.
    """

    model_config = ConfigDict(from_attributes=True)

    screenshot_id: UUID = Field(..., description="Screenshot ID")
    run_id: UUID = Field(..., description="Run ID")
    image_url: str = Field(..., description="URL to full image")
    thumbnail_url: str | None = Field(None, description="URL to thumbnail")
    uploaded_at: UTCDateTime = Field(..., description="Upload time (UTC)")
    file_size_bytes: int = Field(..., description="File size in bytes")


class ExecutionScreenshotDetail(BaseModel):
    """Detailed screenshot information.

    Used when retrieving individual screenshot details.
    """

    model_config = ConfigDict(from_attributes=True)

    id: UUID = Field(..., description="Screenshot ID")
    run_id: UUID = Field(..., description="Run ID")
    action_execution_id: UUID | None = Field(None, description="Associated action ID")
    sequence_number: int = Field(..., description="Sequence number")
    screenshot_type: ScreenshotType = Field(..., description="Screenshot type")
    storage_path: str = Field(..., description="Storage path")
    image_url: str | None = Field(None, description="Public URL")
    thumbnail_url: str | None = Field(None, description="Thumbnail URL")
    width: int = Field(..., description="Width in pixels")
    height: int = Field(..., description="Height in pixels")
    file_size_bytes: int | None = Field(None, description="File size")
    state_name: str | None = Field(None, description="State name")
    perceptual_hash: str | None = Field(None, description="Perceptual hash for diffs")
    captured_at: UTCDateTime = Field(..., description="Capture time (UTC)")
    metadata: dict[str, Any] = Field(default_factory=dict, description="Metadata")
    created_at: UTCDateTime = Field(..., description="Record creation time (UTC)")


class ExecutionScreenshotListResponse(BaseModel):
    """Response schema for paginated screenshot list."""

    screenshots: list[ExecutionScreenshotDetail] = Field(
        ..., description="List of screenshots"
    )
    total: int = Field(..., description="Total count")
    limit: int = Field(..., description="Items per page")
    offset: int = Field(..., description="Items skipped")
    has_more: bool = Field(..., description="Whether more items exist")


__all__ = [
    "ExecutionScreenshotCreate",
    "ExecutionScreenshotResponse",
    "ExecutionScreenshotDetail",
    "ExecutionScreenshotListResponse",
]
