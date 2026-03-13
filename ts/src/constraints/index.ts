/**
 * Constraint Engine Types
 *
 * TypeScript interfaces mirroring the Rust constraint engine types
 * from `qontinui-runner/src-tauri/src/constraint_engine/`.
 */

// ============================================================================
// Severity
// ============================================================================

/**
 * Severity of a constraint violation.
 *
 * - `block`: Reject the fix, inject violation context, re-run agentic phase
 *   without consuming an iteration. After max_retries, consume the iteration.
 * - `warn`: Apply the fix, but inject violation context for the next iteration.
 * - `log`: Record only, don't affect execution.
 */
export type ConstraintSeverity = "block" | "warn" | "log";

// ============================================================================
// Check Types (discriminated union matching Rust's #[serde(tag = "type")])
// ============================================================================

/** Type discriminator for constraint checks. */
export type ConstraintCheckType =
  | "grep_forbidden"
  | "grep_required"
  | "file_scope"
  | "command";

/**
 * Grep modified files for a regex pattern.
 * Violation if the pattern IS found (use for secrets, debug statements, etc.)
 */
export interface GrepForbiddenCheck {
  type: "grep_forbidden";
  /** Regex pattern to search for. */
  pattern: string;
  /** Optional glob to limit which modified files are checked. */
  file_glob?: string;
}

/**
 * Grep modified files for a regex pattern.
 * Violation if the pattern is NOT found (use for required headers, licenses, etc.)
 */
export interface GrepRequiredCheck {
  type: "grep_required";
  /** Regex pattern that must be present. */
  pattern: string;
  /** Optional glob to limit which modified files are checked. */
  file_glob?: string;
}

/**
 * Check that all modified files are within allowed paths.
 * Violation if any modified file is outside the allowed directories.
 */
export interface FileScopeCheck {
  type: "file_scope";
  /** Allowed directory prefixes (relative to project root). */
  allowed_paths: string[];
}

/**
 * Run a shell command. Violation if exit code is non-zero.
 * Useful for quick compilation checks, linting, etc.
 */
export interface CommandCheck {
  type: "command";
  /** The command to run. */
  cmd: string;
  /** Working directory (relative to project root). Default: project root. */
  cwd?: string;
  /** Timeout in seconds. Default: 30. */
  timeout_secs: number;
}

/** Discriminated union of all constraint check types. */
export type ConstraintCheck =
  | GrepForbiddenCheck
  | GrepRequiredCheck
  | FileScopeCheck
  | CommandCheck;

// ============================================================================
// Core Constraint Definition
// ============================================================================

/** A constraint definition. */
export interface Constraint {
  /** Unique identifier (e.g., "builtin:no-secrets", "project:no-todos"). */
  id: string;
  /** Human-readable name. */
  name: string;
  /** Why this constraint exists (shown to the AI on violation). */
  description: string;
  /** What to check. */
  check: ConstraintCheck;
  /** How severe a violation is. */
  severity: ConstraintSeverity;
  /** Whether this constraint is enabled. Default: true. */
  enabled: boolean;
}

// ============================================================================
// Evaluation Results
// ============================================================================

/** A specific violation found during constraint evaluation. */
export interface ConstraintViolation {
  /** File where the violation was found (if applicable). */
  file?: string;
  /** Line number (if applicable). */
  line?: number;
  /** What was found / what went wrong. */
  detail: string;
}

/** Result of evaluating a single constraint. */
export interface ConstraintResult {
  /** The constraint that was evaluated. */
  constraint_id: string;
  constraint_name: string;
  /** Whether the constraint passed. */
  passed: boolean;
  /** Severity of the constraint (for quick filtering). */
  severity: ConstraintSeverity;
  /** Details about the violation (empty if passed). */
  violations: ConstraintViolation[];
}

// ============================================================================
// Resource Limits
// ============================================================================

/**
 * Resource limits for workflow execution.
 * When a limit is approached (within the warning threshold), the tracker
 * emits context injection actions. When exceeded, it emits stronger actions.
 */
export interface ResourceLimits {
  /** Maximum wall-clock time for the entire workflow (seconds). */
  max_wall_time_secs?: number;
  /** Maximum number of unique files modified across all iterations. */
  max_files_modified?: number;
  /** Maximum agentic phase durations summed (milliseconds). */
  max_agentic_time_ms?: number;
  /**
   * Warning threshold as a fraction (0.0-1.0). When resource usage
   * exceeds this fraction of the limit, a warning is injected.
   * Default: 0.75 (warn at 75% of limit).
   */
  warning_threshold?: number;
}

// ============================================================================
// AI Constraint Proposals
// ============================================================================

/** A new constraint proposed by the AI during an agentic phase. */
export interface NewConstraintProposal {
  type: "new_constraint";
  constraint: Constraint;
}

/** A proposal to enable or disable a builtin constraint. */
export interface BuiltinOverrideProposal {
  type: "builtin_override";
  /** Builtin suffix (e.g., "no-secrets", "no-debug-statements"). */
  builtin_suffix: string;
  enabled: boolean;
  reason: string;
}

/** A constraint proposal from the AI. */
export type ConstraintProposal =
  | NewConstraintProposal
  | BuiltinOverrideProposal;

// ============================================================================
// API Request / Response Types
// ============================================================================

/** Request body for POST /constraints/validate. */
export interface ValidateConfigRequest {
  /** Raw TOML content to validate. */
  toml: string;
}

/** Response for POST /constraints/validate. */
export interface ValidateConfigResponse {
  /** Whether the config is fully valid (parseable with no errors or warnings). */
  valid: boolean;
  /** Parse errors or non-fatal warnings (e.g., constraints skipped due to bad regex). */
  errors: string[];
  /** Successfully parsed constraints (may be partial if some were skipped). */
  constraints: Constraint[];
}

/** Response for GET /constraints/config. */
export interface ReadConfigResponse {
  /** Raw TOML content of the constraints.toml file (empty string if not found). */
  toml: string;
  /** Resolved file path, if a config file was found. */
  path?: string;
}

/** Request body for POST /constraints/config. */
export interface WriteConfigRequest {
  /** Project path for the constraints.toml. Defaults to workspace root. */
  project_path?: string;
  /** Raw TOML content to validate and write. */
  toml: string;
}

/** Response for POST /constraints/config. */
export interface WriteConfigResponse {
  /** Whether the config is fully valid (parseable with no errors or warnings). */
  valid: boolean;
  /** Parse errors or non-fatal warnings. */
  errors: string[];
  /** The file path that was written to. */
  path: string;
}
