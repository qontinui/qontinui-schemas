/**
 * Unified Workflow Types
 *
 * Canonical type definitions for the unified Workflow Builder system.
 * Shared across qontinui-runner and qontinui-web.
 *
 * All automation is organized into four phases: Setup, Verification, Agentic, Completion.
 *
 * Execution Order:
 *   Setup (once) -> [Verification <-> Agentic]* -> Completion (once)
 *
 * The Verification/Agentic loop continues until all required checks pass or max iterations.
 * Setup and Completion run exactly once - at the beginning and end respectively.
 *
 * Step Types (4 core types):
 *   command   - Shell commands, checks, check groups, tests
 *   ui_bridge - UI Bridge SDK interactions (navigate, execute, assert, snapshot)
 *   prompt    - AI task instructions
 *   workflow  - Run a saved workflow inline (composition)
 */

// =============================================================================
// Phases
// =============================================================================

export type WorkflowPhase = "setup" | "verification" | "agentic" | "completion";

// =============================================================================
// Log Source Selection
// =============================================================================

export type LogSourceSelection =
  | "default"
  | "ai"
  | "all"
  | { profile_id: string };

// =============================================================================
// Health Check Configuration
// =============================================================================

export interface HealthCheckUrl {
  name: string;
  url: string;
  expected_status?: number;
  timeout_seconds?: number;
  is_critical?: boolean;
}

// =============================================================================
// Skill Origin
// =============================================================================

export interface SkillOrigin {
  skill_id: string;
  skill_slug: string;
  parameter_values: Record<string, unknown>;
}

// =============================================================================
// Step Types
// =============================================================================

export interface BaseStep {
  id: string;
  name: string;
  fail_on_console_errors?: boolean;
  inputs?: Record<string, string>;
  extract?: Record<string, string>;
  depends_on?: string[];
  required?: boolean;
  retry?: { count: number; delay_ms: number };
  skill_origin?: SkillOrigin;
}

// -----------------------------------------------------------------------------
// API Request Builder Types
// -----------------------------------------------------------------------------

export type HttpMethod = "GET" | "POST" | "PUT" | "PATCH" | "DELETE";

export type ApiContentType =
  | "application/json"
  | "application/x-www-form-urlencoded"
  | "text/plain"
  | "none";

export interface ApiVariableExtraction {
  variable_name: string;
  json_path: string;
  default_value?: string;
}

export interface ApiAssertion {
  type:
    | "status_code"
    | "json_path"
    | "header"
    | "body_contains"
    | "response_time";
  expected: string | number;
  json_path?: string;
  header_name?: string;
  operator?: "equals" | "contains" | "matches" | "greater_than" | "less_than";
}

// -----------------------------------------------------------------------------
// Test Types
// -----------------------------------------------------------------------------

export type TestType =
  | "playwright"
  | "qontinui_vision"
  | "python"
  | "repository"
  | "custom_command";

export type PlaywrightExecutionMode = "independent" | "chained";

// -----------------------------------------------------------------------------
// Check Types
// -----------------------------------------------------------------------------

export type CheckType =
  | "lint"
  | "format"
  | "typecheck"
  | "analyze"
  | "security"
  | "custom_command"
  | "http_status"
  | "ai_review"
  | "ci_cd";

// -----------------------------------------------------------------------------
// Command Steps
// -----------------------------------------------------------------------------

