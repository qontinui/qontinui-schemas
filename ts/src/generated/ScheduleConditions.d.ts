/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { IdleCondition } from './IdleCondition';
import type { RepositoryInactiveCondition } from './RepositoryInactiveCondition';
import type { RepositoryWatch } from './RepositoryWatch';

/**
 * Conditions that must ALL be met before task execution.
 */
export interface ScheduleConditions {
  /**
   * Require the runner to be idle.
   */
  require_idle?: IdleCondition | null;
  /**
   * Require repository file inactivity across one or more paths.
   */
  require_repo_inactive?: RepositoryInactiveCondition | null;
  /**
   * Maximum time to wait for conditions (minutes). `None` = wait
   * indefinitely.
   */
  timeout_minutes?: number | null;
  [k: string]: unknown;
}
