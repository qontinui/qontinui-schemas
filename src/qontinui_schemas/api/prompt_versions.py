"""Prompt versioning API schemas.

Schemas for creating, listing, and comparing prompt template versions.
Used by:
- qontinui-web backend (FastAPI)
- qontinui-web frontend (TypeScript, via generated types)
"""

from typing import Any
from uuid import UUID

from pydantic import BaseModel, Field

from qontinui_schemas.common.time import UTCDateTime

# =============================================================================
# Request Schemas
# =============================================================================


class PromptVersionCreate(BaseModel):
    """Request to create a new prompt template version.

    Note: template_id comes from the URL path parameter and is NOT
    included in the request body.
    """

    prompt_content: str = Field(..., description="Full prompt text for this version")
    parameters: list[dict[str, Any]] | None = Field(
        None, description="Template parameters at this version"
    )
    change_description: str | None = Field(
        None, description="Description of what changed in this version"
    )


# =============================================================================
# Response Schemas
# =============================================================================


class PromptVersionResponse(BaseModel):
    """Response for a single prompt template version."""

    id: UUID = Field(..., description="Version record ID")
    template_id: str = Field(..., description="Parent prompt template ID")
    version_number: int = Field(..., description="Sequential version number")
    prompt_content: str = Field(..., description="Full prompt text for this version")
    parameters_json: list[dict[str, Any]] | None = Field(
        None, description="Template parameters at this version"
    )
    content_hash: str = Field(
        ..., description="SHA256 hash of prompt content for deduplication"
    )
    change_description: str | None = Field(
        None, description="Description of what changed"
    )
    created_by: str | None = Field(None, description="User who created this version")
    performance_metrics: dict[str, Any] | None = Field(
        None,
        description=(
            "Aggregated performance metrics (success_rate, avg_cost, avg_latency)"
        ),
    )
    created_at: UTCDateTime = Field(..., description="Creation timestamp (UTC)")


class PromptVersionListResponse(BaseModel):
    """Paginated list of prompt template versions."""

    items: list[PromptVersionResponse] = Field(
        ..., description="List of prompt versions"
    )
    total: int = Field(..., description="Total number of versions for this template")


# =============================================================================
# Diff Schema
# =============================================================================


class PromptVersionDiff(BaseModel):
    """Diff between two prompt template versions."""

    version_a: int = Field(..., description="First version number being compared")
    version_b: int = Field(..., description="Second version number being compared")
    diff_text: str = Field(
        ...,
        description="Unified diff text showing changes between the two versions",
    )
