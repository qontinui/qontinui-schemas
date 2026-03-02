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
// Skill Author
// =============================================================================

export interface SkillAuthor {
  name: string;
  email?: string;
  url?: string;
}

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
  min?: number;
  max?: number;
  pattern?: string;
  depends_on?: { param: string; value: unknown };
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

export interface CompositionTemplate {
  kind: "composition";
  skill_refs: SkillRef[];
}

export interface SkillRef {
  skill_id: string;
  parameter_overrides?: Record<string, unknown>;
}

export type SkillTemplate = SingleStepTemplate | MultiStepTemplate | CompositionTemplate;

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
  source: "builtin" | "user" | "community";
  version?: string;
  author?: SkillAuthor;
  checksum?: string;
  depends_on?: string[];
  usage_count?: number;
  approval_status?: "pending" | "approved" | "rejected";
  forked_from?: string;
}

// =============================================================================
// Skill Origin (attached to steps created from skills)
// =============================================================================

export interface SkillOrigin {
  skill_id: string;
  skill_slug: string;
  parameter_values: Record<string, unknown>;
}

// =============================================================================
// Export / Import
// =============================================================================

export interface SkillExportManifest {
  version: string;
  exported_at: string;
  app_version: string;
  content_type: "skills";
  skill_count: number;
  checksum?: string;  // SHA-256 of all skill content
}

export interface SkillExport {
  manifest: SkillExportManifest;
  skills: SkillDefinition[];
}

export interface SkillImportResult {
  imported: number;
  skipped: number;
  overwritten: number;
  errors: string[];
}
