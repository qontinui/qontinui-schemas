//! RAG API schemas for Qontinui services.
//!
//! Mirrors `src/qontinui_schemas/api/rag.py`. Rust is the source of truth;
//! TS and Python bindings regenerate from the JSON Schemas emitted here.
//!
//! Used by qontinui-web (backend + frontend) and qontinui-runner for the
//! RAG (retrieval-augmented generation) dashboard: embedding compute,
//! batch sync runner → backend, progress events, dashboard stats, and
//! semantic search.
//!
//! Datetime fields are ISO 8601 strings with a `Z` suffix (UTC); see the
//! crate-level docs for the rationale — the types crate is wire-format
//! only and doesn't depend on a chrono version.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Enums
// ============================================================================

/// Status of an embedding-generation job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

/// Status of RAG processing in the runner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RagProcessingStatus {
    NotStarted,
    InProgress,
    Completed,
    Failed,
}

// ============================================================================
// Embedding computation (qontinui-api)
// ============================================================================

/// Request to compute a text embedding for semantic search.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ComputeTextEmbeddingRequest {
    /// Text to encode into embedding space.
    pub text: String,
    /// Embedding model: `"clip"` (512-dim) or `"minilm"` (384-dim).
    #[serde(default = "default_clip_model")]
    pub model: String,
}

fn default_clip_model() -> String {
    "clip".to_string()
}

/// Response with a computed text embedding.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ComputeTextEmbeddingResponse {
    /// Whether the embedding computation succeeded.
    pub success: bool,
    /// Text embedding vector.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,
    /// Dimension of the embedding vector.
    #[serde(default = "default_embedding_dim")]
    pub embedding_dim: i64,
    /// Processing time in milliseconds.
    #[serde(default)]
    pub processing_time_ms: f64,
    /// Error message if the request failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

fn default_embedding_dim() -> i64 {
    512
}

/// Request to compute embeddings for a single image.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ComputeEmbeddingRequest {
    /// Base64-encoded image data.
    pub image_data: String,
    /// Also compute a text embedding from OCR.
    #[serde(default)]
    pub compute_text_embedding: bool,
    /// Optional text description for the text embedding.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_description: Option<String>,
}

/// Response with computed embeddings for a single image.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ComputeEmbeddingResponse {
    pub success: bool,
    /// CLIP image embedding vector (512 dimensions).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_embedding: Option<Vec<f32>>,
    /// Text embedding vector (384 dimensions).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_embedding: Option<Vec<f32>>,
    /// AI-generated text description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_description: Option<String>,
    /// Text extracted via OCR.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ocr_text: Option<String>,
    /// OCR confidence score (0-1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ocr_confidence: Option<f64>,
    /// Processing time in milliseconds.
    #[serde(default)]
    pub processing_time_ms: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Request to compute embeddings for multiple images.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BatchComputeEmbeddingRequest {
    /// Images, each a map with `id`, `image_data` (base64), and optional
    /// `text_description`.
    pub images: Vec<HashMap<String, serde_json::Value>>,
    /// Compute text embeddings for all images.
    #[serde(default = "default_true")]
    pub compute_text_embeddings: bool,
    /// Extract OCR text from images.
    #[serde(default = "default_true")]
    pub extract_ocr: bool,
}

fn default_true() -> bool {
    true
}

/// Result for a single image in batch processing.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BatchEmbeddingResult {
    /// Image identifier from the request.
    pub id: String,
    pub success: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_embedding: Option<Vec<f32>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_embedding: Option<Vec<f32>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ocr_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ocr_confidence: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Response with batch-computed embeddings.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BatchComputeEmbeddingResponse {
    /// Whether the batch succeeded overall.
    pub success: bool,
    pub results: Vec<BatchEmbeddingResult>,
    pub total_processed: i64,
    pub successful: i64,
    pub failed: i64,
    #[serde(default)]
    pub processing_time_ms: f64,
}

// ============================================================================
// Embedding sync (runner → backend)
// ============================================================================

/// Single embedding result from the runner.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EmbeddingResultItem {
    /// ID of the state image that was processed.
    pub state_image_id: String,
    pub success: bool,
    /// CLIP image embedding vector (512 dimensions).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_embedding: Option<Vec<f32>>,
    /// Text embedding vector (384 dimensions for all-MiniLM-L6-v2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_embedding: Option<Vec<f32>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ocr_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ocr_confidence: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Request containing embedding results from the runner.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EmbeddingResultsRequest {
    pub project_id: String,
    pub results: Vec<EmbeddingResultItem>,
    pub total_processed: i64,
    pub successful: i64,
    pub failed: i64,
}

