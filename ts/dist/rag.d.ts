/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Status of an embedding-generation job.
 */
type JobStatus = "pending" | "in_progress" | "completed" | "failed" | "cancelled";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Status of RAG processing in the runner.
 */
type RagProcessingStatus = "not_started" | "in_progress" | "completed" | "failed";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Request to compute a text embedding for semantic search.
 */
interface ComputeTextEmbeddingRequest {
  /**
   * Embedding model: `"clip"` (512-dim) or `"minilm"` (384-dim).
   */
  model: string;
  /**
   * Text to encode into embedding space.
   */
  text: string;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Response with a computed text embedding.
 */
interface ComputeTextEmbeddingResponse {
  /**
   * Text embedding vector.
   */
  embedding?: number[] | null;
  /**
   * Dimension of the embedding vector.
   */
  embedding_dim: number;
  /**
   * Error message if the request failed.
   */
  error?: string | null;
  /**
   * Processing time in milliseconds.
   */
  processing_time_ms: number;
  /**
   * Whether the embedding computation succeeded.
   */
  success: boolean;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Request to compute embeddings for a single image.
 */
interface ComputeEmbeddingRequest {
  /**
   * Also compute a text embedding from OCR.
   */
  compute_text_embedding: boolean;
  /**
   * Base64-encoded image data.
   */
  image_data: string;
  /**
   * Optional text description for the text embedding.
   */
  text_description?: string | null;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Response with computed embeddings for a single image.
 */
interface ComputeEmbeddingResponse {
  error?: string | null;
  /**
   * CLIP image embedding vector (512 dimensions).
   */
  image_embedding?: number[] | null;
  /**
   * OCR confidence score (0-1).
   */
  ocr_confidence?: number | null;
  /**
   * Text extracted via OCR.
   */
  ocr_text?: string | null;
  /**
   * Processing time in milliseconds.
   */
  processing_time_ms: number;
  success: boolean;
  /**
   * AI-generated text description.
   */
  text_description?: string | null;
  /**
   * Text embedding vector (384 dimensions).
   */
  text_embedding?: number[] | null;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Request to compute embeddings for multiple images.
 */
interface BatchComputeEmbeddingRequest {
  /**
   * Compute text embeddings for all images.
   */
  compute_text_embeddings: boolean;
  /**
   * Extract OCR text from images.
   */
  extract_ocr: boolean;
  /**
   * Images, each a map with `id`, `image_data` (base64), and optional
   * `text_description`.
   */
  images: {
    [k: string]: unknown;
  }[];
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Result for a single image in batch processing.
 */
interface BatchEmbeddingResult {
  error?: string | null;
  /**
   * Image identifier from the request.
   */
  id: string;
  image_embedding?: number[] | null;
  ocr_confidence?: number | null;
  ocr_text?: string | null;
  success: boolean;
  text_description?: string | null;
  text_embedding?: number[] | null;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Response with batch-computed embeddings.
 */
interface BatchComputeEmbeddingResponse {
  failed: number;
  processing_time_ms: number;
  results: BatchEmbeddingResult[];
  /**
   * Whether the batch succeeded overall.
   */
  success: boolean;
  successful: number;
  total_processed: number;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Single embedding result from the runner.
 */
interface EmbeddingResultItem {
  error?: string | null;
  /**
   * CLIP image embedding vector (512 dimensions).
   */
  image_embedding?: number[] | null;
  ocr_confidence?: number | null;
  ocr_text?: string | null;
  /**
   * ID of the state image that was processed.
   */
  state_image_id: string;
  success: boolean;
  text_description?: string | null;
  /**
   * Text embedding vector (384 dimensions for all-MiniLM-L6-v2).
   */
  text_embedding?: number[] | null;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Request containing embedding results from the runner.
 */
interface EmbeddingResultsRequest {
  failed: number;
  project_id: string;
  results: EmbeddingResultItem[];
  successful: number;
  total_processed: number;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Response after applying embedding results.
 */
interface EmbeddingResultsResponse {
  applied: number;
  failed: number;
  message: string;
  not_found: number;
  success: boolean;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Progress event emitted during RAG processing.
 */
interface RagProgressEvent {
  elements_processed?: number | null;
  error?: string | null;
  /**
   * Human-readable status message.
   */
  message: string;
  /**
   * Progress percentage (0-100).
   */
  percent?: number | null;
  project_id: string;
  status: RagProcessingStatus;
  total_elements?: number | null;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Completion event emitted when RAG processing finishes.
 */
interface RagCompletionEvent {
  failed: number;
  project_id: string;
  results: EmbeddingResultItem[];
  success: boolean;
  successful: number;
  total_processed: number;
  web_sync_error?: string | null;
  /**
   * Whether sync to the web backend succeeded.
   */
  web_sync_success?: boolean | null;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Summary of an embedding-generation job.
 */
interface JobSummary {
  error_message?: string | null;
  /**
   * Job UUID.
   */
  id: string;
  processed_patterns: number;
  /**
   * Progress percentage (0-100).
   */
  progress_percent: number;
  /**
   * ISO 8601 UTC timestamp (`Z` suffix).
   */
  started_at?: string | null;
  status: JobStatus;
  total_patterns: number;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Summary statistics for the RAG dashboard.
 */
interface RAGDashboardStats {
  /**
   * Currently-running job, if any.
   */
  active_job?: JobSummary | null;
  /**
   * ISO 8601 UTC timestamp (`Z` suffix).
   */
  last_sync_at?: string | null;
  total_embeddings: number;
  total_patterns: number;
  total_states: number;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Single embedding record for display.
 */
interface EmbeddingItem {
  /**
   * ISO 8601 UTC timestamp (`Z` suffix).
   */
  created_at: string;
  embedding_model: string;
  embedding_version: string;
  /**
   * Whether a text embedding vector is available.
   */
  has_text_embedding: boolean;
  /**
   * Embedding UUID.
   */
  id: string;
  image_height: number;
  image_id: string;
  image_storage_path: string;
  /**
   * Presigned URL for displaying the image.
   */
  image_url?: string | null;
  image_width: number;
  pattern_id: string;
  /**
   * Additional pattern metadata.
   */
  pattern_metadata: {
    [k: string]: unknown;
  };
  pattern_name?: string | null;
  state_id: string;
  state_name: string;
  text_description?: string | null;
  /**
   * ISO 8601 UTC timestamp (`Z` suffix).
   */
  updated_at: string;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Paginated list of embeddings.
 */
interface EmbeddingListResponse {
  has_more: boolean;
  items: EmbeddingItem[];
  limit: number;
  /**
   * 1-indexed page number.
   */
  page: number;
  total: number;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Single job record for display.
 */
interface JobItem {
  completed_at?: string | null;
  /**
   * ISO 8601 UTC timestamp (`Z` suffix).
   */
  created_at: string;
  error_message?: string | null;
  id: string;
  job_metadata: {
    [k: string]: unknown;
  };
  max_retries: number;
  processed_patterns: number;
  progress_percent: number;
  retry_count: number;
  started_at?: string | null;
  status: JobStatus;
  total_patterns: number;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Paginated list of jobs.
 */
interface JobListResponse {
  has_more: boolean;
  items: JobItem[];
  limit: number;
  page: number;
  total: number;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Request for semantic search.
 */
interface SemanticSearchRequest {
  /**
   * Max results to return (1–100).
   */
  limit: number;
  /**
   * Minimum similarity threshold (0–1). CLIP text-to-image similarities
   * typically fall in 0.15–0.35, so 0.2 is a reasonable default.
   */
  min_similarity: number;
  /**
   * Search query text (min length 1).
   */
  query: string;
  state_filter?: string | null;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Single search result.
 */
interface SearchResultItem {
  embedding: EmbeddingItem;
  /**
   * Similarity score (0-1).
   */
  similarity_score: number;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Response from semantic search.
 */
interface SemanticSearchResponse {
  /**
   * The original search query.
   */
  query: string;
  results: SearchResultItem[];
  total_found: number;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * State item for the filter dropdown.
 */
interface StateFilterItem {
  /**
   * Number of embeddings in this state.
   */
  count: number;
  state_id: string;
  state_name: string;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Response with list of states for filtering.
 */
interface StatesResponse {
  /**
   * Total number of states.
   */
  count: number;
  states: StateFilterItem[];
}

export type { BatchComputeEmbeddingRequest, BatchComputeEmbeddingResponse, BatchEmbeddingResult, ComputeEmbeddingRequest, ComputeEmbeddingResponse, ComputeTextEmbeddingRequest, ComputeTextEmbeddingResponse, EmbeddingItem, EmbeddingListResponse, EmbeddingResultItem, EmbeddingResultsRequest, EmbeddingResultsResponse, JobItem, JobListResponse, JobStatus, JobSummary, RAGDashboardStats, RagCompletionEvent, RagProcessingStatus, RagProgressEvent, SearchResultItem, SemanticSearchRequest, SemanticSearchResponse, StateFilterItem, StatesResponse };
