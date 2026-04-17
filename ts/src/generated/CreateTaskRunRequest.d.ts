/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { TaskType } from './TaskType';

/**
 * Request body for creating a task run (simplified shape used by the runner's
 * create-task endpoint).
 */
export interface CreateTaskRunRequest {
  /**
   * Whether the task should auto-continue.
   */
  autoContinue?: boolean | null;
  /**
   * Workflow config ID to use.
   */
  configId?: string | null;
  /**
   * Serialized execution steps, if provided ad-hoc.
   */
  executionStepsJson?: string | null;
  /**
   * Serialized log-sources configuration.
   */
  logSourcesJson?: string | null;
  /**
   * Optional cap on AI sessions.
   */
  maxSessions?: number | null;
  /**
   * Prompt text, if applicable.
   */
  prompt?: string | null;
  /**
   * Display name.
   */
  taskName: string;
  /**
   * Task type.
   */
  taskType?: TaskType | null;
  /**
   * Workflow name to use.
   */
  workflowName?: string | null;
  [k: string]: unknown;
}
