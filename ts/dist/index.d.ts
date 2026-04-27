export { ActionPlan, ActionPlanExecuteRequest, ActionPlanResult, AgenticStep, ApiAssertion, ApiContentType, ApiVariableExtraction, BaseStep, BaseStepFields, CanonicalStep, CheckType, CommandMode, CommandStep, CompletionStep, CompositionTemplate, CostAnnotations, CostCategory, CoverageMatrix, DEFAULT_SUMMARY_PROMPT, DependencyEdge, DependencyGraph, DependencyNode, ElementTarget, HealthCheckUrl, HttpMethod, LogSourceSelection, ModelOverrideConfig, ModelOverrides, MultiStepTemplate, PHASE_INFO, PlannedAction, PlannedActionResult, PlannedActionType, PlaywrightExecutionMode, PromptStep, QualityFinding, QualityFindingCategory, QualityFindingSeverity, QualityReport, RetryPolicy, RetrySpec, RoutingRule, STEP_TYPES, SetupStep, SingleStepTemplate, SkillAuthor, SkillCategory, SkillDefinition, SkillExport, SkillExportManifest, SkillImportResult, SkillOrigin, SkillParameter, SkillParameterOption, SkillRef, SkillTemplate, StageCondition, StageInput, StageOutput, StepCost, StepTypeInfo, StepTypeName, TestType, UiBridgeAction, UiBridgeStep, UnifiedStep, UnifiedWorkflow, VerificationCategory, VerificationCategoryKind, VerificationStep, WorkflowArchitecture, WorkflowExport, WorkflowExportManifest, WorkflowFeatures, WorkflowImportResult, WorkflowPhase, WorkflowStage, WorkflowStep } from './workflow.js';
export { CheckIssueDetail, CreateTaskRunRequest, FindingsSummary, GateEvaluationResult, IndividualCheckResult, Pagination, RunPromptRequest, RunPromptResponse, RunPromptResponseData, StepExecutionConfig, TaskRun, TaskRunBackend, TaskRunBackendDetail, TaskRunCreate, TaskRunFilters, TaskRunFinding, TaskRunFindingActionType, TaskRunFindingCategory, TaskRunFindingCreate, TaskRunFindingFilters, TaskRunFindingResponse, TaskRunFindingSeverity, TaskRunFindingStatus, TaskRunFindingSummary, TaskRunFindingUpdate, TaskRunFindingsListResponse, TaskRunListResponse, TaskRunSession, TaskRunStatus, TaskRunUpdate, TaskType, VerificationPhaseResult, VerificationResultResponse, VerificationResultsListResponse, VerificationStepDetails, VerificationStepResult } from './task-run.js';
export { ActionExecutionCreate, ActionExecutionResponse, ActionStatus, ActionType, CompressionResult, CompressionStatus, CoverageData, ErrorType, ExecutionIssueCreate, ExecutionIssueResponse, ExecutionRunComplete, ExecutionRunCompleteResponse, ExecutionRunCreate, ExecutionRunResponse, ExecutionScreenshotCreate, ExecutionScreenshotResponse, ExecutionStats, ExecutionStatus, HookDefinition, HookExecutionResult, HookStatus, HookTrigger, IssueSeverity, LLMMetrics, MatchLocation, RawCompressionEvent, RawCompressionResultPayload, RawExecutionStatusEvent, RawExecutionStatusEventBase, RawHookExecutionEvent, RawHookExecutionPayload, RawHookStartedEvent, RawRetryAttemptEvent, RawRetryAttemptPayload, RawRetryStatePayload, RawRoutingDecisionEvent, RawRoutingDecisionPayload, RawStatusChangeEvent, RawSubStepCompleteEvent, RawSubStepStartedEvent, RawTokenCountPayload, RawTokenCountUpdateEvent, RetryAttempt, RetryState, RetryStatus, RoutingDecision, RoutingFactor, RoutingStatus, RunStatus, RunType, RunnerMetadata, ScreenshotAnnotation, ScreenshotAnnotationShape, ScreenshotType, SubStepInfo, SubStepStatus, SubStepStatusDisplay, TaskComplexity, TokenCount, WorkflowMetadata } from './execution.js';
export { CatchUpPolicy, ConditionScheduleConfig, ConditionStatus, CreateScheduledTaskRequest, IdleCondition, McpConnectionRef, NextTaskInfo, RepositoryInactiveCondition, RepositoryWatch, ScheduleConditions, ScheduleExpression, ScheduledTask, ScheduledTaskStatus, ScheduledTaskType, SchedulerSettings, SchedulerStatus, TaskExecutionRecord, UpdateScheduledTaskRequest } from './scheduler.js';
export { Runner, RunnerCrash, RunnerStatus, RunnerUiError } from './runner.js';
export { AccentColor, AccentColorClasses, ActionColorClasses, ActionColorType, BuiltInCategoryId, CategoryStore, CategorySummary, CheckGroup, CheckItem, CodeContext, Context, ExecutionReport, Finding, FindingActionType, FindingCategory, FindingSeverity, FindingStatus, LibraryItem, Macro, MacroAction, ParsedFinding, PhaseInfo, ReportStatus, ReportSummary, SavedApiRequest, SavedPrompt, SeverityColorClasses, SeverityLevel, ShellCommand, StatusColorClasses, StatusColorType, UserInputOption, UserInputRequest, UserInputType } from './library.js';
export { AiMessage, AiSessionState } from './chat.js';
export { ActiveStatesResult, AvailableTransitionsResult, DiscoveryStrategy, DomainKnowledge, InitialStateRef, InitialStatesSource, MouseButton, NavigationResult, PathfindingRequest, PathfindingResult, PathfindingStep, Point, ResolvedInitialStates, ResolvedInitialStatesResult, ScrollDirection, StandardActionType, StateMachineConfig, StateMachineConfigCreate, StateMachineConfigFull, StateMachineConfigUpdate, StateMachineExportFormat, StateMachineState, StateMachineStateCreate, StateMachineStateUpdate, StateMachineTransition, StateMachineTransitionCreate, StateMachineTransitionUpdate, StateNodeData, TransitionAction, TransitionActionValue, TransitionEdgeData, TransitionExecutionResult, TransitionInfo } from './state-machine.js';
export { BuiltinOverrideProposal, Constraint, ConstraintCheck, ConstraintProposal, ConstraintResult, ConstraintSeverity, ConstraintViolation, NewConstraintProposal, ReadConfigResponse, ResourceLimits, ValidateConfigRequest, ValidateConfigResponse, WriteConfigRequest, WriteConfigResponse } from './constraints.js';
export { a as CoordinateSystem, C as Coordinates, R as Region } from './Region.d-DtT3UphX.js';
export { Monitor, MonitorPosition, VirtualDesktop } from './geometry.js';
export { AccessibilityBackend, AccessibilityBounds, AccessibilityNode, AccessibilityRole, AccessibilitySelector, AccessibilitySnapshot, AccessibilityState } from './accessibility.js';

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

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Source of a state-discovery result.
 *
 * Identifies which discovery pathway produced the state machine. Mirrors
 * Python `DiscoverySourceType(str, Enum)`.
 */
