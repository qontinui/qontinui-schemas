/**
 * Auto-generated TypeScript types from qontinui-schemas
 * DO NOT EDIT - regenerate with: poetry run python scripts/generate_typescript.py
 */

export enum FindingCategory {
  CODE_BUG = "code_bug",
  SECURITY = "security",
  PERFORMANCE = "performance",
  TODO = "todo",
  ENHANCEMENT = "enhancement",
  CONFIG_ISSUE = "config_issue",
  TEST_ISSUE = "test_issue",
  DOCUMENTATION = "documentation",
  RUNTIME_ISSUE = "runtime_issue",
  ALREADY_FIXED = "already_fixed",
  EXPECTED_BEHAVIOR = "expected_behavior",
}

export enum FindingSeverity {
  CRITICAL = "critical",
  HIGH = "high",
  MEDIUM = "medium",
  LOW = "low",
  INFO = "info",
}

export enum FindingStatus {
  DETECTED = "detected",
  IN_PROGRESS = "in_progress",
  NEEDS_INPUT = "needs_input",
  RESOLVED = "resolved",
  WONT_FIX = "wont_fix",
  DEFERRED = "deferred",
}

export enum FindingActionType {
  AUTO_FIX = "auto_fix",
  NEEDS_USER_INPUT = "needs_user_input",
  INFORMATIONAL = "informational",
}

export interface FindingCodeContext {
  /** File path where the finding was detected */
  file?: string | null;
  /** Line number where the finding was detected */
  line?: number | null;
  /** Column number where the finding was detected */
  column?: number | null;
  /** Code snippet related to the finding (max 1000 chars) */
  snippet?: string | null;
}

export interface FindingUserInput {
  /** Question to present to the user */
  question: string;
  /** Type of input expected: 'text' or 'choice' */
  input_type?: string;
  /** Options for choice-type input */
  options?: string[] | null;
}

export interface FindingCreate {
  /** Parent task run ID */
  task_run_id: string;
  /** Session number where the finding was detected */
  session_num: number;
  /** Category of the finding */
  category: FindingCategory;
  /** Severity level of the finding */
  severity: FindingSeverity;
  /** Brief title describing the finding (max 500 chars) */
  title: string;
  /** Detailed description of the finding */
  description: string;
  /** Code context if the finding relates to specific code */
  code_context?: FindingCodeContext | null;
  /** Hash for deduplication across sessions */
  signature_hash?: string | null;
  /** Type of action for this finding */
  action_type: FindingActionType;
  /** User input request if action_type requires user decision */
  user_input?: FindingUserInput | null;
}

export interface FindingBatchCreate {
  /** List of findings to create (1-50 items) */
  findings: FindingCreate[];
}

export interface FindingUpdate {
  /** New status for the finding */
  status?: FindingStatus | null;
  /** Resolution description */
  resolution?: string | null;
  /** Session number where the finding was resolved */
  resolved_in_session?: number | null;
  /** User's response to a finding requiring input */
  user_response?: string | null;
}

export interface FindingDetail {
  /** Finding ID */
  id: string;
  /** Parent task run ID */
  task_run_id: string;
  /** Session number where the finding was detected */
  session_num: number;
  /** Category of the finding */
  category: FindingCategory;
  /** Severity level of the finding */
  severity: FindingSeverity;
  /** Current status of the finding */
  status: FindingStatus;
  /** Brief title describing the finding */
  title: string;
  /** Detailed description of the finding */
  description: string;
  /** Resolution description if resolved */
  resolution?: string | null;
  /** Code context if the finding relates to specific code */
  code_context?: FindingCodeContext | null;
  /** Hash for deduplication across sessions */
  signature_hash?: string | null;
  /** Type of action for this finding */
  action_type: FindingActionType;
  /** User input request if action_type requires user decision */
  user_input?: FindingUserInput | null;
  /** User's response if input was requested */
  user_response?: string | null;
  /** When the finding was detected (UTC) */
  detected_at: string;
  /** When the finding was resolved (UTC) */
  resolved_at?: any | null;
  /** Session number where the finding was resolved */
  resolved_in_session?: number | null;
}

export interface FindingListResponse {
  /** List of findings */
  findings: FindingDetail[];
  /** Total count of findings matching the query */
  total: number;
  /** Maximum items per page */
  limit: number;
  /** Number of items skipped */
  offset: number;
  /** Whether more items exist beyond this page */
  has_more: boolean;
}

export interface FindingSummary {
  /** Task run ID */
  task_run_id: string;
  /** Total number of findings */
  total?: number;
  /** Count of findings by category */
  by_category?: Record<string, any>;
  /** Count of findings by severity */
  by_severity?: Record<string, any>;
  /** Count of findings by status */
  by_status?: Record<string, any>;
  /** Number of findings awaiting user input */
  needs_input_count?: number;
  /** Number of resolved findings */
  resolved_count?: number;
  /** Number of unresolved findings */
  outstanding_count?: number;
}
