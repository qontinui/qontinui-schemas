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
  durationSeconds?: number | null;
  /**
   * ISO 8601 timestamp when the run ended, if it has ended.
   */
  endedAt?: string | null;
  /**
   * Owning project ID.
   */
  projectId: string;
  /**
   * Assigned run identifier.
   */
  runId: string;
  /**
   * Human-readable run name.
   */
  runName: string;
  runType: RunType;
  /**
   * ISO 8601 timestamp when the run started.
   */
  startedAt: string;
  status: RunStatus;
  [k: string]: unknown;
}