type DiscoverySourceType = "playwright" | "ui_bridge" | "recording" | "vision" | "manual";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Type of action that triggers a state transition.
 *
 * Mirrors Python `TransitionTriggerType(str, Enum)`.
 */
type TransitionTriggerType = "click" | "type" | "scroll" | "hover" | "custom";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Bounding box for a discovered image element.
 *
 * Pixel-space rectangle on a source screenshot. `width` / `height` are `> 0`
 * on the Python side (validator not duplicated here — this is a wire-format
 * layer).
 */
interface DiscoveryBoundingBox {
  /**
   * Height of the bounding box (pixels, positive).
   */
  height: number;
  /**
   * Width of the bounding box (pixels, positive).
   */
  width: number;
  /**
   * X coordinate of the top-left corner (pixels).
   */
  x: number;
  /**
   * Y coordinate of the top-left corner (pixels).
   */
  y: number;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Trigger for a discovered state transition.
 *
 * Describes the action (click, type, …) that caused a transition. All
 * identifying fields are optional — different discovery sources populate
 * different subsets.
 */
interface DiscoveryTransitionTrigger {
  /**
   * ID of the DOM element (for web extraction).
   */
  elementId?: string | null;
  /**
   * ID of the image that was clicked/interacted with.
   */
  imageId?: string | null;
  /**
   * CSS selector for the trigger element.
   */
  selector?: string | null;
  /**
   * Type of trigger action. Defaults to `click` when omitted on the wire.
   */
  type?: TransitionTriggerType & string;
  /**
   * Value for type actions (text input).
   */
  value?: string | null;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Visual element within a discovered state.
 *
 * Represents an image crop from a screenshot with its bounding box and
 * optional pixel-level identification.
 */
interface DiscoveredStateImage {
  bbox: DiscoveryBoundingBox;
  /**
   * Confidence score for this image (0.0–1.0). Defaults to `1.0`.
   */
  confidence: number;
  /**
   * Semantic type of the element (e.g. `button`, `input`).
   */
  elementType?: string | null;
  /**
   * Unique identifier for the image.
   */
  id: string;
  /**
   * Human-readable label for the image.
   */
  label?: string | null;
  /**
   * Additional free-form metadata.
   */
  metadata?: {
    [k: string]: unknown;
  } | null;
  /**
   * Hash of pixel data for deduplication.
   */
  pixelHash?: string | null;
  /**
   * ID of the source screenshot.
   */
  screenshotId?: string | null;
  /**
   * URL to the source screenshot.
   */
  screenshotUrl?: string | null;
  /**
   * ID of the state this image belongs to.
   */
  stateId?: string | null;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * A discovered UI state (collection of co-occurring elements).
 *
 * States represent distinct UI screens or views identified by the set of
 * images that consistently appear together.
 */
interface DiscoveredState {
  /**
   * Confidence score for state detection (0.0–1.0). Defaults to `1.0`.
   */
  confidence: number;
  /**
   * Description of what this state represents.
   */
  description?: string | null;
  /**
   * IDs of DOM elements (for web extraction).
   */
  elementIds?: string[];
  /**
   * Unique identifier for the state.
   */
  id: string;
  /**
   * IDs of images in this state.
   */
  imageIds?: string[];
  /**
   * Additional free-form metadata.
   */
  metadata?: {
    [k: string]: unknown;
  } | null;
  /**
   * Human-readable name.
   */
  name: string;
  /**
   * IDs of renders where this state appears.
   */
  renderIds?: string[];
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * A transition between discovered states.
 *
 * Transitions represent actions that change the active set of states on the
 * screen.
 */
interface DiscoveredTransition {
  /**
   * Confidence score for transition detection (0.0–1.0). Defaults to `1.0`.
   */
  confidence: number;
  /**
   * ID of the source state.
   */
  fromStateId: string;
  /**
   * Unique identifier for the transition.
   */
  id: string;
  /**
   * Additional free-form metadata.
   */
  metadata?: {
    [k: string]: unknown;
  } | null;
  /**
   * ID of the target state.
   */
  toStateId: string;
  /**
   * What triggers this transition.
   */
  trigger?: DiscoveryTransitionTrigger | null;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Complete state-machine result from discovery.
 *
 * Unified output format regardless of the source (Playwright, UI Bridge,
 * Recording, Vision, Manual).
 */
interface StateDiscoveryResult {
  /**
   * Overall confidence score (0.0–1.0).
   */
  confidence: number;
  /**
   * ISO 8601 timestamp of creation.
   */
  createdAt: string;
  /**
   * Description of this state machine.
   */
  description?: string | null;
  /**
   * Additional discovery metadata.
   */
  discoveryMetadata?: {
    [k: string]: unknown;
  };
  /**
   * Strategy used for discovery (`auto`, `fingerprint`, `legacy`, …).
   */
  discoveryStrategy?: string | null;
  /**
   * Mapping of element IDs to render IDs where they appear.
   */
  elementToRenders?: {
    [k: string]: string[];
  };
  /**
   * Unique identifier for the result.
   */
  id: string;
  /**
   * Number of images (statistic).
   */
  imageCount: number;
  /**
   * All discovered images.
   */
  images?: DiscoveredStateImage[];
  /**
   * Human-readable name.
   */
  name: string;
  /**
   * ID of the project this belongs to.
   */
  projectId: string;
  /**
   * Number of renders analyzed (statistic).
   */
  renderCount: number;
  /**
   * ID of the source session (extraction, recording, …).
   */
  sourceSessionId?: string | null;
  sourceType: DiscoverySourceType;
  /**
   * Number of states (statistic).
   */
  stateCount: number;
  /**
   * All discovered states.
   */
  states?: DiscoveredState[];
  /**
   * Number of transitions (statistic).
   */
  transitionCount: number;
  /**
   * All discovered transitions.
   */
  transitions?: DiscoveredTransition[];
  /**
   * Number of unique elements (statistic).
   */
  uniqueElementCount: number;
  /**
   * ISO 8601 timestamp of last update.
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
 * Summary of a state-discovery result (for listings).
 *
 * Lightweight projection of `StateDiscoveryResult` used by list endpoints.
 */
interface StateDiscoveryResultSummary {
  /**
   * Confidence score (0.0–1.0).
   */
  confidence: number;
  /**
   * ISO 8601 timestamp of creation.
   */
  createdAt: string;
  /**
   * Description.
   */
  description?: string | null;
  /**
   * Strategy used.
   */
  discoveryStrategy?: string | null;
  /**
   * Unique identifier.
   */
  id: string;
  /**
   * Number of images.
   */
  imageCount: number;
  /**
   * Human-readable name.
   */
  name: string;
  /**
   * ID of the project.
   */
  projectId: string;
  sourceType: DiscoverySourceType;
  /**
   * Number of states.
   */
  stateCount: number;
  /**
   * Number of transitions.
   */
  transitionCount: number;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * API response for listing discovery results.
 */
interface StateDiscoveryResultListResponse {
  /**
   * List of result summaries.
   */
  items: StateDiscoveryResultSummary[];
  /**
   * Total count of results.
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
 * Request payload to create a state-discovery result.
 */
interface StateDiscoveryResultCreate {
  /**
   * Confidence score (0.0–1.0).
   */
  confidence: number;
  /**
   * Description.
   */
  description?: string | null;
  /**
   * Additional metadata.
   */
  discoveryMetadata?: {
    [k: string]: unknown;
  };
  /**
   * Strategy used.
   */
  discoveryStrategy?: string | null;
  /**
   * Element to renders mapping.
   */
  elementToRenders?: {
    [k: string]: string[];
  };
  /**
   * Discovered images.
   */
  images?: DiscoveredStateImage[];
  /**
   * Human-readable name.
   */
  name: string;
  /**
   * ID of the source session.
   */
  sourceSessionId?: string | null;
  sourceType: DiscoverySourceType;
  /**
   * Discovered states.
   */
  states?: DiscoveredState[];
  /**
   * Discovered transitions.
   */
  transitions?: DiscoveredTransition[];
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Request payload to update a state-discovery result.
 *
 * All fields optional; only supplied fields are applied.
 */
interface StateDiscoveryResultUpdate {
  /**
   * Description.
   */
  description?: string | null;
  /**
   * Updated metadata.
   */
  discoveryMetadata?: {
    [k: string]: unknown;
  } | null;
  /**
   * Updated images.
   */
  images?: DiscoveredStateImage[] | null;
  /**
   * Human-readable name.
   */
  name?: string | null;
  /**
   * Updated states.
   */
  states?: DiscoveredState[] | null;
  /**
   * Updated transitions.
   */
  transitions?: DiscoveredTransition[] | null;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Portable export format for state machines.
 *
 * Used when exporting a discovery result to a shareable artifact.
 * `source_type` is kept as a free-form `String` to match Python's
 * `DiscoverySourceType | str` union (enabling imports that predate the enum).
 */
interface StateMachineExport {
  /**
   * Description.
   */
  description?: string | null;
  /**
   * Element to renders mapping.
   */
  elementToRenders?: {
    [k: string]: string[];
  };
  /**
   * State images.
   */
  images?: DiscoveredStateImage[];
  /**
   * Export metadata (original ID, export timestamp, …).
   */
  metadata?: {
    [k: string]: unknown;
  };
  /**
   * State machine name.
   */
  name: string;
  /**
   * Original discovery source (string for forward compatibility — Python
   * accepts `DiscoverySourceType | str`).
   */
  sourceType: string;
  /**
   * States.
   */
  states?: DiscoveredState[];
  /**
   * Transitions.
   */
  transitions?: DiscoveredTransition[];
  /**
   * Export format version. Defaults to `"1.0.0"`.
   */
  version: string;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Request payload to import a state machine.
 */
interface StateMachineImport {
  /**
   * Override name (uses export name when omitted).
   */
  name?: string | null;
  stateMachine: StateMachineExport;
}

/**
 * Unified State Discovery Result types.
 *
 * These types represent the output of state discovery from any source:
 * - Playwright (web extraction)
 * - UI Bridge (render log analysis)
 * - Recording (user session recording)
 * - Vision (screenshot analysis)
 * - Manual (user-defined)
 *
 * All types use camelCase field names to match the generated types from Rust
 * schemas. Converter functions are provided to transform snake_case API
 * responses into these typed objects.
 */

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
type CanvasComponentType = "Markdown" | "CodeDiff" | "Table" | "FileTree" | "KeyValue" | "Terminal" | "Alert" | "Timeline" | "ProgressChart" | "FindingList" | "Checklist" | "SummaryStats" | "StateTimeline" | "Waterfall" | "Sparkline" | "WaffleChart" | "PhaseTimeline" | "IterationComparison" | "StepDurationChart" | "PhaseDistribution" | "DependencyGraph" | "CostBreakdown" | "MissionBrief" | "AcceptanceCriteria";
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
/** Data for PhaseTimeline component. */
interface PhaseTimelineData {
    phases: Array<{
        name: string;
        duration_ms: number;
        status: "completed" | "running" | "pending" | "failed";
        step_count: number;
    }>;
    total_duration_ms: number;
}
/** Data for IterationComparison component. */
interface IterationComparisonData {
    iterations: Array<{
        iteration: number;
        passed: number;
        failed: number;
        total: number;
    }>;
}
/** Data for StepDurationChart component. */
interface StepDurationChartData {
    steps: Array<{
        name: string;
        duration_ms: number;
        status: "success" | "failed" | "running" | "skipped";
        phase?: string;
    }>;
    max_duration_ms: number;
}
/** Data for PhaseDistribution component. */
interface PhaseDistributionData {
    segments: Array<{
        phase: string;
        duration_ms: number;
        percentage: number;
        color?: string;
    }>;
    total_duration_ms: number;
}
/** Data for DependencyGraph component. */
interface DependencyGraphData {
    nodes: Array<{
        id: string;
        label: string;
        type: string;
        phase: string;
        is_referenced: boolean;
        cost_category?: string;
    }>;
    edges: Array<{
        source: string;
        target: string;
        label?: string;
        edge_type: "explicit_depends_on" | "implicit_reference" | "setup_provides";
    }>;
}
/** Data for MissionBrief component. */
interface MissionBriefData {
    workflow_name: string;
    prompt: string;
    model: string;
    reflection: boolean;
    approval_gate: boolean;
    stages: Array<{
        name: string;
        index: number;
        max_iterations: number;
    }>;
    total_stages: number;
    max_iterations: number;
    progress: {
        current_stage_index: number | null;
        current_iteration: number;
        phase: string | null;
        activity: string | null;
    };
}
/** Data for AcceptanceCriteria component. */
interface AcceptanceCriteriaData {
    goal_summary: string;
    criteria: Array<{
        id: string;
        description: string;
        method: "command" | "ui_bridge" | "test" | "manual";
        priority: "critical" | "important" | "optional";
        verification_hint: string;
        category: string;
        status: "pending" | "running" | "passed" | "failed" | "skipped";
        last_error?: string;
    }>;
    assumptions?: string[];
}
/** Data for CostBreakdown component. */
interface CostBreakdownData {
    steps: Array<{
        name: string;
        estimated_ms: number;
        category: string;
        has_side_effects: boolean;
    }>;
    total_estimated_ms: number;
}

/**
 * Known Issues Registry Types
 *
 * Persistent known issue tracking that survives across workflow runs.
 * Issues are scoped to specs, URLs, components, or global.
 */
type IssueCategory = "duplication" | "rendering" | "data_integrity" | "timing" | "layout" | "state" | "performance" | "encoding" | "navigation" | "authentication" | "other";
type ScopeType = "global" | "spec" | "url" | "component" | "feature";
type DetectionMethod = "algorithmic" | "ai_judgment" | "visual" | "command" | "ui_bridge";
type KnownIssueSeverity = "critical" | "high" | "medium" | "low";
type IssueStatus = "active" | "resolved" | "monitoring" | "wont_fix";
type IssueProvenance = "manual" | "auto_detected" | "reflection" | "imported";
interface KnownIssue {
    id: string;
    title: string;
    description: string;
    category: IssueCategory;
    scope_type: ScopeType;
    scope_value: string | null;
    scope_tags: string[];
    detection_method: DetectionMethod;
    detection_config: Record<string, unknown>;
    pattern_template_id: string | null;
    reproduction_context: string | null;
    trigger_conditions: string[];
    severity: KnownIssueSeverity;
    status: IssueStatus;
    confidence: number;
    provenance: IssueProvenance;
    source_finding_ids: string[];
    source_task_run_id: string | null;
    verification_hint: string | null;
    verification_step_template: Record<string, unknown> | null;
    times_detected: number;
    times_checked: number;
    last_detected_at: string | null;
    last_checked_at: string | null;
    resolved_at: string | null;
    created_at: string;
    updated_at: string;
}
interface CreateKnownIssueRequest {
    title: string;
    description: string;
    category: IssueCategory;
    scope_type: ScopeType;
    scope_value?: string | null;
    scope_tags?: string[];
    detection_method: DetectionMethod;
    detection_config?: Record<string, unknown>;
    pattern_template_id?: string | null;
    reproduction_context?: string | null;
    trigger_conditions?: string[];
    severity: KnownIssueSeverity;
    provenance?: IssueProvenance;
    source_finding_ids?: string[];
    source_task_run_id?: string | null;
    verification_hint?: string | null;
    verification_step_template?: Record<string, unknown> | null;
}
interface UpdateKnownIssueRequest {
    title?: string;
    description?: string;
    category?: IssueCategory;
    scope_type?: ScopeType;
    scope_value?: string | null;
    scope_tags?: string[];
    detection_method?: DetectionMethod;
    detection_config?: Record<string, unknown>;
    pattern_template_id?: string | null;
    reproduction_context?: string | null;
    trigger_conditions?: string[];
    severity?: KnownIssueSeverity;
    status?: IssueStatus;
    confidence?: number;
    verification_hint?: string | null;
    verification_step_template?: Record<string, unknown> | null;
}
interface ListKnownIssuesQuery {
    scope_type?: string;
    scope_value?: string;
    category?: string;
    severity?: string;
    status?: string;
    spec_id?: string;
}
interface CreatePatternTemplateRequest {
    name: string;
    description: string;
    category: string;
    detection_type: string;
    ai_prompt_template?: string | null;
    parameters?: string | null;
}
interface TemplateParameter {
    name: string;
    type: string;
    description: string;
    default?: unknown;
}
interface IssuePatternTemplate {
    id: string;
    name: string;
    description: string;
    category: string;
    detection_type: string;
    step_template: Record<string, unknown> | null;
    ai_prompt_template: string | null;
    parameters: TemplateParameter[];
    built_in: boolean;
    status: string;
    created_at: string;
    updated_at: string;
}
/** All issue categories with display labels */
declare const ISSUE_CATEGORIES: {
    value: IssueCategory;
    label: string;
}[];
/** All severity levels with display labels */
declare const ISSUE_SEVERITIES: {
    value: KnownIssueSeverity;
    label: string;
}[];
/** All detection methods with display labels */
declare const DETECTION_METHODS: {
    value: DetectionMethod;
    label: string;
}[];

export { type AcceptanceCriteriaData, type AlertData, type CanvasComponentType, type CanvasPanel, type CanvasUpdateEvent, type ChecklistData, type CodeDiffData, type ComponentRenderLogEntry, type CostBreakdownData, type CreateKnownIssueRequest, type CreatePatternTemplateRequest, type CreateRenderLogRequest, DETECTION_METHODS, type DependencyGraphData, type DetectionMethod, type DiscoveredState, type DiscoveredStateImage, type DiscoveredTransition, type DiscoveryBoundingBox, type DiscoverySourceType, type DiscoveryTransitionTrigger, type DomElementSnapshot, type DomMutationType, type DomSnapshot, type DomSnapshotRenderLogEntry, type ElementRect, type FileTreeData, type FindingListData, type FormSnapshot, ISSUE_CATEGORIES, ISSUE_SEVERITIES, type ImageSnapshot, type IssueCategory, type IssuePatternTemplate, type IssueProvenance, type IssueStatus, type IterationComparisonData, type KeyValueData, type KnownIssue, type KnownIssueSeverity, type LinkSnapshot, type ListKnownIssuesQuery, type MarkdownData, type MissionBriefData, type PhaseDistributionData, type PhaseTimelineData, type ProgressChartData, type RenderLogEntry, type RenderLogEntryBase, type RenderLogList, type RenderLogResponse, type RenderLogStats, type RenderLogSummary, type RenderLogTrigger, SOURCE_TYPE_COLORS, SOURCE_TYPE_LABELS, type ScopeType, type SparklineData, type StateDiscoveryResult, type StateDiscoveryResultCreate, type StateDiscoveryResultListResponse, type StateDiscoveryResultSummary, type StateDiscoveryResultUpdate, type StateMachineExport, type StateMachineImport, type StateTimelineData, type StepDurationChartData, type SummaryStatsData, type TableData, type TemplateParameter, type TerminalData, type TimelineData, type TransitionTriggerType, type UpdateKnownIssueRequest, type WaffleChartData, type WaterfallData, isComponentRenderLog, isDomSnapshotRenderLog, toDiscoveredState, toDiscoveredStateImage, toDiscoveredTransition, toStateDiscoveryResult, toStateDiscoveryResultSummary };
