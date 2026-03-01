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

import type { WorkflowPhase } from "./index";

// =============================================================================
// Skill Categories
// =============================================================================

export type SkillCategory =
  | "code-quality"
  | "testing"
  | "monitoring"
  | "ai-task"
  | "deployment"
  | "composition"
  | "custom";

// =============================================================================
// Skill Parameters
// =============================================================================

export interface SkillParameterOption {
  label: string;
  value: string;
}

export interface SkillParameter {
  name: string;
  type: "string" | "number" | "boolean" | "select";
  label: string;
  description: string;
  required: boolean;
  default?: unknown;
  options?: SkillParameterOption[];
  placeholder?: string;
}

// =============================================================================
// Skill Templates
// =============================================================================

export interface SingleStepTemplate {
  kind: "single_step";
  step: Record<string, unknown>;
}

export interface MultiStepTemplate {
  kind: "multi_step";
  steps: Record<string, unknown>[];
}

export type SkillTemplate = SingleStepTemplate | MultiStepTemplate;

// =============================================================================
// Skill Definition
// =============================================================================

export interface SkillDefinition {
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

// =============================================================================
// Skill Origin (attached to steps created from skills)
// =============================================================================

export interface SkillOrigin {
  skill_id: string;
  skill_slug: string;
  parameter_values: Record<string, unknown>;
}
