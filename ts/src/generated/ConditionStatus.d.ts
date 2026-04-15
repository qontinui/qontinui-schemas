/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Status of condition checking for a deferred task.
 */
export interface ConditionStatus {
  /**
   * Current idle-condition result. `None` if not yet checked,
   * `Some(true)` if idle, `Some(false)` if busy.
   */
  idle_met?: boolean | null;
  /**
   * Current repository-inactive status per repository: `(path, is_inactive)`.
   */
  repo_inactive_met?: [unknown, unknown][] | null;
  /**
   * Whether the overall condition-wait timeout has been exceeded.
   */
  timed_out: boolean;
  /**
   * ISO 8601 timestamp when conditions began being evaluated.
   */
  waiting_since: string;
  [k: string]: unknown;
}
