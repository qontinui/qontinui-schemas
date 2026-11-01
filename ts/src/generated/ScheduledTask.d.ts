/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { CatchUpPolicy } from "./CatchUpPolicy";
import type { ConditionScheduleConfig } from "./ConditionScheduleConfig";
import type { ConditionStatus } from "./ConditionStatus";
import type { IdleCondition } from "./IdleCondition";
import type { McpConnectionRef } from "./McpConnectionRef";
import type { ProbeCondition } from "./ProbeCondition";
import type { RepositoryInactiveCondition } from "./RepositoryInactiveCondition";
import type { RepositoryWatch } from "./RepositoryWatch";
import type { ScheduleConditions } from "./ScheduleConditions";
import type { ScheduleExpression } from "./ScheduleExpression";
import type { ScheduledTaskStatus } from "./ScheduledTaskStatus";
import type { ScheduledTaskType } from "./ScheduledTaskType";
import type { TaskExecutionRecord } from "./TaskExecutionRecord";

/**
 * A scheduled task definition — the full persisted frame, including computed
 * fields (`last_run`, `next_run`) and condition-evaluation state
 * (`conditions`, `condition_status`).
 */
export interface ScheduledTask {
  /**
   * Automatically trigger auto-fix when this task fails.
   */
  autoFixOnFailure: boolean;
  /**
   * Slots within this number of seconds of "now" are not treated as
   * missed (the normal scheduler will pick them up). Default 300s.
   */
  catchUpGraceSeconds: number;
  /**
   * Catch-up policy applied when the runner restarts after missing one
   * or more scheduled slots.
   */
  catchUpPolicy?: CatchUpPolicy & string;
  /**
   * Present while the task is waiting for its conditions to be met.
   */
  conditionStatus?: ConditionStatus | null;
  /**
   * Optional conditions that must be met before execution.
   */
  conditions?: ScheduleConditions | null;
  /**
   * Number of consecutive launch failures since the task last started
   * successfully. Drives exponential backoff. Reset to 0 on the next
   * successful launch.
   */
  consecutiveLaunchFailures: number;
  /**
   * ISO 8601 timestamp of creation.
   */
  createdAt: string;
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
  lastRun?: TaskExecutionRecord | null;
  /**
   * **Base** backoff in seconds for launch failures (not the current
   * accumulated backoff). The runner computes the effective delay as
   * `min(base * 2^(consecutive_launch_failures - 1), 86400)`.
   * Default 60s.
   */
  launchFailureBackoffSeconds: number;
  /**
   * Set when the task was created by the runner's reconciler from a
   * fleet-wide recurring-agent declaration rather than by a user. `None` =
   * an unmanaged, user-created task, which the reconciler never touches.
   */
  managed?: ManagedBy | null;
  /**
   * ISO 8601 timestamp of last modification.
   */
  modifiedAt: string;
  /**
   * Display name for the task.
   */
  name: string;
  /**
   * Next scheduled run time (ISO 8601), computed by the runner.
   */
  nextRun?: string | null;
  /**
   * Schedule configuration.
   */
  schedule: ScheduleExpression;
  /**
   * Skip future runs once the task has succeeded at least once.
   */
  skipIfCompleted: boolean;
  /**
   * Free-form description of success criteria, for human reference.
   */
  successCriteria?: string | null;
  /**
   * Task type and its per-type configuration.
   */
  task: ScheduledTaskType;
  /**
   * IANA time zone this task's schedule is interpreted in (e.g.
   * `Europe/Berlin`). Takes precedence over the global
   * [`SchedulerSettings::timezone`]; `None` = the scheduler's zone. Lets one
   * fleet-wide declaration mean either "local night on every device" or a
   * fixed instant.
   */
  timezone?: string | null;
}
/**
 * Provenance of a [`ScheduledTask`] that the runner reconciles from a
 * fleet-wide declaration (plan
 * `2026-10-29-recurring-agents-declared-in-coord-reconciled-by-runner`, D5).
 *
 * `(source, tenant_id, agent)` is the managed key: the runner holds at most
 * one task per key. What the reconciler compares to decide whether a task is
 * current is `digest`, never `version` — a catalog fix in coord's code moves
 * no row version, and a deleted-and-recreated agent restarts its version at 1.
 */
export interface ManagedBy {
  /**
   * The recurring agent's name — its join key across coord's status, run
   * requests and this managed task (coord forbids renames).
   */
  agent: string;
  /**
   * sha256 (64 lowercase hex) of the resolved spec coord served when this
   * task was applied. `None` = applied from a declaration that carried no
   * digest.
   */
  digest?: string | null;
  /**
   * Which declaration store the task came from — [`MANAGED_SOURCE_COORD`].
   * A plain string rather than an enum, so a stored task from a newer
   * source still loads; the consumer decides what it recognises.
   */
  source: string;
  /**
   * The coord tenant (UUID) whose declaration this task applies. A device
   * bound to several tenants holds one managed task per tenant and agent.
   */
  tenantId: string;
  /**
   * The declaration's row version when applied. History and display only.
   */
  version: number;
}
