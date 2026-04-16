/**
 * Execution Types
 *
 * Type definitions for the unified execution reporting API that supports
 * multiple run types: QA testing, integration testing, live automation,
 * recording sessions, and debug runs.
 *
 * Tier 1 (enums) and Tier 2 (request/response DTOs) are generated from Rust
 * (source of truth: qontinui-schemas/rust/src/execution.rs). Do not edit
 * those sections by hand — regenerate via `just generate-types` (or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`).
 *
 * Tier 3 (UI display / live-status-stream types) remains hand-authored below.
 */

// ============================================================================
// Enums (Tier 1, generated)
// ============================================================================
//
// For enums that had named-constant call-site syntax in the old hand-authored
// TS (e.g. `RunType.QA_TEST`), we import the generated literal union under an
// alias, re-export it as the type name, and declare a companion `const` object
// with the same name so `RunType.QA_TEST` still resolves as a value. TS merges
// the type alias and const value into one declaration usable in both
// positions.

import type { RunType as _RunType } from "../generated/RunType";
export type RunType = _RunType;
export const RunType = {
  QA_TEST: "qa_test",
  INTEGRATION_TEST: "integration_test",
  LIVE_AUTOMATION: "live_automation",
  RECORDING: "recording",
  DEBUG: "debug",
} as const;

import type { RunStatus as _RunStatus } from "../generated/RunStatus";
export type RunStatus = _RunStatus;
export const RunStatus = {
  PENDING: "pending",
  RUNNING: "running",
  COMPLETED: "completed",
  FAILED: "failed",
  TIMEOUT: "timeout",
  CANCELLED: "cancelled",
  PAUSED: "paused",
} as const;

import type { ActionStatus as _ActionStatus } from "../generated/ActionStatus";
export type ActionStatus = _ActionStatus;
export const ActionStatus = {
  SUCCESS: "success",
  FAILED: "failed",
  TIMEOUT: "timeout",
  SKIPPED: "skipped",
  ERROR: "error",
  PENDING: "pending",
} as const;

// `ActionType` collides with `tree_events::ActionType` on the Rust side, so
// the generated symbol is `ExecutionActionType`. We preserve the local name
// `ActionType` for backward compatibility.
import type { ExecutionActionType as _ActionType } from "../generated/ExecutionActionType";
export type ActionType = _ActionType;
export const ActionType = {
  // Vision actions
  FIND: "find",
  FIND_ALL: "find_all",
  WAIT_FOR: "wait_for",
  WAIT_UNTIL_GONE: "wait_until_gone",

  // Input actions
  CLICK: "click",
  DOUBLE_CLICK: "double_click",
  RIGHT_CLICK: "right_click",
  TYPE: "type",
  PRESS_KEY: "press_key",
  HOTKEY: "hotkey",
  SCROLL: "scroll",
  DRAG: "drag",

  // State machine actions
  GO_TO_STATE: "go_to_state",
  TRANSITION: "transition",
  VERIFY_STATE: "verify_state",

  // Control flow
  CONDITIONAL: "conditional",
  LOOP: "loop",
  PARALLEL: "parallel",
  SEQUENCE: "sequence",

  // Utility
  WAIT: "wait",
  SCREENSHOT: "screenshot",
  LOG: "log",
  ASSERT: "assert",

  // AI actions
  AI_PROMPT: "ai_prompt",
  RUN_PROMPT_SEQUENCE: "run_prompt_sequence",

  // Custom/plugin
  CUSTOM: "custom",
} as const;

import type { ErrorType as _ErrorType } from "../generated/ErrorType";
export type ErrorType = _ErrorType;
export const ErrorType = {
  ELEMENT_NOT_FOUND: "element_not_found",
  TIMEOUT: "timeout",
  ASSERTION_FAILED: "assertion_failed",
  CRASH: "crash",
  NETWORK_ERROR: "network_error",
  VALIDATION_ERROR: "validation_error",
  OTHER: "other",
} as const;

import type { IssueSeverity as _IssueSeverity } from "../generated/IssueSeverity";
export type IssueSeverity = _IssueSeverity;
export const IssueSeverity = {
  CRITICAL: "critical",
  HIGH: "high",
  MEDIUM: "medium",
  LOW: "low",
  INFORMATIONAL: "informational",
} as const;

import type { ScreenshotType as _ScreenshotType } from "../generated/ScreenshotType";
export type ScreenshotType = _ScreenshotType;
export const ScreenshotType = {
  ERROR: "error",
  SUCCESS: "success",
  MANUAL: "manual",
  PERIODIC: "periodic",
  ACTION_RESULT: "action_result",
  STATE_VERIFICATION: "state_verification",
} as const;

