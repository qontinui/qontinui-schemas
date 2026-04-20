/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Kind of execution run being reported.
 */
type RunType$1 = "qa_test" | "integration_test" | "live_automation" | "recording" | "debug";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Lifecycle status of an execution run.
 */
type RunStatus$1 = "pending" | "running" | "completed" | "failed" | "timeout" | "cancelled" | "paused";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Outcome of a single action within a run.
 */
type ActionStatus$1 = "success" | "failed" | "timeout" | "skipped" | "error" | "pending";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Type of action executed.
 *
 * Covers vision, input, state-machine, control-flow, utility, AI, and custom
 * action kinds. Variants are serialized as snake_case strings.
 */
type ExecutionActionType =
  | "find"
  | "find_all"
  | "wait_for"
  | "wait_until_gone"
  | "click"
  | "double_click"
  | "right_click"
  | "type"
  | "press_key"
  | "hotkey"
  | "scroll"
  | "drag"
  | "go_to_state"
  | "transition"
  | "verify_state"
  | "conditional"
  | "loop"
  | "parallel"
  | "sequence"
  | "wait"
  | "screenshot"
  | "log"
  | "assert"
  | "ai_prompt"
  | "run_prompt_sequence"
  | "custom";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Category of error that caused an action to fail.
 */
type ErrorType$1 =
  | "element_not_found"
  | "timeout"
  | "assertion_failed"
  | "crash"
  | "network_error"
  | "validation_error"
  | "other";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Severity of an issue reported against a run.
 */
type IssueSeverity$1 = "critical" | "high" | "medium" | "low" | "informational";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Kind of screenshot captured.
 */
type ScreenshotType$1 = "error" | "success" | "manual" | "periodic" | "action_result" | "state_verification";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Kind of shape overlaid on a screenshot annotation.
 */
type ScreenshotAnnotationShape = "box" | "circle" | "arrow" | "text";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Information about the runner environment that produced the run.
 */
