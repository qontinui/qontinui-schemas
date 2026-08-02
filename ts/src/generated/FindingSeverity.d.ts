/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Severity level of a finding.
 *
 * Lifecycle (ordered, most-severe first): `CRITICAL` → `HIGH` → `MEDIUM` →
 * `LOW` → `INFO`. Mirrors Python `FindingSeverity(str, Enum)`.
 */
export type FindingSeverity = "critical" | "high" | "medium" | "low" | "info";
