export { AgenticStep, ApiAssertion, ApiContentType, ApiVariableExtraction, BaseStep, CheckType, CommandStep, CompletionStep, CompositionTemplate, DEFAULT_SUMMARY_PROMPT, HealthCheckUrl, HttpMethod, LogSourceSelection, ModelOverrideConfig, ModelOverrides, MultiStepTemplate, PHASE_INFO, PlaywrightExecutionMode, PromptStep, RoutingRule, STEP_TYPES, SetupStep, SingleStepTemplate, SkillAuthor, SkillCategory, SkillDefinition, SkillExport, SkillExportManifest, SkillImportResult, SkillOrigin, SkillParameter, SkillParameterOption, SkillRef, SkillTemplate, StageCondition, StepTypeInfo, StepTypeName, TestType, UiBridgeStep, UnifiedStep, UnifiedWorkflow, VerificationStep, WorkflowExport, WorkflowExportManifest, WorkflowFeatures, WorkflowImportResult, WorkflowPhase, WorkflowStage, WorkflowStep } from './workflow.cjs';
export { CheckIssueDetail, CreateTaskRunRequest, FindingsSummary, GateEvaluationResult, IndividualCheckResult, Pagination, RunPromptRequest, RunPromptResponse, StepExecutionConfig, TaskRun, TaskRunBackend, TaskRunBackendDetail, TaskRunCreate, TaskRunFilters, TaskRunFinding, TaskRunFindingActionType, TaskRunFindingCategory, TaskRunFindingCreate, TaskRunFindingFilters, TaskRunFindingResponse, TaskRunFindingSeverity, TaskRunFindingStatus, TaskRunFindingSummary, TaskRunFindingUpdate, TaskRunFindingsListResponse, TaskRunListResponse, TaskRunSession, TaskRunStatus, TaskRunUpdate, TaskType, VerificationPhaseResult, VerificationResultResponse, VerificationResultsListResponse, VerificationStepDetails, VerificationStepResult } from './task-run.cjs';
export { ActionExecutionCreate, ActionExecutionResponse, ActionStatus, ActionType, CompressionResult, CompressionStatus, CoverageData, ErrorType, ExecutionIssueCreate, ExecutionIssueResponse, ExecutionRunComplete, ExecutionRunCompleteResponse, ExecutionRunCreate, ExecutionRunResponse, ExecutionScreenshotCreate, ExecutionScreenshotResponse, ExecutionStats, ExecutionStatus, HookDefinition, HookExecutionResult, HookStatus, HookTrigger, IssueSeverity, RawCompressionEvent, RawCompressionResultPayload, RawExecutionStatusEvent, RawExecutionStatusEventBase, RawHookExecutionEvent, RawHookExecutionPayload, RawHookStartedEvent, RawRetryAttemptEvent, RawRetryAttemptPayload, RawRetryStatePayload, RawRoutingDecisionEvent, RawRoutingDecisionPayload, RawStatusChangeEvent, RawSubStepCompleteEvent, RawSubStepStartedEvent, RawTokenCountPayload, RawTokenCountUpdateEvent, RetryAttempt, RetryState, RetryStatus, RoutingDecision, RoutingFactor, RoutingStatus, RunStatus, RunType, RunnerMetadata, ScreenshotType, SubStepInfo, SubStepStatus, SubStepStatusDisplay, TaskComplexity, TokenCount, WorkflowMetadata } from './execution.cjs';
export { AutoFixTask, ConditionStatus, CreateScheduledTaskRequest, IdleCondition, NextTaskInfo, PromptTask, RepositoryInactiveCondition, RepositoryWatch, ScheduleConditions, ScheduleCron, ScheduleExpression, ScheduleInterval, ScheduleOnce, ScheduleState, ScheduledTask, ScheduledTaskStatus, ScheduledTaskType, SchedulerSettings, SchedulerStatus, TaskExecutionRecord, UpdateScheduledTaskRequest, WorkflowTask } from './scheduler.cjs';
export { AccentColor, AccentColorClasses, ActionColorClasses, ActionColorType, BuiltInCategoryId, CategoryStore, CategorySummary, CheckGroup, CheckItem, CodeContext, Context, ExecutionReport, Finding, FindingActionType, FindingCategory, FindingSeverity, FindingStatus, LibraryItem, Macro, MacroAction, ParsedFinding, PhaseInfo, ReportStatus, ReportSummary, SavedApiRequest, SavedPrompt, SeverityColorClasses, SeverityLevel, ShellCommand, StatusColorClasses, StatusColorType, UserInputOption, UserInputRequest, UserInputType } from './library.cjs';
export { AiMessage, AiSessionState } from './chat.cjs';

