/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ProcessState } from "./ProcessState";

/**
 * A managed process attributed to a project, reduced to what the dashboard
 * renders. Full detail stays in `ProcessStatus` / `ProcessConfig`.
 */
export interface ProcessStatusLite {
  /**
   * Working directory — the field that attributed this process to the
   * project in the first place.
   */
  cwd: string;
  /**
   * Health port, when the process declares one.
   */
  healthPort?: number | null;
  /**
   * `ProcessConfig.id`.
   */
  id: string;
  /**
   * Human-readable process name ("Website").
   */
  name: string;
  /**
   * Whether the health port is currently responding. `None` when the
   * process declares no health port.
   */
  portHealthy?: boolean | null;
  /**
   * Lifecycle state as reported by the process manager.
   */
  state: ProcessState;
}
