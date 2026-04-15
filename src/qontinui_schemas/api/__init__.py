"""API schemas for Qontinui services.

This module contains request/response schemas for API endpoints
shared between backend, frontend, and runner.
"""

from qontinui_schemas.api.evaluation import (
    DatasetItemCreate,
    DatasetItemListResponse,
    DatasetItemResponse,
    EvaluationDatasetCreate,
    EvaluationDatasetListResponse,
    EvaluationDatasetResponse,
    EvaluationExperimentCreate,
    EvaluationExperimentListResponse,
    EvaluationExperimentResponse,
    ExperimentResultCreate,
    ExperimentResultListResponse,
    ExperimentResultResponse,
    ExperimentStatusUpdate,
    ExperimentSummary,
)
from qontinui_schemas.api.execution import (
    ActionExecutionBatch,
    ActionExecutionBatchResponse,
    ActionExecutionCreate,
    ActionExecutionListResponse,
    ActionExecutionResponse,
    ActionReliabilityStats,
    ActionStatus,
    ActionType,
    CostTrendDataPoint,
    CostTrendResponse,
    CoverageData,
    ExecutionIssueBatch,
    ExecutionIssueBatchResponse,
    ExecutionIssueCreate,
    ExecutionIssueDetail,
    ExecutionIssueListResponse,
    ExecutionIssueResponse,
    ExecutionIssueUpdate,
    ExecutionRunComplete,
    ExecutionRunCompleteResponse,
    ExecutionRunCreate,
    ExecutionRunDetail,
    ExecutionRunListResponse,
    ExecutionRunResponse,
    ExecutionScreenshotCreate,
    ExecutionScreenshotResponse,
    ExecutionStats,
    ExecutionTrendDataPoint,
    ExecutionTrendResponse,
    ExecutionWorkflowMetadata,
    HistoricalActionQuery,
    HistoricalActionResult,
    IssueSeverity,
    IssueSource,
    IssueStatus,
    IssueType,
    LLMCostSummary,
    LLMMetrics,
    ModelCostBreakdown,
    PlaybackFrameRequest,
    RunnerMetadata,
    RunStatus,
    RunType,
    VisualComparisonResult,
)
from qontinui_schemas.api.execution import Pagination as ExecutionPagination
from qontinui_schemas.api.execution import (
    # Enums; Metadata; Execution Run; Action Execution;
    # Screenshot; Issue; Analytics; Historical Playback; Pagination
    ScreenshotType as ExecutionScreenshotType,
)
from qontinui_schemas.api.feedback import (
    FeedbackScoreBatchResponse,
    FeedbackScoreCreate,
    FeedbackScoreListResponse,
    FeedbackScoreResponse,
    FeedbackScoreSummary,
)
from qontinui_schemas.api.prompt_versions import (  # Prompt Versioning
    PromptVersionCreate,
    PromptVersionDiff,
    PromptVersionListResponse,
    PromptVersionResponse,
)
from qontinui_schemas.api.rag import (
    # Enums; Embedding Computation; Embedding Sync;
    # RAG Progress Events; RAG Dashboard; Semantic Search; State Filter
    BatchComputeEmbeddingRequest,
    BatchComputeEmbeddingResponse,
    BatchEmbeddingResult,
    ComputeEmbeddingRequest,
    ComputeEmbeddingResponse,
    ComputeTextEmbeddingRequest,
    ComputeTextEmbeddingResponse,
    EmbeddingItem,
    EmbeddingListResponse,
    EmbeddingResultItem,
    EmbeddingResultsRequest,
    EmbeddingResultsResponse,
    JobItem,
    JobListResponse,
    JobStatus,
    JobSummary,
    RagCompletionEvent,
    RAGDashboardStats,
    RagProcessingStatus,
    RagProgressEvent,
    SearchResultItem,
    SemanticSearchRequest,
    SemanticSearchResponse,
    StateFilterItem,
    StatesResponse,
)
from qontinui_schemas.api.testing import (
    # Enums; Request schemas; Response schemas;
    # Analytics; Historical Data (Config Testing)
    ActionDataBatch,
    ActionDataBatchResponse,
    ActionDataCreate,
    CoverageTrendDataPoint,
    CoverageTrendResponse,
    CoverageUpdate,
    CoverageUpdateResponse,
    DeficiencyBatchCreate,
    DeficiencyBatchResponse,
    DeficiencyCreate,
    DeficiencyDetail,
    DeficiencyListResponse,
    DeficiencyResponse,
    DeficiencySeverity,
    DeficiencyStatus,
    DeficiencyType,
    DeficiencyUpdate,
    HistoricalFrameResponse,
    HistoricalResultRequest,
    HistoricalResultResponse,
    PaginatedResponse,
    Pagination,
    PlaybackRequest,
    ReliabilityResponse,
    ScreenshotMetadata,
    ScreenshotType,
    ScreenshotUploadResponse,
    TestRunComplete,
    TestRunCompleteResponse,
    TestRunCreate,
    TestRunDetail,
    TestRunListResponse,
    TestRunResponse,
    TestRunStatus,
    TransitionBatchCreate,
    TransitionBatchResponse,
    TransitionCreate,
    TransitionReliabilityStats,
    TransitionResponse,
    TransitionStatus,
    VisualComparisonSummary,
)

