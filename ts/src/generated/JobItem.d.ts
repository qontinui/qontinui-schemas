/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { JobStatus } from './JobStatus';

/**
 * Single job record for display.
 */
export interface JobItem {
  completed_at?: string | null;
  /**
   * ISO 8601 UTC timestamp (`Z` suffix).
   */
  created_at: string;
  error_message?: string | null;
  id: string;
  job_metadata: {
    [k: string]: unknown;
  };
  max_retries: number;
  processed_patterns: number;
  progress_percent: number;
  retry_count: number;
  started_at?: string | null;
  status: JobStatus;
  total_patterns: number;
}
