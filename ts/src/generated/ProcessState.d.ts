/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * State of a managed process.
 *
 * Ordering mirrors the lifecycle progression:
 * `Stopped → Starting → (Building) → Running → Healthy → Stopping → Stopped`,
 * or to `Failed` on any abnormal exit.
 * `ExternallyOwned` is a special out-of-band state meaning "a process is
 * running on our port, but this runner did not spawn it" — used by the
 * process-reconcile feature to surface port squatters without touching the
 * normal lifecycle.
 */
export type ProcessState =
  | (
      | "stopped"
      | "starting"
      | "building"
      | "running"
      | "healthy"
      | "stopping"
      | "failed"
    )
  | "externally_owned";
