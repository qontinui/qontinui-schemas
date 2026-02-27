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

// -----------------------------------------------------------------------------
// Workflow Steps (composition -- run a saved workflow inline)
// -----------------------------------------------------------------------------

export interface WorkflowStep extends BaseStep {
  type: "workflow";
  phase: "setup" | "verification" | "completion";
  workflow_id: string;
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
// Workflow Stages
// =============================================================================

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
// Feature Detection
// =============================================================================

export interface WorkflowFeatures {
  hasSetup: boolean;
  hasVerification: boolean;
  hasAgentic: boolean;
  hasCompletion: boolean;
  hasUiBridge: boolean;
  showIterationSettings: boolean;
  hasAiPrompts: boolean;
}

// =============================================================================
// Step Type Display Info
// =============================================================================

export interface StepTypeInfo {
  type: string;
  label: string;
  description: string;
  icon: string;
  color: string;
  phase: WorkflowPhase;
}

// =============================================================================
// Step Type Constants
// =============================================================================

export const STEP_TYPES: Record<WorkflowPhase, StepTypeInfo[]> = {
  setup: [
    {
      type: "command",
      label: "Command",
      description: "Run shell commands, checks, or tests",
      icon: "Terminal",
      color: "gray",
      phase: "setup",
    },
    {
      type: "ui_bridge",
      label: "UI Bridge",
      description: "Interact with UI via UI Bridge SDK",
      icon: "Monitor",
      color: "emerald",
      phase: "setup",
    },
    {
      type: "prompt",
      label: "AI Task",
      description: "AI-driven task",
      icon: "Bot",
      color: "violet",
      phase: "setup",
    },
    {
      type: "workflow",
      label: "Workflow",
      description: "Run a saved workflow",
      icon: "Workflow",
      color: "blue",
      phase: "setup",
    },
  ],
  verification: [
    {
      type: "command",
      label: "Command",
      description: "Run commands, checks, or tests for verification",
      icon: "Terminal",
      color: "gray",
      phase: "verification",
    },
    {
      type: "ui_bridge",
      label: "UI Bridge",
      description: "Verify UI state via UI Bridge",
      icon: "Monitor",
      color: "emerald",
      phase: "verification",
    },
    {
      type: "prompt",
      label: "AI Verification",
      description: "AI-evaluated criteria",
      icon: "Bot",
      color: "violet",
      phase: "verification",
    },
    {
      type: "workflow",
      label: "Workflow",
      description: "Run a saved workflow for verification",
      icon: "Workflow",
      color: "blue",
      phase: "verification",
    },
  ],
  agentic: [
    {
      type: "prompt",
      label: "Prompt",
      description: "AI task instructions",
      icon: "MessageSquare",
      color: "amber",
      phase: "agentic",
    },
  ],
  completion: [
    {
      type: "command",
      label: "Command",
      description: "Run cleanup commands or final tests",
      icon: "Terminal",
      color: "gray",
      phase: "completion",
    },
    {
      type: "ui_bridge",
      label: "UI Bridge",
      description: "Final UI interactions",
      icon: "Monitor",
      color: "emerald",
      phase: "completion",
    },
    {
      type: "prompt",
      label: "AI Completion",
      description: "Final AI actions",
      icon: "Bot",
      color: "violet",
      phase: "completion",
    },
    {
      type: "workflow",
      label: "Workflow",
      description: "Run a saved workflow as a completion step",
      icon: "Workflow",
      color: "blue",
      phase: "completion",
    },
  ],
};

// =============================================================================
// Phase Display Info
// =============================================================================

export const PHASE_INFO: Record<
  WorkflowPhase,
  { label: string; description: string; color: string }
> = {
  setup: {
    label: "Setup",
    description: "Runs once at the beginning",
    color: "blue",
  },
  verification: {
    label: "Verification",
    description: "Checks success criteria, loops with agentic",
    color: "green",
  },
  agentic: {
    label: "Agentic",
    description: "AI work, iterates until verification passes",
    color: "amber",
  },
  completion: {
    label: "Completion",
    description: "Runs once after the loop exits",
    color: "purple",
  },
};

// =============================================================================
// Summary Step Constants
// =============================================================================

export const DEFAULT_SUMMARY_PROMPT = `Write a one-paragraph summary of all the tasks completed in this workflow. Include what was accomplished, whether the stated goal was achieved, any issues encountered and how they were resolved, and remaining work if the goal was not fully achieved. Be concise but comprehensive.`;
