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
// Generated frame types — source of truth is qontinui-schemas/rust/src/workflow.rs
// These are produced by the schemars -> JSON Schema -> json-schema-to-typescript
// pipeline. Do NOT redeclare them here; re-export from ../generated/*.
//
// Step arrays on UnifiedWorkflow / WorkflowStage are typed as `unknown[]` by
// the generator. Typed step arrays are part of the Wave 4 step migration and
// remain hand-authored below (BaseStep, CommandStep, PromptStep, UiBridgeStep,
// WorkflowStep, and the per-phase aliases).
// =============================================================================

export type { LogSourceSelection } from "../generated/LogSourceSelection";
export type { HealthCheckUrl } from "../generated/HealthCheckUrl";
export type { RoutingRule } from "../generated/RoutingRule";
export type { ModelOverrideConfig } from "../generated/ModelOverrideConfig";
export type { StageCondition } from "../generated/StageCondition";
export type { RetryPolicy } from "../generated/RetryPolicy";
export type { StageOutput } from "../generated/StageOutput";
export type { StageInput } from "../generated/StageInput";
export type { WorkflowStage } from "../generated/WorkflowStage";
export type { UnifiedWorkflow } from "../generated/UnifiedWorkflow";
export type { WorkflowArchitecture } from "../generated/WorkflowArchitecture";

// Step DTOs (generated from qontinui-types::workflow_step)
export type { CommandStep } from "../generated/CommandStep";
export type { PromptStep } from "../generated/PromptStep";
export type { UiBridgeStep } from "../generated/UiBridgeStep";
export type { WorkflowStep } from "../generated/WorkflowStep";
export type { UnifiedStep } from "../generated/UnifiedStep";
export type { HttpMethod } from "../generated/HttpMethod";
export type { ApiContentType } from "../generated/ApiContentType";
export type { ApiAssertion } from "../generated/ApiAssertion";
export type { ApiVariableExtraction } from "../generated/ApiVariableExtraction";
export type { TestType } from "../generated/TestType";
export type { PlaywrightExecutionMode } from "../generated/PlaywrightExecutionMode";
export type { CheckType } from "../generated/CheckType";
export type { CommandMode } from "../generated/CommandMode";
export type { UiBridgeAction } from "../generated/UiBridgeAction";
export type { VerificationCategoryKind } from "../generated/VerificationCategoryKind";

// ModelOverrides is a hand-authored convenience alias: the generated
// UnifiedWorkflow / WorkflowStage type `model_overrides` as an open
// `{ [k: string]: ModelOverrideConfig }`, but downstream code prefers a
// named type with known phase keys. Keep as a local alias over the generated
// per-phase config.
import type { ModelOverrideConfig as _ModelOverrideConfig } from "../generated/ModelOverrideConfig";

export type ModelOverrides = {
  setup?: _ModelOverrideConfig;
  agentic?: _ModelOverrideConfig;
  completion?: _ModelOverrideConfig;
  verification?: _ModelOverrideConfig;
  investigation?: _ModelOverrideConfig;
  summary?: _ModelOverrideConfig;
  generation?: _ModelOverrideConfig;
};

// =============================================================================
// Phases
// =============================================================================

export type WorkflowPhase = "setup" | "verification" | "agentic" | "completion";

// =============================================================================
// Skill Types (re-exported from ./skill)
// =============================================================================

export type {
  SkillCategory,
  SkillAuthor,
  SkillParameterOption,
  SkillParameter,
  SingleStepTemplate,
  MultiStepTemplate,
  CompositionTemplate,
  SkillRef,
  SkillTemplate,
  SkillDefinition,
  SkillOrigin,
  SkillExportManifest,
  SkillExport,
  SkillImportResult,
} from "./skill";

export type {
  PlannedActionType,
  ElementTarget,
  PlannedAction,
  ActionPlan,
  PlannedActionResult,
  ActionPlanResult,
  ActionPlanExecuteRequest,
} from "./action-plan";

// =============================================================================
// Step Types — generated from qontinui-types::workflow_step
//
// The four canonical step variants (CommandStep, PromptStep, UiBridgeStep,
// WorkflowStep) and their helper unions (HttpMethod, ApiContentType, …) are
// re-exported at the top of this file from ../generated/*. The remaining
// hand-authored types in this section are TS-only sugar: a `BaseStep` alias
// for consumers that still want a name for the shared-fields subset, a
// `StepTypeName` literal that includes non-canonical runner-side variants
// (e.g. native_accessibility) for UI listings, and the phase-alias types.
//
// Phase narrowing is TS-only sugar — not a wire concern. The generator emits
// a narrow `phase` field on each variant; these aliases just group the four
// variants by the phase that can carry them.
// =============================================================================

