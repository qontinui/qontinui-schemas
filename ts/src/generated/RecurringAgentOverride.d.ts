/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ConditionScheduleConfig } from "./ConditionScheduleConfig";
import type { IdleCondition } from "./IdleCondition";
import type { McpConnectionRef } from "./McpConnectionRef";
import type { ProbeCondition } from "./ProbeCondition";
import type { RepositoryInactiveCondition } from "./RepositoryInactiveCondition";
import type { RepositoryWatch } from "./RepositoryWatch";
import type { ScheduleConditions } from "./ScheduleConditions";
import type { ScheduleExpression } from "./ScheduleExpression";
import type { ScheduledTaskType } from "./ScheduledTaskType";

/**
 * A local override of a managed task on one machine (plan D6), stored by the
 * runner and reported to coord so the web shows it. Only the fields set are
 * overridden; the rest follow the fleet declaration.
 */
export interface RecurringAgentOverride {
  /**
   * Replacement conditions (e.g. require idle / repo inactive).
   */
  conditions?: ScheduleConditions | null;
  /**
   * ISO 8601 timestamp when the override was made.
   */
  created_at: string;
  /**
   * Free-form note from whoever made the override.
   */
  note?: string | null;
  /**
   * Replacement schedule.
   */
  schedule?: ScheduleExpression | null;
  /**
   * Replacement task.
   */
  task?: ScheduledTaskType | null;
  /**
   * Replacement IANA time zone.
   */
  timezone?: string | null;
}
