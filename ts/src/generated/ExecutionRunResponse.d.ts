/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { RunStatus } from './RunStatus';
import type { RunType } from './RunType';

/**
 * Response envelope returned when a run is created or fetched.
 */
export interface ExecutionRunResponse {
  /**
   * Total duration in seconds, if the run has ended.
   */
  duration_seconds?: number | null;
  /**
   * ISO 8601 timestamp when the run ended, if it has ended.
   */
  ended_at?: string | null;
  /**
   * Owning project ID.
   */
  project_id: string;
  /**
   * Assigned run identifier.
   */
  run_id: string;
  /**
   * Human-readable run name.
   */
  run_name: string;
  run_type: RunType;
  /**
   * ISO 8601 timestamp when the run started.
   */
  started_at: string;
  status: RunStatus;
  [k: string]: unknown;
}
