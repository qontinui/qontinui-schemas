/**
 * Execution Types
 *
 * Type definitions for the unified execution reporting API that supports
 * multiple run types: QA testing, integration testing, live automation,
 * recording sessions, and debug runs.
 */
declare enum RunType {
    QA_TEST = "qa_test",
    INTEGRATION_TEST = "integration_test",
    LIVE_AUTOMATION = "live_automation",
    RECORDING = "recording",
    DEBUG = "debug"
}
declare enum RunStatus {
    PENDING = "pending",
    RUNNING = "running",
    COMPLETED = "completed",
    FAILED = "failed",
    TIMEOUT = "timeout",
    CANCELLED = "cancelled",
    PAUSED = "paused"
}
declare enum ActionStatus {
    SUCCESS = "success",
    FAILED = "failed",
    TIMEOUT = "timeout",
    SKIPPED = "skipped",
    ERROR = "error",
    PENDING = "pending"
}
declare enum ActionType {
    FIND = "find",
    FIND_ALL = "find_all",
    WAIT_FOR = "wait_for",
    WAIT_UNTIL_GONE = "wait_until_gone",
    CLICK = "click",
    DOUBLE_CLICK = "double_click",
    RIGHT_CLICK = "right_click",
    TYPE = "type",
    PRESS_KEY = "press_key",
    HOTKEY = "hotkey",
    SCROLL = "scroll",
    DRAG = "drag",
    GO_TO_STATE = "go_to_state",
    TRANSITION = "transition",
    VERIFY_STATE = "verify_state",
    CONDITIONAL = "conditional",
    LOOP = "loop",
    PARALLEL = "parallel",
    SEQUENCE = "sequence",
    WAIT = "wait",
    SCREENSHOT = "screenshot",
    LOG = "log",
    ASSERT = "assert",
    AI_PROMPT = "ai_prompt",
    RUN_PROMPT_SEQUENCE = "run_prompt_sequence",
    CUSTOM = "custom"
}
declare enum ErrorType {
    ELEMENT_NOT_FOUND = "element_not_found",
    TIMEOUT = "timeout",
    ASSERTION_FAILED = "assertion_failed",
    CRASH = "crash",
    NETWORK_ERROR = "network_error",
    VALIDATION_ERROR = "validation_error",
    OTHER = "other"
}
declare enum IssueSeverity {
    CRITICAL = "critical",
    HIGH = "high",
    MEDIUM = "medium",
    LOW = "low",
    INFORMATIONAL = "informational"
}
declare enum ScreenshotType {
    ERROR = "error",
    SUCCESS = "success",
    MANUAL = "manual",
    PERIODIC = "periodic",
    ACTION_RESULT = "action_result",
    STATE_VERIFICATION = "state_verification"
}
interface RunnerMetadata {
    runner_version: string;
    os: string;
    hostname: string;
    screen_resolution?: string;
    cpu_info?: string;
    memory_mb?: number;
    extra?: Record<string, unknown>;
}
interface WorkflowMetadata {
    workflow_id: string;
    workflow_name: string;
    workflow_version?: string;
    total_states?: number;
    total_transitions?: number;
    tags?: string[];
    description?: string;
    initial_state_ids?: string[];
}
interface ExecutionStats {
    total_actions: number;
    successful_actions: number;
    failed_actions: number;
    timeout_actions: number;
    skipped_actions: number;
    total_duration_ms: number;
    avg_action_duration_ms?: number;
    /** Aggregate input tokens across all LLM actions */
    total_tokens_input?: number;
    /** Aggregate output tokens across all LLM actions */
    total_tokens_output?: number;
    /** Aggregate estimated cost in USD across all LLM actions */
    total_cost_usd?: number;
    /** Number of actions that used an LLM */
    llm_action_count?: number;
}
interface CoverageData {
    coverage_percentage: number;
    states_covered: number;
    total_states: number;
    transitions_covered: number;
    total_transitions: number;
    uncovered_states?: string[];
    uncovered_transitions?: string[];
    state_visit_counts?: Record<string, number>;
    transition_execution_counts?: Record<string, number>;
}
/** Token and cost metrics for an LLM-powered action. */
interface LLMMetrics {
    /** LLM model identifier */
    model?: string;
    /** Provider name (e.g. anthropic, openai) */
    provider?: string;
    /** Input/prompt token count */
    tokens_input?: number;
    /** Completion token count */
    tokens_output?: number;
    /** Computed total token count */
    tokens_total?: number;
    /** Estimated cost in USD */
    cost_usd?: number;
    /** Generation parameters (temperature, max_tokens, etc.) */
    generation_params?: Record<string, unknown>;
}
interface ExecutionRunCreate {
    project_id: string;
    run_type: RunType;
    run_name: string;
    description?: string;
    runner_metadata: RunnerMetadata;
    workflow_metadata?: WorkflowMetadata;
    configuration?: Record<string, unknown>;
}
interface ExecutionRunResponse {
    run_id: string;
    project_id: string;
    run_type: RunType;
    run_name: string;
    status: RunStatus;
    started_at: string;
    ended_at?: string;
    duration_seconds?: number;
}
interface ActionExecutionCreate {
    sequence_number: number;
    action_type: ActionType;
    action_name: string;
    status: ActionStatus;
    started_at: string;
    completed_at: string;
    duration_ms: number;
    from_state?: string;
    to_state?: string;
    active_states?: string[];
    pattern_id?: string;
    pattern_name?: string;
    confidence_score?: number;
    match_location?: {
        x: number;
        y: number;
        width?: number;
        height?: number;
    };
    error_message?: string;
    error_type?: ErrorType;
    error_stack?: string;
    screenshot_id?: string;
    parent_action_id?: string;
    input_data?: Record<string, unknown>;
    output_data?: Record<string, unknown>;
    metadata?: Record<string, unknown>;
    /** LLM token and cost metrics if action used an LLM */
    llm_metrics?: LLMMetrics;
    /** Span type for tracing (e.g. "llm", "tool", "agent") */
    span_type?: string;
    /** Trace ID for correlating related actions */
    trace_id?: string;
    /** Parent action ID for child actions within a sequence */
    parent_id?: string;
}
interface ActionExecutionResponse {
    recorded: number;
    run_id: string;
    action_ids?: string[];
}
interface ExecutionScreenshotCreate {
    screenshot_id: string;
    sequence_number: number;
    screenshot_type: ScreenshotType;
    timestamp: string;
    width: number;
    height: number;
    action_sequence_number?: number;
    state?: string;
    active_states?: string[];
    annotations?: Array<{
        type: "box" | "circle" | "arrow" | "text";
        x: number;
        y: number;
        width?: number;
        height?: number;
        label?: string;
        color?: string;
    }>;
    metadata?: Record<string, unknown>;
}
interface ExecutionScreenshotResponse {
    screenshot_id: string;
    run_id: string;
    image_url: string;
    thumbnail_url?: string;
    uploaded_at: string;
    file_size_bytes: number;
}
interface ExecutionIssueCreate {
    title: string;
    description: string;
    severity: IssueSeverity;
    issue_type: string;
    action_sequence_number?: number;
    state?: string;
    screenshot_ids?: string[];
    reproduction_steps?: string[];
    expected_behavior?: string;
    actual_behavior?: string;
    metadata?: Record<string, unknown>;
}
interface ExecutionIssueResponse {
    recorded: number;
    run_id: string;
    issue_ids?: string[];
}
interface ExecutionRunComplete {
    status: RunStatus;
    ended_at: string;
    stats: ExecutionStats;
    coverage?: CoverageData;
    summary?: string;
    error_message?: string;
}
interface ExecutionRunCompleteResponse {
    run_id: string;
    status: RunStatus;
    started_at: string;
    ended_at: string;
    duration_seconds: number;
    stats: ExecutionStats;
    coverage?: CoverageData;
}
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

