/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ConditionScheduleConfig } from './ConditionScheduleConfig';
import type { IdleCondition } from './IdleCondition';
import type { RepositoryInactiveCondition } from './RepositoryInactiveCondition';
import type { RepositoryWatch } from './RepositoryWatch';
import type { ScheduleConditions } from './ScheduleConditions';
import type { ScheduleExpression } from './ScheduleExpression';
import type { ScheduledTaskType } from './ScheduledTaskType';

/**
 * Request body for updating an existing scheduled task. All fields are
 * optional; only those supplied are applied.
 */
export interface UpdateScheduledTaskRequest {
  /**
   * Update `auto_fix_on_failure`.
   */
  autoFixOnFailure?: boolean | null;
  /**
   * Replace the conditions block (pass `null` to clear).
   */
  conditions?: ScheduleConditions | null;
  /**
   * New description (pass `null` to clear).
   */
  description?: string | null;
  /**
   * Enable/disable the task.
   */
  enabled?: boolean | null;
  /**
   * New display name.
   */
  name?: string | null;
  /**
   * Replace the schedule expression.
   */
  schedule?: ScheduleExpression | null;
  /**
   * Update `skip_if_completed`.
   */
  skipIfCompleted?: boolean | null;
  /**
   * Update the success criteria (pass `null` to clear).
   */
  successCriteria?: string | null;
  /**
   * Replace the task definition.
   */
  task?: ScheduledTaskType | null;
  [k: string]: unknown;
}
