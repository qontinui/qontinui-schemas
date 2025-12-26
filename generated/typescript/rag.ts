/**
 * Auto-generated TypeScript types from qontinui-schemas
 * DO NOT EDIT - regenerate with: poetry run python scripts/generate_typescript.py
 */

export enum JobStatus {
  PENDING = "pending",
  IN_PROGRESS = "in_progress",
  COMPLETED = "completed",
  FAILED = "failed",
  CANCELLED = "cancelled",
}

export enum RagProcessingStatus {
  NOT_STARTED = "not_started",
  IN_PROGRESS = "in_progress",
  COMPLETED = "completed",
  FAILED = "failed",
}

export interface ComputeTextEmbeddingRequest {
  /** Text to encode into embedding space */
  text: string;
}

export interface ComputeTextEmbeddingResponse {
  /** Whether embedding computation succeeded */
  success: boolean;
  /** CLIP text embedding vector (512 dimensions) */
  embedding?: number[] | null;
  /** Dimension of the embedding vector */
  embedding_dim?: number;
  /** Processing time in milliseconds */
  processing_time_ms?: number;
  /** Error message if failed */
  error?: string | null;
}

export interface ComputeEmbeddingRequest {
  /** Base64 encoded image data */
  image_data: string;
  /** Also compute text embedding from OCR */
  compute_text_embedding?: boolean;
  /** Optional text description for text embedding */
  text_description?: string | null;
}

export interface ComputeEmbeddingResponse {
  /** Whether embedding computation succeeded */
  success: boolean;
  /** CLIP image embedding vector (512 dimensions) */
  image_embedding?: number[] | null;
  /** Text embedding vector (384 dimensions) */
  text_embedding?: number[] | null;
  /** AI-generated text description */
  text_description?: string | null;
  /** Text extracted via OCR */
  ocr_text?: string | null;
  /** OCR confidence score (0-1) */
  ocr_confidence?: number | null;
  /** Processing time in milliseconds */
  processing_time_ms?: number;
  /** Error message if failed */
  error?: string | null;
}

export interface BatchComputeEmbeddingRequest {
  /** List of images with 'id', 'image_data' (base64), and optional 'text_description' */
  images: Record<string, any>[];
  /** Compute text embeddings for all images */
  compute_text_embeddings?: boolean;
  /** Extract OCR text from images */
  extract_ocr?: boolean;
}

export interface BatchEmbeddingResult {
  /** Image identifier from the request */
  id: string;
  /** Whether embedding computation succeeded */
  success: boolean;
  /** CLIP image embedding vector (512 dimensions) */
  image_embedding?: number[] | null;
  /** Text embedding vector (384 dimensions) */
  text_embedding?: number[] | null;
  /** AI-generated text description */
  text_description?: string | null;
  /** Text extracted via OCR */
  ocr_text?: string | null;
  /** OCR confidence score (0-1) */
  ocr_confidence?: number | null;
  /** Error message if failed */
  error?: string | null;
}

export interface BatchComputeEmbeddingResponse {
  /** Whether batch processing succeeded overall */
  success: boolean;
  /** Results for each image */
  results: BatchEmbeddingResult[];
  /** Total number of images processed */
  total_processed: number;
  /** Number of successful embeddings */
  successful: number;
  /** Number of failed embeddings */
  failed: number;
  /** Total processing time in milliseconds */
  processing_time_ms?: number;
}

export interface EmbeddingResultItem {
  /** ID of the state image that was processed */
  state_image_id: string;
  /** Whether embedding generation succeeded */
  success: boolean;
  /** CLIP image embedding vector (512 dimensions) */
  image_embedding?: number[] | null;
  /** Text embedding vector (384 dimensions for all-MiniLM-L6-v2) */
  text_embedding?: number[] | null;
  /** AI-generated text description of the element */
  text_description?: string | null;
  /** Text extracted via OCR from the image */
  ocr_text?: string | null;
  /** Confidence score of OCR extraction (0-1) */
  ocr_confidence?: number | null;
  /** Error message if processing failed */
  error?: string | null;
}

export interface EmbeddingResultsRequest {
  /** Project ID the embeddings belong to */
  project_id: string;
  /** List of embedding results */
  results: EmbeddingResultItem[];
  /** Total number of elements processed */
  total_processed: number;
  /** Number of successfully processed elements */
  successful: number;
  /** Number of failed elements */
  failed: number;
}

export interface EmbeddingResultsResponse {
  /** Whether the sync operation succeeded */
  success: boolean;
  /** Status message */
  message: string;
  /** Number of embeddings applied to config */
  applied: number;
  /** Number of embeddings that failed to apply */
  failed: number;
  /** Number of state images not found in config */
  not_found: number;
}

