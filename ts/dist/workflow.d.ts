/**
 * Structured Action Plan Types
 *
 * Defines the schema for LLM-generated UI Bridge action plans.
 * Instead of natural language instructions interpreted by a second LLM call,
 * action plans let the agentic-phase LLM directly specify typed UI actions
 * that map to UI Bridge's control API.
 *
 * Inspired by Skyvern's structured action protocol: each action carries
 * reasoning, confidence, and typed parameters so that execution is
 * deterministic and auditable.
 */
/**
 * Action types that can appear in an action plan.
 * Maps directly to UI Bridge's StandardAction type with additions
 * for navigation and waiting.
 */
type PlannedActionType = "click" | "doubleClick" | "rightClick" | "type" | "clear" | "select" | "check" | "uncheck" | "toggle" | "hover" | "focus" | "scroll" | "scrollIntoView" | "setValue" | "drag" | "submit" | "sendKeys" | "autocomplete" | "navigate" | "wait";
/**
 * How to find the target element for an action.
 * Supports multiple resolution strategies with fallback.
 */
interface ElementTarget {
    /** Direct element ID from a prior snapshot (fastest resolution) */
    elementId?: string;
    /** data-testid attribute value */
    testId?: string;
    /** Natural language description for fuzzy search (fallback) */
    searchText?: string;
    /** Element type hint to narrow search (e.g., "button", "input") */
    elementType?: string;
    /** CSS selector */
    selector?: string;
}
/**
 * A single action in an action plan.
 *
 * The LLM produces an array of these, each mapping to one UI Bridge
 * control action. The reasoning and confidence fields enable auditing
 * and confidence-gated execution.
 */
interface PlannedAction {
    /** Action type to execute */
    action: PlannedActionType;
    /** How to find the target element */
    target: ElementTarget;
    /** LLM's reasoning for this action (audit trail) */
    reasoning?: string;
    /** LLM's confidence that this is the right action (0.0–1.0) */
    confidence: number;
    /**
     * Action-specific parameters.
     * - type/setValue: { text: string }
     * - select: { value: string } or { label: string }
     * - scroll: { direction: "up"|"down"|"left"|"right" }
     * - drag: { targetPosition: { x: number, y: number } }
     * - sendKeys: { keys: string }
     * - navigate: { url: string }
     * - wait: { ms: number }
     */
    params?: Record<string, unknown>;
    /**
     * Separates the generic intent from specific data.
     * Enables action plan caching: the query stays stable across runs,
     * only the answer changes per user context.
     *
     * Example:
     *   userDetailQuery: "What email should be entered?"
     *   userDetailAnswer: "test@example.com"
     */
    userDetailQuery?: string;
    userDetailAnswer?: string;
}
/**
 * A complete action plan: an ordered sequence of UI actions
 * produced by the agentic-phase LLM.
 */
interface ActionPlan {
    /** Ordered list of actions to execute */
    actions: PlannedAction[];
    /** High-level goal this plan achieves */
    goal: string;
    /**
     * Minimum confidence threshold. Actions below this confidence
     * are skipped (or trigger verification) rather than executed.
     * Default: 0.5
     */
    confidenceThreshold?: number;
    /** Whether to stop on first action failure (default: true) */
    stopOnFailure?: boolean;
}
/** Result of executing a single planned action */
interface PlannedActionResult {
    /** Index of the action in the plan */
    index: number;
    /** Whether the action succeeded */
    success: boolean;
    /** The action that was executed */
    action: PlannedActionType;
    /** Element ID that was resolved and acted upon */
    resolvedElementId?: string;
    /** Error message if the action failed */
    error?: string;
    /** Whether the action was skipped due to low confidence */
    skippedLowConfidence?: boolean;
    /** Duration in milliseconds */
    durationMs: number;
    /** Post-action element state (if available) */
    elementState?: Record<string, unknown>;
}
/** Aggregated result of executing a full action plan */
interface ActionPlanResult {
    /** Whether all executed actions succeeded */
    success: boolean;
    /** The goal from the action plan */
    goal?: string;
    /** Per-action results */
    results: PlannedActionResult[];
    /** Count of actions executed (excludes skipped) */
    executedCount: number;
    /** Count of actions skipped due to low confidence */
    skippedCount: number;
    /** Count of actions that failed */
    failedCount: number;
    /** Total duration in milliseconds */
    totalDurationMs: number;
    /** Whether this plan was stored in the cache for future reuse */
    cached?: boolean;
}
/**
 * Extended action plan request with caching fields.
 * Used when calling the endpoint directly (not via workflow steps).
 */
