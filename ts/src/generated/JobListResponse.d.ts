/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { JobItem } from './JobItem';
import type { JobStatus } from './JobStatus';

/**
 * Paginated list of jobs.
 */
export interface JobListResponse {
  has_more: boolean;
  items: JobItem[];
  limit: number;
  page: number;
  total: number;
}