export interface RagProgressEvent {
  /** Project ID being processed */
  project_id: string;
  /** Current processing status */
  status: RagProcessingStatus;
  /** Human-readable status message */
  message: string;
  /** Progress percentage (0-100) */
  percent?: number | null;
  /** Number of elements processed so far */
  elements_processed?: number | null;
  /** Total number of elements to process */
  total_elements?: number | null;
  /** Error message if failed */
  error?: string | null;
}

export interface RagCompletionEvent {
  /** Project ID that was processed */
  project_id: string;
  /** Whether processing completed successfully */
  success: boolean;
  /** Individual element results */
  results: EmbeddingResultItem[];
  /** Total elements processed */
  total_processed: number;
  /** Number of successful elements */
  successful: number;
  /** Number of failed elements */
  failed: number;
  /** Whether sync to web backend succeeded */
  web_sync_success?: boolean | null;
  /** Error message if web sync failed */
  web_sync_error?: string | null;
}

export interface JobSummary {
  /** Job UUID */
  id: string;
  /** Current job status */
  status: JobStatus;
  /** Progress percentage (0-100) */
  progress_percent: number;
  /** Total patterns to process */
  total_patterns: number;
  /** Patterns processed so far */
  processed_patterns: number;
  /** When the job started */
  started_at?: string | null;
  /** Error message if job failed */
  error_message?: string | null;
}

export interface RAGDashboardStats {
  /** Total number of indexed embeddings */
  total_embeddings: number;
  /** Number of unique states with embeddings */
  total_states: number;
  /** Number of unique patterns */
  total_patterns: number;
  /** When runner last synced embeddings */
  last_sync_at?: string | null;
  /** Currently running job if any */
  active_job?: JobSummary | null;
}

export interface EmbeddingItem {
  /** Embedding UUID */
  id: string;
  /** Pattern identifier */
  pattern_id: string;
  /** Human-readable pattern name */
  pattern_name?: string | null;
  /** State identifier */
  state_id: string;
  /** Human-readable state name */
  state_name: string;
  /** Image identifier */
  image_id: string;
  /** Path to image in object storage */
  image_storage_path: string;
  /** Presigned URL for displaying the image */
  image_url?: string | null;
  /** Model used for embedding (e.g., clip-vit-base-patch32) */
  embedding_model: string;
  /** Version of the embedding model */
  embedding_version: string;
  /** Image width in pixels */
  image_width: number;
  /** Image height in pixels */
  image_height: number;
  /** AI-generated or manual text description */
  text_description?: string | null;
  /** Whether text embedding vector is available */
  has_text_embedding?: boolean;
  /** Additional pattern metadata */
  pattern_metadata?: Record<string, any>;
  /** When the embedding was created */
  created_at: string;
  /** When the embedding was last updated */
  updated_at: string;
}

export interface EmbeddingListResponse {
  /** List of embedding items */
  items: EmbeddingItem[];
  /** Total number of embeddings matching filter */
  total: number;
  /** Current page number (1-indexed) */
  page: number;
  /** Items per page */
  limit: number;
  /** Whether more pages exist */
  has_more: boolean;
}

export interface JobItem {
  /** Job UUID */
  id: string;
  /** Current job status */
  status: JobStatus;
  /** Total patterns to process */
  total_patterns: number;
  /** Patterns processed so far */
  processed_patterns: number;
  /** Progress percentage (0-100) */
  progress_percent: number;
  /** Error message if job failed */
  error_message?: string | null;
  /** Number of retry attempts */
  retry_count: number;
  /** Maximum retry attempts allowed */
  max_retries: number;
  /** Additional job metadata */
  job_metadata?: Record<string, any>;
  /** When the job was created */
  created_at: string;
  /** When the job started */
  started_at?: string | null;
  /** When the job completed */
  completed_at?: string | null;
}

export interface JobListResponse {
  /** List of job items */
  items: JobItem[];
  /** Total number of jobs matching filter */
  total: number;
  /** Current page number (1-indexed) */
  page: number;
  /** Items per page */
  limit: number;
  /** Whether more pages exist */
  has_more: boolean;
}

export interface SemanticSearchRequest {
  /** Search query text */
  query: string;
  /** Max results to return */
  limit?: number;
  /** Minimum similarity threshold */
  min_similarity?: number;
  /** Filter by state ID */
  state_filter?: string | null;
}

export interface SearchResultItem {
  /** The matched embedding */
  embedding: EmbeddingItem;
  /** Similarity score (0-1) */
  similarity_score: number;
}

export interface SemanticSearchResponse {
  /** List of search results */
  results: SearchResultItem[];
  /** The original search query */
  query: string;
  /** Total number of results found */
  total_found: number;
}

export interface StateFilterItem {
  /** State identifier */
  state_id: string;
  /** Human-readable state name */
  state_name: string;
  /** Number of embeddings in this state */
  count: number;
}

export interface StatesResponse {
  /** List of states */
  states: StateFilterItem[];
  /** Total number of states */
  count: number;
}