import type { CommandStep as _CommandStep } from "../generated/CommandStep";
import type { PromptStep as _PromptStep } from "../generated/PromptStep";
import type { UiBridgeStep as _UiBridgeStep } from "../generated/UiBridgeStep";
import type { WorkflowStep as _WorkflowStep } from "../generated/WorkflowStep";

/**
 * Shared-fields subset carried by every canonical step variant.
 *
 * Kept as a hand-authored alias so downstream code can still reference
 * `BaseStep` by name. Fields mirror `qontinui-types::workflow_step::BaseStepFields`.
 */
export interface BaseStep {
  id: string;
  name: string;
  fail_on_console_errors?: boolean;
  inputs?: Record<string, string>;
  extract?: Record<string, string>;
  depends_on?: string[];
  required?: boolean;
  retry?: { count: number; delay_ms: number };
  skill_origin?: import("./skill").SkillOrigin;
  /** Acceptance criterion IDs this step verifies (supports multiple) */
  criterion_ids?: string[];
  /** Verification depth category for this step */
  verification_category?: VerificationCategory;
}

// =============================================================================
// Step Type Names
// =============================================================================

// Includes runner-side non-canonical variants (e.g. `native_accessibility`)
// that appear in the UI listing but are not part of the `UnifiedStep` DTO.
export type StepTypeName = "command" | "ui_bridge" | "prompt" | "workflow" | "native_accessibility";

// =============================================================================
// Phase-alias types (TS-only sugar over the generated UnifiedStep)
// =============================================================================

export type SetupStep = _CommandStep | _PromptStep | _UiBridgeStep | _WorkflowStep;
export type VerificationStep = _CommandStep | _PromptStep | _UiBridgeStep | _WorkflowStep;
export type AgenticStep = _PromptStep;
export type CompletionStep = _CommandStep | _PromptStep | _UiBridgeStep | _WorkflowStep;

// =============================================================================
// Verification Categories
// =============================================================================

export type VerificationCategory =
  | "existence"
  | "uniqueness"
  | "referential_integrity"
  | "semantic_correctness"
  | "runtime_behavior";

// =============================================================================
// Dependency Graph
// =============================================================================

export interface DependencyNode {
  id: string;
  label: string;
  type: string;
  phase: WorkflowPhase;
  is_referenced: boolean;
  cost_category?: string;
}

export interface DependencyEdge {
  source: string;
  target: string;
  label?: string;
  edge_type: "explicit_depends_on" | "implicit_reference" | "setup_provides";
}

export interface DependencyGraph {
  nodes: DependencyNode[];
  edges: DependencyEdge[];
}

// =============================================================================
// Cost Annotations
// =============================================================================

export type CostCategory =
  | "network"
  | "ai_call"
  | "setup"
  | "ui_interaction";

export interface StepCost {
  step_id: string;
  name: string;
  estimated_ms: number;
  category: CostCategory;
  has_side_effects: boolean;
}

export interface CostAnnotations {
  steps: StepCost[];
  total_estimated_ms: number;
}

// =============================================================================
// Quality Report
// =============================================================================

export type QualityFindingSeverity = "critical" | "warning" | "info";

export type QualityFindingCategory =
  | "verification_gap"
  | "missing_criterion"
  | "unnecessary_step"
  | "weak_retry"
  | "required_flag_violation"
  | "retry_inconsistency"
  | "data_contract_violation"
  | "false_positive_risk";

export interface QualityFinding {
  finding_id: string;
  step_id?: string;
  severity: QualityFindingSeverity;
  category: QualityFindingCategory;
  description: string;
  suggested_fix?: string;
}

export interface CoverageMatrix {
  criteria_to_steps: Record<string, string[]>;
  steps_to_criteria: Record<string, string[]>;
  uncovered_criteria: string[];
  unlinked_steps: string[];
}

export interface QualityReport {
  findings: QualityFinding[];
  score: number;
  pass: boolean;
  coverage_matrix?: CoverageMatrix;
}

// =============================================================================
// Export/Import Types
// =============================================================================

import type { UnifiedWorkflow as _UnifiedWorkflow } from "../generated/UnifiedWorkflow";

export interface WorkflowExportManifest {
  version: string;
  exported_at: string;
  app_version: string;
  content_type: "unified_workflow";
}

export interface WorkflowExport {
  manifest: WorkflowExportManifest;
  workflow: _UnifiedWorkflow;
}

export interface WorkflowImportResult {
  workflow: _UnifiedWorkflow;
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
    {
      type: "native_accessibility",
      label: "Native Accessibility",
      description: "Run a native accessibility audit using OS-level APIs",
      icon: "Accessibility",
      color: "teal",
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