// Shape of annotation overlay on a screenshot. Was inline on
// `ExecutionScreenshotCreate.annotations[].type` previously. No call sites
// used a named-constant syntax, so a plain re-export suffices.
export type { ScreenshotAnnotationShape } from "../generated/ScreenshotAnnotationShape";

// ============================================================================
// Metadata & DTO Types (Tier 2, generated)
// ============================================================================

export type { RunnerMetadata } from "../generated/RunnerMetadata";
export type { WorkflowMetadata } from "../generated/WorkflowMetadata";
export type { ExecutionStats } from "../generated/ExecutionStats";
export type { CoverageData } from "../generated/CoverageData";
export type { LLMMetrics } from "../generated/LLMMetrics";

// ============================================================================
// Request/Create + Response Types (Tier 2, generated)
// ============================================================================

export type { ExecutionRunCreate } from "../generated/ExecutionRunCreate";
export type { ExecutionRunResponse } from "../generated/ExecutionRunResponse";

// `MatchLocation` collides with `tree_events::MatchLocation` on the Rust
// side, so the generated symbol is `ExecutionMatchLocation`. Preserve the
// local name via a re-export alias.
export type { ExecutionMatchLocation as MatchLocation } from "../generated/ExecutionMatchLocation";

export type { ActionExecutionCreate } from "../generated/ActionExecutionCreate";
export type { ActionExecutionResponse } from "../generated/ActionExecutionResponse";

export type { ScreenshotAnnotation } from "../generated/ScreenshotAnnotation";
export type { ExecutionScreenshotCreate } from "../generated/ExecutionScreenshotCreate";
export type { ExecutionScreenshotResponse } from "../generated/ExecutionScreenshotResponse";

export type { ExecutionIssueCreate } from "../generated/ExecutionIssueCreate";
export type { ExecutionIssueResponse } from "../generated/ExecutionIssueResponse";

export type { ExecutionRunComplete } from "../generated/ExecutionRunComplete";
export type { ExecutionRunCompleteResponse } from "../generated/ExecutionRunCompleteResponse";

// ============================================================================
// Execution Status Types (real-time display) — Tier 3, hand-authored
// ============================================================================

export type TaskComplexity = "simple" | "medium" | "complex";

export interface RoutingFactor {
  description: string;
  complexity: TaskComplexity;
  weight: number;
}

export interface RoutingDecision {
  complexity: TaskComplexity;
  confidence: number;
  factors: string[];
  selectedModel: string;
  timestamp: number;
  promptPreview?: string;
  fileCount?: number;
  criteriaCount?: number;
}

export interface RoutingStatus {
  enabled: boolean;
  decision: RoutingDecision | null;
  config: {
    simpleModel: string;
    mediumModel: string;
    complexModel: string;
  };
}

export interface RetryAttempt {
  attemptNumber: number;
  error: string;
  timestamp: string;
  delayMs: number;
  feedbackInjected: boolean;
}

export interface RetryState {
  attempt: number;
  lastError: string | null;
  lastAttemptAt: string | null;
  totalDelayMs: number;
  errorHistory: RetryAttempt[];
}

export interface RetryStatus {
  enabled: boolean;
  maxRetries: number;
  feedbackInjection: boolean;
  state: RetryState;
  nextRetryDelayMs: number | null;
  exhausted: boolean;
}

export interface TokenCount {
  total: number;
  findings: number;
  observations: number;
  feedback: number;
  solutions: number;
  other: number;
  entryCount: number;
}

export interface CompressionResult {
  originalTokens: number;
  compressedTokens: number;
  itemsSummarized: number;
  summaryEntriesCreated: number;
  compressedCategories: string[];
  timestamp: string;
}

export interface CompressionStatus {
  enabled: boolean;
  thresholdTokens: number;
  targetTokens: number;
  currentTokenCount: TokenCount | null;
  lastCompression: CompressionResult | null;
  thresholdPercentage: number;
  compressionImminent: boolean;
}

export type HookTrigger =
  | "pre_execution"
  | "post_execution"
  | "on_error"
  | "on_verification_fail"
  | "on_complete"
  | "pre_iteration"
  | "post_iteration";

export interface HookExecutionResult {
  hookId: string;
  hookName: string;
  trigger: HookTrigger;
  success: boolean;
  output: string | null;
  error: string | null;
  durationMs: number;
  timestamp: string;
}

export interface HookDefinition {
  id: string;
  name: string;
  trigger: HookTrigger;
  enabled: boolean;
  actionType: string;
}

