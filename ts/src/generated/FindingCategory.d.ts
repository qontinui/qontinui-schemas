/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Category of a detected finding.
 *
 * Determines the kind of issue or observation surfaced during analysis.
 * Mirrors Python `FindingCategory(str, Enum)`.
 */
export type FindingCategory =
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
  | "expected_behavior";
