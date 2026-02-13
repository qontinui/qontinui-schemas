"""RAG API schemas for Qontinui services.

These schemas are shared between:
- qontinui-web backend (Python)
- qontinui-web frontend (TypeScript)
- qontinui-runner (TypeScript)

All datetime fields use UTCDateTime for consistent UTC timezone handling
and ISO 8601 format strings with 'Z' suffix for JSON serialization.
"""

from enum import Enum
from typing import Any

from pydantic import BaseModel, Field

from qontinui_schemas.common.time import UTCDateTime

# ============================================================================
# Enums
# ============================================================================


class JobStatus(str, Enum):
    """Status of an embedding generation job."""

    PENDING = "pending"
    IN_PROGRESS = "in_progress"
    COMPLETED = "completed"
    FAILED = "failed"
    CANCELLED = "cancelled"


class RagProcessingStatus(str, Enum):
    """Status of RAG processing in the runner."""

    NOT_STARTED = "not_started"
    IN_PROGRESS = "in_progress"
    COMPLETED = "completed"
    FAILED = "failed"


# ============================================================================
# Embedding Computation (qontinui-api)
# ============================================================================


class ComputeTextEmbeddingRequest(BaseModel):
    """Request to compute text embedding for semantic search."""

    text: str = Field(description="Text to encode into embedding space")
    model: str = Field(
        default="clip",
        description="Embedding model: 'clip' (512-dim) or 'minilm' (384-dim)",
    )


class ComputeTextEmbeddingResponse(BaseModel):
    """Response with computed text embedding."""

    success: bool = Field(description="Whether embedding computation succeeded")
    embedding: list[float] | None = Field(
        default=None, description="Text embedding vector"
    )
    embedding_dim: int = Field(
        default=512, description="Dimension of the embedding vector"
    )
    processing_time_ms: float = Field(
        default=0.0, description="Processing time in milliseconds"
    )
    error: str | None = Field(default=None, description="Error message if failed")


class ComputeEmbeddingRequest(BaseModel):
    """Request to compute embedding for a single image."""

    image_data: str = Field(description="Base64 encoded image data")
    compute_text_embedding: bool = Field(
        default=False, description="Also compute text embedding from OCR"
    )
    text_description: str | None = Field(
        default=None, description="Optional text description for text embedding"
    )


class ComputeEmbeddingResponse(BaseModel):
    """Response with computed embeddings for a single image."""

    success: bool = Field(description="Whether embedding computation succeeded")
    image_embedding: list[float] | None = Field(
        default=None, description="CLIP image embedding vector (512 dimensions)"
    )
    text_embedding: list[float] | None = Field(
        default=None, description="Text embedding vector (384 dimensions)"
    )
    text_description: str | None = Field(
        default=None, description="AI-generated text description"
    )
    ocr_text: str | None = Field(default=None, description="Text extracted via OCR")
    ocr_confidence: float | None = Field(
        default=None, description="OCR confidence score (0-1)"
    )
    processing_time_ms: float = Field(
        default=0.0, description="Processing time in milliseconds"
    )
    error: str | None = Field(default=None, description="Error message if failed")


class BatchComputeEmbeddingRequest(BaseModel):
    """Request to compute embeddings for multiple images."""

    images: list[dict[str, Any]] = Field(
        description="List of images with 'id', 'image_data' (base64), and optional 'text_description'"
    )
    compute_text_embeddings: bool = Field(
        default=True, description="Compute text embeddings for all images"
    )
    extract_ocr: bool = Field(default=True, description="Extract OCR text from images")


class BatchEmbeddingResult(BaseModel):
    """Result for a single image in batch processing."""

    id: str = Field(description="Image identifier from the request")
    success: bool = Field(description="Whether embedding computation succeeded")
    image_embedding: list[float] | None = Field(
        default=None, description="CLIP image embedding vector (512 dimensions)"
    )
    text_embedding: list[float] | None = Field(
        default=None, description="Text embedding vector (384 dimensions)"
    )
    text_description: str | None = Field(
        default=None, description="AI-generated text description"
    )
    ocr_text: str | None = Field(default=None, description="Text extracted via OCR")
    ocr_confidence: float | None = Field(
        default=None, description="OCR confidence score (0-1)"
    )
    error: str | None = Field(default=None, description="Error message if failed")


