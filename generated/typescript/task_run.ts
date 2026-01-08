/**
 * Auto-generated TypeScript types from qontinui-schemas
 * DO NOT EDIT - regenerate with: poetry run python scripts/generate_typescript.py
 */

export enum TaskType {
  TASK = "task",
  AUTOMATION = "automation",
  SCHEDULED = "scheduled",
}

export enum TaskRunStatus {
  RUNNING = "running",
  COMPLETE = "complete",
  FAILED = "failed",
  STOPPED = "stopped",
}

export enum AutomationStatus {
  RUNNING = "running",
  SUCCESS = "success",
  FAILED = "failed",
  TIMEOUT = "timeout",
  CANCELLED = "cancelled",
}

export interface TaskRunCreate {
  /** Human-readable name for this task */
  task_name: string;
  /** Task prompt (NULL for pure automation) */
  prompt?: string | null;
  /** Type of task: 'task', 'automation', or 'scheduled' */
  task_type?: TaskType;
  /** Whether to auto-continue on session completion */
  auto_continue?: boolean;
  /** Maximum number of AI sessions allowed */
  max_sessions?: number | null;
  /** JSON-encoded execution steps to run before each AI session */
  execution_steps_json?: string | null;
  /** JSON-encoded log sources to capture during execution */
  log_sources_json?: string | null;
  /** Config ID for automation-enabled tasks */
  config_id?: string | null;
  /** Workflow name being executed */
  workflow_name?: string | null;
}

export interface TaskRunResponse {
  /** Unique task run identifier */
  id: string;
  /** Task name */
  task_name: string;
  /** Type of task */
  task_type: TaskType;
  /** Current status */
  status: TaskRunStatus;
  /** Number of AI sessions run */
  sessions_count?: number;
  /** Maximum sessions allowed */
  max_sessions?: number | null;
  /** Auto-continue enabled */
  auto_continue?: boolean;
  /** Config ID */
  config_id?: string | null;
  /** Workflow name */
  workflow_name?: string | null;
  /** When the task was created (UTC) */
  created_at: string;
  /** When the task completed (UTC) */
  completed_at?: any | null;
}

export interface TaskRunDetail {
  /** Unique task run identifier */
  id: string;
  /** Task name */
  task_name: string;
  /** Type of task */
  task_type: TaskType;
  /** Current status */
  status: TaskRunStatus;
  /** Number of AI sessions run */
  sessions_count?: number;
  /** Maximum sessions allowed */
  max_sessions?: number | null;
  /** Auto-continue enabled */
  auto_continue?: boolean;
  /** Config ID */
  config_id?: string | null;
  /** Workflow name */
  workflow_name?: string | null;
  /** When the task was created (UTC) */
  created_at: string;
  /** When the task completed (UTC) */
  completed_at?: any | null;
  /** Task prompt */
  prompt?: string | null;
  /** Full task output log */
  output_log?: string;
  /** Error message if failed */
  error_message?: string | null;
  /** Execution steps JSON */
  execution_steps_json?: string | null;
  /** Log sources JSON */
  log_sources_json?: string | null;
  /** AI-generated paragraph summary of the task run */
  summary?: string | null;
  /** Whether the stated goal was achieved */
  goal_achieved?: boolean | null;
  /** What remains to be done if goal not achieved */
  remaining_work?: string | null;
  /** When the summary was generated */
  summary_generated_at?: any | null;
  /** Last update time (UTC) */
  updated_at: string;
}

export interface TaskRunUpdate {
  /** New status */
  status?: TaskRunStatus | null;
  /** Output to append */
  output_log?: string | null;
  /** Error message */
  error_message?: string | null;
  /** Update session count */
  sessions_count?: number | null;
  /** AI summary */
  summary?: string | null;
  /** Goal achieved flag */
  goal_achieved?: boolean | null;
  /** Remaining work description */
  remaining_work?: string | null;
}

export interface TaskRunComplete {
  /** Final status (complete, failed, stopped) */
  status: TaskRunStatus;
  /** Error message if failed */
  error_message?: string | null;
  /** Optional execution summary */
  summary?: string | null;
  /** Whether goal was achieved */
  goal_achieved?: boolean | null;
  /** What remains if goal not achieved */
  remaining_work?: string | null;
}

export interface TaskRunReopen {
  /** Number of additional sessions to add */
  additional_sessions: number;
}

export interface TaskRunListResponse {
  /** List of task runs */
  runs: TaskRunResponse[];
  /** Total number of runs */
  total: number;
  /** Items per page */
  limit: number;
  /** Items skipped */
  offset: number;
  /** Whether more items exist */
  has_more: boolean;
}

export interface TaskRunAutomationCreate {
  /** Parent task run ID */
  task_run_id: string;
  /** Workflow being executed */
  workflow_name?: string | null;
  /** Iteration number within the task */
  iteration_number?: number;
}

export interface TaskRunAutomationResponse {
  /** Unique automation record ID */
  id: string;
  /** Parent task run ID */
  task_run_id: string;
  /** Workflow name */
  workflow_name?: string | null;
  /** Automation status */
  automation_status: AutomationStatus;
  /** Iteration number */
  iteration_number?: number;
  /** When automation started (UTC) */
  started_at: string;
  /** When automation ended (UTC) */
  ended_at?: any | null;
  /** Duration in milliseconds */
  duration_ms?: number | null;
}

export interface TaskRunAutomationDetail {
  /** Unique automation record ID */
  id: string;
  /** Parent task run ID */
  task_run_id: string;
  /** Workflow name */
  workflow_name?: string | null;
  /** Automation status */
  automation_status: AutomationStatus;
  /** Iteration number */
  iteration_number?: number;
  /** When automation started (UTC) */
  started_at: string;
  /** When automation ended (UTC) */
  ended_at?: any | null;
  /** Duration in milliseconds */
  duration_ms?: number | null;
  /** Whether automation succeeded */
  success?: boolean | null;
  /** Type of error if failed */
  error_type?: string | null;
  /** Error message if failed */
  error_message?: string | null;
  /** Summary of actions executed */
  actions_summary?: Record<string, any> | null;
  /** States visited */
  states_visited?: string[] | null;
  /** Transitions executed */
  transitions_executed?: Record<string, any>[] | null;
  /** Template matching results */
  template_matches?: Record<string, any>[] | null;
  /** Anomalies detected */
  anomalies?: Record<string, any>[] | null;
}

export interface TaskRunAutomationComplete {
  /** Final status */
  automation_status: AutomationStatus;
  /** Whether automation succeeded */
  success: boolean;
  /** Error type if failed */
  error_type?: string | null;
  /** Error message if failed */
  error_message?: string | null;
  /** Actions summary */
  actions_summary?: Record<string, any> | null;
  /** States visited */
  states_visited?: string[] | null;
  /** Transitions executed */
  transitions_executed?: Record<string, any>[] | null;
  /** Template matches */
  template_matches?: Record<string, any>[] | null;
  /** Anomalies */
  anomalies?: Record<string, any>[] | null;
}

export interface TaskRunAutomationListResponse {
  /** List of automation executions */
  automations: TaskRunAutomationResponse[];
  /** Total count */
  total: number;
}

export interface TaskRunSyncPayload {
  /** Task run details */
  task_run: TaskRunDetail;
  /** Automation execution records */
  automations?: TaskRunAutomationDetail[] | null;
  /** Code/automation findings */
  findings?: Record<string, any>[] | null;
  /** Discoveries from automation */
  discoveries?: Record<string, any>[] | null;
}
