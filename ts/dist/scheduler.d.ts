/**
 * Scheduler Types
 *
 * TypeScript interfaces for the CI/CD scheduler system.
 */
interface ScheduleOnce {
    type: "Once";
    value: string;
}
interface ScheduleCron {
    type: "Cron";
    value: string;
}
interface ScheduleInterval {
    type: "Interval";
    value: number;
}
interface ScheduleState {
    type: "State";
    /** The state machine state ID that triggers this task */
    state_id: string;
    /** Delay in seconds after entering the state before triggering */
    check_delay_seconds?: number;
    /** Minimum seconds between re-triggers if state is re-entered */
    rebuild_delay_seconds?: number;
}
type ScheduleExpression = ScheduleOnce | ScheduleCron | ScheduleInterval | ScheduleState;
interface IdleCondition {
    enabled: boolean;
}
interface RepositoryWatch {
    path: string;
    inactive_minutes: number;
}
interface RepositoryInactiveCondition {
    enabled: boolean;
    repositories: RepositoryWatch[];
}
interface ScheduleConditions {
    require_idle?: IdleCondition;
    require_repo_inactive?: RepositoryInactiveCondition;
    timeout_minutes?: number;
}
interface ConditionStatus {
    waiting_since: string;
    idle_met?: boolean;
    repo_inactive_met?: Array<[string, boolean]>;
    timed_out: boolean;
}
interface WorkflowTask {
    task_type: "Workflow";
    workflow_name: string;
    config_path?: string;
    monitor_index?: number;
    /** If set, run unified workflow by ID instead of legacy workflow by name */
    workflow_id?: string;
}
interface PromptTask {
    task_type: "Prompt";
    prompt_id: string;
    max_sessions?: number;
}
interface AutoFixTask {
    task_type: "AutoFix";
    check_findings: boolean;
    force_run: boolean;
}
type ScheduledTaskType = WorkflowTask | PromptTask | AutoFixTask;
type ScheduledTaskStatus = "pending" | "running" | "completed" | "failed" | "skipped" | "cancelled";
interface TaskExecutionRecord {
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
interface ScheduledTask {
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
interface SchedulerSettings {
    enabled: boolean;
    max_concurrent: number;
    default_auto_fix_on_failure: boolean;
    timezone?: string;
}
interface NextTaskInfo {
    id: string;
    name: string;
    next_run: string;
}
interface SchedulerStatus {
    enabled: boolean;
    running_tasks: number;
    pending_tasks: number;
    next_task?: NextTaskInfo;
}
interface CreateScheduledTaskRequest {
    name: string;
    description?: string;
    schedule: ScheduleExpression;
    task: ScheduledTaskType;
    skip_if_completed?: boolean;
    auto_fix_on_failure?: boolean;
    success_criteria?: string;
    conditions?: ScheduleConditions;
}
interface UpdateScheduledTaskRequest {
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

export type { AutoFixTask, ConditionStatus, CreateScheduledTaskRequest, IdleCondition, NextTaskInfo, PromptTask, RepositoryInactiveCondition, RepositoryWatch, ScheduleConditions, ScheduleCron, ScheduleExpression, ScheduleInterval, ScheduleOnce, ScheduleState, ScheduledTask, ScheduledTaskStatus, ScheduledTaskType, SchedulerSettings, SchedulerStatus, TaskExecutionRecord, UpdateScheduledTaskRequest, WorkflowTask };
