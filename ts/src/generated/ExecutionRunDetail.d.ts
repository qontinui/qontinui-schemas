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
  createdAt: string;
  /**
   * Run description.
   */
  description?: string | null;
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
  runnerMetadata: RunnerMetadata;
  /**
   * ISO 8601 timestamp when the run started.
   */
  startedAt: string;
  stats: ExecutionStats;
  status: RunStatus;
  /**
   * ISO 8601 timestamp when the record was last updated.
   */
  updatedAt?: string | null;
  /**
   * Workflow metadata, if applicable.
   */
  workflowMetadata?: WorkflowMetadata | null;
}
