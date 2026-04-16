/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { JobStatus } from './JobStatus';

/**
 * Summary of an embedding-generation job.
 */
export interface JobSummary {
  error_message?: string | null;
  /**
   * Job UUID.
   */
  id: string;
  processed_patterns: number;
  /**
   * Progress percentage (0-100).
   */
  progress_percent: number;
  /**
   * ISO 8601 UTC timestamp (`Z` suffix).
   */
  started_at?: string | null;
  status: JobStatus;
  total_patterns: number;
  [k: string]: unknown;
}
