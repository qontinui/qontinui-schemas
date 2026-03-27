"""Feedback Score API schemas.

This module provides schemas for the feedback score system used by the
Opik integration to track quality metrics on execution runs and action
executions.

Used by:
- qontinui-web backend (FastAPI)
- qontinui-web frontend (TypeScript, via generated types)
- qontinui-runner (feedback reporting)
"""

from typing import Any, Literal
from uuid import UUID

from pydantic import BaseModel, Field

from qontinui_schemas.common.time import UTCDateTime

# =============================================================================
# Feedback Score Schemas
# =============================================================================


class FeedbackScoreCreate(BaseModel):
    """Request to create a new feedback score."""

    target_type: Literal["run", "action"] = Field(
        ..., description="Whether this score targets a run or an action"
    )
    target_id: UUID = Field(..., description="ID of the target run or action execution")
    name: str = Field(
        ..., description="Score name (e.g. 'accuracy', 'helpfulness')", max_length=255
    )
    value: float = Field(..., description="Numeric score value")
    category_value: str | None = Field(
        None, description="Optional categorical label", max_length=255
    )
    source: Literal["manual", "automated", "llm_judge", "runner_agentic_metrics"] = (
        Field("manual", description="Source of the feedback score")
    )
    reason: str | None = Field(None, description="Optional reason for the score")
    metadata: dict[str, Any] | None = Field(None, description="Additional metadata")


class FeedbackScoreResponse(BaseModel):
    """Response for a single feedback score."""

    id: UUID = Field(..., description="Feedback score ID")
    target_type: Literal["run", "action"] = Field(
        ..., description="Whether this score targets a run or an action"
    )
    target_id: UUID = Field(..., description="ID of the target run or action execution")
    name: str = Field(..., description="Score name")
    value: float = Field(..., description="Numeric score value")
    category_value: str | None = Field(None, description="Categorical label")
    source: str = Field(..., description="Source of the feedback score")
    reason: str | None = Field(None, description="Reason for the score")
    metadata: dict[str, Any] | None = Field(None, description="Additional metadata")
    created_at: UTCDateTime = Field(..., description="Creation time (UTC)")
    created_by: str | None = Field(None, description="User who created the score")


class FeedbackScoreSummary(BaseModel):
    """Aggregated summary for a feedback score name."""

    name: str = Field(..., description="Score name")
    count: int = Field(..., description="Number of scores")
    avg_value: float = Field(..., description="Average score value")
    min_value: float = Field(..., description="Minimum score value")
    max_value: float = Field(..., description="Maximum score value")


class FeedbackScoreBatchResponse(BaseModel):
    """Response for batch-creating feedback scores."""

    created: int = Field(..., description="Number of scores successfully created")


class FeedbackScoreListResponse(BaseModel):
    """Response for listing feedback scores."""

    items: list[FeedbackScoreResponse] = Field(
        ..., description="List of feedback scores"
    )
    total: int = Field(..., description="Total number of matching scores")