/**
 * Shared Render Log Types
 *
 * These types define the render logging schema used across:
 * - qontinui-runner (Tauri desktop app) - Component-level logging
 * - qontinui-web (Next.js web app) - Full DOM snapshot logging
 *
 * The runner reads both its own logs and web's logs for verification
 * in AI feedback loops.
 *
 * NOTE: Field names use snake_case to match Rust/Python JSON serialization.
 * This ensures compatibility across all services.
 */
/**
 * Trigger types for render log capture.
 * What caused the render log to be captured.
 */
type RenderLogTrigger = "mount" | "navigation" | "mutation" | "data_update" | "prop_change" | "manual" | "interval";
/**
 * DOM mutation types (for web's MutationObserver).
 */
type DomMutationType = "childList" | "attributes" | "characterData";
/**
 * Base render log entry with common fields.
 * Extended by component logs and DOM snapshot logs.
 */
interface RenderLogEntryBase {
    /** Unique identifier for this log entry */
    id: string;
    /** Unix timestamp in milliseconds when the capture occurred */
    timestamp: number;
    /** What triggered this capture */
    trigger: RenderLogTrigger;
    /** Session ID for grouping related captures */
    session_id?: string;
}
/**
 * Component-level render log entry.
 * Used by qontinui-runner to log what data components render.
 *
 * Components explicitly call logRender() with their rendered data.
 * This is lightweight and focused on verifying data display.
 */
interface ComponentRenderLogEntry extends RenderLogEntryBase {
    /** Type discriminator */
    type: "component";
    /** Component name (e.g., "RunRecapTab", "StatusBanner") */
    component: string;
    /** The data that was rendered (component-specific structure) */
    data: Record<string, unknown>;
    /** Sections that are visible/expanded in the component */
    visible_sections?: string[];
    /** Associated task run ID if applicable */
    task_run_id?: number;
}
/**
 * Bounding rectangle for an element.
 */
interface ElementRect {
    x: number;
    y: number;
    width: number;
    height: number;
}
/**
 * Single DOM element in a snapshot.
 * Recursive structure representing the DOM tree.
 */
interface DomElementSnapshot {
    /** HTML tag name (lowercase) */
    tag: string;
    /** Element ID if present */
    id?: string;
    /** CSS class list */
    class_list: string[];
    /** Direct text content (not from children) */
    text_content?: string;
    /** Length of innerHTML (for size estimation) */
    inner_html_length?: number;
    /** Bounding rectangle relative to viewport */
    rect?: ElementRect;
    /** Whether the element is visible */
    is_visible: boolean;
    /** Computed opacity */
    opacity?: number;
    /** Computed display value */
    display?: string;
    /** Key HTML attributes */
    attributes: Record<string, string>;
    /** Selected computed styles */
    computed_styles?: Record<string, string>;
    /** Child elements (recursive) */
    children: DomElementSnapshot[];
}
/**
 * Form data extracted from the page.
 */
interface FormSnapshot {
    id?: string;
    action?: string;
    method?: string;
    inputs: Array<{
        type: string;
        name: string;
        value: string;
    }>;
}
/**
 * Link data extracted from the page.
 */
interface LinkSnapshot {
    href: string;
    text: string;
}
/**
 * Image data extracted from the page.
 */
interface ImageSnapshot {
    src: string;
    alt?: string;
    width?: number;
    height?: number;
}
/**
 * Full DOM snapshot data.
 * Contains the entire page state for debugging.
 *
 * NOTE: The snapshot is stored as JSONB in the database, so field names
 * in the nested structure (root, forms, links, images) may vary by
 * implementation. The web frontend uses camelCase internally but that's
 * stored as-is in the JSONB field.
 */
interface DomSnapshot {
    /** Root element tree (document.body) */
    root: DomElementSnapshot | null;
    /** Total number of elements in the tree */
    total_elements: number;
    /** Concatenated visible text (truncated) */
    visible_text?: string;
    /** Forms on the page */
    forms: FormSnapshot[];
    /** Links on the page */
    links: LinkSnapshot[];
    /** Images on the page */
    images: ImageSnapshot[];
    /** JavaScript console errors captured */
    errors: string[];
    /** JavaScript console warnings captured */
    warnings: string[];
}
/**
 * Full DOM snapshot render log entry.
 * Used by qontinui-web to capture entire page state.
 *
 * Automatically captured via MutationObserver on DOM changes.
 * Provides comprehensive debugging data.
 */
