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
 * Request payload for marking a run complete.
 */
export interface ExecutionRunComplete {
  /**
   * Coverage data, if the run executed a workflow.
   */
  coverage?: CoverageData | null;
  /**
   * ISO 8601 timestamp when the run ended.
   */
  ended_at: string;
  /**
   * Error message if the run failed.
   */
  error_message?: string | null;
  stats: ExecutionStats;
  status: RunStatus;
  /**
   * Free-form run summary.
   */
  summary?: string | null;
  [k: string]: unknown;
}
