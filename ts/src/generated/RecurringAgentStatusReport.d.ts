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
import type { RecurringAgentStatus } from "./RecurringAgentStatus";
import type { RepositoryInactiveCondition } from "./RepositoryInactiveCondition";
import type { RepositoryWatch } from "./RepositoryWatch";
import type { ScheduleConditions } from "./ScheduleConditions";
import type { ScheduleExpression } from "./ScheduleExpression";
import type { ScheduledTaskStatus } from "./ScheduledTaskStatus";
import type { ScheduledTaskType } from "./ScheduledTaskType";
import type { TaskExecutionRecord } from "./TaskExecutionRecord";

/**
 * What a runner posts to coord after every reconcile and every run
 * (`POST /coord/recurring-agents/status/:device_id`, plan D7): one entry per
 * recurring agent the tenant's declaration targets at this device.
 */
export interface RecurringAgentStatusReport {
  /**
   * The device's "accept managed tasks" setting (D9; default on). When
   * off, the runner applies nothing and reports each agent `unapplied`
   * with [`UNAPPLIED_MANAGED_TASKS_REFUSED`].
   */
  accepts_managed: boolean;
  /**
   * One entry per agent, names unique.
   */
  agents: RecurringAgentStatus[];
  /**
   * RFC 3339 timestamp when the device built this report.
   */
  reported_at: string;
  /**
   * The tenant (UUID) the reported agents were declared by. A device bound
   * to several tenants posts one report per proven tenant.
   */
  tenant_id: string;
}
