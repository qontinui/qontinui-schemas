/**
 * Library Types
 *
 * Normalized types for library items that both apps agree on.
 * These represent the various reusable building blocks available
 * in the workflow builder's library pickers.
 */

// ============================================================================
// Base Library Item
// ============================================================================

export interface LibraryItem {
  id: string;
  name: string;
  description?: string;
}

// ============================================================================
// Check Items
// ============================================================================

export interface CheckItem extends LibraryItem {
  check_type: string;
  tool?: string;
  command?: string;
  working_directory?: string;
  config_path?: string;
  auto_fix?: boolean;
  fail_on_warning?: boolean;
  timeout_seconds?: number;
  enabled?: boolean;
}

export interface CheckGroup extends LibraryItem {
  checks: string[];
  parallel?: boolean;
  stop_on_first_failure?: boolean;
}

// ============================================================================
// Shell Commands
// ============================================================================

export interface ShellCommand extends LibraryItem {
  command: string;
  working_directory?: string;
  timeout_seconds?: number;
  platform?: string;
}

// ============================================================================
// Saved Prompts
// ============================================================================

export interface SavedPrompt extends LibraryItem {
  content: string;
  provider?: string;
  model?: string;
  category?: string;
  language?: string;
}

// ============================================================================
// Saved API Requests
// ============================================================================

export interface SavedApiRequest extends LibraryItem {
  method: string;
  url: string;
  headers?: Record<string, string>;
  body?: string;
  content_type?: string;
  auth_type?: string;
  variables?: Record<string, string>;
}

// ============================================================================
// Contexts
// ============================================================================

export interface Context extends LibraryItem {
  content: string;
  auto_include?: boolean;
  priority?: number;
}

// ============================================================================
// Macros
// ============================================================================

export interface Macro extends LibraryItem {
  actions: MacroAction[];
  timeout_seconds?: number;
}

export interface MacroAction {
  type: string;
  config: Record<string, unknown>;
  delay_ms?: number;
}

// ============================================================================
// Findings Types (runner-side finding system)
// ============================================================================

export type BuiltInCategoryId =
  | "code_bug"
  | "todo"
  | "config_issue"
  | "already_fixed"
  | "expected_behavior"
  | "data_migration"
  | "runtime_issue"
  | "test_issue"
  | "enhancement"
  | "warning"
  | "documentation"
  | "security"
  | "performance";

export type FindingActionType =
  | "auto_fix"
  | "needs_user_input"
  | "manual"
  | "informational";

export type FindingStatus =
  | "detected"
  | "in_progress"
  | "needs_input"
  | "resolved"
  | "wont_fix"
  | "deferred";

export type FindingSeverity = "critical" | "high" | "medium" | "low" | "info";

export type UserInputType = "text" | "choice" | "boolean" | "code";

export interface FindingCategory {
  id: string;
  name: string;
  description: string;
  icon: string;
  color: string;
  isBuiltIn: boolean;
  defaultActionType: FindingActionType;
  sortOrder: number;
}

export interface UserInputOption {
  value: string;
  label: string;
  description?: string;
}

export interface UserInputRequest {
  id: string;
  findingId: string;
  question: string;
  context?: string;
  inputType: UserInputType;
  options?: UserInputOption[];
  required: boolean;
  defaultValue?: string;
}

export interface CodeContext {
  file?: string;
  line?: number;
  endLine?: number;
  snippet?: string;
}

export interface Finding {
  id: string;
  categoryId: string;
  severity: FindingSeverity;
  status: FindingStatus;
  title: string;
  description: string;
  sourceSessionId?: string;
  sourcePromptName?: string;
  detectedAt: number;
  actionType: FindingActionType;
  actionable: boolean;
  pendingQuestion?: UserInputRequest;
  userResponse?: string;
  resolvedAt?: number;
  resolution?: string;
  codeContext?: CodeContext;
  metadata?: Record<string, unknown>;
}

export interface CategorySummary {
  count: number;
  actionable: number;
  resolved: number;
}

export interface ReportSummary {
  totalFindings: number;
  byCategory: Record<string, CategorySummary>;
  bySeverity: Record<FindingSeverity, number>;
  byStatus: Record<FindingStatus, number>;
  actionable: number;
  needsUserInput: number;
  autoFixed: number;
  informational: number;
}

export type ReportStatus =
  | "running"
  | "completed"
  | "paused_for_input"
  | "failed"
  | "cancelled";

export interface PhaseInfo {
  phase: number;
  startedAt: number;
  completedAt?: number;
  findingsDetected: number;
  findingsResolved: number;
}

export interface ExecutionReport {
  id: string;
  sessionId: string;
  promptName: string;
  promptId?: string;
  startedAt: number;
  completedAt?: number;
  duration?: number;
  status: ReportStatus;
  findings: Finding[];
  summary: ReportSummary;
  pendingInputs: UserInputRequest[];
  phases: PhaseInfo[];
}

export interface ParsedFinding {
  categoryId: string;
  severity: FindingSeverity;
  title: string;
  description: string;
  needsInput: boolean;
  question?: string;
  options?: string[];
  file?: string;
  line?: number;
  resolution?: string;
}

export interface CategoryStore {
  customCategories: FindingCategory[];
  categoryOrder: string[];
  hiddenCategories: string[];
}

// ============================================================================
// Color Types (design-system colors)
// ============================================================================

/** @deprecated Use FindingSeverity instead */
export type SeverityLevel = FindingSeverity;

export interface SeverityColorClasses {
  bg: string;
  text: string;
  border: string;
  dot: string;
}

export type StatusColorType =
  | "idle"
  | "running"
  | "success"
  | "error"
  | "warning"
  | "pending"
  | "paused"
  | "cancelled";

export interface StatusColorClasses {
  bg: string;
  text: string;
  border: string;
  icon: string;
  pulse?: string;
}

export type ActionColorType =
  | "auto_fix"
  | "needs_user_input"
  | "manual"
  | "informational"
  | "skip"
  | "defer";

export interface ActionColorClasses {
  bg: string;
  text: string;
  border: string;
  badge: string;
}

export type AccentColor =
  | "red"
  | "orange"
  | "amber"
  | "yellow"
  | "green"
  | "emerald"
  | "blue"
  | "cyan"
  | "purple"
  | "slate";

export interface AccentColorClasses {
  bg: string;
  bgSolid: string;
  text: string;
  border: string;
}
