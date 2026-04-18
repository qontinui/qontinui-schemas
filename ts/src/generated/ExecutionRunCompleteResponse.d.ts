/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { CoverageData } from './CoverageData';
import type { ExecutionStats } from './ExecutionStats';
import type { RunStatus } from './RunStatus';

/**
 * Response envelope returned after completing a run.
 */
export interface ExecutionRunCompleteResponse {
  /**
   * Coverage data, if the run executed a workflow.
   */
  coverage?: CoverageData | null;
  /**
   * Total duration in seconds.
   */
  durationSeconds: number;
  /**
   * ISO 8601 timestamp when the run ended.
   */
  endedAt: string;
  /**
   * Associated run ID.
   */
  runId: string;
  /**
   * ISO 8601 timestamp when the run started.
   */
  startedAt: string;
  stats: ExecutionStats;
  status: RunStatus;
}