class BatchComputeEmbeddingResponse(BaseModel):
    """Response with batch computed embeddings."""

    success: bool = Field(description="Whether batch processing succeeded overall")
    results: list[BatchEmbeddingResult] = Field(description="Results for each image")
    total_processed: int = Field(description="Total number of images processed")
    successful: int = Field(description="Number of successful embeddings")
    failed: int = Field(description="Number of failed embeddings")
    processing_time_ms: float = Field(
        default=0.0, description="Total processing time in milliseconds"
    )


# ============================================================================
# Embedding Sync (Runner → Backend)
# ============================================================================


class EmbeddingResultItem(BaseModel):
    """Single embedding result from runner."""

    state_image_id: str = Field(description="ID of the state image that was processed")
    success: bool = Field(description="Whether embedding generation succeeded")
    image_embedding: list[float] | None = Field(
        default=None, description="CLIP image embedding vector (512 dimensions)"
    )
    text_embedding: list[float] | None = Field(
        default=None,
        description="Text embedding vector (384 dimensions for all-MiniLM-L6-v2)",
    )
    text_description: str | None = Field(
        default=None, description="AI-generated text description of the element"
    )
    ocr_text: str | None = Field(
        default=None, description="Text extracted via OCR from the image"
    )
    ocr_confidence: float | None = Field(
        default=None, description="Confidence score of OCR extraction (0-1)"
    )
    error: str | None = Field(
        default=None, description="Error message if processing failed"
    )


class EmbeddingResultsRequest(BaseModel):
    """Request containing embedding results from runner."""

    project_id: str = Field(description="Project ID the embeddings belong to")
    results: list[EmbeddingResultItem] = Field(description="List of embedding results")
    total_processed: int = Field(description="Total number of elements processed")
    successful: int = Field(description="Number of successfully processed elements")
    failed: int = Field(description="Number of failed elements")


class EmbeddingResultsResponse(BaseModel):
    """Response after applying embedding results."""

    success: bool = Field(description="Whether the sync operation succeeded")
    message: str = Field(description="Status message")
    applied: int = Field(description="Number of embeddings applied to config")
    failed: int = Field(description="Number of embeddings that failed to apply")
    not_found: int = Field(description="Number of state images not found in config")


# ============================================================================
# RAG Progress Events (Runner → UI)
# ============================================================================


class RagProgressEvent(BaseModel):
    """Progress event emitted during RAG processing."""

    project_id: str = Field(description="Project ID being processed")
    status: RagProcessingStatus = Field(description="Current processing status")
    message: str = Field(description="Human-readable status message")
    percent: float | None = Field(
        default=None, description="Progress percentage (0-100)"
    )
    elements_processed: int | None = Field(
        default=None, description="Number of elements processed so far"
    )
    total_elements: int | None = Field(
        default=None, description="Total number of elements to process"
    )
    error: str | None = Field(default=None, description="Error message if failed")


class RagCompletionEvent(BaseModel):
    """Completion event emitted when RAG processing finishes."""

    project_id: str = Field(description="Project ID that was processed")
    success: bool = Field(description="Whether processing completed successfully")
    results: list[EmbeddingResultItem] = Field(description="Individual element results")
    total_processed: int = Field(description="Total elements processed")
    successful: int = Field(description="Number of successful elements")
    failed: int = Field(description="Number of failed elements")
    web_sync_success: bool | None = Field(
        default=None, description="Whether sync to web backend succeeded"
    )
    web_sync_error: str | None = Field(
        default=None, description="Error message if web sync failed"
    )


# ============================================================================
# RAG Dashboard (Backend → Frontend)
# ============================================================================


class JobSummary(BaseModel):
    """Summary of an embedding generation job."""

    id: str = Field(description="Job UUID")
    status: JobStatus = Field(description="Current job status")
    progress_percent: float = Field(description="Progress percentage (0-100)")
    total_patterns: int = Field(description="Total patterns to process")
    processed_patterns: int = Field(description="Patterns processed so far")
    started_at: UTCDateTime | None = Field(
        default=None, description="When the job started (UTC)"
    )
    error_message: str | None = Field(
        default=None, description="Error message if job failed"
    )


class RAGDashboardStats(BaseModel):
    """Summary statistics for RAG dashboard."""

    total_embeddings: int = Field(description="Total number of indexed embeddings")
    total_states: int = Field(description="Number of unique states with embeddings")
    total_patterns: int = Field(description="Number of unique patterns")
    last_sync_at: UTCDateTime | None = Field(
        default=None, description="When runner last synced embeddings (UTC)"
    )
    active_job: JobSummary | None = Field(
        default=None, description="Currently running job if any"
    )


