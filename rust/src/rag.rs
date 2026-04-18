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
#[schemars(deny_unknown_fields)]
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
#[schemars(deny_unknown_fields)]
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
#[schemars(deny_unknown_fields)]
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
#[schemars(deny_unknown_fields)]
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
#[schemars(deny_unknown_fields)]
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
#[schemars(deny_unknown_fields)]
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
#[schemars(deny_unknown_fields)]
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
#[schemars(deny_unknown_fields)]
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
#[schemars(deny_unknown_fields)]
pub struct EmbeddingResultsRequest {
    pub project_id: String,
    pub results: Vec<EmbeddingResultItem>,
    pub total_processed: i64,
    pub successful: i64,
    pub failed: i64,
}

/// Response after applying embedding results.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
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
#[schemars(deny_unknown_fields)]
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
#[schemars(deny_unknown_fields)]
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
#[schemars(deny_unknown_fields)]
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
#[schemars(deny_unknown_fields)]
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
#[schemars(deny_unknown_fields)]
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
#[schemars(deny_unknown_fields)]
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
#[schemars(deny_unknown_fields)]
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
#[schemars(deny_unknown_fields)]
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
#[schemars(deny_unknown_fields)]
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
#[schemars(deny_unknown_fields)]
pub struct SearchResultItem {
    pub embedding: EmbeddingItem,
    /// Similarity score (0-1).
    pub similarity_score: f64,
}

/// Response from semantic search.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
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
#[schemars(deny_unknown_fields)]
pub struct StateFilterItem {
    pub state_id: String,
    pub state_name: String,
    /// Number of embeddings in this state.
    pub count: i64,
}

/// Response with list of states for filtering.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct StatesResponse {
    pub states: Vec<StateFilterItem>,
    /// Total number of states.
    pub count: i64,
}

// ============================================================================
// GUI element chunking & retrieval (rag/models.py)
// ============================================================================

/// Type of GUI element detected and stored for RAG retrieval.
///
/// Mirrors `rag.models.ElementType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ElementType {
    // Button types
    Button,
    IconButton,
    ToggleButton,
    DropdownButton,

    // Input types
    TextInput,
    SearchInput,
    PasswordInput,
    Textarea,

    // Selection types
    Checkbox,
    RadioButton,
    Dropdown,
    Combobox,
    Slider,

    // Navigation types
    Link,
    Tab,
    MenuItem,
    Breadcrumb,

    // Container types
    Modal,
    Dialog,
    Panel,
    Card,

    // Display types
    Icon,
    Image,
    Label,
    Badge,
    Tooltip,

    // Data display types
    TableCell,
    TableHeader,
    ListItem,

    // Feedback types
    Progress,
    Spinner,

    // Unknown
    Unknown,
}

/// Bounding box coordinates for a GUI element.
///
/// Mirrors `rag.models.BoundingBox`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct BoundingBox {
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
}

