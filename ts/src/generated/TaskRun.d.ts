/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { TaskRunStatus } from './TaskRunStatus';
import type { TaskType } from './TaskType';

/**
 * A task run as tracked by the local runner during execution.
 *
 * Mirrors `TaskRun` in the runner's `taskRun.ts`. Optional fields here use
 * `?` in TypeScript, so they are omitted on the wire when missing.
 */
export interface TaskRun {
  /**
   * Whether the task will auto-continue into another session on exit.
   */
  auto_continue: boolean;
  /**
   * ISO 8601 timestamp when the task completed.
   */
  completed_at?: string | null;
  /**
   * ID of the workflow config used to run this task, if any.
   */
  config_id?: string | null;
  /**
   * ISO 8601 timestamp when the task was created.
   */
  created_at: string;
  /**
   * Error message if the task failed.
   */
  error_message?: string | null;
  /**
   * Whether the task's goal was achieved (AI assessment).
   */
  goal_achieved?: boolean | null;
  /**
   * Unique identifier (UUID v4 string).
   */
  id: string;
  /**
   * Optional cap on AI sessions.
   */
  max_sessions?: number | null;
  /**
   * Accumulated output log for the task run.
   */
  output_log: string;
  /**
   * Original prompt text, if applicable.
   */
  prompt?: string | null;
  /**
   * Description of any remaining work (AI assessment).
   */
  remaining_work?: string | null;
  /**
   * Number of AI sessions that have been run.
   */
  sessions_count: number;
  status: TaskRunStatus;
  /**
   * AI-generated summary of the task run.
   */
  summary?: string | null;
  /**
   * ISO 8601 timestamp when the summary was generated.
   */
  summary_generated_at?: string | null;
  /**
   * Display name of the task.
   */
  task_name: string;
  task_type: TaskType;
  /**
   * ISO 8601 timestamp when the task record was last updated.
   */
  updated_at: string;
  /**
   * Name of the workflow used to run this task, if any.
   */
  workflow_name?: string | null;
  [k: string]: unknown;
}
