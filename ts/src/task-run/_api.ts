/**
 * Task Run Types
 *
 * Merged type definitions from both runner (taskRun.ts) and web (task-runs.ts).
 * Runner types are used for local execution, web types for backend API communication.
 *
 * Types generated from Rust (source of truth: qontinui-schemas/rust/src/task_run.rs).
 * Do not edit by hand — regenerate via `just generate-types` (or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`).
 */

// =============================================================================
// Status Types
// =============================================================================

export type { TaskRunStatus } from "../generated/TaskRunStatus";
export type { TaskType } from "../generated/TaskType";

// =============================================================================
// Runner Task Run (local execution)
// =============================================================================

export type { TaskRun } from "../generated/TaskRun";

// =============================================================================
// Web Backend Task Run (API responses)
// =============================================================================

export type { TaskRunBackend } from "../generated/TaskRunBackend";

// =============================================================================
// Session Types
// =============================================================================

export type { TaskRunSession } from "../generated/TaskRunSession";

// =============================================================================
// Finding Types
// =============================================================================

export type { TaskRunFindingCategory } from "../generated/TaskRunFindingCategory";
export type { TaskRunFindingSeverity } from "../generated/TaskRunFindingSeverity";
export type { TaskRunFindingStatus } from "../generated/TaskRunFindingStatus";
export type { TaskRunFindingActionType } from "../generated/TaskRunFindingActionType";
export type { TaskRunFinding } from "../generated/TaskRunFinding";

// Local alias: `TaskRunFindingResponse` is a TS-only convenience synonym for
// `TaskRunFinding` (the Rust side uses `TaskRunFinding` directly).
import type { TaskRunFinding as _TaskRunFinding } from "../generated/TaskRunFinding";
export type TaskRunFindingResponse = _TaskRunFinding;

// =============================================================================
// Detail Types
// =============================================================================

export type { TaskRunFindingSummary } from "../generated/TaskRunFindingSummary";
export type { TaskRunBackendDetail } from "../generated/TaskRunBackendDetail";

// =============================================================================
// Request/Update Types
// =============================================================================

export type { TaskRunCreate } from "../generated/TaskRunCreate";
export type { TaskRunUpdate } from "../generated/TaskRunUpdate";
export type { TaskRunFindingCreate } from "../generated/TaskRunFindingCreate";
export type { TaskRunFindingUpdate } from "../generated/TaskRunFindingUpdate";

// =============================================================================
// Runner-Specific Request Types
// =============================================================================

// `RunPromptResponse.data` was an inline object in the old hand-authored TS;
// the Rust side lifted it to a named `RunPromptResponseData` struct, so we
// re-export both.
export type { RunPromptResponseData } from "../generated/RunPromptResponseData";
export type { RunPromptResponse } from "../generated/RunPromptResponse";
export type { RunPromptRequest } from "../generated/RunPromptRequest";
export type { CreateTaskRunRequest } from "../generated/CreateTaskRunRequest";

// =============================================================================
// Filter Types
// =============================================================================

export type { TaskRunFilters } from "../generated/TaskRunFilters";
export type { TaskRunFindingFilters } from "../generated/TaskRunFindingFilters";

// =============================================================================
// Response Types
// =============================================================================

export type { Pagination } from "../generated/Pagination";
export type { TaskRunListResponse } from "../generated/TaskRunListResponse";
export type { TaskRunFindingsListResponse } from "../generated/TaskRunFindingsListResponse";
export type { FindingsSummary } from "../generated/FindingsSummary";

// =============================================================================
// Verification Result Types
// =============================================================================

export type { CheckIssueDetail } from "../generated/CheckIssueDetail";
export type { IndividualCheckResult } from "../generated/IndividualCheckResult";
export type { VerificationStepDetails } from "../generated/VerificationStepDetails";
export type { StepExecutionConfig } from "../generated/StepExecutionConfig";
export type { VerificationStepResult } from "../generated/VerificationStepResult";
export type { GateEvaluationResult } from "../generated/GateEvaluationResult";
export type { VerificationPhaseResult } from "../generated/VerificationPhaseResult";
export type { VerificationResultResponse } from "../generated/VerificationResultResponse";
export type { VerificationResultsListResponse } from "../generated/VerificationResultsListResponse";