interface ActionPlanExecuteRequest extends ActionPlan {
    /** Page URL for cache keying */
    pageUrl?: string;
    /** Element snapshot for cache fingerprinting (array of {id, type, role, label}) */
    elementSnapshot?: Array<{
        id?: string;
        type?: string;
        role?: string;
        label?: string;
    }>;
}

/**
 * Skill Types
 *
 * A skill is a named, parameterized template that produces pre-configured
 * workflow step(s) when instantiated. Skills sit between raw step types
 * and full workflows:
 *
 *   Raw Step Types  (command, prompt, ui_bridge, workflow)  ← execution primitives
 *        ↑ instantiates
 *   Skills          ("Lint Project", "API Health Check")    ← named capability templates
 *        ↑ composes into
 *   Workflows       (multi-phase verification-agentic loops) ← orchestration
 *
 * Skills are purely a configuration-time abstraction — they produce steps,
 * they do NOT add new runtime behavior.
 */

type SkillCategory = "code-quality" | "testing" | "monitoring" | "ai-task" | "deployment" | "composition" | "custom";
interface SkillAuthor {
    name: string;
    email?: string;
    url?: string;
}
interface SkillParameterOption {
    label: string;
    value: string;
}
interface SkillParameter {
    name: string;
    type: "string" | "number" | "boolean" | "select";
    label: string;
    description: string;
    required: boolean;
    default?: unknown;
    options?: SkillParameterOption[];
    placeholder?: string;
    min?: number;
    max?: number;
    pattern?: string;
    depends_on?: {
        param: string;
        value: unknown;
    };
}
interface SingleStepTemplate {
    kind: "single_step";
    step: Record<string, unknown>;
}
interface MultiStepTemplate {
    kind: "multi_step";
    steps: Record<string, unknown>[];
}
interface CompositionTemplate {
    kind: "composition";
    skill_refs: SkillRef[];
}
interface SkillRef {
    skill_id: string;
    parameter_overrides?: Record<string, unknown>;
}
type SkillTemplate = SingleStepTemplate | MultiStepTemplate | CompositionTemplate;
interface SkillDefinition {
    id: string;
    name: string;
    slug: string;
    description: string;
    category: SkillCategory;
    tags: string[];
    icon: string;
    color: string;
    allowed_phases: WorkflowPhase[];
    parameters: SkillParameter[];
    template: SkillTemplate;
    source: "builtin" | "user" | "community";
    version?: string;
    author?: SkillAuthor;
    checksum?: string;
    depends_on?: string[];
    usage_count?: number;
    approval_status?: "pending" | "approved" | "rejected";
    forked_from?: string;
}
interface SkillOrigin {
    skill_id: string;
    skill_slug: string;
    parameter_values: Record<string, unknown>;
}
interface SkillExportManifest {
    version: string;
    exported_at: string;
    app_version: string;
    content_type: "skills";
    skill_count: number;
    checksum?: string;
}
interface SkillExport {
    manifest: SkillExportManifest;
    skills: SkillDefinition[];
}
interface SkillImportResult {
    imported: number;
    skipped: number;
    overwritten: number;
    errors: string[];
}