interface DomSnapshotRenderLogEntry extends RenderLogEntryBase {
    /** Type discriminator */
    type: "dom_snapshot";
    /** Current page URL */
    page_url: string;
    /** Page title */
    page_title?: string;
    /** Type of DOM mutation that triggered capture (if mutation trigger) */
    mutation_type?: DomMutationType;
    /** CSS selector of the element that was mutated */
    target_selector?: string;
    /** The full DOM snapshot */
    snapshot: DomSnapshot;
    /** Viewport dimensions */
    viewport_width?: number;
    viewport_height?: number;
    /** Scroll position */
    scroll_x?: number;
    scroll_y?: number;
    /** Time taken to capture the snapshot in milliseconds */
    capture_duration_ms?: number;
    /** Number of elements captured */
    element_count?: number;
    /** User ID if authenticated (web only) */
    user_id?: string;
}
/**
 * Any render log entry.
 * Use the `type` field to discriminate.
 */
type RenderLogEntry = ComponentRenderLogEntry | DomSnapshotRenderLogEntry;
/**
 * Request to create a render log entry (web API).
 */
interface CreateRenderLogRequest {
    session_id: string;
    page_url: string;
    page_title?: string;
    trigger: RenderLogTrigger;
    mutation_type?: DomMutationType;
    target_selector?: string;
    snapshot: DomSnapshot;
    viewport_width?: number;
    viewport_height?: number;
    scroll_x?: number;
    scroll_y?: number;
    capture_duration_ms?: number;
    element_count?: number;
}
/**
 * Render log response from web API.
 * Includes database-generated fields.
 */
interface RenderLogResponse {
    id: number;
    session_id: string;
    timestamp: string;
    page_url: string;
    page_title?: string;
    trigger: RenderLogTrigger;
    mutation_type?: DomMutationType;
    target_selector?: string;
    snapshot: DomSnapshot;
    viewport_width?: number;
    viewport_height?: number;
    scroll_x?: number;
    scroll_y?: number;
    capture_duration_ms?: number;
    element_count?: number;
    user_id?: string;
}
/**
 * Summary of a render log (without full snapshot).
 */
interface RenderLogSummary {
    id: number;
    session_id: string;
    timestamp: string;
    page_url: string;
    page_title?: string;
    trigger: RenderLogTrigger;
    mutation_type?: DomMutationType;
    element_count?: number;
    capture_duration_ms?: number;
}
/**
 * Paginated list of render logs.
 */
interface RenderLogList {
    items: RenderLogSummary[];
    total: number;
    page: number;
    page_size: number;
    has_more: boolean;
}
/**
 * Render logging statistics.
 */
interface RenderLogStats {
    enabled: boolean;
    total_snapshots: number;
    total_sessions: number;
    oldest_snapshot?: string;
    newest_snapshot?: string;
    storage_used_bytes: number;
    image_count: number;
}
/**
 * Check if a render log entry is a component log.
 */
declare function isComponentRenderLog(entry: RenderLogEntry): entry is ComponentRenderLogEntry;
/**
 * Check if a render log entry is a DOM snapshot log.
 */
declare function isDomSnapshotRenderLog(entry: RenderLogEntry): entry is DomSnapshotRenderLogEntry;

/**
 * Unified State Discovery Result types.
 *
 * These types represent the output of state discovery from any source:
 * - Playwright (web extraction)
 * - UI Bridge (render log analysis)
 * - Recording (user session recording)
 * - Vision (screenshot analysis)
 * - Manual (user-defined)
 */
