/**
 * Scheduler Types
 *
 * TypeScript interfaces for the CI/CD scheduler system.
 */

// ============================================================================
// Schedule Expression Types
// ============================================================================

export interface ScheduleOnce {
  type: "Once";
  value: string;
}

export interface ScheduleCron {
  type: "Cron";
  value: string;
}

export interface ScheduleInterval {
  type: "Interval";
  value: number;
}

export type ScheduleExpression = ScheduleOnce | ScheduleCron | ScheduleInterval;

// ============================================================================
// Schedule Conditions
// ============================================================================

export interface IdleCondition {
  enabled: boolean;
}

export interface RepositoryWatch {
  path: string;
  inactive_minutes: number;
}

export interface RepositoryInactiveCondition {
  enabled: boolean;
  repositories: RepositoryWatch[];
}

export interface ScheduleConditions {
  require_idle?: IdleCondition;
  require_repo_inactive?: RepositoryInactiveCondition;
  timeout_minutes?: number;
}

export interface ConditionStatus {
  waiting_since: string;
  idle_met?: boolean;
  repo_inactive_met?: Array<[string, boolean]>;
  timed_out: boolean;
}

// ============================================================================
// Task Type Definitions
// ============================================================================

export interface WorkflowTask {
  task_type: "Workflow";
  workflow_name: string;
  config_path?: string;
  monitor_index?: number;
}

export interface PromptTask {
  task_type: "Prompt";
  prompt_id: string;
  max_sessions?: number;
}

export interface AutoFixTask {
  task_type: "AutoFix";
  check_findings: boolean;
  force_run: boolean;
}

export type ScheduledTaskType = WorkflowTask | PromptTask | AutoFixTask;

// ============================================================================
// Task Status
// ============================================================================

export type ScheduledTaskStatus =
  | "pending"
  | "running"
  | "completed"
  | "failed"
  | "skipped"
  | "cancelled";

// ============================================================================
// Execution Record
// ============================================================================

export interface TaskExecutionRecord {
  execution_id: string;
  session_id?: string;
  started_at: string;
  ended_at?: string;
  status: ScheduledTaskStatus;
  success: boolean;
  error_message?: string;
  triggered_auto_fix: boolean;
  auto_fix_session_id?: string;
}

// ============================================================================
// Scheduled Task
// ============================================================================

export interface ScheduledTask {
  id: string;
  name: string;
  description?: string;
  enabled: boolean;
  schedule: ScheduleExpression;
  task: ScheduledTaskType;
  skip_if_completed: boolean;
  auto_fix_on_failure: boolean;
  success_criteria?: string;
  created_at: string;
  modified_at: string;
  last_run?: TaskExecutionRecord;
  next_run?: string;
  conditions?: ScheduleConditions;
  condition_status?: ConditionStatus;
}

// ============================================================================
// Scheduler Settings
// ============================================================================

export interface SchedulerSettings {
  enabled: boolean;
  max_concurrent: number;
  default_auto_fix_on_failure: boolean;
  timezone?: string;
}

// ============================================================================
// Scheduler Status
// ============================================================================

export interface NextTaskInfo {
  id: string;
  name: string;
  next_run: string;
}

export interface SchedulerStatus {
  enabled: boolean;
  running_tasks: number;
  pending_tasks: number;
  next_task?: NextTaskInfo;
}

// ============================================================================
// API Request/Response Types
// ============================================================================

export interface CreateScheduledTaskRequest {
  name: string;
  description?: string;
  schedule: ScheduleExpression;
  task: ScheduledTaskType;
  skip_if_completed?: boolean;
  auto_fix_on_failure?: boolean;
  success_criteria?: string;
  conditions?: ScheduleConditions;
}

export interface UpdateScheduledTaskRequest {
  name?: string;
  description?: string | null;
  enabled?: boolean;
  schedule?: ScheduleExpression;
  task?: ScheduledTaskType;
  skip_if_completed?: boolean;
  auto_fix_on_failure?: boolean;
  success_criteria?: string | null;
  conditions?: ScheduleConditions | null;
}