/**
 * Workflow Types
 *
 * Canonical type definitions for the unified Workflow Builder system.
 * Shared across qontinui-runner and qontinui-web.
 *
 * All automation is organized into four phases: Setup, Verification, Agentic, Completion.
 *
 * Execution Order:
 *   Setup (once) -> [Verification <-> Agentic]* -> Completion (once)
 *
 * Step Types (4 core types):
 *   command   - Shell commands, checks, check groups, tests
 *   ui_bridge - UI Bridge SDK interactions (navigate, execute, assert, snapshot)
 *   prompt    - AI task instructions
 *   workflow  - Run a saved workflow inline (composition)
 */
type WorkflowPhase = "setup" | "verification" | "agentic" | "completion";
type LogSourceSelection = "default" | "ai" | "all" | {
    profile_id: string;
};
interface HealthCheckUrl {
    name: string;
    url: string;
    expected_status?: number;
    timeout_seconds?: number;
    is_critical?: boolean;
}

interface BaseStep {
    id: string;
    name: string;
    fail_on_console_errors?: boolean;
    inputs?: Record<string, string>;
    extract?: Record<string, string>;
    depends_on?: string[];
    required?: boolean;
    retry?: {
        count: number;
        delay_ms: number;
    };
    skill_origin?: SkillOrigin;
    /** Acceptance criterion IDs this step verifies (supports multiple) */
    criterion_ids?: string[];
    /** Verification depth category for this step */
    verification_category?: VerificationCategory;
}
type HttpMethod = "GET" | "POST" | "PUT" | "PATCH" | "DELETE";
type ApiContentType = "application/json" | "application/x-www-form-urlencoded" | "text/plain" | "none";
interface ApiVariableExtraction {
    variable_name: string;
    json_path: string;
    default_value?: string;
}
interface ApiAssertion {
    type: "status_code" | "json_path" | "header" | "body_contains" | "response_time";
    expected: string | number;
    json_path?: string;
    header_name?: string;
    operator?: "equals" | "contains" | "matches" | "greater_than" | "less_than";
}
type TestType = "playwright" | "qontinui_vision" | "python" | "repository" | "custom_command";
type PlaywrightExecutionMode = "independent" | "chained";
type CheckType = "lint" | "format" | "typecheck" | "analyze" | "security" | "custom_command" | "http_status" | "ai_review" | "ci_cd";
interface CommandStep extends BaseStep {
    type: "command";
    phase: "setup" | "verification" | "completion";
    mode?: "shell" | "check" | "check_group" | "test";
    command?: string;
    working_directory?: string;
    timeout_seconds?: number;
    fail_on_error?: boolean;
    run_on_subsequent_iterations?: boolean;
    shell_command_id?: string;
    check_type?: CheckType;
    tool?: string;
    check_id?: string;
    config_path?: string;
    auto_fix?: boolean;
    fail_on_warning?: boolean;
    repository?: string;
    workflow_name?: string;
    branch?: string;
    wait_for_completion?: boolean;
    check_group_id?: string;
    test_type?: TestType;
    test_id?: string;
    code?: string;
    script_id?: string;
    script_content?: string;
    target_url?: string;
    fused_script_id?: string;
    execution_mode?: PlaywrightExecutionMode;
}
interface PromptStep extends BaseStep {
    type: "prompt";
    phase: "setup" | "verification" | "agentic" | "completion";
    content: string;
    prompt_id?: string;
    provider?: string;
    model?: string;
    is_summary_step?: boolean;
}
interface UiBridgeStep extends BaseStep {
    type: "ui_bridge";
    phase: "setup" | "verification" | "completion";
    action: "navigate" | "execute" | "assert" | "snapshot" | "compare" | "snapshot_assert" | "action_plan";
    url?: string;
    instruction?: string;
    target?: string;
    assert_type?: "exists" | "text_equals" | "contains" | "visible" | "enabled";
    expected?: string;
    timeout_ms?: number;
    comparison_mode?: "structural" | "visual" | "both";
    reference_snapshot_id?: string;
    severity_threshold?: "critical" | "major" | "minor" | "info";
    /** Snapshot target: "control" (runner UI), "sdk" (connected app), or "proxy:PORT" */
    ui_bridge_snapshot_target?: string;
    /** Structured action plan for the "action_plan" action type */
    action_plan?: ActionPlan;
}
interface WorkflowStep extends BaseStep {
    type: "workflow";
    phase: "setup" | "verification" | "completion";
    workflow_id: string;
    workflow_name: string;
}
type StepTypeName = "command" | "ui_bridge" | "prompt" | "workflow" | "native_accessibility";
type UnifiedStep = CommandStep | PromptStep | UiBridgeStep | WorkflowStep;
type SetupStep = CommandStep | PromptStep | UiBridgeStep | WorkflowStep;
type VerificationStep = CommandStep | PromptStep | UiBridgeStep | WorkflowStep;
type AgenticStep = PromptStep;
type CompletionStep = CommandStep | PromptStep | UiBridgeStep | WorkflowStep;
/** A conditional routing rule that selects model/provider based on runtime context. */
interface RoutingRule {
    /** Condition expression, e.g. "verification_failures >= 2" */
    condition: string;
    model?: string;
    provider?: string;
    temperature?: number;
    max_tokens?: number;
}
interface ModelOverrideConfig {
    provider?: string;
    model?: string;
    /** Temperature override for this phase (0.0–1.0). */
    temperature?: number;
    /** Max output tokens override for this phase. */
    max_tokens?: number;
    /** Fallback provider if the primary fails with a retryable error. */
    fallback_provider?: string;
    /** Fallback model if the primary fails with a retryable error. */
    fallback_model?: string;
    /** Conditional routing rules evaluated at runtime. First matching rule wins. */
    routing_rules?: RoutingRule[];
}
type ModelOverrides = {
    setup?: ModelOverrideConfig;
    agentic?: ModelOverrideConfig;
    completion?: ModelOverrideConfig;
    verification?: ModelOverrideConfig;
    investigation?: ModelOverrideConfig;
    summary?: ModelOverrideConfig;
    generation?: ModelOverrideConfig;
};
/**
 * Condition for conditional stage execution.
 *
 * When attached to a WorkflowStage, the stage is skipped if the condition
 * evaluates to "should skip". All fields are optional and combine with AND
 * semantics — all specified conditions must be met for the stage to run.
 */
