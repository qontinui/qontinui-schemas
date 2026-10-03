/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
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
  idleMet?: boolean | null;
  /**
   * Human-readable outcome of the last probe evaluation — e.g. `exit 0`,
   * `exit 3`, `timed out after 30s`, `spawn failed: ...`, `probe running;
   * awaiting its result`, `not run: another condition is not met` — with a
   * bounded stderr tail appended when the probe wrote one.
   */
  probeDetail?: string | null;
  /**
   * Current probe-condition result. `None` if no probe is configured or
   * none has been evaluated yet, `Some(true)` if the last probe exited 0.
   */
  probeMet?: boolean | null;
  /**
   * Current repository-inactive status per repository: `(path, is_inactive)`.
   */
  repoInactiveMet?: [unknown, unknown][] | null;
  /**
   * Whether the overall condition-wait timeout has been exceeded.
   */
  timedOut: boolean;
  /**
   * ISO 8601 timestamp when conditions began being evaluated.
   */
  waitingSince: string;
}
