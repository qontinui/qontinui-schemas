/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { CoverageData } from './CoverageData';
import type { ExecutionStats } from './ExecutionStats';
import type { RunStatus } from './RunStatus';
import type { RunType } from './RunType';
import type { RunnerMetadata } from './RunnerMetadata';
import type { WorkflowMetadata } from './WorkflowMetadata';

/**
 * Detailed execution run information (superset of [`ExecutionRunResponse`]).
 *
 * Python models this via inheritance; here all fields are inlined because Rust
 * has no inheritance and the wire form is a flat object.
 */
export interface ExecutionRunDetail {
  /**
   * Configuration snapshot captured at run start.
   */
  configuration?: {
    [k: string]: unknown;
  };
  /**
   * Coverage data, if applicable.
   */
  coverage?: CoverageData | null;
  /**
   * ISO 8601 timestamp when the record was created.
   */
  created_at: string;
  /**
   * Run description.
   */
  description?: string | null;
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
  runner_metadata: RunnerMetadata;
  /**
   * ISO 8601 timestamp when the run started.
   */
  started_at: string;
  stats: ExecutionStats;
  status: RunStatus;
  /**
   * ISO 8601 timestamp when the record was last updated.
   */
  updated_at?: string | null;
  /**
   * Workflow metadata, if applicable.
   */
  workflow_metadata?: WorkflowMetadata | null;
  [k: string]: unknown;
}