interface StageCondition {
    /** Run only if previous stage had this outcome: "passed", "failed", or "any" */
    if_previous?: "passed" | "failed" | "any";
    /** Run only after this many total iterations have occurred across all stages */
    min_iteration?: number;
    /** Run only if this many stages have failed verification so far */
    min_failures?: number;
}
interface RetryPolicy {
    /** Number of retry attempts (0 = no retries) */
    count: number;
    /** Delay between retries in milliseconds */
    delay_ms: number;
    /** Whether to use exponential backoff */
    backoff?: boolean;
}
interface StageOutput {
    /** Unique key for this output (e.g. "api_url", "auth_token") */
    key: string;
    /** Human-readable description */
    description?: string;
}
interface StageInput {
    /** The key to bind (matches a StageOutput.key from a prior stage) */
    key: string;
    /** Which stage provides this input (stage id). If omitted, searches all prior stages. */
    from_stage?: string;
    /** Whether this input is required (default: true) */
    required?: boolean;
}
interface WorkflowStage {
    id: string;
    name: string;
    description?: string;
    setup_steps: SetupStep[];
    verification_steps: VerificationStep[];
    agentic_steps: AgenticStep[];
    completion_steps: CompletionStep[];
    /** `null` (or omitted) means unlimited iterations. */
    max_iterations?: number | null;
    timeout_seconds?: number | null;
    provider?: string;
    model?: string;
    model_overrides?: ModelOverrides;
    /** Optional condition for conditional stage execution */
    condition?: StageCondition;
    /** Retry policy for this stage (overrides per-step defaults) */
    retry_policy?: RetryPolicy;
    /** Declared outputs that this stage produces for downstream stages */
    outputs?: StageOutput[];
    /** Inputs required from prior stages */
    inputs?: StageInput[];
}
interface UnifiedWorkflow {
    id: string;
    name: string;
    description: string;
    setup_steps: SetupStep[];
    verification_steps: VerificationStep[];
    agentic_steps: AgenticStep[];
    completion_steps: CompletionStep[];
    /** `null` (or omitted) means unlimited iterations. */
    max_iterations?: number | null;
    timeout_seconds?: number | null;
    provider?: string;
    model?: string;
    model_overrides?: ModelOverrides;
    log_source_selection?: LogSourceSelection;
    context_ids?: string[];
    disabled_context_ids?: string[];
    auto_include_contexts?: boolean;
    skip_ai_summary?: boolean;
    log_watch_enabled?: boolean;
    health_check_enabled?: boolean;
    health_check_urls?: HealthCheckUrl[];
    prompt_template?: string | null;
    stages?: WorkflowStage[];
    stop_on_failure?: boolean;
    reflection_mode?: boolean;
    /** Create a new git branch and worktree for this run. Changes stay isolated until merged. */
    use_worktree?: boolean;
    /** Run in multi-agent mode, spawning parallel sub-agents for independent tasks. */
    multi_agent_mode?: boolean;
    /** Workflow execution architecture override. */
    workflow_architecture?: "traditional" | "agentic_verification" | "multi_agent_pipeline";
    /** Configuration for the multi-agent pipeline architecture. */
    multi_agent_pipeline_config?: Record<string, unknown>;
    /** Restrict working directory resolution to the workspace boundary. Steps cannot resolve paths outside the workspace root. */
    strict_cwd?: boolean;
    /** Tags for per-execution tool whitelisting. When non-empty, only skills matching at least one tag are included in AI prompt context. */
    tool_tags?: string[];
    /** Policy for automatic git rollback when the workflow fails: "none" (default), "last_good", "clean". */
    rollback_policy?: "none" | "last_good" | "clean";
    /** Per-constraint overrides: map of constraint_id to enabled (true) / disabled (false) */
    constraint_overrides?: Record<string, boolean>;
    /** Dependency graph computed during generation */
    dependency_graph?: DependencyGraph;
    /** Cost annotations computed during generation */
    cost_annotations?: CostAnnotations;
    /** Quality report from the revision phase */
    quality_report?: QualityReport;
    /** Acceptance criteria from the specification agent (JSON blob) */
    acceptance_criteria?: Record<string, unknown> | null;
    /** When true, stop execution if accumulated tokens exceed max_context_tokens. */
    enforce_token_budget?: boolean;
    /** Whether this workflow is marked as a favorite */
    is_favorite?: boolean | null;
    /** JSON-serialized flow control configuration (concurrency, throttle, rate limit, debounce) */
    flow_control_json?: string | null;
    /** JSON-serialized per-phase timeout configuration */
    phase_timeouts_json?: string | null;
    category: string;
    tags: string[];
    created_at: string;
    modified_at: string;
}
type VerificationCategory = "existence" | "uniqueness" | "referential_integrity" | "semantic_correctness" | "runtime_behavior";
interface DependencyNode {
    id: string;
    label: string;
    type: string;
    phase: WorkflowPhase;
    is_referenced: boolean;
    cost_category?: string;
}
interface DependencyEdge {
    source: string;
    target: string;
    label?: string;
    edge_type: "explicit_depends_on" | "implicit_reference" | "setup_provides";
}
interface DependencyGraph {
    nodes: DependencyNode[];
    edges: DependencyEdge[];
}
type CostCategory = "network" | "ai_call" | "setup" | "ui_interaction";
interface StepCost {
    step_id: string;
    name: string;
    estimated_ms: number;
    category: CostCategory;
    has_side_effects: boolean;
}
interface CostAnnotations {
    steps: StepCost[];
    total_estimated_ms: number;
}
type QualityFindingSeverity = "critical" | "warning" | "info";
type QualityFindingCategory = "verification_gap" | "missing_criterion" | "unnecessary_step" | "weak_retry" | "required_flag_violation" | "retry_inconsistency" | "data_contract_violation" | "false_positive_risk";
interface QualityFinding {
    finding_id: string;
    step_id?: string;
    severity: QualityFindingSeverity;
    category: QualityFindingCategory;
    description: string;
    suggested_fix?: string;
}
interface CoverageMatrix {
    criteria_to_steps: Record<string, string[]>;
    steps_to_criteria: Record<string, string[]>;
    uncovered_criteria: string[];
    unlinked_steps: string[];
}
interface QualityReport {
    findings: QualityFinding[];
    score: number;
    pass: boolean;
    coverage_matrix?: CoverageMatrix;
}
interface WorkflowExportManifest {
    version: string;
    exported_at: string;
    app_version: string;
    content_type: "unified_workflow";
}
interface WorkflowExport {
    manifest: WorkflowExportManifest;
    workflow: UnifiedWorkflow;
}
interface WorkflowImportResult {
    workflow: UnifiedWorkflow;
    overwritten: boolean;
    original_id: string | null;
}
interface WorkflowFeatures {
    hasSetup: boolean;
    hasVerification: boolean;
    hasAgentic: boolean;
    hasCompletion: boolean;
    hasUiBridge: boolean;
    showIterationSettings: boolean;
    hasAiPrompts: boolean;
}
interface StepTypeInfo {
    type: string;
    label: string;
    description: string;
    icon: string;
    color: string;
    phase: WorkflowPhase;
}
declare const STEP_TYPES: Record<WorkflowPhase, StepTypeInfo[]>;
declare const PHASE_INFO: Record<WorkflowPhase, {
    label: string;
    description: string;
    color: string;
}>;
declare const DEFAULT_SUMMARY_PROMPT = "Write a one-paragraph summary of all the tasks completed in this workflow. Include what was accomplished, whether the stated goal was achieved, any issues encountered and how they were resolved, and remaining work if the goal was not fully achieved. Be concise but comprehensive.";

