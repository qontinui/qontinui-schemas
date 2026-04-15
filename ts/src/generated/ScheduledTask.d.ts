/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ConditionScheduleConfig } from './ConditionScheduleConfig';
import type { ConditionStatus } from './ConditionStatus';
import type { IdleCondition } from './IdleCondition';
import type { RepositoryInactiveCondition } from './RepositoryInactiveCondition';
import type { RepositoryWatch } from './RepositoryWatch';
import type { ScheduleConditions } from './ScheduleConditions';
import type { ScheduleExpression } from './ScheduleExpression';
import type { ScheduledTaskStatus } from './ScheduledTaskStatus';
import type { ScheduledTaskType } from './ScheduledTaskType';
import type { TaskExecutionRecord } from './TaskExecutionRecord';

/**
 * A scheduled task definition — the full persisted frame, including computed
 * fields (`last_run`, `next_run`) and condition-evaluation state
 * (`conditions`, `condition_status`).
 */
export interface ScheduledTask {
  /**
   * Automatically trigger auto-fix when this task fails.
   */
  auto_fix_on_failure: boolean;
  /**
   * Present while the task is waiting for its conditions to be met.
   */
  condition_status?: ConditionStatus | null;
  /**
   * Optional conditions that must be met before execution.
   */
  conditions?: ScheduleConditions | null;
  /**
   * ISO 8601 timestamp of creation.
   */
  created_at: string;
  /**
   * Optional human-readable description.
   */
  description?: string | null;
  /**
   * Whether the task is enabled and eligible to run.
   */
  enabled: boolean;
  /**
   * Unique identifier (UUID v4 string).
   */
  id: string;
  /**
   * Record of the most recent execution.
   */
  last_run?: TaskExecutionRecord | null;
  /**
   * ISO 8601 timestamp of last modification.
   */
  modified_at: string;
  /**
   * Display name for the task.
   */
  name: string;
  /**
   * Next scheduled run time (ISO 8601), computed by the runner.
   */
  next_run?: string | null;
  schedule: ScheduleExpression;
  /**
   * Skip future runs once the task has succeeded at least once.
   */
  skip_if_completed: boolean;
  /**
   * Free-form description of success criteria, for human reference.
   */
  success_criteria?: string | null;
  task: ScheduledTaskType;
  [k: string]: unknown;
}
