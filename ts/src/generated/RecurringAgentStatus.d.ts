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
import type { RecurringAgentOverride } from "./RecurringAgentOverride";
import type { RecurringAgentState } from "./RecurringAgentState";
import type { RepositoryInactiveCondition } from "./RepositoryInactiveCondition";
import type { RepositoryWatch } from "./RepositoryWatch";
import type { ScheduleConditions } from "./ScheduleConditions";
import type { ScheduleExpression } from "./ScheduleExpression";
import type { ScheduledTaskStatus } from "./ScheduledTaskStatus";
import type { ScheduledTaskType } from "./ScheduledTaskType";
import type { TaskExecutionRecord } from "./TaskExecutionRecord";

/**
 * One device's status for one recurring agent — a
 * `coord.recurring_agent_device_status` row (plan D7).
 */
export interface RecurringAgentStatus {
  /**
   * The `request_id` of the newest "Run now" request the device ran,
   * echoed so coord can stop serving it as pending (plan D10).
   */
  acknowledged_request_id?: string | null;
  /**
   * The digest of the spec applied. `None` = none applied.
   */
  applied_digest?: string | null;
  /**
   * The declaration version applied. Display only.
   */
  applied_version?: number | null;
  /**
   * ISO 8601 end of the newest completed run. `None` = none known.
   */
  last_completed_at?: string | null;
  /**
   * The newest run.
   */
  last_run?: TaskExecutionRecord | null;
  /**
   * ISO 8601 end of the newest successful run. `None` = none known.
   */
  last_success_at?: string | null;
  /**
   * The agent's name.
   */
  name: string;
  /**
   * ISO 8601 next scheduled run.
   */
  next_run?: string | null;
  /**
   * The local override record. Required when `state` is `overridden`; a
   * `paused` task may also carry one.
   */
  override?: RecurringAgentOverride | null;
  /**
   * The last N runs, newest first.
   */
  recent_runs?: TaskExecutionRecord[];
  /**
   * What the device did with the agent.
   */
  state: RecurringAgentState;
  /**
   * Why the agent is `unapplied` or `refused` (e.g.
   * [`UNAPPLIED_WORKSPACE_ROOT_UNSET`], [`REFUSED_COMMAND`]). Present
   * exactly for those two states.
   */
  unapplied_reason?: string | null;
}