/// Response after applying embedding results.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EmbeddingResultsResponse {
    pub success: bool,
    pub message: String,
    pub applied: i64,
    pub failed: i64,
    pub not_found: i64,
}

// ============================================================================
// RAG progress events (runner → UI)
// ============================================================================

/// Progress event emitted during RAG processing.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RagProgressEvent {
    pub project_id: String,
    pub status: RagProcessingStatus,
    /// Human-readable status message.
    pub message: String,
    /// Progress percentage (0-100).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub percent: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elements_processed: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_elements: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Completion event emitted when RAG processing finishes.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RagCompletionEvent {
    pub project_id: String,
    pub success: bool,
    pub results: Vec<EmbeddingResultItem>,
    pub total_processed: i64,
    pub successful: i64,
    pub failed: i64,
    /// Whether sync to the web backend succeeded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub web_sync_success: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub web_sync_error: Option<String>,
}

// ============================================================================
// RAG dashboard (backend → frontend)
// ============================================================================

/// Summary of an embedding-generation job.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct JobSummary {
    /// Job UUID.
    pub id: String,
    pub status: JobStatus,
    /// Progress percentage (0-100).
    pub progress_percent: f64,
    pub total_patterns: i64,
    pub processed_patterns: i64,
    /// ISO 8601 UTC timestamp (`Z` suffix).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

/// Summary statistics for the RAG dashboard.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RAGDashboardStats {
    pub total_embeddings: i64,
    pub total_states: i64,
    pub total_patterns: i64,
    /// ISO 8601 UTC timestamp (`Z` suffix).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_sync_at: Option<String>,
    /// Currently-running job, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_job: Option<JobSummary>,
}

/// Single embedding record for display.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EmbeddingItem {
    /// Embedding UUID.
    pub id: String,
    pub pattern_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pattern_name: Option<String>,
    pub state_id: String,
    pub state_name: String,
    pub image_id: String,
    pub image_storage_path: String,
    /// Presigned URL for displaying the image.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    pub embedding_model: String,
    pub embedding_version: String,
    pub image_width: i64,
    pub image_height: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_description: Option<String>,
    /// Whether a text embedding vector is available.
    #[serde(default)]
    pub has_text_embedding: bool,
    /// Additional pattern metadata.
    #[serde(default)]
    pub pattern_metadata: HashMap<String, serde_json::Value>,
    /// ISO 8601 UTC timestamp (`Z` suffix).
    pub created_at: String,
    /// ISO 8601 UTC timestamp (`Z` suffix).
    pub updated_at: String,
}

/// Paginated list of embeddings.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EmbeddingListResponse {
    pub items: Vec<EmbeddingItem>,
    pub total: i64,
    /// 1-indexed page number.
    pub page: i64,
    pub limit: i64,
    pub has_more: bool,
}

/// Single job record for display.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct JobItem {
    pub id: String,
    pub status: JobStatus,
    pub total_patterns: i64,
    pub processed_patterns: i64,
    pub progress_percent: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    pub retry_count: i64,
    pub max_retries: i64,
    #[serde(default)]
    pub job_metadata: HashMap<String, serde_json::Value>,
    /// ISO 8601 UTC timestamp (`Z` suffix).
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
}

/// Paginated list of jobs.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct JobListResponse {
    pub items: Vec<JobItem>,
    pub total: i64,
    pub page: i64,
    pub limit: i64,
    pub has_more: bool,
}

// ============================================================================
// Semantic search
// ============================================================================

/// Request for semantic search.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SemanticSearchRequest {
    /// Search query text (min length 1).
    pub query: String,
    /// Max results to return (1–100).
    #[serde(default = "default_search_limit")]
    pub limit: i64,
    /// Minimum similarity threshold (0–1). CLIP text-to-image similarities
    /// typically fall in 0.15–0.35, so 0.2 is a reasonable default.
    #[serde(default = "default_min_similarity")]
    pub min_similarity: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_filter: Option<String>,
}

fn default_search_limit() -> i64 {
    20
}
fn default_min_similarity() -> f64 {
    0.2
}

/// Single search result.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchResultItem {
    pub embedding: EmbeddingItem,
    /// Similarity score (0-1).
    pub similarity_score: f64,
}

/// Response from semantic search.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SemanticSearchResponse {
    pub results: Vec<SearchResultItem>,
    /// The original search query.
    pub query: String,
    pub total_found: i64,
}

// ============================================================================
// State filter
// ============================================================================

/// State item for the filter dropdown.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StateFilterItem {
    pub state_id: String,
    pub state_name: String,
    /// Number of embeddings in this state.
    pub count: i64,
}

/// Response with list of states for filtering.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StatesResponse {
    pub states: Vec<StateFilterItem>,
    /// Total number of states.
    pub count: i64,
}