interface RunnerMetadata {
  /**
   * CPU description.
   */
  cpuInfo?: string | null;
  /**
   * Arbitrary additional runner context.
   */
  extra?: {
    [k: string]: unknown;
  } | null;
  /**
   * Host machine name.
   */
  hostname: string;
  /**
   * Installed system memory in megabytes.
   */
  memoryMb?: number | null;
  /**
   * Operating system identifier (e.g., `"windows"`, `"macos"`, `"linux"`).
   */
  os: string;
  /**
   * Semantic version of the runner binary.
   */
  runnerVersion: string;
  /**
   * Screen resolution as a free-form string (e.g., `"1920x1080"`).
   */
  screenResolution?: string | null;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Information about the workflow being executed in the run.
 */
interface WorkflowMetadata {
  /**
   * Workflow description.
   */
  description?: string | null;
  /**
   * IDs of the states that are active when the workflow starts.
   */
  initialStateIds?: string[] | null;
  /**
   * Free-form tags attached to the workflow.
   */
  tags?: string[] | null;
  /**
   * Number of states declared by the workflow.
   */
  totalStates?: number | null;
  /**
   * Number of transitions declared by the workflow.
   */
  totalTransitions?: number | null;
  /**
   * Workflow identifier.
   */
  workflowId: string;
  /**
   * Human-readable workflow name.
   */
  workflowName: string;
  /**
   * Workflow version, if tracked.
   */
  workflowVersion?: string | null;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Aggregate execution statistics for a completed run.
 */
interface ExecutionStats {
  /**
   * Mean per-action duration, in milliseconds.
   */
  avgActionDurationMs?: number | null;
  /**
   * Number of actions with [`ActionStatus::Failed`].
   */
  failedActions: number;
  /**
   * Number of actions that used an LLM.
   */
  llmActionCount?: number | null;
  /**
   * Number of actions with [`ActionStatus::Skipped`].
   */
  skippedActions: number;
  /**
   * Number of actions with [`ActionStatus::Success`].
   */
  successfulActions: number;
  /**
   * Number of actions with [`ActionStatus::Timeout`].
   */
  timeoutActions: number;
  /**
   * Total number of actions executed.
   */
  totalActions: number;
  /**
   * Aggregate estimated cost in USD across all LLM actions.
   */
  totalCostUsd?: number | null;
  /**
   * Sum of all action durations, in milliseconds.
   */
  totalDurationMs: number;
  /**
   * Aggregate input tokens across all LLM actions.
   */
  totalTokensInput?: number | null;
  /**
   * Aggregate output tokens across all LLM actions.
   */
  totalTokensOutput?: number | null;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Coverage data computed for a workflow run.
 */
interface CoverageData {
  /**
   * Overall coverage as a percentage in the range `[0.0, 100.0]`.
   */
  coveragePercentage: number;
  /**
   * Per-state visit counts, keyed by state ID.
   */
  stateVisitCounts?: {
    [k: string]: number;
  } | null;
  /**
   * Number of states visited at least once.
   */
  statesCovered: number;
  /**
   * Total number of states in the workflow.
   */
  totalStates: number;
  /**
   * Total number of transitions in the workflow.
   */
  totalTransitions: number;
  /**
   * Per-transition execution counts, keyed by transition ID.
   */
  transitionExecutionCounts?: {
    [k: string]: number;
  } | null;
  /**
   * Number of transitions executed at least once.
   */
  transitionsCovered: number;
  /**
   * IDs of states that were not visited.
   */
  uncoveredStates?: string[] | null;
  /**
   * IDs of transitions that were not executed.
   */
  uncoveredTransitions?: string[] | null;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Token usage and cost metrics for an LLM-powered action.
 */
interface LLMMetrics {
  /**
   * Estimated cost in USD.
   */
  costUsd?: number | null;
  /**
   * Generation parameters (temperature, max_tokens, etc.).
   */
  generationParams?: {
    [k: string]: unknown;
  } | null;
  /**
   * LLM model identifier.
   */
  model?: string | null;
  /**
   * Provider name (e.g., `"anthropic"`, `"openai"`).
   */
  provider?: string | null;
  /**
   * Input/prompt token count.
   */
  tokensInput?: number | null;
  /**
   * Completion token count.
   */
  tokensOutput?: number | null;
  /**
   * Computed total token count.
   */
  tokensTotal?: number | null;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Request payload for creating a new execution run.
 */
interface ExecutionRunCreate {
  /**
   * Run configuration bag (opaque to this layer).
   */
  configuration?: {
    [k: string]: unknown;
  } | null;
  /**
   * Optional free-form description.
   */
  description?: string | null;
  /**
   * Owning project ID.
   */
  projectId: string;
  /**
   * Human-readable run name.
   */
  runName: string;
  runType: RunType$1;
  runnerMetadata: RunnerMetadata;
  /**
   * Workflow metadata, if the run executes a workflow.
   */
  workflowMetadata?: WorkflowMetadata | null;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Response envelope returned when a run is created or fetched.
 */
interface ExecutionRunResponse {
  /**
   * Total duration in seconds, if the run has ended.
   */
  durationSeconds?: number | null;
  /**
   * ISO 8601 timestamp when the run ended, if it has ended.
   */
  endedAt?: string | null;
  /**
   * Owning project ID.
   */
  projectId: string;
  /**
   * Assigned run identifier.
   */
  runId: string;
  /**
   * Human-readable run name.
   */
  runName: string;
  runType: RunType$1;
  /**
   * ISO 8601 timestamp when the run started.
   */
  startedAt: string;
  status: RunStatus$1;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Pixel coordinates and optional size of a matched pattern.
 *
 * Inline object on [`ActionExecutionCreate::match_location`]. Lifted to a
 * named struct so it round-trips through JSON Schema.
 */
interface ExecutionMatchLocation {
  /**
   * Match height in pixels.
   */
  height?: number | null;
  /**
   * Match width in pixels.
   */
  width?: number | null;
  /**
   * X coordinate in pixels.
   */
  x: number;
  /**
   * Y coordinate in pixels.
   */
  y: number;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Request payload for reporting a single action execution.
 *
 * Usually submitted as part of a batch; see [`ActionExecutionResponse`].
 */
interface ActionExecutionCreate {
  /**
   * Human-readable action name.
   */
  actionName: string;
  actionType: ExecutionActionType;
  /**
   * IDs of states active when the action ran.
   */
  activeStates?: string[] | null;
  /**
   * ISO 8601 timestamp when the action completed.
   */
  completedAt: string;
  /**
   * Confidence score of the match in the range `[0.0, 1.0]`.
   */
  confidenceScore?: number | null;
  /**
   * Action duration in milliseconds.
   */
  durationMs: number;
  /**
   * Error message if the action failed.
   */
  errorMessage?: string | null;
  /**
   * Captured stack trace, if any.
   */
  errorStack?: string | null;
  /**
   * Category of error, if any.
   */
  errorType?: ErrorType$1 | null;
  /**
   * Source state ID for state-machine actions.
   */
  fromState?: string | null;
  /**
   * Opaque input data captured for the action.
   */
  inputData?: {
    [k: string]: unknown;
  } | null;
  /**
   * LLM token and cost metrics, if the action used an LLM.
   */
  llmMetrics?: LLMMetrics | null;
  /**
   * Pixel location of the match.
   */
  matchLocation?: ExecutionMatchLocation | null;
  /**
   * Opaque additional metadata.
   */
  metadata?: {
    [k: string]: unknown;
  } | null;
  /**
   * Opaque output data produced by the action.
   */
  outputData?: {
    [k: string]: unknown;
  } | null;
  /**
   * ID of a parent action, if this is a sub-action.
   */
  parentActionId?: string | null;
  /**
   * Parent action ID for child actions within a sequence.
   */
  parentId?: string | null;
  /**
   * ID of the pattern the action targeted.
   */
  patternId?: string | null;
  /**
   * Human-readable pattern name.
   */
  patternName?: string | null;
  /**
   * ID of a screenshot associated with the action.
   */
  screenshotId?: string | null;
  /**
   * Zero-based sequence number of the action within the run.
   */
  sequenceNumber: number;
  /**
   * Span type for tracing (e.g., `"llm"`, `"tool"`, `"agent"`).
   */
  spanType?: string | null;
  /**
   * ISO 8601 timestamp when the action started.
   */
  startedAt: string;
  status: ActionStatus$1;
  /**
   * Destination state ID for state-machine actions.
   */
  toState?: string | null;
  /**
   * Trace ID correlating related actions.
   */
  traceId?: string | null;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Response envelope returned after reporting action executions.
 */
interface ActionExecutionResponse {
  /**
   * Assigned action IDs, in the same order as the submitted batch.
   */
  actionIds?: string[] | null;
  /**
   * Number of actions recorded.
   */
  recorded: number;
  /**
   * Associated run ID.
   */
  runId: string;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Annotation overlaid on a screenshot (box, circle, arrow, or text).
 */
interface ScreenshotAnnotation {
  /**
   * CSS-style color string (e.g., `"#FF0000"`).
   */
  color?: string | null;
  /**
   * Annotation height in pixels.
   */
  height?: number | null;
  /**
   * Free-form label for the annotation.
   */
  label?: string | null;
  type: ScreenshotAnnotationShape;
  /**
   * Annotation width in pixels.
   */
  width?: number | null;
  /**
   * X coordinate in pixels.
   */
  x: number;
  /**
   * Y coordinate in pixels.
   */
  y: number;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Request payload describing a screenshot being uploaded.
 */
interface ExecutionScreenshotCreate {
  /**
   * Sequence number of the associated action, if any.
   */
  actionSequenceNumber?: number | null;
  /**
   * IDs of states active when the screenshot was taken.
   */
  activeStates?: string[] | null;
  /**
   * Overlaid annotations.
   */
  annotations?: ScreenshotAnnotation[] | null;
  /**
   * Image height in pixels.
   */
  height: number;
  /**
   * Opaque additional metadata.
   */
  metadata?: {
    [k: string]: unknown;
  } | null;
  /**
   * Client-generated screenshot identifier.
   */
  screenshotId: string;
  screenshotType: ScreenshotType$1;
  /**
   * Sequence number of the screenshot within the run.
   */
  sequenceNumber: number;
  /**
   * State ID active when the screenshot was taken.
   */
  state?: string | null;
  /**
   * ISO 8601 timestamp when the screenshot was taken.
   */
  timestamp: string;
  /**
   * Image width in pixels.
   */
  width: number;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Response envelope returned after uploading a screenshot.
 */
interface ExecutionScreenshotResponse {
  /**
   * File size in bytes.
   */
  fileSizeBytes: number;
  /**
   * URL to the full-size image.
   */
  imageUrl: string;
  /**
   * Associated run ID.
   */
  runId: string;
  /**
   * Assigned screenshot identifier.
   */
  screenshotId: string;
  /**
   * URL to a thumbnail image, if generated.
   */
  thumbnailUrl?: string | null;
  /**
   * ISO 8601 timestamp when the image was uploaded.
   */
  uploadedAt: string;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Request payload for reporting an issue observed during a run.
 */
interface ExecutionIssueCreate {
  /**
   * Sequence number of the associated action, if any.
   */
  actionSequenceNumber?: number | null;
  /**
   * Actual observed behavior.
   */
  actualBehavior?: string | null;
  /**
   * Full issue description.
   */
  description: string;
  /**
   * Expected behavior.
   */
  expectedBehavior?: string | null;
  /**
   * Free-form issue type label (e.g., `"visual_regression"`, `"flaky"`).
   */
  issueType: string;
  /**
   * Opaque additional metadata.
   */
  metadata?: {
    [k: string]: unknown;
  } | null;
  /**
   * Steps to reproduce the issue.
   */
  reproductionSteps?: string[] | null;
  /**
   * IDs of screenshots illustrating the issue.
   */
  screenshotIds?: string[] | null;
  severity: IssueSeverity$1;
  /**
   * State ID active when the issue was observed.
   */
  state?: string | null;
  /**
   * Short human-readable title.
   */
  title: string;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Response envelope returned after reporting issues.
 */
interface ExecutionIssueResponse {
  /**
   * Assigned issue IDs, in the same order as the submitted batch.
   */
  issueIds?: string[] | null;
  /**
   * Number of issues recorded.
   */
  recorded: number;
  /**
   * Associated run ID.
   */
  runId: string;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Request payload for marking a run complete.
 */
interface ExecutionRunComplete {
  /**
   * Coverage data, if the run executed a workflow.
   */
  coverage?: CoverageData | null;
  /**
   * ISO 8601 timestamp when the run ended.
   */
  endedAt: string;
  /**
   * Error message if the run failed.
   */
  errorMessage?: string | null;
  stats: ExecutionStats;
  status: RunStatus$1;
  /**
   * Free-form run summary.
   */
  summary?: string | null;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Response envelope returned after completing a run.
 */
interface ExecutionRunCompleteResponse {
  /**
   * Coverage data, if the run executed a workflow.
   */
  coverage?: CoverageData | null;
  /**
   * Total duration in seconds.
   */
  durationSeconds: number;
  /**
   * ISO 8601 timestamp when the run ended.
   */
  endedAt: string;
  /**
   * Associated run ID.
   */
  runId: string;
  /**
   * ISO 8601 timestamp when the run started.
   */
  startedAt: string;
  stats: ExecutionStats;
  status: RunStatus$1;
}

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

type RunType = RunType$1;
declare const RunType: {
    readonly QA_TEST: "qa_test";
    readonly INTEGRATION_TEST: "integration_test";
    readonly LIVE_AUTOMATION: "live_automation";
    readonly RECORDING: "recording";
    readonly DEBUG: "debug";
};

type RunStatus = RunStatus$1;
declare const RunStatus: {
    readonly PENDING: "pending";
    readonly RUNNING: "running";
    readonly COMPLETED: "completed";
    readonly FAILED: "failed";
    readonly TIMEOUT: "timeout";
    readonly CANCELLED: "cancelled";
    readonly PAUSED: "paused";
};

type ActionStatus = ActionStatus$1;
declare const ActionStatus: {
    readonly SUCCESS: "success";
    readonly FAILED: "failed";
    readonly TIMEOUT: "timeout";
    readonly SKIPPED: "skipped";
    readonly ERROR: "error";
    readonly PENDING: "pending";
};

type ActionType = ExecutionActionType;
declare const ActionType: {
    readonly FIND: "find";
    readonly FIND_ALL: "find_all";
    readonly WAIT_FOR: "wait_for";
    readonly WAIT_UNTIL_GONE: "wait_until_gone";
    readonly CLICK: "click";
    readonly DOUBLE_CLICK: "double_click";
    readonly RIGHT_CLICK: "right_click";
    readonly TYPE: "type";
    readonly PRESS_KEY: "press_key";
    readonly HOTKEY: "hotkey";
    readonly SCROLL: "scroll";
    readonly DRAG: "drag";
    readonly GO_TO_STATE: "go_to_state";
    readonly TRANSITION: "transition";
    readonly VERIFY_STATE: "verify_state";
    readonly CONDITIONAL: "conditional";
    readonly LOOP: "loop";
    readonly PARALLEL: "parallel";
    readonly SEQUENCE: "sequence";
    readonly WAIT: "wait";
    readonly SCREENSHOT: "screenshot";
    readonly LOG: "log";
    readonly ASSERT: "assert";
    readonly AI_PROMPT: "ai_prompt";
    readonly RUN_PROMPT_SEQUENCE: "run_prompt_sequence";
    readonly CUSTOM: "custom";
};

type ErrorType = ErrorType$1;
declare const ErrorType: {
    readonly ELEMENT_NOT_FOUND: "element_not_found";
    readonly TIMEOUT: "timeout";
    readonly ASSERTION_FAILED: "assertion_failed";
    readonly CRASH: "crash";
    readonly NETWORK_ERROR: "network_error";
    readonly VALIDATION_ERROR: "validation_error";
    readonly OTHER: "other";
};

type IssueSeverity = IssueSeverity$1;
declare const IssueSeverity: {
    readonly CRITICAL: "critical";
    readonly HIGH: "high";
    readonly MEDIUM: "medium";
    readonly LOW: "low";
    readonly INFORMATIONAL: "informational";
};

type ScreenshotType = ScreenshotType$1;
declare const ScreenshotType: {
    readonly ERROR: "error";
    readonly SUCCESS: "success";
    readonly MANUAL: "manual";
    readonly PERIODIC: "periodic";
    readonly ACTION_RESULT: "action_result";
    readonly STATE_VERIFICATION: "state_verification";
};

type TaskComplexity = "simple" | "medium" | "complex";
interface RoutingFactor {
    description: string;
    complexity: TaskComplexity;
    weight: number;
}
interface RoutingDecision {
    complexity: TaskComplexity;
    confidence: number;
    factors: string[];
    selectedModel: string;
    timestamp: number;
    promptPreview?: string;
    fileCount?: number;
    criteriaCount?: number;
}
interface RoutingStatus {
    enabled: boolean;
    decision: RoutingDecision | null;
    config: {
        simpleModel: string;
        mediumModel: string;
        complexModel: string;
    };
}
interface RetryAttempt {
    attemptNumber: number;
    error: string;
    timestamp: string;
    delayMs: number;
    feedbackInjected: boolean;
}
interface RetryState {
    attempt: number;
    lastError: string | null;
    lastAttemptAt: string | null;
    totalDelayMs: number;
    errorHistory: RetryAttempt[];
}
interface RetryStatus {
    enabled: boolean;
    maxRetries: number;
    feedbackInjection: boolean;
    state: RetryState;
    nextRetryDelayMs: number | null;
    exhausted: boolean;
}
interface TokenCount {
    total: number;
    findings: number;
    observations: number;
    feedback: number;
    solutions: number;
    other: number;
    entryCount: number;
}
interface CompressionResult {
    originalTokens: number;
    compressedTokens: number;
    itemsSummarized: number;
    summaryEntriesCreated: number;
    compressedCategories: string[];
    timestamp: string;
}
interface CompressionStatus {
    enabled: boolean;
    thresholdTokens: number;
    targetTokens: number;
    currentTokenCount: TokenCount | null;
    lastCompression: CompressionResult | null;
    thresholdPercentage: number;
    compressionImminent: boolean;
}
type HookTrigger = "pre_execution" | "post_execution" | "on_error" | "on_verification_fail" | "on_complete" | "pre_iteration" | "post_iteration";
interface HookExecutionResult {
    hookId: string;
    hookName: string;
    trigger: HookTrigger;
    success: boolean;
    output: string | null;
    error: string | null;
    durationMs: number;
    timestamp: string;
}
interface HookDefinition {
    id: string;
    name: string;
    trigger: HookTrigger;
    enabled: boolean;
    actionType: string;
}
interface HookStatus {
    hooks: HookDefinition[];
    executionHistory: HookExecutionResult[];
    currentlyExecuting: string | null;
}
type SubStepStatus = "pending" | "running" | "completed" | "failed" | "skipped";
interface SubStepInfo {
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
interface SubStepStatusDisplay {
    current: SubStepInfo | null;
    steps: SubStepInfo[];
    progressPercent: number;
    completedCount: number;
    totalCount: number;
    isActive: boolean;
    currentPhase: string | null;
}
interface ExecutionStatus {
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
interface RawExecutionStatusEventBase {
    type: string;
    task_run_id: string;
    timestamp: number;
}
interface RawRoutingDecisionPayload {
    complexity: string;
    confidence: number;
    factors: string[];
    selected_model: string;
    prompt_preview?: string;
    file_count?: number;
    criteria_count?: number;
}
interface RawRetryAttemptPayload {
    attempt_number: number;
    error: string;
    attempt_timestamp: string;
    delay_ms: number;
    feedback_injected: boolean;
}
interface RawRetryStatePayload {
    attempt: number;
    last_error: string | null;
    last_attempt_at: string | null;
    total_delay_ms: number;
    error_history: RawRetryAttemptPayload[];
}
interface RawTokenCountPayload {
    total: number;
    findings: number;
    observations: number;
    feedback: number;
    solutions: number;
    other: number;
    entry_count: number;
}
interface RawCompressionResultPayload {
    original_tokens: number;
    compressed_tokens: number;
    items_summarized: number;
    summary_entries_created: number;
    compressed_categories: string[];
    timestamp: string;
}
interface RawHookExecutionPayload {
    hook_id: string;
    hook_name: string;
    trigger: string;
    success: boolean;
    output: string | null;
    error: string | null;
    duration_ms: number;
    timestamp: string;
}
interface RawRoutingDecisionEvent extends RawExecutionStatusEventBase {
    type: "routing_decision";
    decision: RawRoutingDecisionPayload;
}
interface RawRetryAttemptEvent extends RawExecutionStatusEventBase {
    type: "retry_attempt";
    attempt: RawRetryAttemptPayload;
    state: RawRetryStatePayload;
    exhausted: boolean;
    next_retry_delay_ms: number | null;
}
interface RawCompressionEvent extends RawExecutionStatusEventBase {
    type: "compression";
    result: RawCompressionResultPayload;
    current_token_count: RawTokenCountPayload;
}
interface RawTokenCountUpdateEvent extends RawExecutionStatusEventBase {
    type: "token_count_update";
    token_count: RawTokenCountPayload;
    threshold_percentage: number;
    compression_imminent: boolean;
}
interface RawHookExecutionEvent extends RawExecutionStatusEventBase {
    type: "hook_execution";
    result: RawHookExecutionPayload;
}
interface RawHookStartedEvent extends RawExecutionStatusEventBase {
    type: "hook_started";
    hook_id: string;
    hook_name: string;
    trigger: string;
}
interface RawStatusChangeEvent extends RawExecutionStatusEventBase {
    type: "status_change";
    status: string;
    iteration: number;
    task_name: string | null;
}
interface RawSubStepCompleteEvent {
    type: "sub_step_complete";
    checkpoint_id: string;
    task_run_id: string;
    sub_step_id: string;
    description: string | null;
    timestamp: number;
}
interface RawSubStepStartedEvent {
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
type RawExecutionStatusEvent = RawRoutingDecisionEvent | RawRetryAttemptEvent | RawCompressionEvent | RawTokenCountUpdateEvent | RawHookExecutionEvent | RawHookStartedEvent | RawStatusChangeEvent;

export { type ActionExecutionCreate, type ActionExecutionResponse, ActionStatus, ActionType, type CompressionResult, type CompressionStatus, type CoverageData, ErrorType, type ExecutionIssueCreate, type ExecutionIssueResponse, type ExecutionRunComplete, type ExecutionRunCompleteResponse, type ExecutionRunCreate, type ExecutionRunResponse, type ExecutionScreenshotCreate, type ExecutionScreenshotResponse, type ExecutionStats, type ExecutionStatus, type HookDefinition, type HookExecutionResult, type HookStatus, type HookTrigger, IssueSeverity, type LLMMetrics, type ExecutionMatchLocation as MatchLocation, type RawCompressionEvent, type RawCompressionResultPayload, type RawExecutionStatusEvent, type RawExecutionStatusEventBase, type RawHookExecutionEvent, type RawHookExecutionPayload, type RawHookStartedEvent, type RawRetryAttemptEvent, type RawRetryAttemptPayload, type RawRetryStatePayload, type RawRoutingDecisionEvent, type RawRoutingDecisionPayload, type RawStatusChangeEvent, type RawSubStepCompleteEvent, type RawSubStepStartedEvent, type RawTokenCountPayload, type RawTokenCountUpdateEvent, type RetryAttempt, type RetryState, type RetryStatus, type RoutingDecision, type RoutingFactor, type RoutingStatus, RunStatus, RunType, type RunnerMetadata, type ScreenshotAnnotation, type ScreenshotAnnotationShape, ScreenshotType, type SubStepInfo, type SubStepStatus, type SubStepStatusDisplay, type TaskComplexity, type TokenCount, type WorkflowMetadata };
