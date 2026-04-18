/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { TaskType } from './TaskType';

/**
 * Request payload for creating a task run.
 */
export interface TaskRunCreate {
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
   * Optional client-generated ID.
   */
  id?: string | null;
  /**
   * Serialized log-sources configuration.
   */
  logSourcesJson?: string | null;
  /**
   * Optional cap on AI sessions.
   */
  maxSessions?: number | null;
  /**
   * Optional owning project.
   */
  projectId?: string | null;
  /**
   * Prompt text, if applicable.
   */
  prompt?: string | null;
  /**
   * Optional runner that will execute the task.
   */
  runnerId?: string | null;
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
}
