/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Category of a finding surfaced during a task run.
 */
type TaskRunFindingCategory =
  | "code_bug"
  | "security"
  | "performance"
  | "todo"
  | "enhancement"
  | "config_issue"
  | "test_issue"
  | "documentation"
  | "runtime_issue"
  | "already_fixed"
  | "expected_behavior"
  | "data_migration"
  | "warning";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Severity of a finding.
 */
type TaskRunFindingSeverity = "critical" | "high" | "medium" | "low" | "info";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Lifecycle status of a finding.
 */
type TaskRunFindingStatus = "detected" | "in_progress" | "needs_input" | "resolved" | "wont_fix" | "deferred";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * How a finding should be acted upon.
 */
type TaskRunFindingActionType = "auto_fix" | "needs_user_input" | "manual" | "informational";

export type { TaskRunFindingActionType as T, TaskRunFindingCategory as a, TaskRunFindingSeverity as b, TaskRunFindingStatus as c };
