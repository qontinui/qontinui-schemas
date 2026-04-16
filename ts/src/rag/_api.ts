/**
 * RAG (Retrieval-Augmented Generation) API Types
 *
 * Covers embedding computation (qontinui-api), sync (runner → backend),
 * progress events, dashboard stats, semantic search, and state filter.
 *
 * Source of truth: qontinui-schemas/rust/src/rag.rs.
 */

// Enums
export type { JobStatus } from "../generated/JobStatus";
export type { RagProcessingStatus } from "../generated/RagProcessingStatus";

// Embedding computation
export type { ComputeTextEmbeddingRequest } from "../generated/ComputeTextEmbeddingRequest";
export type { ComputeTextEmbeddingResponse } from "../generated/ComputeTextEmbeddingResponse";
export type { ComputeEmbeddingRequest } from "../generated/ComputeEmbeddingRequest";
export type { ComputeEmbeddingResponse } from "../generated/ComputeEmbeddingResponse";
export type { BatchComputeEmbeddingRequest } from "../generated/BatchComputeEmbeddingRequest";
export type { BatchEmbeddingResult } from "../generated/BatchEmbeddingResult";
export type { BatchComputeEmbeddingResponse } from "../generated/BatchComputeEmbeddingResponse";

// Embedding sync
export type { EmbeddingResultItem } from "../generated/EmbeddingResultItem";
export type { EmbeddingResultsRequest } from "../generated/EmbeddingResultsRequest";
export type { EmbeddingResultsResponse } from "../generated/EmbeddingResultsResponse";

// Progress events
export type { RagProgressEvent } from "../generated/RagProgressEvent";
export type { RagCompletionEvent } from "../generated/RagCompletionEvent";

// Dashboard
export type { JobSummary } from "../generated/JobSummary";
export type { RAGDashboardStats } from "../generated/RAGDashboardStats";
export type { EmbeddingItem } from "../generated/EmbeddingItem";
export type { EmbeddingListResponse } from "../generated/EmbeddingListResponse";
export type { JobItem } from "../generated/JobItem";
export type { JobListResponse } from "../generated/JobListResponse";

// Semantic search
export type { SemanticSearchRequest } from "../generated/SemanticSearchRequest";
export type { SearchResultItem } from "../generated/SearchResultItem";
export type { SemanticSearchResponse } from "../generated/SemanticSearchResponse";

// State filter
export type { StateFilterItem } from "../generated/StateFilterItem";
export type { StatesResponse } from "../generated/StatesResponse";