__all__ = [
    # ==========================================================================
    # Unified Execution API (NEW - replaces fragmented systems)
    # ==========================================================================
    # Execution Enums
    "RunType",
    "RunStatus",
    "ActionType",
    "ActionStatus",
    "ExecutionScreenshotType",
    "IssueSeverity",
    "IssueStatus",
    "IssueType",
    "IssueSource",
    # Metadata
    "RunnerMetadata",
    "ExecutionWorkflowMetadata",
    # Execution Run
    "ExecutionRunCreate",
    "ExecutionRunResponse",
    "ExecutionRunDetail",
    "ExecutionRunListResponse",
    "ExecutionRunComplete",
    "ExecutionRunCompleteResponse",
    "ExecutionStats",
    "CoverageData",
    # Action Execution
    "ActionExecutionCreate",
    "ActionExecutionBatch",
    "ActionExecutionResponse",
    "ActionExecutionBatchResponse",
    "ActionExecutionListResponse",
    # Screenshot
    "ExecutionScreenshotCreate",
    "ExecutionScreenshotResponse",
    "VisualComparisonResult",
    # Issue
    "ExecutionIssueCreate",
    "ExecutionIssueBatch",
    "ExecutionIssueResponse",
    "ExecutionIssueDetail",
    "ExecutionIssueListResponse",
    "ExecutionIssueBatchResponse",
    "ExecutionIssueUpdate",
    # LLM Metrics
    "LLMMetrics",
    "LLMCostSummary",
    "ModelCostBreakdown",
    # Analytics
    "ActionReliabilityStats",
    "ExecutionTrendDataPoint",
    "ExecutionTrendResponse",
    "CostTrendDataPoint",
    "CostTrendResponse",
    # Historical Playback
    "HistoricalActionQuery",
    "HistoricalActionResult",
    "PlaybackFrameRequest",
    "ExecutionPagination",
    # ==========================================================================
    # RAG API
    # ==========================================================================
    # RAG Enums
    "JobStatus",
    "RagProcessingStatus",
    # RAG Embedding Computation (qontinui-api)
    "ComputeTextEmbeddingRequest",
    "ComputeTextEmbeddingResponse",
    "ComputeEmbeddingRequest",
    "ComputeEmbeddingResponse",
    "BatchComputeEmbeddingRequest",
    "BatchEmbeddingResult",
    "BatchComputeEmbeddingResponse",
    # RAG Embedding Sync (Runner → Backend)
    "EmbeddingResultItem",
    "EmbeddingResultsRequest",
    "EmbeddingResultsResponse",
    # RAG Progress Events (Runner → UI)
    "RagProgressEvent",
    "RagCompletionEvent",
    # RAG Dashboard (Backend → Frontend)
    "RAGDashboardStats",
    "JobSummary",
    "JobItem",
    "JobListResponse",
    "EmbeddingItem",
    "EmbeddingListResponse",
    # Semantic Search
    "SemanticSearchRequest",
    "SearchResultItem",
    "SemanticSearchResponse",
    # State Filter
    "StateFilterItem",
    "StatesResponse",
    # ==========================================================================
    # Prompt Versioning API
    # ==========================================================================
    "PromptVersionCreate",
    "PromptVersionResponse",
    "PromptVersionListResponse",
    "PromptVersionDiff",
    # Testing Enums
    "TestRunStatus",
    "TransitionStatus",
    "DeficiencySeverity",
    "DeficiencyStatus",
    "DeficiencyType",
    "ScreenshotType",
    # Testing Request schemas
    "TestRunCreate",
    "TransitionCreate",
    "TransitionBatchCreate",
    "DeficiencyCreate",
    "DeficiencyBatchCreate",
    "DeficiencyUpdate",
    "CoverageUpdate",
    "TestRunComplete",
    "ScreenshotMetadata",
    # Testing Response schemas
    "TestRunResponse",
    "TestRunDetail",
    "TestRunListResponse",
    "TransitionResponse",
    "TransitionBatchResponse",
    "DeficiencyResponse",
    "DeficiencyDetail",
    "DeficiencyListResponse",
    "DeficiencyBatchResponse",
    "CoverageUpdateResponse",
    "TestRunCompleteResponse",
    "ScreenshotUploadResponse",
    "VisualComparisonSummary",
    "Pagination",
    "PaginatedResponse",
    # Analytics
    "CoverageTrendDataPoint",
    "CoverageTrendResponse",
    "TransitionReliabilityStats",
    "ReliabilityResponse",
    # Historical Data (Config Testing)
    "HistoricalResultRequest",
    "HistoricalResultResponse",
    "ActionDataCreate",
    "ActionDataBatch",
    "ActionDataBatchResponse",
    "HistoricalFrameResponse",
    "PlaybackRequest",
    # ==========================================================================
    # Feedback Scores API (Opik integration)
    # ==========================================================================
    "FeedbackScoreBatchResponse",
    "FeedbackScoreCreate",
    "FeedbackScoreResponse",
    "FeedbackScoreSummary",
    "FeedbackScoreListResponse",
    # ==========================================================================
    # Evaluation Datasets & Experiments API
    # ==========================================================================
    # Datasets
    "EvaluationDatasetCreate",
    "EvaluationDatasetResponse",
    "EvaluationDatasetListResponse",
    "DatasetItemCreate",
    "DatasetItemResponse",
    # Experiments
    "EvaluationExperimentCreate",
    "EvaluationExperimentResponse",
    "EvaluationExperimentListResponse",
    "ExperimentResultCreate",
    "ExperimentResultResponse",
    "DatasetItemListResponse",
    "ExperimentResultListResponse",
    "ExperimentStatusUpdate",
    "ExperimentSummary",
]
