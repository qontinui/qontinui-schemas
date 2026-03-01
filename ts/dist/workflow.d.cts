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
}
interface SingleStepTemplate {
    kind: "single_step";
    step: Record<string, unknown>;
}
interface MultiStepTemplate {
    kind: "multi_step";
    steps: Record<string, unknown>[];
}
type SkillTemplate = SingleStepTemplate | MultiStepTemplate;
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
    source: "builtin" | "user";
}
interface SkillOrigin {
    skill_id: string;
    skill_slug: string;
    parameter_values: Record<string, unknown>;
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
    action: "navigate" | "execute" | "assert" | "snapshot" | "compare";
    url?: string;
    instruction?: string;
    target?: string;
    assert_type?: "exists" | "text_equals" | "contains" | "visible" | "enabled";
    expected?: string;
    timeout_ms?: number;
    comparison_mode?: "structural" | "visual" | "both";
    reference_snapshot_id?: string;
    severity_threshold?: "critical" | "major" | "minor" | "info";
}
interface WorkflowStep extends BaseStep {
    type: "workflow";
    phase: "setup" | "verification" | "completion";
    workflow_id: string;
    workflow_name: string;
}
type StepTypeName = "command" | "ui_bridge" | "prompt" | "workflow";
type UnifiedStep = CommandStep | PromptStep | UiBridgeStep | WorkflowStep;
type SetupStep = CommandStep | PromptStep | UiBridgeStep | WorkflowStep;
type VerificationStep = CommandStep | PromptStep | UiBridgeStep | WorkflowStep;
type AgenticStep = PromptStep;
type CompletionStep = CommandStep | PromptStep | UiBridgeStep | WorkflowStep;
interface WorkflowStage {
    id: string;
    name: string;
    description?: string;
    setup_steps: SetupStep[];
    verification_steps: VerificationStep[];
    agentic_steps: AgenticStep[];
    completion_steps: CompletionStep[];
    max_iterations?: number;
    timeout_seconds?: number | null;
    provider?: string;
    model?: string;
}
interface UnifiedWorkflow {
    id: string;
    name: string;
    description: string;
    setup_steps: SetupStep[];
    verification_steps: VerificationStep[];
    agentic_steps: AgenticStep[];
    completion_steps: CompletionStep[];
    max_iterations?: number;
    timeout_seconds?: number | null;
    provider?: string;
    model?: string;
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
    category: string;
    tags: string[];
    created_at: string;
    modified_at: string;
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

export { type AgenticStep, type ApiAssertion, type ApiContentType, type ApiVariableExtraction, type BaseStep, type CheckType, type CommandStep, type CompletionStep, DEFAULT_SUMMARY_PROMPT, type HealthCheckUrl, type HttpMethod, type LogSourceSelection, type MultiStepTemplate, PHASE_INFO, type PlaywrightExecutionMode, type PromptStep, STEP_TYPES, type SetupStep, type SingleStepTemplate, type SkillCategory, type SkillDefinition, type SkillOrigin, type SkillParameter, type SkillParameterOption, type SkillTemplate, type StepTypeInfo, type StepTypeName, type TestType, type UiBridgeStep, type UnifiedStep, type UnifiedWorkflow, type VerificationStep, type WorkflowExport, type WorkflowExportManifest, type WorkflowFeatures, type WorkflowImportResult, type WorkflowPhase, type WorkflowStage, type WorkflowStep };