class EmbeddingItem(BaseModel):
    """Single embedding record for display."""

    id: str = Field(description="Embedding UUID")
    pattern_id: str = Field(description="Pattern identifier")
    pattern_name: str | None = Field(
        default=None, description="Human-readable pattern name"
    )
    state_id: str = Field(description="State identifier")
    state_name: str = Field(description="Human-readable state name")
    image_id: str = Field(description="Image identifier")
    image_storage_path: str = Field(description="Path to image in object storage")
    image_url: str | None = Field(
        default=None, description="Presigned URL for displaying the image"
    )
    embedding_model: str = Field(
        description="Model used for embedding (e.g., clip-vit-base-patch32)"
    )
    embedding_version: str = Field(description="Version of the embedding model")
    image_width: int = Field(description="Image width in pixels")
    image_height: int = Field(description="Image height in pixels")
    text_description: str | None = Field(
        default=None, description="AI-generated or manual text description"
    )
    has_text_embedding: bool = Field(
        default=False, description="Whether text embedding vector is available"
    )
    pattern_metadata: dict[str, Any] = Field(
        default_factory=dict, description="Additional pattern metadata"
    )
    created_at: UTCDateTime = Field(description="When the embedding was created (UTC)")
    updated_at: UTCDateTime = Field(
        description="When the embedding was last updated (UTC)"
    )


class EmbeddingListResponse(BaseModel):
    """Paginated list of embeddings."""

    items: list[EmbeddingItem] = Field(description="List of embedding items")
    total: int = Field(description="Total number of embeddings matching filter")
    page: int = Field(description="Current page number (1-indexed)")
    limit: int = Field(description="Items per page")
    has_more: bool = Field(description="Whether more pages exist")


class JobItem(BaseModel):
    """Single job record for display."""

    id: str = Field(description="Job UUID")
    status: JobStatus = Field(description="Current job status")
    total_patterns: int = Field(description="Total patterns to process")
    processed_patterns: int = Field(description="Patterns processed so far")
    progress_percent: float = Field(description="Progress percentage (0-100)")
    error_message: str | None = Field(
        default=None, description="Error message if job failed"
    )
    retry_count: int = Field(description="Number of retry attempts")
    max_retries: int = Field(description="Maximum retry attempts allowed")
    job_metadata: dict[str, Any] = Field(
        default_factory=dict, description="Additional job metadata"
    )
    created_at: UTCDateTime = Field(description="When the job was created (UTC)")
    started_at: UTCDateTime | None = Field(
        default=None, description="When the job started (UTC)"
    )
    completed_at: UTCDateTime | None = Field(
        default=None, description="When the job completed (UTC)"
    )


class JobListResponse(BaseModel):
    """Paginated list of jobs."""

    items: list[JobItem] = Field(description="List of job items")
    total: int = Field(description="Total number of jobs matching filter")
    page: int = Field(description="Current page number (1-indexed)")
    limit: int = Field(description="Items per page")
    has_more: bool = Field(description="Whether more pages exist")


# ============================================================================
# Semantic Search
# ============================================================================


class SemanticSearchRequest(BaseModel):
    """Request for semantic search."""

    query: str = Field(min_length=1, description="Search query text")
    limit: int = Field(default=20, ge=1, le=100, description="Max results to return")
    min_similarity: float = Field(
        default=0.2,
        ge=0.0,
        le=1.0,
        description="Minimum similarity threshold. CLIP text-to-image similarities "
        "typically range from 0.15-0.35, so 0.2 is a reasonable default.",
    )
    state_filter: str | None = Field(default=None, description="Filter by state ID")


class SearchResultItem(BaseModel):
    """Single search result."""

    embedding: EmbeddingItem = Field(description="The matched embedding")
    similarity_score: float = Field(description="Similarity score (0-1)")


class SemanticSearchResponse(BaseModel):
    """Response from semantic search."""

    results: list[SearchResultItem] = Field(description="List of search results")
    query: str = Field(description="The original search query")
    total_found: int = Field(description="Total number of results found")


# ============================================================================
# State Filter
# ============================================================================


class StateFilterItem(BaseModel):
    """State item for filter dropdown."""

    state_id: str = Field(description="State identifier")
    state_name: str = Field(description="Human-readable state name")
    count: int = Field(description="Number of embeddings in this state")


class StatesResponse(BaseModel):
    """Response with list of states for filtering."""

    states: list[StateFilterItem] = Field(description="List of states")
    count: int = Field(description="Total number of states")
