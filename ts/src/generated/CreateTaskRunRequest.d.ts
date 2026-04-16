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
  auto_continue?: boolean | null;
  /**
   * Workflow config ID to use.
   */
  config_id?: string | null;
  /**
   * Serialized execution steps, if provided ad-hoc.
   */
  execution_steps_json?: string | null;
  /**
   * Serialized log-sources configuration.
   */
  log_sources_json?: string | null;
  /**
   * Optional cap on AI sessions.
   */
  max_sessions?: number | null;
  /**
   * Prompt text, if applicable.
   */
  prompt?: string | null;
  /**
   * Display name.
   */
  task_name: string;
  /**
   * Task type.
   */
  task_type?: TaskType | null;
  /**
   * Workflow name to use.
   */
  workflow_name?: string | null;
  [k: string]: unknown;
}