export { type ActionPlan, type ActionPlanExecuteRequest, type ActionPlanResult, type AgenticStep, type ApiAssertion, type ApiContentType, type ApiVariableExtraction, type BaseStep, type CheckType, type CommandStep, type CompletionStep, type CompositionTemplate, type CostAnnotations, type CostCategory, type CoverageMatrix, DEFAULT_SUMMARY_PROMPT, type DependencyEdge, type DependencyGraph, type DependencyNode, type ElementTarget, type HealthCheckUrl, type HttpMethod, type LogSourceSelection, type ModelOverrideConfig, type ModelOverrides, type MultiStepTemplate, PHASE_INFO, type PlannedAction, type PlannedActionResult, type PlannedActionType, type PlaywrightExecutionMode, type PromptStep, type QualityFinding, type QualityFindingCategory, type QualityFindingSeverity, type QualityReport, type RetryPolicy, type RoutingRule, STEP_TYPES, type SetupStep, type SingleStepTemplate, type SkillAuthor, type SkillCategory, type SkillDefinition, type SkillExport, type SkillExportManifest, type SkillImportResult, type SkillOrigin, type SkillParameter, type SkillParameterOption, type SkillRef, type SkillTemplate, type StageCondition, type StageInput, type StageOutput, type StepCost, type StepTypeInfo, type StepTypeName, type TestType, type UiBridgeStep, type UnifiedStep, type UnifiedWorkflow, type VerificationCategory, type VerificationStep, type WorkflowExport, type WorkflowExportManifest, type WorkflowFeatures, type WorkflowImportResult, type WorkflowPhase, type WorkflowStage, type WorkflowStep };
