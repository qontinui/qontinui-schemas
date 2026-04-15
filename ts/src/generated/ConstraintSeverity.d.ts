/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Severity of a constraint violation.
 *
 * - `Block`: Reject the fix, inject violation context, re-run agentic phase
 *   without consuming an iteration. After `max_retries`, consume the iteration.
 * - `Warn`: Apply the fix, but inject violation context for the next iteration.
 * - `Log`: Record only, don't affect execution.
 */
export type ConstraintSeverity = "block" | "warn" | "log";