type DiscoverySourceType = "playwright" | "ui_bridge" | "recording" | "vision" | "manual";
type TransitionTriggerType = "click" | "type" | "scroll" | "hover" | "custom";
interface DiscoveryBoundingBox {
    x: number;
    y: number;
    width: number;
    height: number;
}
interface DiscoveryTransitionTrigger {
    type: TransitionTriggerType;
    image_id?: string;
    element_id?: string;
    selector?: string;
    value?: string;
}
interface DiscoveredStateImage {
    id: string;
    screenshot_id?: string;
    screenshot_url?: string;
    bbox: DiscoveryBoundingBox;
    pixel_hash?: string;
    state_id?: string;
    element_type?: string;
    label?: string;
    confidence: number;
    metadata?: Record<string, unknown>;
}
interface DiscoveredState {
    id: string;
    name: string;
    image_ids: string[];
    render_ids: string[];
    element_ids: string[];
    confidence: number;
    description?: string;
    metadata?: Record<string, unknown>;
}
interface DiscoveredTransition {
    id: string;
    from_state_id: string;
    to_state_id: string;
    trigger?: DiscoveryTransitionTrigger;
    confidence: number;
    metadata?: Record<string, unknown>;
}
interface StateDiscoveryResult {
    id: string;
    project_id: string;
    name: string;
    description?: string;
    source_type: DiscoverySourceType;
    source_session_id?: string;
    discovery_strategy?: string;
    images: DiscoveredStateImage[];
    states: DiscoveredState[];
    transitions: DiscoveredTransition[];
    element_to_renders: Record<string, string[]>;
    image_count: number;
    state_count: number;
    transition_count: number;
    render_count: number;
    unique_element_count: number;
    confidence: number;
    discovery_metadata: Record<string, unknown>;
    created_at: string;
    updated_at: string;
}
interface StateDiscoveryResultSummary {
    id: string;
    project_id: string;
    name: string;
    description?: string;
    source_type: DiscoverySourceType;
    discovery_strategy?: string;
    image_count: number;
    state_count: number;
    transition_count: number;
    confidence: number;
    created_at: string;
}
interface StateDiscoveryResultListResponse {
    items: StateDiscoveryResultSummary[];
    total: number;
}
interface StateDiscoveryResultCreate {
    name: string;
    description?: string;
    source_type: DiscoverySourceType;
    source_session_id?: string;
    discovery_strategy?: string;
    images: DiscoveredStateImage[];
    states: DiscoveredState[];
    transitions: DiscoveredTransition[];
    element_to_renders: Record<string, string[]>;
    confidence: number;
    discovery_metadata: Record<string, unknown>;
}
interface StateDiscoveryResultUpdate {
    name?: string;
    description?: string;
    images?: DiscoveredStateImage[];
    states?: DiscoveredState[];
    transitions?: DiscoveredTransition[];
    discovery_metadata?: Record<string, unknown>;
}
interface StateMachineExport {
    version: string;
    name: string;
    description?: string;
    source_type: DiscoverySourceType | string;
    images: DiscoveredStateImage[];
    states: DiscoveredState[];
    transitions: DiscoveredTransition[];
    element_to_renders: Record<string, string[]>;
    metadata: Record<string, unknown>;
}
interface StateMachineImport {
    state_machine: StateMachineExport;
    name?: string;
}
declare const SOURCE_TYPE_LABELS: Record<DiscoverySourceType, string>;
declare const SOURCE_TYPE_COLORS: Record<DiscoverySourceType, string>;
declare function toStateDiscoveryResult(data: Record<string, unknown>): StateDiscoveryResult;
declare function toDiscoveredStateImage(data: unknown): DiscoveredStateImage;
declare function toDiscoveredState(data: unknown): DiscoveredState;
declare function toDiscoveredTransition(data: unknown): DiscoveredTransition;
declare function toStateDiscoveryResultSummary(data: Record<string, unknown>): StateDiscoveryResultSummary;

/**
 * Canvas Types
 *
 * Type definitions for the A2UI (Agent-to-UI) Canvas system.
 * Canvas panels allow the AI agent to render rich, structured visual content
 * in the dashboard during workflow execution.
 */
/**
 * Supported canvas component types.
 * These are validated server-side against an allowlist.
 */
type CanvasComponentType = "Markdown" | "CodeDiff" | "Table" | "FileTree" | "KeyValue" | "Terminal" | "Alert" | "Timeline" | "ProgressChart" | "FindingList" | "Checklist" | "SummaryStats" | "StateTimeline" | "Waterfall" | "Sparkline" | "WaffleChart";
/**
 * A canvas panel rendered in the dashboard.
 */
interface CanvasPanel {
    panel_id: string;
    component: CanvasComponentType;
    title: string;
    data: Record<string, unknown>;
    priority?: number;
    size?: "compact" | "normal" | "large";
    group?: string;
    task_run_id?: string;
    created_at?: string;
    updated_at?: string;
}
/**
 * Event emitted when a canvas panel is created, updated, or deleted.
 */
