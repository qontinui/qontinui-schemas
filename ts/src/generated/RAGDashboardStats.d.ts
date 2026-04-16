/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { JobStatus } from './JobStatus';
import type { JobSummary } from './JobSummary';

/**
 * Summary statistics for the RAG dashboard.
 */
export interface RAGDashboardStats {
  /**
   * Currently-running job, if any.
   */
  active_job?: JobSummary | null;
  /**
   * ISO 8601 UTC timestamp (`Z` suffix).
   */
  last_sync_at?: string | null;
  total_embeddings: number;
  total_patterns: number;
  total_states: number;
  [k: string]: unknown;
}
