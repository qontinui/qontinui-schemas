"""Evaluation Dataset and Experiment API schemas.

This module provides schemas for the evaluation system used to manage
datasets of test cases and run experiments comparing prompt variants
against those datasets.

Used by:
- qontinui-web backend (FastAPI)
- qontinui-web frontend (TypeScript, via generated types)
- qontinui-runner (evaluation reporting)
"""

from typing import Any, Literal
from uuid import UUID

from pydantic import BaseModel, Field

from qontinui_schemas.common.time import UTCDateTime

# =============================================================================
# Evaluation Dataset Schemas
# =============================================================================


class EvaluationDatasetCreate(BaseModel):
    """Request to create a new evaluation dataset."""

    name: str = Field(..., description="Dataset name", max_length=255)
    description: str | None = Field(None, description="Optional dataset description")


class EvaluationDatasetResponse(BaseModel):
    """Response for a single evaluation dataset."""

    id: UUID = Field(..., description="Dataset ID")
    name: str = Field(..., description="Dataset name")
    description: str | None = Field(None, description="Dataset description")
    version: int = Field(..., description="Dataset version number")
    item_count: int = Field(..., description="Number of items in the dataset")
    content_hash: str | None = Field(
        None, description="Hash of dataset contents for change detection"
    )
    created_at: UTCDateTime = Field(..., description="Creation time (UTC)")
    updated_at: UTCDateTime | None = Field(None, description="Last update time (UTC)")


class EvaluationDatasetListResponse(BaseModel):
    """Response for listing evaluation datasets."""

    items: list[EvaluationDatasetResponse] = Field(
        ..., description="List of evaluation datasets"
    )
    total: int = Field(..., description="Total number of matching datasets")


# =============================================================================
# Dataset Item Schemas
# =============================================================================


class DatasetItemCreate(BaseModel):
    """Request to create a new dataset item."""

    input: dict[str, Any] = Field(..., description="Input data for the evaluation case")
    expected_output: dict[str, Any] | None = Field(
        None, description="Expected output for comparison"
    )
    metadata: dict[str, Any] | None = Field(None, description="Additional metadata")


class DatasetItemResponse(BaseModel):
    """Response for a single dataset item."""

    id: UUID = Field(..., description="Dataset item ID")
    dataset_id: UUID = Field(..., description="Parent dataset ID")
    input: dict[str, Any] = Field(..., description="Input data")
    expected_output: dict[str, Any] | None = Field(None, description="Expected output")
    metadata: dict[str, Any] | None = Field(None, description="Additional metadata")
    content_hash: str = Field(..., description="Hash of item content for deduplication")
    created_at: UTCDateTime = Field(..., description="Creation time (UTC)")


# =============================================================================
# Evaluation Experiment Schemas
# =============================================================================


class EvaluationExperimentCreate(BaseModel):
    """Request to create a new evaluation experiment."""

    name: str = Field(..., description="Experiment name", max_length=255)
    dataset_id: UUID = Field(..., description="ID of the dataset to evaluate against")
    dataset_version: int | None = Field(
        None, description="Dataset version to pin (defaults to current)"
    )
    prompt_variant_id: str | None = Field(
        None, description="ID of the prompt variant being tested"
    )
    description: str | None = Field(None, description="Optional experiment description")


class EvaluationExperimentResponse(BaseModel):
    """Response for a single evaluation experiment."""

    id: UUID = Field(..., description="Experiment ID")
    name: str = Field(..., description="Experiment name")
    dataset_id: UUID = Field(..., description="Dataset ID")
    dataset_version: int = Field(..., description="Dataset version used")
    prompt_variant_id: str | None = Field(
        None, description="Prompt variant being tested"
    )
    description: str | None = Field(None, description="Experiment description")
    status: str = Field(
        ..., description="Experiment status: pending, running, completed, failed"
    )
    metrics: dict[str, Any] | None = Field(None, description="Aggregate result metrics")
    item_count: int = Field(..., description="Total items in the experiment")
    completed_count: int = Field(..., description="Number of completed items")
    created_at: UTCDateTime = Field(..., description="Creation time (UTC)")
    completed_at: UTCDateTime | None = Field(None, description="Completion time (UTC)")


class EvaluationExperimentListResponse(BaseModel):
    """Response for listing evaluation experiments."""

    items: list[EvaluationExperimentResponse] = Field(
        ..., description="List of evaluation experiments"
    )
    total: int = Field(..., description="Total number of matching experiments")


# =============================================================================
# Experiment Result Schemas
# =============================================================================


class ExperimentResultCreate(BaseModel):
    """Request to create a new experiment result."""

    experiment_id: UUID = Field(..., description="ID of the parent experiment")
    dataset_item_id: UUID = Field(..., description="ID of the dataset item evaluated")
    output: dict[str, Any] = Field(..., description="Actual output from evaluation")
    scores: dict[str, float] | None = Field(
        None, description="Score metrics (e.g. accuracy, similarity)"
    )
    duration_ms: float | None = Field(
        None, description="Evaluation duration in milliseconds"
    )
    cost_usd: float | None = Field(None, description="Cost of evaluation in USD")
    tokens_total: int | None = Field(None, description="Total tokens consumed")


class ExperimentResultResponse(BaseModel):
    """Response for a single experiment result."""

    id: UUID = Field(..., description="Result ID")
    experiment_id: UUID = Field(..., description="Parent experiment ID")
    dataset_item_id: UUID = Field(..., description="Dataset item ID")
    output: dict[str, Any] = Field(..., description="Actual output")
    scores: dict[str, float] | None = Field(None, description="Score metrics")
    duration_ms: float | None = Field(None, description="Duration in milliseconds")
    cost_usd: float | None = Field(None, description="Cost in USD")
    tokens_total: int | None = Field(None, description="Total tokens consumed")
    created_at: UTCDateTime = Field(..., description="Creation time (UTC)")


class DatasetItemListResponse(BaseModel):
    """Response for listing dataset items."""

    items: list[DatasetItemResponse] = Field(..., description="List of dataset items")
    total: int = Field(..., description="Total number of matching items")


class ExperimentResultListResponse(BaseModel):
    """Response for listing experiment results."""

    items: list[ExperimentResultResponse] = Field(
        ..., description="List of experiment results"
    )
    total: int = Field(..., description="Total number of matching results")


class ExperimentStatusUpdate(BaseModel):
    """Request to update an experiment's status."""

    status: Literal["pending", "running", "completed", "failed"] = Field(
        ..., description="New experiment status"
    )
    metrics: dict[str, Any] | None = Field(
        None, description="Optional aggregate metrics to set"
    )


class ExperimentSummary(BaseModel):
    """Aggregated summary of experiment results."""

    total_results: int = Field(..., description="Total number of results")
    avg_duration_ms: float | None = Field(
        None, description="Average duration in milliseconds"
    )
    total_cost_usd: float | None = Field(None, description="Total cost in USD")
    total_tokens: int | None = Field(None, description="Total tokens consumed")
    avg_scores: dict[str, float] | None = Field(
        None, description="Average scores across all results"
    )
