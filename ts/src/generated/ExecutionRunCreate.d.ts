/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { RunType } from './RunType';
import type { RunnerMetadata } from './RunnerMetadata';
import type { WorkflowMetadata } from './WorkflowMetadata';

/**
 * Request payload for creating a new execution run.
 */
export interface ExecutionRunCreate {
  /**
   * Run configuration bag (opaque to this layer).
   */
  configuration?: {
    [k: string]: unknown;
  } | null;
  /**
   * Optional free-form description.
   */
  description?: string | null;
  /**
   * Owning project ID.
   */
  project_id: string;
  /**
   * Human-readable run name.
   */
  run_name: string;
  run_type: RunType;
  runner_metadata: RunnerMetadata;
  /**
   * Workflow metadata, if the run executes a workflow.
   */
  workflow_metadata?: WorkflowMetadata | null;
  [k: string]: unknown;
}
