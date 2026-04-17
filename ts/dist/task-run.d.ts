/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Lifecycle status of a task run.
 */
type TaskRunStatus = "running" | "complete" | "failed" | "stopped";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Kind of task being tracked.
 */
type TaskType = "task" | "automation" | "scheduled";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * A task run as tracked by the local runner during execution.
 *
 * Mirrors `TaskRun` in the runner's `taskRun.ts`. Optional fields here use
 * `?` in TypeScript, so they are omitted on the wire when missing.
 */
interface TaskRun {
  /**
   * Whether the task will auto-continue into another session on exit.
   */
  auto_continue: boolean;
  /**
   * ISO 8601 timestamp when the task completed.
   */
  completed_at?: string | null;
  /**
   * ID of the workflow config used to run this task, if any.
   */
  config_id?: string | null;
  /**
   * ISO 8601 timestamp when the task was created.
   */
  created_at: string;
  /**
   * Error message if the task failed.
   */
  error_message?: string | null;
  /**
   * Whether the task's goal was achieved (AI assessment).
   */
  goal_achieved?: boolean | null;
  /**
   * Unique identifier (UUID v4 string).
   */
  id: string;
  /**
   * Optional cap on AI sessions.
   */
  max_sessions?: number | null;
  /**
   * Accumulated output log for the task run.
   */
  output_log: string;
  /**
   * Original prompt text, if applicable.
   */
  prompt?: string | null;
  /**
   * Description of any remaining work (AI assessment).
   */
  remaining_work?: string | null;
  /**
   * Number of AI sessions that have been run.
   */
  sessions_count: number;
  status: TaskRunStatus;
  /**
   * AI-generated summary of the task run.
   */
  summary?: string | null;
  /**
   * ISO 8601 timestamp when the summary was generated.
   */
  summary_generated_at?: string | null;
  /**
   * Display name of the task.
   */
  task_name: string;
  task_type: TaskType;
  /**
   * ISO 8601 timestamp when the task record was last updated.
   */
  updated_at: string;
  /**
   * Name of the workflow used to run this task, if any.
   */
  workflow_name?: string | null;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * A task run as returned by the backend API.
 *
 * Mirrors `TaskRunBackend` in the web app's `task-runs.ts`. Nullable
 * ownership fields (`project_id`, `created_by_user_id`, `runner_id`,
 * `max_sessions`, `output_summary`, `error_message`, `duration_seconds`,
 * `completed_at`) are required on the wire but may be `null`; they are
 * `Option<T>` with `serde(default)` so deserialize tolerates missing, but
 * are always serialized (including as `null`) to preserve the wire shape.
 */
interface TaskRunBackend {
  /**
   * Whether the task will auto-continue into another session on exit.
   */
  auto_continue: boolean;
  /**
   * ISO 8601 timestamp when the task completed.
   */
  completed_at: string | null;
  /**
   * ISO 8601 timestamp when the task was created.
   */
  created_at: string;
  /**
   * User who created the task run, if known.
   */
  created_by_user_id: string | null;
  /**
   * Total duration in seconds.
   */
  duration_seconds: number | null;
  /**
   * Error message if the task failed.
   */
  error_message: string | null;
  /**
   * Whether the full output log was persisted.
   */
  full_output_stored: boolean;
  /**
   * Unique identifier (UUID v4 string).
   */
  id: string;
  /**
   * Optional cap on AI sessions.
   */
  max_sessions: number | null;
  /**
   * Short summary of the run output, if stored.
   */
  output_summary: string | null;
  /**
   * Owning project ID, if scoped to a project.
   */
  project_id: string | null;
  /**
   * Original prompt text.
   */
  prompt: string;
  /**
   * Runner instance that executed the task, if known.
   */
  runner_id: string | null;
  /**
   * Number of AI sessions that have been run.
   */
  sessions_count: number;
  status: TaskRunStatus;
  /**
   * Display name.
   */
  task_name: string;
  /**
   * ISO 8601 timestamp when the record was last updated.
   */
  updated_at: string;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * A single AI session within a task run.
 */
interface TaskRunSession {
  /**
   * Duration of the session in seconds.
   */
  duration_seconds: number | null;
  /**
   * ISO 8601 timestamp when the session ended.
   */
  ended_at: string | null;
  /**
   * Unique identifier (UUID v4 string).
   */
  id: string;
  /**
   * Short summary of the session output, if stored.
   */
  output_summary: string | null;
  /**
   * 1-based session index within the parent task run.
   */
  session_number: number;
  /**
   * ISO 8601 timestamp when the session started.
   */
  started_at: string;
  /**
   * Parent task run ID.
   */
  task_run_id: string;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Category of a finding surfaced during a task run.
 */
type TaskRunFindingCategory =
  | "code_bug"
  | "security"
  | "performance"
  | "todo"
  | "enhancement"
  | "config_issue"
  | "test_issue"
  | "documentation"
  | "runtime_issue"
  | "already_fixed"
  | "expected_behavior"
  | "data_migration"
  | "warning";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Severity of a finding.
 */
type TaskRunFindingSeverity = "critical" | "high" | "medium" | "low" | "info";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Lifecycle status of a finding.
 */
type TaskRunFindingStatus = "detected" | "in_progress" | "needs_input" | "resolved" | "wont_fix" | "deferred";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * How a finding should be acted upon.
 */
type TaskRunFindingActionType = "auto_fix" | "needs_user_input" | "manual" | "informational";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * A finding surfaced during a task run (bug, enhancement, TODO, etc.).
 *
 * All nullable fields here are required-nullable on the wire (always present,
 * possibly `null`), so they use `serde(default)` without `skip_serializing_if`.
 */
interface TaskRunFinding {
  action_type: TaskRunFindingActionType;
  category: TaskRunFindingCategory;
  /**
   * Snippet of code illustrating the issue.
   */
  code_snippet: string | null;
  /**
   * Column number where the issue was found.
   */
  column_number: number | null;
  /**
   * Full description.
   */
  description: string;
  /**
   * ISO 8601 timestamp when the finding was detected.
   */
  detected_at: string;
  /**
   * Session number in which the finding was detected.
   */
  detected_in_session: number;
  /**
   * File path where the issue was found.
   */
  file_path: string | null;
  /**
   * Unique identifier (UUID v4 string).
   */
  id: string;
  /**
   * Suggested response options for the user, if input is needed.
   */
  input_options: string[] | null;
  /**
   * Line number where the issue was found.
   */
  line_number: number | null;
  /**
   * Whether this finding requires user input.
   */
  needs_input: boolean;
  /**
   * Question posed to the user, if input is needed.
   */
  question: string | null;
  /**
   * How the finding was resolved, if applicable.
   */
  resolution: string | null;
  /**
   * ISO 8601 timestamp when the finding was resolved.
   */
  resolved_at: string | null;
  /**
   * Session number in which the finding was resolved.
   */
  resolved_in_session: number | null;
  severity: TaskRunFindingSeverity;
  /**
   * Hash used to deduplicate findings across runs.
   */
  signature_hash: string | null;
  status: TaskRunFindingStatus;
  /**
   * Parent task run ID.
   */
  task_run_id: string;
  /**
   * Short human-readable title.
   */
  title: string;
  /**
   * ISO 8601 timestamp when the record was last updated.
   */
  updated_at: string;
  /**
   * The user's response, if any.
   */
  user_response: string | null;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Aggregated finding counts grouped along each axis.
 */
interface TaskRunFindingSummary {
  /**
   * Count of findings by category.
   */
  by_category: {
    [k: string]: number;
  };
  /**
   * Count of findings by severity.
   */
  by_severity: {
    [k: string]: number;
  };
  /**
   * Count of findings by status.
   */
  by_status: {
    [k: string]: number;
  };
  /**
   * Total number of findings.
   */
  total: number;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Detailed view of a backend task run, including its sessions and findings.
 *
 * The TypeScript `TaskRunBackendDetail extends TaskRunBackend` is modeled in
 * Rust by flattening a [`TaskRunBackend`] base so the wire shape stays flat.
 */
interface TaskRunBackendDetail {
  /**
   * Whether the task will auto-continue into another session on exit.
   */
  auto_continue: boolean;
  /**
   * ISO 8601 timestamp when the task completed.
   */
  completed_at: string | null;
  /**
   * ISO 8601 timestamp when the task was created.
   */
  created_at: string;
  /**
   * User who created the task run, if known.
   */
  created_by_user_id: string | null;
  /**
   * Total duration in seconds.
   */
  duration_seconds: number | null;
  /**
   * Error message if the task failed.
   */
  error_message: string | null;
  finding_summary: TaskRunFindingSummary;
  /**
   * Findings surfaced during this task run.
   */
  findings: TaskRunFinding[];
  /**
   * Whether the full output log was persisted.
   */
  full_output_stored: boolean;
  /**
   * Unique identifier (UUID v4 string).
   */
  id: string;
  /**
   * Optional cap on AI sessions.
   */
  max_sessions: number | null;
  /**
   * Short summary of the run output, if stored.
   */
  output_summary: string | null;
  /**
   * Owning project ID, if scoped to a project.
   */
  project_id: string | null;
  /**
   * Original prompt text.
   */
  prompt: string;
  /**
   * Runner instance that executed the task, if known.
   */
  runner_id: string | null;
  /**
   * AI sessions associated with this task run.
   */
  sessions: TaskRunSession[];
  /**
   * Number of AI sessions that have been run.
   */
  sessions_count: number;
  status: TaskRunStatus;
  /**
   * Display name.
   */
  task_name: string;
  /**
   * ISO 8601 timestamp when the record was last updated.
   */
  updated_at: string;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Request payload for creating a task run.
 */
interface TaskRunCreate {
  /**
   * Whether the task should auto-continue.
   */
  auto_continue?: boolean | null;
  /**
   * Workflow config ID to use.
   */
  config_id?: string | null;
  /**
   * Serialized execution steps, if provided ad-hoc.
   */
  execution_steps_json?: string | null;
  /**
   * Optional client-generated ID.
   */
  id?: string | null;
  /**
   * Serialized log-sources configuration.
   */
  log_sources_json?: string | null;
  /**
   * Optional cap on AI sessions.
   */
  max_sessions?: number | null;
  /**
   * Optional owning project.
   */
  project_id?: string | null;
  /**
   * Prompt text, if applicable.
   */
  prompt?: string | null;
  /**
   * Optional runner that will execute the task.
   */
  runner_id?: string | null;
  /**
   * Display name.
   */
  task_name: string;
  /**
   * Task type.
   */
  task_type?: TaskType | null;
  /**
   * Workflow name to use.
   */
  workflow_name?: string | null;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Request payload for updating an existing task run. All fields are optional;
 * only those supplied are applied.
 */
interface TaskRunUpdate {
  /**
   * ISO 8601 timestamp when the task completed.
   */
  completed_at?: string | null;
  /**
   * Total duration in seconds.
   */
  duration_seconds?: number | null;
  /**
   * Error message to attach.
   */
  error_message?: string | null;
  /**
   * Full output log to persist.
   */
  full_output?: string | null;
  /**
   * Whether the full output log has been stored.
   */
  full_output_stored?: boolean | null;
  /**
   * Updated output summary.
   */
  output_summary?: string | null;
  /**
   * Updated session count.
   */
  sessions_count?: number | null;
  /**
   * New lifecycle status.
   */
  status?: TaskRunStatus | null;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Request payload for creating a finding.
 */
interface TaskRunFindingCreate {
  /**
   * Action type (defaults server-side if omitted).
   */
  action_type?: TaskRunFindingActionType | null;
  category: TaskRunFindingCategory;
  /**
   * Snippet of code illustrating the issue.
   */
  code_snippet?: string | null;
  /**
   * Column number where the issue was found.
   */
  column_number?: number | null;
  /**
   * Full description.
   */
  description: string;
  /**
   * Session number in which the finding was detected.
   */
  detected_in_session: number;
  /**
   * File path where the issue was found.
   */
  file_path?: string | null;
  /**
   * Optional client-generated ID.
   */
  id?: string | null;
  /**
   * Suggested response options for the user.
   */
  input_options?: string[] | null;
  /**
   * Line number where the issue was found.
   */
  line_number?: number | null;
  /**
   * Whether this finding requires user input.
   */
  needs_input?: boolean | null;
  /**
   * Question to pose to the user, if input is needed.
   */
  question?: string | null;
  /**
   * Resolution text, if already resolved.
   */
  resolution?: string | null;
  severity: TaskRunFindingSeverity;
  /**
   * Deduplication hash.
   */
  signature_hash?: string | null;
  /**
   * Initial status (defaults server-side if omitted).
   */
  status?: TaskRunFindingStatus | null;
  /**
   * Short title.
   */
  title: string;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Request payload for updating a finding. All fields are optional.
 */
interface TaskRunFindingUpdate {
  /**
   * Resolution text.
   */
  resolution?: string | null;
  /**
   * ISO 8601 timestamp of resolution.
   */
  resolved_at?: string | null;
  /**
   * Session number in which the finding was resolved.
   */
  resolved_in_session?: number | null;
  /**
   * New lifecycle status.
   */
  status?: TaskRunFindingStatus | null;
  /**
   * User response, if the finding needed input.
   */
  user_response?: string | null;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Inline `data` payload on a [`RunPromptResponse`].
 */
interface RunPromptResponseData {
  /**
   * AI output text.
   */
  output?: string | null;
  /**
   * Final response text.
   */
  response?: string | null;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Response from the runner's `run_prompt` endpoint.
 */
interface RunPromptResponse {
  /**
   * Structured data payload from a synchronous prompt run.
   */
  data?: RunPromptResponseData | null;
  /**
   * Error message if the call failed.
   */
  error?: string | null;
  /**
   * Path to the log file for the session.
   */
  log_file?: string | null;
  /**
   * Immediate output if the call ran synchronously.
   */
  output?: string | null;
  /**
   * OS process ID of the spawned AI session, if any.
   */
  pid?: number | null;
  /**
   * ID of the created AI session, if any.
   */
  session_id?: string | null;
  /**
   * Path to the state file tracking the session.
   */
  state_file?: string | null;
  /**
   * Whether the prompt was accepted and started successfully.
   */
  success: boolean;
  /**
   * ID of the created task run, if any.
   */
  task_run_id?: string | null;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Request body for the runner's `run_prompt` endpoint.
 */
interface RunPromptRequest {
  /**
   * Prompt content (the actual text sent to the AI).
   */
  content: string;
  /**
   * Optional free-form context string appended to the prompt.
   */
  context?: string | null;
  /**
   * Display-only version of the prompt (shown in the UI).
   */
  display_prompt?: string | null;
  /**
   * Attached image paths.
   */
  image_paths?: string[] | null;
  /**
   * Optional cap on AI sessions.
   */
  max_sessions?: number | null;
  /**
   * Cap on trace screenshots to include.
   */
  max_trace_screenshots?: number | null;
  /**
   * Cap on video frames to extract for the prompt.
   */
  max_video_frames?: number | null;
  /**
   * Display name for the task.
   */
  name: string;
  /**
   * Hard timeout in seconds.
   */
  timeout_seconds?: number | null;
  /**
   * Optional path to a trace file.
   */
  trace_path?: string | null;
  /**
   * Attached video paths.
   */
  video_paths?: string[] | null;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Request body for creating a task run (simplified shape used by the runner's
 * create-task endpoint).
 */
interface CreateTaskRunRequest {
  /**
   * Whether the task should auto-continue.
   */
  auto_continue?: boolean | null;
  /**
   * Workflow config ID to use.
   */
  config_id?: string | null;
  /**
   * Serialized execution steps, if provided ad-hoc.
   */
  execution_steps_json?: string | null;
  /**
   * Serialized log-sources configuration.
   */
  log_sources_json?: string | null;
  /**
   * Optional cap on AI sessions.
   */
  max_sessions?: number | null;
  /**
   * Prompt text, if applicable.
   */
  prompt?: string | null;
  /**
   * Display name.
   */
  task_name: string;
  /**
   * Task type.
   */
  task_type?: TaskType | null;
  /**
   * Workflow name to use.
   */
  workflow_name?: string | null;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Filter parameters for listing task runs.
 */
interface TaskRunFilters {
  /**
   * Include only runs before this ISO 8601 timestamp.
   */
  end_date?: string | null;
  /**
   * Pagination limit.
   */
  limit?: number | null;
  /**
   * Pagination offset.
   */
  offset?: number | null;
  /**
   * Restrict to a given project.
   */
  project_id?: string | null;
  /**
   * Include only runs after this ISO 8601 timestamp.
   */
  start_date?: string | null;
  /**
   * Restrict to a given status.
   */
  status?: TaskRunStatus | null;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Filter parameters for listing findings.
 */
interface TaskRunFindingFilters {
  /**
   * Restrict to a given category.
   */
  category?: TaskRunFindingCategory | null;
  /**
   * Restrict to a given severity.
   */
  severity?: TaskRunFindingSeverity | null;
  /**
   * Restrict to a given status.
   */
  status?: TaskRunFindingStatus | null;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Pagination envelope attached to list responses.
 */
interface Pagination {
  /**
   * Whether additional records are available after this page.
   */
  has_more: boolean;
  /**
   * Maximum number of records returned per page.
   */
  limit: number;
  /**
   * Offset into the full result set.
   */
  offset: number;
  /**
   * Total number of matching records.
   */
  total: number;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Response for `GET /task-runs`.
 */
interface TaskRunListResponse {
  pagination: Pagination;
  /**
   * Page of matching task runs.
   */
  tasks: TaskRunBackend[];
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Response for `GET /task-runs/{id}/findings`.
 */
interface TaskRunFindingsListResponse {
  /**
   * Findings for the task run.
   */
  findings: TaskRunFinding[];
  summary: TaskRunFindingSummary;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Compact findings summary including the most recent findings.
 *
 * The TS type `TaskRunFindingResponse` is a type alias for `TaskRunFinding`;
 * in Rust we use `TaskRunFinding` directly.
 */
interface FindingsSummary {
  /**
   * Count by category.
   */
  by_category: {
    [k: string]: number;
  };
  /**
   * Count by severity.
   */
  by_severity: {
    [k: string]: number;
  };
  /**
   * Count by status.
   */
  by_status: {
    [k: string]: number;
  };
  /**
   * Most recent findings.
   */
  recent: TaskRunFinding[];
  /**
   * Total number of findings.
   */
  total: number;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * A specific issue detail from an individual verification check.
 */
interface CheckIssueDetail {
  /**
   * Error code or lint rule, if applicable.
   */
  code: string | null;
  /**
   * Column number, if applicable.
   */
  column: number | null;
  /**
   * File path where the issue was detected.
   */
  file: string;
  /**
   * Whether the check can auto-fix this issue.
   */
  fixable: boolean;
  /**
   * Line number, if applicable.
   */
  line: number | null;
  /**
   * Human-readable message.
   */
  message: string;
  /**
   * Severity label (free-form string from the upstream check).
   */
  severity: string;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Result of a single named verification check (e.g., `"eslint"`, `"mypy"`).
 */
interface IndividualCheckResult {
  /**
   * How long the check took, in milliseconds.
   */
  duration_ms: number;
  /**
   * Error message, if the check itself failed to run.
   */
  error_message: string | null;
  /**
   * Number of files the check inspected.
   */
  files_checked: number;
  /**
   * Specific issue details.
   */
  issues: CheckIssueDetail[];
  /**
   * Number of issues auto-fixed by this check.
   */
  issues_fixed: number;
  /**
   * Number of issues surfaced by this check.
   */
  issues_found: number;
  /**
   * Name of the check.
   */
  name: string;
  /**
   * Raw check output, if captured.
   */
  output: string | null;
  /**
   * Free-form status string (e.g., `"passed"`, `"failed"`, `"skipped"`).
   */
  status: string;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Detailed output captured for a single verification step.
 */
interface VerificationStepDetails {
  /**
   * Number of assertions that passed.
   */
  assertions_passed: number | null;
  /**
   * Total number of assertions.
   */
  assertions_total: number | null;
  /**
   * Results of individual named checks (e.g., lint, type, test).
   */
  check_results: IndividualCheckResult[] | null;
  /**
   * Captured browser/console output.
   */
  console_output: string | null;
  /**
   * Exit code of the spawned process.
   */
  exit_code: number | null;
  /**
   * Captured page snapshot (HTML or serialized representation).
   */
  page_snapshot: string | null;
  /**
   * Phase the step belongs to (e.g., `"setup"`, `"verification"`).
   */
  phase: string;
  /**
   * Captured stderr, if any.
   */
  stderr: string | null;
  /**
   * Captured stdout, if any.
   */
  stdout: string | null;
  /**
   * ID of the step this detail belongs to.
   */
  step_id: string;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Execution config captured for a verification step.
 *
 * The TypeScript type includes an index signature `[key: string]: unknown`,
 * so extra arbitrary fields are captured in `extra` via `serde(flatten)` and
 * passed through opaquely.
 */
interface StepExecutionConfig {
  /**
   * Action type (e.g., click, type, wait).
   */
  action_type?: string | null;
  /**
   * Check type for verification steps.
   */
  check_type?: string | null;
  /**
   * Target image ID, if the action references an image.
   */
  target_image_id?: string | null;
  /**
   * Target image name, if the action references an image.
   */
  target_image_name?: string | null;
  /**
   * Timeout in seconds.
   */
  timeout_seconds?: number | null;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Result of a single step within a verification phase.
 */
interface VerificationStepResult {
  config: StepExecutionConfig;
  /**
   * Step duration in milliseconds.
   */
  duration_ms: number;
  /**
   * ISO 8601 timestamp when the step ended.
   */
  ended_at: string | null;
  /**
   * Error message if the step failed.
   */
  error: string | null;
  /**
   * Arbitrary structured output produced by the step.
   */
  output_data: {
    [k: string]: unknown;
  } | null;
  /**
   * Path to a screenshot captured for the step.
   */
  screenshot_path: string | null;
  /**
   * ISO 8601 timestamp when the step started.
   */
  started_at: string | null;
  /**
   * ID of the step, if assigned.
   */
  step_id: string | null;
  /**
   * Zero-based index of the step within the phase.
   */
  step_index: number;
  /**
   * Display name of the step.
   */
  step_name: string;
  /**
   * Free-form step type label.
   */
  step_type: string;
  /**
   * Whether the step succeeded.
   */
  success: boolean;
  /**
   * Detailed captured output, if any.
   */
  verification_details: VerificationStepDetails | null;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Result of evaluating a named gate across a set of steps.
 */
interface GateEvaluationResult {
  /**
   * IDs of required steps that failed.
   */
  failed_step_ids: string[];
  /**
   * Name of the gate.
   */
  gate_name: string;
  /**
   * IDs of required steps that were missing.
   */
  missing_step_ids: string[];
  /**
   * Whether the gate passed overall.
   */
  passed: boolean;
  /**
   * IDs of required steps that passed.
   */
  passed_step_ids: string[];
  /**
   * IDs of steps the gate required.
   */
  required_step_ids: string[];
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Result of a single iteration of the verification phase.
 */
interface VerificationPhaseResult {
  /**
   * Whether all steps passed.
   */
  all_passed: boolean;
  /**
   * Whether a critical step failure short-circuited the phase.
   */
  critical_failure: boolean;
  /**
   * Number of steps that failed.
   */
  failed_steps: number;
  /**
   * Whether pass/fail is determined by gates rather than overall step counts.
   */
  gate_based_evaluation: boolean;
  /**
   * Per-gate evaluation results.
   */
  gate_results: GateEvaluationResult[];
  /**
   * 1-based iteration index within the workflow run.
   */
  iteration: number;
  /**
   * Number of steps that passed.
   */
  passed_steps: number;
  /**
   * Number of steps that were skipped.
   */
  skipped_steps: number;
  /**
   * Per-step results.
   */
  step_results: VerificationStepResult[];
  /**
   * Total duration of the phase in milliseconds.
   */
  total_duration_ms: number;
  /**
   * Total number of steps executed.
   */
  total_steps: number;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Response record for a single stored verification result.
 */
interface VerificationResultResponse {
  /**
   * Whether all steps passed.
   */
  all_passed: boolean;
  /**
   * ISO 8601 timestamp when the record was created.
   */
  created_at: string;
  /**
   * Whether a critical step failure short-circuited the phase.
   */
  critical_failure: boolean;
  /**
   * Number of steps that failed.
   */
  failed_steps: number;
  /**
   * Unique identifier (UUID v4 string).
   */
  id: string;
  /**
   * 1-based iteration index within the workflow run.
   */
  iteration: number;
  /**
   * Number of steps that passed.
   */
  passed_steps: number;
  result_json: VerificationPhaseResult;
  /**
   * Number of steps that were skipped.
   */
  skipped_steps: number;
  /**
   * ID of the owning task run.
   */
  task_run_id: string;
  /**
   * Total duration of the phase in milliseconds.
   */
  total_duration_ms: number;
  /**
   * Total number of steps executed.
   */
  total_steps: number;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Response for listing verification results for a task run.
 */
interface VerificationResultsListResponse {
  /**
   * Total number of verification results.
   */
  count: number;
  /**
   * Number of iterations that failed.
   */
  failed_iterations: number;
  /**
   * Number of iterations that passed.
   */
  passed_iterations: number;
  /**
   * All verification results for the task run.
   */
  results: VerificationResultResponse[];
  /**
   * ID of the owning task run.
   */
  task_run_id: string;
  [k: string]: unknown;
}

/**
 * Task Run Types
 *
 * Merged type definitions from both runner (taskRun.ts) and web (task-runs.ts).
 * Runner types are used for local execution, web types for backend API communication.
 *
 * Types generated from Rust (source of truth: qontinui-schemas/rust/src/task_run.rs).
 * Do not edit by hand — regenerate via `just generate-types` (or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`).
 */

type TaskRunFindingResponse = TaskRunFinding;

export type { CheckIssueDetail, CreateTaskRunRequest, FindingsSummary, GateEvaluationResult, IndividualCheckResult, Pagination, RunPromptRequest, RunPromptResponse, RunPromptResponseData, StepExecutionConfig, TaskRun, TaskRunBackend, TaskRunBackendDetail, TaskRunCreate, TaskRunFilters, TaskRunFinding, TaskRunFindingActionType, TaskRunFindingCategory, TaskRunFindingCreate, TaskRunFindingFilters, TaskRunFindingResponse, TaskRunFindingSeverity, TaskRunFindingStatus, TaskRunFindingSummary, TaskRunFindingUpdate, TaskRunFindingsListResponse, TaskRunListResponse, TaskRunSession, TaskRunStatus, TaskRunUpdate, TaskType, VerificationPhaseResult, VerificationResultResponse, VerificationResultsListResponse, VerificationStepDetails, VerificationStepResult };
