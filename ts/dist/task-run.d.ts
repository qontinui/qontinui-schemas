import { T as TaskRunFindingActionType, a as TaskRunFindingCategory, b as TaskRunFindingSeverity, c as TaskRunFindingStatus } from './TaskRunFindingActionType.d-CNIWogcU.js';

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
  autoContinue: boolean;
  /**
   * ISO 8601 timestamp when the task completed.
   */
  completedAt?: string | null;
  /**
   * ID of the workflow config used to run this task, if any.
   */
  configId?: string | null;
  /**
   * ISO 8601 timestamp when the task was created.
   */
  createdAt: string;
  /**
   * Error message if the task failed.
   */
  errorMessage?: string | null;
  /**
   * Whether the task's goal was achieved (AI assessment).
   */
  goalAchieved?: boolean | null;
  /**
   * Unique identifier (UUID v4 string).
   */
  id: string;
  /**
   * Optional cap on AI sessions.
   */
  maxSessions?: number | null;
  /**
   * Accumulated output log for the task run.
   */
  outputLog: string;
  /**
   * Original prompt text, if applicable.
   */
  prompt?: string | null;
  /**
   * Description of any remaining work (AI assessment).
   */
  remainingWork?: string | null;
  /**
   * Number of AI sessions that have been run.
   */
  sessionsCount: number;
  status: TaskRunStatus;
  /**
   * AI-generated summary of the task run.
   */
  summary?: string | null;
  /**
   * ISO 8601 timestamp when the summary was generated.
   */
  summaryGeneratedAt?: string | null;
  /**
   * Display name of the task.
   */
  taskName: string;
  taskType: TaskType;
  /**
   * ISO 8601 timestamp when the task record was last updated.
   */
  updatedAt: string;
  /**
   * Name of the workflow used to run this task, if any.
   */
  workflowName?: string | null;
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
  autoContinue: boolean;
  /**
   * ISO 8601 timestamp when the task completed.
   */
  completedAt: string | null;
  /**
   * ISO 8601 timestamp when the task was created.
   */
  createdAt: string;
  /**
   * User who created the task run, if known.
   */
  createdByUserId: string | null;
  /**
   * Total duration in seconds.
   */
  durationSeconds: number | null;
  /**
   * Error message if the task failed.
   */
  errorMessage: string | null;
  /**
   * Whether the full output log was persisted.
   */
  fullOutputStored: boolean;
  /**
   * Unique identifier (UUID v4 string).
   */
  id: string;
  /**
   * Optional cap on AI sessions.
   */
  maxSessions: number | null;
  /**
   * Short summary of the run output, if stored.
   */
  outputSummary: string | null;
  /**
   * Owning project ID, if scoped to a project.
   */
  projectId: string | null;
  /**
   * Original prompt text.
   */
  prompt: string;
  /**
   * Runner instance that executed the task, if known.
   */
  runnerId: string | null;
  /**
   * Number of AI sessions that have been run.
   */
  sessionsCount: number;
  status: TaskRunStatus;
  /**
   * Display name.
   */
  taskName: string;
  /**
   * ISO 8601 timestamp when the record was last updated.
   */
  updatedAt: string;
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
  durationSeconds: number | null;
  /**
   * ISO 8601 timestamp when the session ended.
   */
  endedAt: string | null;
  /**
   * Unique identifier (UUID v4 string).
   */
  id: string;
  /**
   * Short summary of the session output, if stored.
   */
  outputSummary: string | null;
  /**
   * 1-based session index within the parent task run.
   */
  sessionNumber: number;
  /**
   * ISO 8601 timestamp when the session started.
   */
  startedAt: string;
  /**
   * Parent task run ID.
   */
  taskRunId: string;
}

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
  actionType: TaskRunFindingActionType;
  category: TaskRunFindingCategory;
  /**
   * Snippet of code illustrating the issue.
   */
  codeSnippet: string | null;
  /**
   * Column number where the issue was found.
   */
  columnNumber: number | null;
  /**
   * Full description.
   */
  description: string;
  /**
   * ISO 8601 timestamp when the finding was detected.
   */
  detectedAt: string;
  /**
   * Session number in which the finding was detected.
   */
  detectedInSession: number;
  /**
   * File path where the issue was found.
   */
  filePath: string | null;
  /**
   * Unique identifier (UUID v4 string).
   */
  id: string;
  /**
   * Suggested response options for the user, if input is needed.
   */
  inputOptions: string[] | null;
  /**
   * Line number where the issue was found.
   */
  lineNumber: number | null;
  /**
   * Whether this finding requires user input.
   */
  needsInput: boolean;
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
  resolvedAt: string | null;
  /**
   * Session number in which the finding was resolved.
   */
  resolvedInSession: number | null;
  severity: TaskRunFindingSeverity;
  /**
   * Hash used to deduplicate findings across runs.
   */
  signatureHash: string | null;
  status: TaskRunFindingStatus;
  /**
   * Parent task run ID.
   */
  taskRunId: string;
  /**
   * Short human-readable title.
   */
  title: string;
  /**
   * ISO 8601 timestamp when the record was last updated.
   */
  updatedAt: string;
  /**
   * The user's response, if any.
   */
  userResponse: string | null;
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
  byCategory: {
    [k: string]: number;
  };
  /**
   * Count of findings by severity.
   */
  bySeverity: {
    [k: string]: number;
  };
  /**
   * Count of findings by status.
   */
  byStatus: {
    [k: string]: number;
  };
  /**
   * Total number of findings.
   */
  total: number;
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
  autoContinue: boolean;
  /**
   * ISO 8601 timestamp when the task completed.
   */
  completedAt: string | null;
  /**
   * ISO 8601 timestamp when the task was created.
   */
  createdAt: string;
  /**
   * User who created the task run, if known.
   */
  createdByUserId: string | null;
  /**
   * Total duration in seconds.
   */
  durationSeconds: number | null;
  /**
   * Error message if the task failed.
   */
  errorMessage: string | null;
  findingSummary: TaskRunFindingSummary;
  /**
   * Findings surfaced during this task run.
   */
  findings: TaskRunFinding[];
  /**
   * Whether the full output log was persisted.
   */
  fullOutputStored: boolean;
  /**
   * Unique identifier (UUID v4 string).
   */
  id: string;
  /**
   * Optional cap on AI sessions.
   */
  maxSessions: number | null;
  /**
   * Short summary of the run output, if stored.
   */
  outputSummary: string | null;
  /**
   * Owning project ID, if scoped to a project.
   */
  projectId: string | null;
  /**
   * Original prompt text.
   */
  prompt: string;
  /**
   * Runner instance that executed the task, if known.
   */
  runnerId: string | null;
  /**
   * AI sessions associated with this task run.
   */
  sessions: TaskRunSession[];
  /**
   * Number of AI sessions that have been run.
   */
  sessionsCount: number;
  status: TaskRunStatus;
  /**
   * Display name.
   */
  taskName: string;
  /**
   * ISO 8601 timestamp when the record was last updated.
   */
  updatedAt: string;
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
  autoContinue?: boolean | null;
  /**
   * Workflow config ID to use.
   */
  configId?: string | null;
  /**
   * Serialized execution steps, if provided ad-hoc.
   */
  executionStepsJson?: string | null;
  /**
   * Optional client-generated ID.
   */
  id?: string | null;
  /**
   * Serialized log-sources configuration.
   */
  logSourcesJson?: string | null;
  /**
   * Optional cap on AI sessions.
   */
  maxSessions?: number | null;
  /**
   * Optional owning project.
   */
  projectId?: string | null;
  /**
   * Prompt text, if applicable.
   */
  prompt?: string | null;
  /**
   * Optional runner that will execute the task.
   */
  runnerId?: string | null;
  /**
   * Display name.
   */
  taskName: string;
  /**
   * Task type.
   */
  taskType?: TaskType | null;
  /**
   * Workflow name to use.
   */
  workflowName?: string | null;
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
  completedAt?: string | null;
  /**
   * Total duration in seconds.
   */
  durationSeconds?: number | null;
  /**
   * Error message to attach.
   */
  errorMessage?: string | null;
  /**
   * Full output log to persist.
   */
  fullOutput?: string | null;
  /**
   * Whether the full output log has been stored.
   */
  fullOutputStored?: boolean | null;
  /**
   * Updated output summary.
   */
  outputSummary?: string | null;
  /**
   * Updated session count.
   */
  sessionsCount?: number | null;
  /**
   * New lifecycle status.
   */
  status?: TaskRunStatus | null;
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
  actionType?: TaskRunFindingActionType | null;
  category: TaskRunFindingCategory;
  /**
   * Snippet of code illustrating the issue.
   */
  codeSnippet?: string | null;
  /**
   * Column number where the issue was found.
   */
  columnNumber?: number | null;
  /**
   * Full description.
   */
  description: string;
  /**
   * Session number in which the finding was detected.
   */
  detectedInSession: number;
  /**
   * File path where the issue was found.
   */
  filePath?: string | null;
  /**
   * Optional client-generated ID.
   */
  id?: string | null;
  /**
   * Suggested response options for the user.
   */
  inputOptions?: string[] | null;
  /**
   * Line number where the issue was found.
   */
  lineNumber?: number | null;
  /**
   * Whether this finding requires user input.
   */
  needsInput?: boolean | null;
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
  signatureHash?: string | null;
  /**
   * Initial status (defaults server-side if omitted).
   */
  status?: TaskRunFindingStatus | null;
  /**
   * Short title.
   */
  title: string;
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
  resolvedAt?: string | null;
  /**
   * Session number in which the finding was resolved.
   */
  resolvedInSession?: number | null;
  /**
   * New lifecycle status.
   */
  status?: TaskRunFindingStatus | null;
  /**
   * User response, if the finding needed input.
   */
  userResponse?: string | null;
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
  logFile?: string | null;
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
  sessionId?: string | null;
  /**
   * Path to the state file tracking the session.
   */
  stateFile?: string | null;
  /**
   * Whether the prompt was accepted and started successfully.
   */
  success: boolean;
  /**
   * ID of the created task run, if any.
   */
  taskRunId?: string | null;
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
  displayPrompt?: string | null;
  /**
   * Attached image paths.
   */
  imagePaths?: string[] | null;
  /**
   * Optional cap on AI sessions.
   */
  maxSessions?: number | null;
  /**
   * Cap on trace screenshots to include.
   */
  maxTraceScreenshots?: number | null;
  /**
   * Cap on video frames to extract for the prompt.
   */
  maxVideoFrames?: number | null;
  /**
   * Display name for the task.
   */
  name: string;
  /**
   * Hard timeout in seconds.
   */
  timeoutSeconds?: number | null;
  /**
   * Optional path to a trace file.
   */
  tracePath?: string | null;
  /**
   * Attached video paths.
   */
  videoPaths?: string[] | null;
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
  autoContinue?: boolean | null;
  /**
   * Workflow config ID to use.
   */
  configId?: string | null;
  /**
   * Serialized execution steps, if provided ad-hoc.
   */
  executionStepsJson?: string | null;
  /**
   * Serialized log-sources configuration.
   */
  logSourcesJson?: string | null;
  /**
   * Optional cap on AI sessions.
   */
  maxSessions?: number | null;
  /**
   * Prompt text, if applicable.
   */
  prompt?: string | null;
  /**
   * Display name.
   */
  taskName: string;
  /**
   * Task type.
   */
  taskType?: TaskType | null;
  /**
   * Workflow name to use.
   */
  workflowName?: string | null;
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
  endDate?: string | null;
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
  projectId?: string | null;
  /**
   * Include only runs after this ISO 8601 timestamp.
   */
  startDate?: string | null;
  /**
   * Restrict to a given status.
   */
  status?: TaskRunStatus | null;
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
  hasMore: boolean;
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
  byCategory: {
    [k: string]: number;
  };
  /**
   * Count by severity.
   */
  bySeverity: {
    [k: string]: number;
  };
  /**
   * Count by status.
   */
  byStatus: {
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
  durationMs: number;
  /**
   * Error message, if the check itself failed to run.
   */
  errorMessage: string | null;
  /**
   * Number of files the check inspected.
   */
  filesChecked: number;
  /**
   * Specific issue details.
   */
  issues: CheckIssueDetail[];
  /**
   * Number of issues auto-fixed by this check.
   */
  issuesFixed: number;
  /**
   * Number of issues surfaced by this check.
   */
  issuesFound: number;
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
  assertionsPassed: number | null;
  /**
   * Total number of assertions.
   */
  assertionsTotal: number | null;
  /**
   * Results of individual named checks (e.g., lint, type, test).
   */
  checkResults: IndividualCheckResult[] | null;
  /**
   * Captured browser/console output.
   */
  consoleOutput: string | null;
  /**
   * Exit code of the spawned process.
   */
  exitCode: number | null;
  /**
   * Captured page snapshot (HTML or serialized representation).
   */
  pageSnapshot: string | null;
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
  stepId: string;
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
  actionType?: string | null;
  /**
   * Check type for verification steps.
   */
  checkType?: string | null;
  /**
   * Target image ID, if the action references an image.
   */
  targetImageId?: string | null;
  /**
   * Target image name, if the action references an image.
   */
  targetImageName?: string | null;
  /**
   * Timeout in seconds.
   */
  timeoutSeconds?: number | null;
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
  durationMs: number;
  /**
   * ISO 8601 timestamp when the step ended.
   */
  endedAt: string | null;
  /**
   * Error message if the step failed.
   */
  error: string | null;
  /**
   * Arbitrary structured output produced by the step.
   */
  outputData: {
    [k: string]: unknown;
  } | null;
  /**
   * Path to a screenshot captured for the step.
   */
  screenshotPath: string | null;
  /**
   * ISO 8601 timestamp when the step started.
   */
  startedAt: string | null;
  /**
   * ID of the step, if assigned.
   */
  stepId: string | null;
  /**
   * Zero-based index of the step within the phase.
   */
  stepIndex: number;
  /**
   * Display name of the step.
   */
  stepName: string;
  /**
   * Free-form step type label.
   */
  stepType: string;
  /**
   * Whether the step succeeded.
   */
  success: boolean;
  /**
   * Detailed captured output, if any.
   */
  verificationDetails: VerificationStepDetails | null;
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
  failedStepIds: string[];
  /**
   * Name of the gate.
   */
  gateName: string;
  /**
   * IDs of required steps that were missing.
   */
  missingStepIds: string[];
  /**
   * Whether the gate passed overall.
   */
  passed: boolean;
  /**
   * IDs of required steps that passed.
   */
  passedStepIds: string[];
  /**
   * IDs of steps the gate required.
   */
  requiredStepIds: string[];
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
  allPassed: boolean;
  /**
   * Whether a critical step failure short-circuited the phase.
   */
  criticalFailure: boolean;
  /**
   * Number of steps that failed.
   */
  failedSteps: number;
  /**
   * Whether pass/fail is determined by gates rather than overall step counts.
   */
  gateBasedEvaluation: boolean;
  /**
   * Per-gate evaluation results.
   */
  gateResults: GateEvaluationResult[];
  /**
   * 1-based iteration index within the workflow run.
   */
  iteration: number;
  /**
   * Number of steps that passed.
   */
  passedSteps: number;
  /**
   * Number of steps that were skipped.
   */
  skippedSteps: number;
  /**
   * Per-step results.
   */
  stepResults: VerificationStepResult[];
  /**
   * Total duration of the phase in milliseconds.
   */
  totalDurationMs: number;
  /**
   * Total number of steps executed.
   */
  totalSteps: number;
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
  allPassed: boolean;
  /**
   * ISO 8601 timestamp when the record was created.
   */
  createdAt: string;
  /**
   * Whether a critical step failure short-circuited the phase.
   */
  criticalFailure: boolean;
  /**
   * Number of steps that failed.
   */
  failedSteps: number;
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
  passedSteps: number;
  resultJson: VerificationPhaseResult;
  /**
   * Number of steps that were skipped.
   */
  skippedSteps: number;
  /**
   * ID of the owning task run.
   */
  taskRunId: string;
  /**
   * Total duration of the phase in milliseconds.
   */
  totalDurationMs: number;
  /**
   * Total number of steps executed.
   */
  totalSteps: number;
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
  failedIterations: number;
  /**
   * Number of iterations that passed.
   */
  passedIterations: number;
  /**
   * All verification results for the task run.
   */
  results: VerificationResultResponse[];
  /**
   * ID of the owning task run.
   */
  taskRunId: string;
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

export { type CheckIssueDetail, type CreateTaskRunRequest, type FindingsSummary, type GateEvaluationResult, type IndividualCheckResult, type Pagination, type RunPromptRequest, type RunPromptResponse, type RunPromptResponseData, type StepExecutionConfig, type TaskRun, type TaskRunBackend, type TaskRunBackendDetail, type TaskRunCreate, type TaskRunFilters, type TaskRunFinding, TaskRunFindingActionType, TaskRunFindingCategory, type TaskRunFindingCreate, type TaskRunFindingFilters, type TaskRunFindingResponse, TaskRunFindingSeverity, TaskRunFindingStatus, type TaskRunFindingSummary, type TaskRunFindingUpdate, type TaskRunFindingsListResponse, type TaskRunListResponse, type TaskRunSession, type TaskRunStatus, type TaskRunUpdate, type TaskType, type VerificationPhaseResult, type VerificationResultResponse, type VerificationResultsListResponse, type VerificationStepDetails, type VerificationStepResult };
