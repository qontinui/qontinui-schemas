/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Status of a finding.
 *
 * Lifecycle: `DETECTED` → `IN_PROGRESS` → (`RESOLVED` | `WONT_FIX` |
 * `DEFERRED`). `NEEDS_INPUT` is a special state requiring user decision.
 * Mirrors Python `FindingStatus(str, Enum)`.
 */
export type FindingStatus = "detected" | "in_progress" | "needs_input" | "resolved" | "wont_fix" | "deferred";
