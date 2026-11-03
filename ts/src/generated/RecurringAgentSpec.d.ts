/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { CatchUpPolicy } from "./CatchUpPolicy";
import type { ConditionScheduleConfig } from "./ConditionScheduleConfig";
import type { IdleCondition } from "./IdleCondition";
import type { McpConnectionRef } from "./McpConnectionRef";
import type { ProbeCondition } from "./ProbeCondition";
import type { RecurringAgentDispatchRole } from "./RecurringAgentDispatchRole";
import type { RecurringAgentOs } from "./RecurringAgentOs";
import type { RecurringAgentTarget } from "./RecurringAgentTarget";
import type { RepositoryInactiveCondition } from "./RepositoryInactiveCondition";
import type { RepositoryWatch } from "./RepositoryWatch";
import type { ScheduleConditions } from "./ScheduleConditions";
import type { ScheduleExpression } from "./ScheduleExpression";
import type { ScheduledTaskType } from "./ScheduledTaskType";

/**
 * One recurring agent as coord serves it: a catalog entry merged with the
 * tenant's overrides, or a custom agent's full spec (plan D2).
 *
 * On coord's device read route `task_spec` carries the resolved task (for a
 * catalog agent, the template rendered from `params`; a working directory may
 * still be the `{workspace_root}` token, which the runner resolves). A
 * custom agent (no `catalog_ref`) must carry a `RemoteAgent` task and no
 * `requireProbe` condition (D9): `Script` and probe commands come only from
 * the catalog, and the runner applies them only when they equal its own
 * bundled pin.
 */
export interface RecurringAgentSpec {
  /**
   * The catalog entry this agent is, or overrides. `None` = a custom agent.
   */
  catalog_ref?: string | null;
  /**
   * What to do about slots missed while a runner was down.
   */
  catch_up_policy?: CatchUpPolicy & string;
  /**
   * Conditions the task waits for before running.
   */
  conditions?: ScheduleConditions | null;
  /**
   * sha256 (64 lowercase hex) of the device-independent resolved spec,
   * computed by coord. What a runner stores as `managed.digest` and echoes
   * back as `applied_digest`.
   */
  digest?: string | null;
  /**
   * Whether the agent is enabled for the tenant.
   */
  enabled: boolean;
  /**
   * The agent's name — unique per tenant, and the join key for device
   * status, run requests and managed tasks (coord forbids renames).
   */
  name: string;
  /**
   * Values for the catalog entry's [`RecurringAgentParamSchema`]. Always a
   * JSON object; empty = every parameter at its default.
   */
  params: {
    [k: string]: unknown;
  };
  /**
   * When the agent runs.
   */
  schedule: ScheduleExpression;
  /**
   * Which devices the agent applies to.
   */
  target: RecurringAgentTarget;
  /**
   * The task to run. Required for a custom agent; for a catalog agent, the
   * resolved template when coord serves it to a device.
   */
  task_spec?: ScheduledTaskType | null;
  /**
   * IANA time zone for `schedule`. `None` = each device's scheduler zone.
   */
  timezone?: string | null;
  /**
   * The declaration's row version (starts at 1). History and display only —
   * compare `digest`, never `version`, to decide whether a device is current.
   */
  version: number;
}
