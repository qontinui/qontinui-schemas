/**
 * Known Issues Registry Types
 *
 * Persistent known issue tracking that survives across workflow runs.
 * Issues are scoped to specs, URLs, components, or global.
 */

export type IssueCategory =
  | "duplication"
  | "rendering"
  | "data_integrity"
  | "timing"
  | "layout"
  | "state"
  | "performance"
  | "encoding"
  | "navigation"
  | "authentication"
  | "other";

export type ScopeType = "global" | "spec" | "url" | "component" | "feature";

export type DetectionMethod =
  | "algorithmic"
  | "ai_judgment"
  | "visual"
  | "command"
  | "ui_bridge";

export type KnownIssueSeverity = "critical" | "high" | "medium" | "low";

export type IssueStatus = "active" | "resolved" | "monitoring" | "wont_fix";

export type IssueProvenance =
  | "manual"
  | "auto_detected"
  | "reflection"
  | "imported";

export interface KnownIssue {
  id: string;
  title: string;
  description: string;
  category: IssueCategory;
  scope_type: ScopeType;
  scope_value: string | null;
  scope_tags: string[];
  detection_method: DetectionMethod;
  detection_config: Record<string, unknown>;
  pattern_template_id: string | null;
  reproduction_context: string | null;
  trigger_conditions: string[];
  severity: KnownIssueSeverity;
  status: IssueStatus;
  confidence: number;
  provenance: IssueProvenance;
  source_finding_ids: string[];
  source_task_run_id: string | null;
  verification_hint: string | null;
  verification_step_template: Record<string, unknown> | null;
  times_detected: number;
  times_checked: number;
  last_detected_at: string | null;
  last_checked_at: string | null;
  resolved_at: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateKnownIssueRequest {
  title: string;
  description: string;
  category: IssueCategory;
  scope_type: ScopeType;
  scope_value?: string | null;
  scope_tags?: string[];
  detection_method: DetectionMethod;
  detection_config?: Record<string, unknown>;
  pattern_template_id?: string | null;
  reproduction_context?: string | null;
  trigger_conditions?: string[];
  severity: KnownIssueSeverity;
  provenance?: IssueProvenance;
  source_finding_ids?: string[];
  source_task_run_id?: string | null;
  verification_hint?: string | null;
  verification_step_template?: Record<string, unknown> | null;
}

export interface UpdateKnownIssueRequest {
  title?: string;
  description?: string;
  category?: IssueCategory;
  scope_type?: ScopeType;
  scope_value?: string | null;
  scope_tags?: string[];
  detection_method?: DetectionMethod;
  detection_config?: Record<string, unknown>;
  pattern_template_id?: string | null;
  reproduction_context?: string | null;
  trigger_conditions?: string[];
  severity?: KnownIssueSeverity;
  status?: IssueStatus;
  confidence?: number;
  verification_hint?: string | null;
  verification_step_template?: Record<string, unknown> | null;
}

export interface ListKnownIssuesQuery {
  scope_type?: string;
  scope_value?: string;
  category?: string;
  severity?: string;
  status?: string;
  spec_id?: string;
}

export interface CreatePatternTemplateRequest {
  name: string;
  description: string;
  category: string;
  detection_type: string;
  ai_prompt_template?: string | null;
  parameters?: string | null;
}

export interface TemplateParameter {
  name: string;
  type: string;
  description: string;
  default?: unknown;
}

export interface IssuePatternTemplate {
  id: string;
  name: string;
  description: string;
  category: string;
  detection_type: string;
  step_template: Record<string, unknown> | null;
  ai_prompt_template: string | null;
  parameters: TemplateParameter[];
  built_in: boolean;
  status: string;
  created_at: string;
  updated_at: string;
}

/** All issue categories with display labels */
export const ISSUE_CATEGORIES: { value: IssueCategory; label: string }[] = [
  { value: "duplication", label: "Duplication" },
  { value: "rendering", label: "Rendering" },
  { value: "data_integrity", label: "Data Integrity" },
  { value: "timing", label: "Timing" },
  { value: "layout", label: "Layout" },
  { value: "state", label: "State" },
  { value: "performance", label: "Performance" },
  { value: "encoding", label: "Encoding" },
  { value: "navigation", label: "Navigation" },
  { value: "authentication", label: "Authentication" },
  { value: "other", label: "Other" },
];

/** All severity levels with display labels */
export const ISSUE_SEVERITIES: { value: KnownIssueSeverity; label: string }[] = [
  { value: "critical", label: "Critical" },
  { value: "high", label: "High" },
  { value: "medium", label: "Medium" },
  { value: "low", label: "Low" },
];

/** All detection methods with display labels */
export const DETECTION_METHODS: { value: DetectionMethod; label: string }[] = [
  { value: "algorithmic", label: "Algorithmic (automatic)" },
  { value: "ai_judgment", label: "AI Judgment" },
  { value: "visual", label: "Visual (screenshot)" },
  { value: "command", label: "Shell Command" },
  { value: "ui_bridge", label: "UI Bridge" },
];
