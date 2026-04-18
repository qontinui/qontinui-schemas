/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Configuration for condition-only schedules.
 */
interface ConditionScheduleConfig {
  /**
   * Minutes to wait after an execution completes before re-evaluating
   * conditions for another run.
   */
  rearmDelayMinutes: number;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * How a task should be scheduled.
 *
 * Serialized with the external tag `type` and payload under `value` so that
 * `Once("...")`, `Cron("...")`, and `Interval(60)` round-trip as
 * `{ "type": "Once", "value": "..." }` etc. The `Condition` variant wraps a
 * [`ConditionScheduleConfig`] rather than a scalar, but uses the same
 * `{ type, value }` envelope.
 */
type ScheduleExpression =
  | {
      type: "Once";
      value: string;
      [k: string]: unknown;
    }
  | {
      type: "Cron";
      value: string;
      [k: string]: unknown;
    }
  | {
      type: "Interval";
      value: number;
      [k: string]: unknown;
    }
  | {
      type: "Condition";
      value: ConditionScheduleConfig;
      [k: string]: unknown;
    };

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Condition that requires the runner to be idle (not executing workflows or
 * AI tasks) before the task may run.
 */
interface IdleCondition {
  /**
   * Whether this condition is active.
   */
  enabled: boolean;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * A single repository to monitor for inactivity.
 */
interface RepositoryWatch {
  /**
   * Minutes of inactivity required before the watch is considered met.
   */
  inactiveMinutes: number;
  /**
   * Path to the repository directory.
   */
  path: string;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Condition that requires repositories to have no file modifications for a
 * period. ALL configured repositories must be inactive for the overall
 * condition to be met.
 */
interface RepositoryInactiveCondition {
  /**
   * Whether this condition is active.
   */
  enabled: boolean;
  /**
   * List of repositories to watch. ALL must be inactive simultaneously.
   */
  repositories: RepositoryWatch[];
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Conditions that must ALL be met before task execution.
 */
interface ScheduleConditions {
  /**
   * Require the runner to be idle.
   */
  requireIdle?: IdleCondition | null;
  /**
   * Require repository file inactivity across one or more paths.
   */
  requireRepoInactive?: RepositoryInactiveCondition | null;
  /**
   * Maximum time to wait for conditions (minutes). `None` = wait
   * indefinitely.
   */
  timeoutMinutes?: number | null;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Status of condition checking for a deferred task.
 */
interface ConditionStatus {
  /**
   * Current idle-condition result. `None` if not yet checked,
   * `Some(true)` if idle, `Some(false)` if busy.
   */
  idleMet?: boolean | null;
  /**
   * Current repository-inactive status per repository: `(path, is_inactive)`.
   */
  repoInactiveMet?: [unknown, unknown][] | null;
  /**
   * Whether the overall condition-wait timeout has been exceeded.
   */
  timedOut: boolean;
  /**
   * ISO 8601 timestamp when conditions began being evaluated.
   */
  waitingSince: string;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Status of a scheduled task execution.
 */
type ScheduledTaskStatus = "pending" | "running" | "completed" | "failed" | "skipped" | "cancelled";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Type of task to schedule.
 *
 * Internally tagged by `task_type`: the variant fields are inlined alongside
 * the discriminator rather than nested under a `value` key.
 */
type ScheduledTaskType =
  | {
      /**
       * Optional path to a workflow config file.
       */
      config_path?: string | null;
      /**
       * Optional monitor index to target.
       */
      monitor_index?: number | null;
      task_type: "Workflow";
      /**
       * If set, run a unified workflow by ID instead of a legacy workflow
       * by name.
       */
      workflow_id?: string | null;
      /**
       * Display name (also used to look up legacy workflows).
       */
      workflow_name: string;
      [k: string]: unknown;
    }
  | {
      /**
       * Optional override for `max_sessions`. `None` uses the prompt's own
       * setting.
       */
      max_sessions?: number | null;
      /**
       * ID of the prompt to run.
       */
      prompt_id: string;
      task_type: "Prompt";
      [k: string]: unknown;
    }
  | {
      /**
       * Whether to check the findings queue before running.
       */
      check_findings: boolean;
      /**
       * Force a run even if no findings are present.
       */
      force_run: boolean;
      task_type: "AutoFix";
      [k: string]: unknown;
    }
  | {
      task_type: "Watcher";
      /**
       * ID of the watcher definition in PostgreSQL.
       */
      watcher_id: string;
      [k: string]: unknown;
    }
  | {
      /**
       * Seconds between successive captures.
       */
      capture_interval_secs: number;
      /**
       * Whether to also trigger a capture on window focus change.
       */
      capture_on_focus_change: boolean;
      /**
       * Optional monitor index to capture.
       */
      monitor_index?: number | null;
      task_type: "BackgroundCapture";
      [k: string]: unknown;
    };

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Record of a single task execution.
 */
interface TaskExecutionRecord {
  /**
   * Session ID of the auto-fix session, if one was triggered.
   */
  autoFixSessionId?: string | null;
  /**
   * ISO 8601 timestamp when execution ended.
   */
  endedAt?: string | null;
  /**
   * Error message if the execution failed.
   */
  errorMessage?: string | null;
  /**
   * Unique ID for this execution (UUID v4 string).
   */
  executionId: string;
  /**
   * Session ID if this execution triggered an AI session, used for
   * downstream success tracking.
   */
  sessionId?: string | null;
  /**
   * ISO 8601 timestamp when execution started.
   */
  startedAt: string;
  status: ScheduledTaskStatus;
  /**
   * Whether the task succeeded, read from the session checkpoint.
   */
  success: boolean;
  /**
   * Whether auto-fix was triggered after this execution.
   */
  triggeredAutoFix: boolean;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * A scheduled task definition — the full persisted frame, including computed
 * fields (`last_run`, `next_run`) and condition-evaluation state
 * (`conditions`, `condition_status`).
 */
interface ScheduledTask {
  /**
   * Automatically trigger auto-fix when this task fails.
   */
  autoFixOnFailure: boolean;
  /**
   * Present while the task is waiting for its conditions to be met.
   */
  conditionStatus?: ConditionStatus | null;
  /**
   * Optional conditions that must be met before execution.
   */
  conditions?: ScheduleConditions | null;
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
  schedule: ScheduleExpression;
  /**
   * Skip future runs once the task has succeeded at least once.
   */
  skipIfCompleted: boolean;
  /**
   * Free-form description of success criteria, for human reference.
   */
  successCriteria?: string | null;
  task: ScheduledTaskType;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Global scheduler settings.
 */
interface SchedulerSettings {
  /**
   * Default `auto_fix_on_failure` value for newly created tasks.
   */
  defaultAutoFixOnFailure: boolean;
  /**
   * Whether the scheduler is enabled globally.
   */
  enabled: boolean;
  /**
   * Maximum number of scheduled tasks allowed to run concurrently.
   */
  maxConcurrent: number;
  /**
   * Timezone for schedule interpretation (IANA name). `None` = local time.
   */
  timezone?: string | null;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Minimal description of the next task due to run.
 */
interface NextTaskInfo {
  /**
   * Task ID.
   */
  id: string;
  /**
   * Task display name.
   */
  name: string;
  /**
   * ISO 8601 timestamp of the next scheduled run.
   */
  nextRun: string;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Summary of the scheduler's current runtime state, returned from the status
 * API.
 */
interface SchedulerStatus {
  /**
   * Whether the scheduler is enabled.
   */
  enabled: boolean;
  /**
   * The next task scheduled to run, if any.
   */
  nextTask?: NextTaskInfo | null;
  /**
   * Number of tasks scheduled but not yet running.
   */
  pendingTasks: number;
  /**
   * Number of tasks currently running.
   */
  runningTasks: number;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Request body for creating a new scheduled task.
 */
interface CreateScheduledTaskRequest {
  /**
   * Trigger auto-fix on failure.
   */
  autoFixOnFailure?: boolean | null;
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
  skipIfCompleted?: boolean | null;
  /**
   * Free-form success criteria description.
   */
  successCriteria?: string | null;
  task: ScheduledTaskType;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Request body for updating an existing scheduled task. All fields are
 * optional; only those supplied are applied.
 */
interface UpdateScheduledTaskRequest {
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

export type { ConditionScheduleConfig, ConditionStatus, CreateScheduledTaskRequest, IdleCondition, NextTaskInfo, RepositoryInactiveCondition, RepositoryWatch, ScheduleConditions, ScheduleExpression, ScheduledTask, ScheduledTaskStatus, ScheduledTaskType, SchedulerSettings, SchedulerStatus, TaskExecutionRecord, UpdateScheduledTaskRequest };