export interface HookStatus {
  hooks: HookDefinition[];
  executionHistory: HookExecutionResult[];
  currentlyExecuting: string | null;
}

export type SubStepStatus =
  | "pending"
  | "running"
  | "completed"
  | "failed"
  | "skipped";

export interface SubStepInfo {
  id: string;
  parentId: string;
  name: string;
  status: SubStepStatus;
  index: number;
  totalCount: number;
  startedAt: number | null;
  completedAt: number | null;
  durationMs: number | null;
  phase?: string;
}

export interface SubStepStatusDisplay {
  current: SubStepInfo | null;
  steps: SubStepInfo[];
  progressPercent: number;
  completedCount: number;
  totalCount: number;
  isActive: boolean;
  currentPhase: string | null;
}

export interface ExecutionStatus {
  taskRunId: string | null;
  taskName: string | null;
  iteration: number;
  status: "idle" | "running" | "completed" | "failed" | "paused";
  routing: RoutingStatus;
  retry: RetryStatus;
  compression: CompressionStatus;
  hooks: HookStatus;
  subSteps: SubStepStatusDisplay;
  lastUpdated: number;
}

// ============================================================================
// Raw Event Types (snake_case from backend) — Tier 3, hand-authored
// ============================================================================

export interface RawExecutionStatusEventBase {
  type: string;
  task_run_id: string;
  timestamp: number;
}

export interface RawRoutingDecisionPayload {
  complexity: string;
  confidence: number;
  factors: string[];
  selected_model: string;
  prompt_preview?: string;
  file_count?: number;
  criteria_count?: number;
}

export interface RawRetryAttemptPayload {
  attempt_number: number;
  error: string;
  attempt_timestamp: string;
  delay_ms: number;
  feedback_injected: boolean;
}

export interface RawRetryStatePayload {
  attempt: number;
  last_error: string | null;
  last_attempt_at: string | null;
  total_delay_ms: number;
  error_history: RawRetryAttemptPayload[];
}

export interface RawTokenCountPayload {
  total: number;
  findings: number;
  observations: number;
  feedback: number;
  solutions: number;
  other: number;
  entry_count: number;
}

export interface RawCompressionResultPayload {
  original_tokens: number;
  compressed_tokens: number;
  items_summarized: number;
  summary_entries_created: number;
  compressed_categories: string[];
  timestamp: string;
}

export interface RawHookExecutionPayload {
  hook_id: string;
  hook_name: string;
  trigger: string;
  success: boolean;
  output: string | null;
  error: string | null;
  duration_ms: number;
  timestamp: string;
}

export interface RawRoutingDecisionEvent extends RawExecutionStatusEventBase {
  type: "routing_decision";
  decision: RawRoutingDecisionPayload;
}

export interface RawRetryAttemptEvent extends RawExecutionStatusEventBase {
  type: "retry_attempt";
  attempt: RawRetryAttemptPayload;
  state: RawRetryStatePayload;
  exhausted: boolean;
  next_retry_delay_ms: number | null;
}

export interface RawCompressionEvent extends RawExecutionStatusEventBase {
  type: "compression";
  result: RawCompressionResultPayload;
  current_token_count: RawTokenCountPayload;
}

export interface RawTokenCountUpdateEvent extends RawExecutionStatusEventBase {
  type: "token_count_update";
  token_count: RawTokenCountPayload;
  threshold_percentage: number;
  compression_imminent: boolean;
}

export interface RawHookExecutionEvent extends RawExecutionStatusEventBase {
  type: "hook_execution";
  result: RawHookExecutionPayload;
}

export interface RawHookStartedEvent extends RawExecutionStatusEventBase {
  type: "hook_started";
  hook_id: string;
  hook_name: string;
  trigger: string;
}

export interface RawStatusChangeEvent extends RawExecutionStatusEventBase {
  type: "status_change";
  status: string;
  iteration: number;
  task_name: string | null;
}

export interface RawSubStepCompleteEvent {
  type: "sub_step_complete";
  checkpoint_id: string;
  task_run_id: string;
  sub_step_id: string;
  description: string | null;
  timestamp: number;
}

export interface RawSubStepStartedEvent {
  type: "sub_step_started";
  checkpoint_id: string;
  task_run_id: string;
  sub_step_id: string;
  sub_step_index: number;
  total_sub_steps: number;
  description: string | null;
  phase: string | null;
  timestamp: number;
}

export type RawExecutionStatusEvent =
  | RawRoutingDecisionEvent
  | RawRetryAttemptEvent
  | RawCompressionEvent
  | RawTokenCountUpdateEvent
  | RawHookExecutionEvent
  | RawHookStartedEvent
  | RawStatusChangeEvent;