/// Complete representation of a GUI element chunk for RAG retrieval.
///
/// Contains all information needed for storing, searching, and retrieving GUI
/// elements from a vector database. Mirrors `rag.models.GUIElementChunk`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct GUIElementChunk {
    // ── Identity ──────────────────────────────────────────────────────────
    /// Unique identifier (UUID).
    pub id: String,
    /// ISO 8601 UTC timestamp (`Z` suffix).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// ISO 8601 UTC timestamp (`Z` suffix).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,

    // ── Source information ─────────────────────────────────────────────────
    /// Application name / identifier.
    #[serde(default)]
    pub source_app: String,
    /// State-machine state ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_state_id: Option<String>,
    /// Screenshot identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_screenshot_id: Option<String>,
    /// How the element was extracted (e.g. `"manual"`).
    #[serde(default = "default_extraction_method")]
    pub extraction_method: String,

    // ── Geometry ───────────────────────────────────────────────────────────
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bounding_box: Option<BoundingBox>,
    #[serde(default)]
    pub width: i64,
    #[serde(default)]
    pub height: i64,
    #[serde(default)]
    pub aspect_ratio: f64,
    #[serde(default)]
    pub area: i64,
    /// `"top-left"`, `"top-right"`, `"bottom-left"`, `"bottom-right"`, or `"center"`.
    #[serde(default)]
    pub position_quadrant: String,

    // ── Visual features ───────────────────────────────────────────────────
    /// Dominant RGB colour tuples.
    #[serde(default)]
    pub dominant_colors: Vec<Vec<i64>>,
    #[serde(default)]
    pub color_histogram: Vec<i64>,
    #[serde(default)]
    pub average_brightness: f64,
    #[serde(default)]
    pub contrast_ratio: f64,
    #[serde(default)]
    pub edge_density: f64,

    // ── Text content ──────────────────────────────────────────────────────
    #[serde(default)]
    pub has_text: bool,
    #[serde(default)]
    pub ocr_text: String,
    #[serde(default)]
    pub ocr_confidence: f64,
    #[serde(default)]
    pub text_length: i64,

    // ── Classification ────────────────────────────────────────────────────
    #[serde(default = "default_element_type")]
    pub element_type: ElementType,
    /// More specific classification.
    #[serde(default)]
    pub element_subtype: String,
    #[serde(default)]
    pub is_interactive: bool,
    /// `"click"`, `"type"`, `"select"`, etc.
    #[serde(default)]
    pub interaction_type: String,

    // ── State indicators ──────────────────────────────────────────────────
    /// `"normal"`, `"hover"`, `"pressed"`, `"disabled"`.
    #[serde(default = "default_visual_state")]
    pub visual_state: String,
    #[serde(default = "default_true_element")]
    pub is_enabled: bool,
    #[serde(default)]
    pub is_selected: bool,
    #[serde(default)]
    pub is_focused: bool,

    // ── Context ───────────────────────────────────────────────────────────
    /// Parent container / region identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_region: Option<String>,
    #[serde(default)]
    pub depth_in_hierarchy: i64,
    #[serde(default)]
    pub sibling_count: i64,

    // ── Platform ──────────────────────────────────────────────────────────
    /// `"windows"`, `"macos"`, `"linux"`, `"web"`.
    #[serde(default)]
    pub platform: String,

    // ── Embeddings ────────────────────────────────────────────────────────
    /// Dense vector for text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_embedding: Option<Vec<f32>>,
    /// Human-readable description for embedding.
    #[serde(default)]
    pub text_description: String,
    /// Dense vector for visual features.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_embedding: Option<Vec<f32>>,

    // ── State-machine integration ─────────────────────────────────────────
    /// Associated state ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_id: Option<String>,
    #[serde(default)]
    pub state_name: String,
    /// Is this element required for state identification?
    #[serde(default)]
    pub is_defining_element: bool,
    /// Is this element optional in the state?
    #[serde(default)]
    pub is_optional_element: bool,
    /// Threshold for matching (0-1).
    #[serde(default = "default_similarity_threshold")]
    pub similarity_threshold: f64,
    /// Does the element stay in the same position?
    #[serde(default)]
    pub is_fixed_position: bool,
    /// Is element shared across multiple states?
    #[serde(default)]
    pub is_shared: bool,
    /// Probability of finding this element (0-1).
    #[serde(default = "default_probability")]
    pub probability: f64,
    /// Search-region identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search_region_id: Option<String>,

    // ── Cross-application semantics ───────────────────────────────────────
    /// `"save"`, `"cancel"`, `"submit"`, `"close"`, etc.
    #[serde(default)]
    pub semantic_role: String,
    /// Expected action when interacted with.
    #[serde(default)]
    pub semantic_action: String,
    /// UI toolkit / style (e.g. `"material"`, `"fluent"`, `"gtk"`).
    #[serde(default)]
    pub style_family: String,
}

fn default_extraction_method() -> String {
    "manual".to_string()
}

fn default_element_type() -> ElementType {
    ElementType::Unknown
}

fn default_visual_state() -> String {
    "normal".to_string()
}

fn default_true_element() -> bool {
    true
}

fn default_similarity_threshold() -> f64 {
    0.8
}

fn default_probability() -> f64 {
    1.0
}

/// Result of embedding a GUI element.
///
/// Contains the original element plus computed embeddings.
/// Mirrors `rag.models.EmbeddedElement`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct EmbeddedElement {
    /// The GUI element that was embedded.
    pub element: GUIElementChunk,
    /// Text embedding vector.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_embedding: Option<Vec<f32>>,
    /// Image embedding vector.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_embedding: Option<Vec<f32>>,
    /// Model used for embedding.
    #[serde(default)]
    pub embedding_model: String,
    /// ISO 8601 UTC timestamp (`Z` suffix).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embedding_timestamp: Option<String>,
}

/// Result from a vector database search query.
///
/// Contains the matched element and relevance scores.
/// Mirrors `rag.models.SearchResult`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct VectorSearchResult {
    /// The matched GUI element.
    pub element: GUIElementChunk,
    /// Similarity score (0-1).
    pub score: f64,
    /// Distance metric from query.
    #[serde(default)]
    pub distance: f64,
    /// Position in result list.
    #[serde(default)]
    pub rank: i64,
    /// Which embedding was matched: `"text"`, `"image"`, or `"hybrid"`.
    #[serde(default = "default_matched_on")]
    pub matched_on: String,
    /// Type of search performed.
    #[serde(default = "default_search_type")]
    pub search_type: String,
    /// The original query text.
    #[serde(default)]
    pub query_text: String,
    /// ISO 8601 UTC timestamp (`Z` suffix).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query_timestamp: Option<String>,
}

fn default_matched_on() -> String {
    "text".to_string()
}

fn default_search_type() -> String {
    "text".to_string()
}

/// Result from the export pipeline.
///
/// Tracks what was exported and any errors that occurred.
/// Mirrors `rag.models.ExportResult`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct ExportResult {
    pub success: bool,
    #[serde(default)]
    pub exported_count: i64,
    #[serde(default)]
    pub failed_count: i64,
    #[serde(default)]
    pub skipped_count: i64,
    #[serde(default)]
    pub errors: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    /// ISO 8601 UTC timestamp (`Z` suffix).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub export_timestamp: Option<String>,
    #[serde(default)]
    pub export_path: String,
    /// `"json"`, `"csv"`, `"parquet"`, etc.
    #[serde(default = "default_export_format")]
    pub format: String,
}

fn default_export_format() -> String {
    "json".to_string()
}
