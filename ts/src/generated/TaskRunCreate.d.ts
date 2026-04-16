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
   * Optional client-generated ID.
   */
  id?: string | null;
  /**
   * Serialized log-sources configuration.
   */
  log_sources_json?: string | null;
  /**
   * Optional cap on AI sessions.
   */
  max_sessions?: number | null;
  /**
   * Optional owning project.
   */
  project_id?: string | null;
  /**
   * Prompt text, if applicable.
   */
  prompt?: string | null;
  /**
   * Optional runner that will execute the task.
   */
  runner_id?: string | null;
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