export { type ActionExecutionCreate, type ActionExecutionResponse, ActionStatus, ActionType, type CompressionResult, type CompressionStatus, type CoverageData, ErrorType, type ExecutionIssueCreate, type ExecutionIssueResponse, type ExecutionRunComplete, type ExecutionRunCompleteResponse, type ExecutionRunCreate, type ExecutionRunResponse, type ExecutionScreenshotCreate, type ExecutionScreenshotResponse, type ExecutionStats, type ExecutionStatus, type HookDefinition, type HookExecutionResult, type HookStatus, type HookTrigger, IssueSeverity, type LLMMetrics, type RawCompressionEvent, type RawCompressionResultPayload, type RawExecutionStatusEvent, type RawExecutionStatusEventBase, type RawHookExecutionEvent, type RawHookExecutionPayload, type RawHookStartedEvent, type RawRetryAttemptEvent, type RawRetryAttemptPayload, type RawRetryStatePayload, type RawRoutingDecisionEvent, type RawRoutingDecisionPayload, type RawStatusChangeEvent, type RawSubStepCompleteEvent, type RawSubStepStartedEvent, type RawTokenCountPayload, type RawTokenCountUpdateEvent, type RetryAttempt, type RetryState, type RetryStatus, type RoutingDecision, type RoutingFactor, type RoutingStatus, RunStatus, RunType, type RunnerMetadata, ScreenshotType, type SubStepInfo, type SubStepStatus, type SubStepStatusDisplay, type TaskComplexity, type TokenCount, type WorkflowMetadata };
