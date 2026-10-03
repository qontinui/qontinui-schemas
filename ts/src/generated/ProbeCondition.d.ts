/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Condition that runs an external command and is met iff it exits 0.
 *
 * The runner execs `command` directly — argv, never a shell — so `command[0]`
 * is the program and the rest are its arguments; wrap it in `sh -c` yourself
 * if you want shell syntax. The condition is **NOT met** on a non-zero exit,
 * on a timeout (the probe is killed at `timeout_seconds`) and on a spawn
 * failure (including an empty `command`); each is logged distinctly. An
 * unobserved or failed probe never reads as met.
 *
 * The probe is rate-limited to one run per `poll_seconds` per task: between
 * polls the runner reuses the last result. `poll_seconds` is floored at 60
 * because the scheduler re-evaluates conditions once per 60 s tick — a shorter
 * interval could not be honoured. The runner's create/update API REFUSES a
 * value below the floor (and an empty `command`, and a `timeout_seconds` of
 * 0 or above 3600) rather than silently rewriting it; a stored row that
 * predates that validation is clamped up to 60 at evaluation time.
 */
export interface ProbeCondition {
  /**
   * Program and arguments to exec (no shell). Must be non-empty.
   */
  command: string[];
  /**
   * Whether this condition is active.
   */
  enabled: boolean;
  /**
   * Minimum seconds between two probe runs for the same task. Minimum 60.
   */
  pollSeconds: number;
  /**
   * Seconds a single probe run may take before it is killed and counted as
   * NOT met. 1..=3600.
   */
  timeoutSeconds: number;
}
