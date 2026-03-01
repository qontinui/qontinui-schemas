/**
 * Task Run Types
 *
 * Merged type definitions from both runner (taskRun.ts) and web (task-runs.ts).
 * Runner types are used for local execution, web types for backend API communication.
 */
type TaskRunStatus = "running" | "complete" | "failed" | "stopped";
type TaskType = "task" | "automation" | "scheduled";
interface TaskRun {
    id: string;
    task_name: string;
    prompt?: string | null;
    task_type: TaskType;
    config_id?: string | null;
    workflow_name?: string | null;
    status: TaskRunStatus;
    sessions_count: number;
    max_sessions?: number | null;
    auto_continue: boolean;
    output_log: string;
    error_message?: string | null;
    summary?: string | null;
    goal_achieved?: boolean | null;
    remaining_work?: string | null;
    summary_generated_at?: string | null;
    created_at: string;
    updated_at: string;
    completed_at?: string | null;
}
interface TaskRunBackend {
    id: string;
    project_id: string | null;
    created_by_user_id: string | null;
    runner_id: string | null;
    task_name: string;
    prompt: string;
    status: TaskRunStatus;
    sessions_count: number;
    max_sessions: number | null;
    auto_continue: boolean;
    output_summary: string | null;
    full_output_stored: boolean;
    error_message: string | null;
    duration_seconds: number | null;
    created_at: string;
    updated_at: string;
    completed_at: string | null;
}
interface TaskRunSession {
    id: string;
    task_id: string;
    session_number: number;
    started_at: string;
    ended_at: string | null;
    duration_seconds: number | null;
    output_summary: string | null;
}
type TaskRunFindingCategory = "code_bug" | "security" | "performance" | "todo" | "enhancement" | "config_issue" | "test_issue" | "documentation" | "runtime_issue" | "already_fixed" | "expected_behavior" | "data_migration" | "warning";
type TaskRunFindingSeverity = "critical" | "high" | "medium" | "low" | "info";
type TaskRunFindingStatus = "detected" | "in_progress" | "needs_input" | "resolved" | "wont_fix" | "deferred";
type TaskRunFindingActionType = "auto_fix" | "needs_user_input" | "manual" | "informational";
interface TaskRunFinding {
    id: string;
    task_id: string;
    category: TaskRunFindingCategory;
    severity: TaskRunFindingSeverity;
    status: TaskRunFindingStatus;
    action_type: TaskRunFindingActionType;
    signature_hash: string | null;
    title: string;
    description: string;
    resolution: string | null;
    file_path: string | null;
    line_number: number | null;
    column_number: number | null;
    code_snippet: string | null;
    detected_in_session: number;
    resolved_in_session: number | null;
    needs_input: boolean;
    question: string | null;
    input_options: string[] | null;
    user_response: string | null;
    detected_at: string;
    resolved_at: string | null;
    updated_at: string;
}
type TaskRunFindingResponse = TaskRunFinding;
interface TaskRunFindingSummary {
    by_category: Record<string, number>;
    by_severity: Record<string, number>;
    by_status: Record<string, number>;
    total: number;
}
interface TaskRunBackendDetail extends TaskRunBackend {
    sessions: TaskRunSession[];
    findings: TaskRunFinding[];
    finding_summary: TaskRunFindingSummary;
}
interface TaskRunCreate {
    id?: string;
    project_id?: string;
    runner_id?: string;
    task_name: string;
    prompt?: string;
    max_sessions?: number;
    auto_continue?: boolean;
    task_type?: TaskType;
    config_id?: string;
    workflow_name?: string;
    execution_steps_json?: string;
    log_sources_json?: string;
}
interface TaskRunUpdate {
    status?: TaskRunStatus;
    sessions_count?: number;
    output_summary?: string;
    full_output?: string;
    full_output_stored?: boolean;
    error_message?: string;
    duration_seconds?: number;
    completed_at?: string;
}
interface TaskRunFindingCreate {
    id?: string;
    category: TaskRunFindingCategory;
    severity: TaskRunFindingSeverity;
    status?: TaskRunFindingStatus;
    action_type?: TaskRunFindingActionType;
    signature_hash?: string;
    title: string;
    description: string;
    resolution?: string;
    file_path?: string;
    line_number?: number;
    column_number?: number;
    code_snippet?: string;
    detected_in_session: number;
    needs_input?: boolean;
    question?: string;
    input_options?: string[];
}
interface TaskRunFindingUpdate {
    status?: TaskRunFindingStatus;
    resolution?: string;
    resolved_in_session?: number;
    resolved_at?: string;
    user_response?: string;
}
interface RunPromptResponse {
    success: boolean;
    task_run_id?: string;
    session_id?: string;
    state_file?: string;
    log_file?: string;
    pid?: number;
    error?: string;
    output?: string;
    data?: {
        output?: string;
        response?: string;
    };
}
interface RunPromptRequest {
    name: string;
    content: string;
    max_sessions?: number;
    display_prompt?: string;
    timeout_seconds?: number;
    context?: string;
    image_paths?: string[];
    video_paths?: string[];
    trace_path?: string;
    max_video_frames?: number;
    max_trace_screenshots?: number;
}
interface CreateTaskRunRequest {
    task_name: string;
    prompt?: string;
    task_type?: TaskType;
    config_id?: string;
    workflow_name?: string;
    max_sessions?: number;
    auto_continue?: boolean;
    execution_steps_json?: string;
    log_sources_json?: string;
}
interface TaskRunFilters {
    project_id?: string;
    status?: TaskRunStatus;
    start_date?: string;
    end_date?: string;
    offset?: number;
    limit?: number;
}
interface TaskRunFindingFilters {
    category?: TaskRunFindingCategory;
    severity?: TaskRunFindingSeverity;
    status?: TaskRunFindingStatus;
}
interface Pagination {
    total: number;
    limit: number;
    offset: number;
    has_more: boolean;
}
interface TaskRunListResponse {
    tasks: TaskRunBackend[];
    pagination: Pagination;
}
interface TaskRunFindingsListResponse {
    findings: TaskRunFinding[];
    summary: TaskRunFindingSummary;
}
interface FindingsSummary {
    total: number;
    by_severity: Record<string, number>;
    by_category: Record<string, number>;
    by_status: Record<string, number>;
    recent: TaskRunFindingResponse[];
}
interface CheckIssueDetail {
    file: string;
    line: number | null;
    column: number | null;
    code: string | null;
    message: string;
    severity: string;
    fixable: boolean;
}
interface IndividualCheckResult {
    name: string;
    status: string;
    duration_ms: number;
    issues_found: number;
    issues_fixed: number;
    files_checked: number;
    error_message: string | null;
    output: string | null;
    issues: CheckIssueDetail[];
}
interface VerificationStepDetails {
    step_id: string;
    phase: string;
    stdout: string | null;
    stderr: string | null;
    assertions_passed: number | null;
    assertions_total: number | null;
    console_output: string | null;
    page_snapshot: string | null;
    exit_code: number | null;
    check_results: IndividualCheckResult[] | null;
}
interface StepExecutionConfig {
    action_type?: string | null;
    target_image_id?: string | null;
    target_image_name?: string | null;
    check_type?: string | null;
    timeout_seconds?: number | null;
    [key: string]: unknown;
}
interface VerificationStepResult {
    step_index: number;
    step_type: string;
    step_name: string;
    step_id: string | null;
    success: boolean;
    error: string | null;
    screenshot_path: string | null;
    started_at: string | null;
    ended_at: string | null;
    duration_ms: number;
    config: StepExecutionConfig;
    verification_details: VerificationStepDetails | null;
    output_data: Record<string, unknown> | null;
}
interface GateEvaluationResult {
    gate_name: string;
    required_step_ids: string[];
    passed_step_ids: string[];
    failed_step_ids: string[];
    missing_step_ids: string[];
    passed: boolean;
}
interface VerificationPhaseResult {
    iteration: number;
    all_passed: boolean;
    total_steps: number;
    passed_steps: number;
    failed_steps: number;
    skipped_steps: number;
    total_duration_ms: number;
    step_results: VerificationStepResult[];
    critical_failure: boolean;
    gate_results: GateEvaluationResult[];
    gate_based_evaluation: boolean;
}
interface VerificationResultResponse {
    id: string;
    task_run_id: string;
    iteration: number;
    all_passed: boolean;
    total_steps: number;
    passed_steps: number;
    failed_steps: number;
    skipped_steps: number;
    total_duration_ms: number;
    critical_failure: boolean;
    result_json: VerificationPhaseResult;
    created_at: string;
}
interface VerificationResultsListResponse {
    task_run_id: string;
    results: VerificationResultResponse[];
    count: number;
    passed_iterations: number;
    failed_iterations: number;
}

export type { CheckIssueDetail, CreateTaskRunRequest, FindingsSummary, GateEvaluationResult, IndividualCheckResult, Pagination, RunPromptRequest, RunPromptResponse, StepExecutionConfig, TaskRun, TaskRunBackend, TaskRunBackendDetail, TaskRunCreate, TaskRunFilters, TaskRunFinding, TaskRunFindingActionType, TaskRunFindingCategory, TaskRunFindingCreate, TaskRunFindingFilters, TaskRunFindingResponse, TaskRunFindingSeverity, TaskRunFindingStatus, TaskRunFindingSummary, TaskRunFindingUpdate, TaskRunFindingsListResponse, TaskRunListResponse, TaskRunSession, TaskRunStatus, TaskRunUpdate, TaskType, VerificationPhaseResult, VerificationResultResponse, VerificationResultsListResponse, VerificationStepDetails, VerificationStepResult };
