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
 * Request body for creating a new scheduled task.
 */
export interface CreateScheduledTaskRequest {
  /**
   * Trigger auto-fix on failure.
   */
  auto_fix_on_failure?: boolean | null;
  /**
   * Optional conditions that must be met before execution.
   */
  conditions?: ScheduleConditions | null;
  /**
   * Optional description.
   */
  description?: string | null;
  /**
   * Display name.
   */
  name: string;
  schedule: ScheduleExpression;
  /**
   * Skip future runs once the task has succeeded.
   */
  skip_if_completed?: boolean | null;
  /**
   * Free-form success criteria description.
   */
  success_criteria?: string | null;
  task: ScheduledTaskType;
  [k: string]: unknown;
}
