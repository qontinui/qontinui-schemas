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
  autoContinue: boolean;
  /**
   * ISO 8601 timestamp when the task completed.
   */
  completedAt?: string | null;
  /**
   * ID of the workflow config used to run this task, if any.
   */
  configId?: string | null;
  /**
   * ISO 8601 timestamp when the task was created.
   */
  createdAt: string;
  /**
   * Error message if the task failed.
   */
  errorMessage?: string | null;
  /**
   * Whether the task's goal was achieved (AI assessment).
   */
  goalAchieved?: boolean | null;
  /**
   * Unique identifier (UUID v4 string).
   */
  id: string;
  /**
   * Optional cap on AI sessions.
   */
  maxSessions?: number | null;
  /**
   * Accumulated output log for the task run.
   */
  outputLog: string;
  /**
   * Original prompt text, if applicable.
   */
  prompt?: string | null;
  /**
   * Description of any remaining work (AI assessment).
   */
  remainingWork?: string | null;
  /**
   * Number of AI sessions that have been run.
   */
  sessionsCount: number;
  status: TaskRunStatus;
  /**
   * AI-generated summary of the task run.
   */
  summary?: string | null;
  /**
   * ISO 8601 timestamp when the summary was generated.
   */
  summaryGeneratedAt?: string | null;
  /**
   * Display name of the task.
   */
  taskName: string;
  taskType: TaskType;
  /**
   * ISO 8601 timestamp when the task record was last updated.
   */
  updatedAt: string;
  /**
   * Name of the workflow used to run this task, if any.
   */
  workflowName?: string | null;
}