interface CanvasUpdateEvent {
    action: "create" | "update" | "delete" | "clear";
    panel_id: string;
    panel?: CanvasPanel;
    task_run_id?: string;
}
/** Data for Markdown component. */
interface MarkdownData {
    content: string;
}
/** Data for Table component. */
interface TableData {
    columns: string[];
    rows: (string | number | boolean | null)[][];
    sortable?: boolean;
}
/** Data for CodeDiff component. */
interface CodeDiffData {
    file_path: string;
    language?: string;
    old_content?: string;
    new_content?: string;
    unified_diff?: string;
}
/** Data for KeyValue component. */
interface KeyValueData {
    pairs: Array<{
        key: string;
        value: string | number | boolean;
        style?: "default" | "success" | "warning" | "error";
    }>;
}
/** Data for Alert component. */
interface AlertData {
    severity: "info" | "success" | "warning" | "error";
    message: string;
    details?: string;
}
/** Data for Terminal component. */
interface TerminalData {
    lines: string[];
    max_lines?: number;
}
/** Data for Timeline component. */
interface TimelineData {
    events: Array<{
        timestamp?: string;
        title: string;
        description?: string;
        status?: "pending" | "running" | "success" | "failed";
    }>;
}
/** Data for FileTree component. */
interface FileTreeData {
    root: string;
    entries: Array<{
        path: string;
        type: "file" | "directory";
        status?: "added" | "modified" | "deleted";
    }>;
}
/** Data for ProgressChart component. */
interface ProgressChartData {
    segments: Array<{
        label: string;
        value: number;
        color?: string;
    }>;
    total?: number;
}
/** Data for FindingList component. */
interface FindingListData {
    findings: Array<{
        id?: string;
        title: string;
        description?: string;
        severity?: "info" | "low" | "medium" | "high" | "critical";
        location?: string;
    }>;
}
/** Data for Checklist component. */
interface ChecklistData {
    items: Array<{
        id: string;
        label: string;
        checked: boolean;
        description?: string;
    }>;
}
/** Data for SummaryStats component. */
interface SummaryStatsData {
    total: number;
    passed: number;
    failed: number;
    skipped?: number;
    label?: string;
}
/** Data for StateTimeline component. */
interface StateTimelineData {
    steps: Array<{
        name: string;
        iterations: Array<{
            iteration: number;
            status: "pass" | "fail" | "skip" | "pending";
        }>;
    }>;
}
/** Data for Waterfall component. */
interface WaterfallData {
    entries: Array<{
        name: string;
        start_ms: number;
        duration_ms: number;
        status?: "running" | "success" | "failed" | "skipped" | "pending";
        phase?: string;
    }>;
    total_duration_ms: number;
}
/** Data for Sparkline component. */
interface SparklineData {
    series: Array<{
        name: string;
        values: Array<{
            iteration: number;
            outcome: "pass" | "fail";
        }>;
    }>;
}
/** Data for WaffleChart component. */
interface WaffleChartData {
    cells: Array<{
        label: string;
        status: "pass" | "fail" | "pending" | "running" | "skip";
    }>;
    columns?: number;
}

export { type AlertData, type CanvasComponentType, type CanvasPanel, type CanvasUpdateEvent, type ChecklistData, type CodeDiffData, type ComponentRenderLogEntry, type CreateRenderLogRequest, type DiscoveredState, type DiscoveredStateImage, type DiscoveredTransition, type DiscoveryBoundingBox, type DiscoverySourceType, type DiscoveryTransitionTrigger, type DomElementSnapshot, type DomMutationType, type DomSnapshot, type DomSnapshotRenderLogEntry, type ElementRect, type FileTreeData, type FindingListData, type FormSnapshot, type ImageSnapshot, type KeyValueData, type LinkSnapshot, type MarkdownData, type ProgressChartData, type RenderLogEntry, type RenderLogEntryBase, type RenderLogList, type RenderLogResponse, type RenderLogStats, type RenderLogSummary, type RenderLogTrigger, SOURCE_TYPE_COLORS, SOURCE_TYPE_LABELS, type SparklineData, type StateDiscoveryResult, type StateDiscoveryResultCreate, type StateDiscoveryResultListResponse, type StateDiscoveryResultSummary, type StateDiscoveryResultUpdate, type StateMachineExport, type StateMachineImport, type StateTimelineData, type SummaryStatsData, type TableData, type TerminalData, type TimelineData, type TransitionTriggerType, type WaffleChartData, type WaterfallData, isComponentRenderLog, isDomSnapshotRenderLog, toDiscoveredState, toDiscoveredStateImage, toDiscoveredTransition, toStateDiscoveryResult, toStateDiscoveryResultSummary };