export interface CommandStep extends BaseStep {
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

// -----------------------------------------------------------------------------
// Prompt Steps
// -----------------------------------------------------------------------------

export interface PromptStep extends BaseStep {
  type: "prompt";
  phase: "setup" | "verification" | "agentic" | "completion";
  content: string;
  prompt_id?: string;
  provider?: string;
  model?: string;
  is_summary_step?: boolean;
}

// -----------------------------------------------------------------------------
// UI Bridge Steps
// -----------------------------------------------------------------------------

export interface UiBridgeStep extends BaseStep {
  type: "ui_bridge";
  phase: "setup" | "verification" | "completion";
  action: "navigate" | "execute" | "assert" | "snapshot" | "compare" | "snapshot_assert";
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
}

// -----------------------------------------------------------------------------
// Workflow Steps (composition — run a saved workflow inline)
// -----------------------------------------------------------------------------

export interface WorkflowStep extends BaseStep {
  type: "workflow";
  phase: "setup" | "verification" | "completion";
  /** ID of the referenced workflow to execute */
  workflow_id: string;
  /** Cached display name of the referenced workflow */
  workflow_name: string;
}

// =============================================================================
// Step Type Names
// =============================================================================

export type StepTypeName = "command" | "ui_bridge" | "prompt" | "workflow";

// =============================================================================
// Unified Step Types
// =============================================================================

export type UnifiedStep =
  | CommandStep
  | PromptStep
  | UiBridgeStep
  | WorkflowStep;

export type SetupStep = CommandStep | PromptStep | UiBridgeStep | WorkflowStep;
export type VerificationStep =
  | CommandStep
  | PromptStep
  | UiBridgeStep
  | WorkflowStep;
export type AgenticStep = PromptStep;
export type CompletionStep =
  | CommandStep
  | PromptStep
  | UiBridgeStep
  | WorkflowStep;

// =============================================================================
// Per-Phase Model Overrides
// =============================================================================

/** A conditional routing rule that selects model/provider based on runtime context. */
export interface RoutingRule {
  condition: string;
  model?: string;
  provider?: string;
  temperature?: number;
  max_tokens?: number;
}

export interface ModelOverrideConfig {
  provider?: string;
  model?: string;
  temperature?: number;
  max_tokens?: number;
  fallback_provider?: string;
  fallback_model?: string;
  routing_rules?: RoutingRule[];
}

export type ModelOverrides = {
  setup?: ModelOverrideConfig;
  agentic?: ModelOverrideConfig;
  completion?: ModelOverrideConfig;
  verification?: ModelOverrideConfig;
  investigation?: ModelOverrideConfig;
  summary?: ModelOverrideConfig;
  generation?: ModelOverrideConfig;
};

// =============================================================================
// Workflow Stages
// =============================================================================

/**
 * Condition for conditional stage execution.
 *
 * When attached to a WorkflowStage, the stage is skipped if the condition
 * evaluates to "should skip". All fields are optional and combine with AND
 * semantics — all specified conditions must be met for the stage to run.
 */
export interface StageCondition {
  /** Run only if previous stage had this outcome: "passed", "failed", or "any" */
  if_previous?: "passed" | "failed" | "any";
  /** Run only after this many total iterations have occurred across all stages */
  min_iteration?: number;
  /** Run only if this many stages have failed verification so far */
  min_failures?: number;
}

/**
 * A workflow stage — a self-contained unit of execution with its own
 * setup/verification/agentic/completion steps and verification-agentic loop.
 *
 * Multi-stage workflows execute stages sequentially. Each stage gets its own
 * verification-agentic loop, and later stages see full output from all prior stages.
 */
export interface WorkflowStage {
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
  model_overrides?: ModelOverrides;
  /** Optional condition for conditional stage execution */
  condition?: StageCondition;
}

// =============================================================================
// Workflow
// =============================================================================

export interface UnifiedWorkflow {
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
  is_favorite?: boolean;
  category: string;
  tags: string[];
  created_at: string;
  modified_at: string;
}

// =============================================================================
// Export/Import Types
// =============================================================================

export interface WorkflowExportManifest {
  version: string;
  exported_at: string;
  app_version: string;
  content_type: "unified_workflow";
}

export interface WorkflowExport {
  manifest: WorkflowExportManifest;
  workflow: UnifiedWorkflow;
}

export interface WorkflowImportResult {
  workflow: UnifiedWorkflow;
  overwritten: boolean;
  original_id: string | null;
}

// =============================================================================
// Phase Normalization Helpers
// =============================================================================

/** Convert any workflow to its phases (stages) representation */
export function normalizeToPhases(workflow: UnifiedWorkflow): WorkflowStage[] {
  if (workflow.stages && workflow.stages.length > 0) {
    return workflow.stages;
  }
  // Wrap top-level steps as a single phase
  return [
    {
      id: workflow.id + "-phase-1",
      name: workflow.name,
      description: workflow.description,
      setup_steps: workflow.setup_steps,
      verification_steps: workflow.verification_steps,
      agentic_steps: workflow.agentic_steps,
      completion_steps: workflow.completion_steps ?? [],
      max_iterations: workflow.max_iterations,
      timeout_seconds: workflow.timeout_seconds,
      provider: workflow.provider,
      model: workflow.model,
      model_overrides: workflow.model_overrides,
    },
  ];
}

/** Get the number of phases in a workflow */
export function getPhaseCount(workflow: UnifiedWorkflow): number {
  return normalizeToPhases(workflow).length;
}
