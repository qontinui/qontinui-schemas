/**
 * Auto-generated TypeScript types from qontinui-schemas
 * DO NOT EDIT - regenerate with: poetry run python scripts/generate_typescript.py
 */

import type {
  TreeNode,
  PathElement,
  NodeType,
  NodeStatus,
  NodeMetadata,
  DisplayNode,
} from "./tree_events";

export enum RunType {
  QA_TEST = "qa_test",
  INTEGRATION_TEST = "integration_test",
  LIVE_AUTOMATION = "live_automation",
  RECORDING = "recording",
  DEBUG = "debug",
}

export enum RunStatus {
  PENDING = "pending",
  RUNNING = "running",
  COMPLETED = "completed",
  FAILED = "failed",
  TIMEOUT = "timeout",
  CANCELLED = "cancelled",
  PAUSED = "paused",
}

export enum ActionType {
  FIND = "find",
  FIND_ALL = "find_all",
  WAIT_FOR = "wait_for",
  WAIT_UNTIL_GONE = "wait_until_gone",
  CLICK = "click",
  DOUBLE_CLICK = "double_click",
  RIGHT_CLICK = "right_click",
  SCROLL = "scroll",
  DRAG = "drag",
  TYPE = "type",
  PRESS_KEY = "press_key",
  HOTKEY = "hotkey",
  GO_TO_STATE = "go_to_state",
  TRANSITION = "transition",
  VERIFY_STATE = "verify_state",
  CONDITIONAL = "conditional",
  LOOP = "loop",
  PARALLEL = "parallel",
  SEQUENCE = "sequence",
  WAIT = "wait",
  SCREENSHOT = "screenshot",
  LOG = "log",
  ASSERT = "assert",
  CUSTOM = "custom",
}

export enum ActionStatus {
  SUCCESS = "success",
  FAILED = "failed",
  TIMEOUT = "timeout",
  SKIPPED = "skipped",
  ERROR = "error",
  PENDING = "pending",
}

export enum ErrorType {
  ELEMENT_NOT_FOUND = "element_not_found",
  TIMEOUT = "timeout",
  ASSERTION_FAILED = "assertion_failed",
  CRASH = "crash",
  NETWORK_ERROR = "network_error",
  VALIDATION_ERROR = "validation_error",
  STATE_MISMATCH = "state_mismatch",
  NAVIGATION_ERROR = "navigation_error",
  SCRIPT_ERROR = "script_error",
  OTHER = "other",
}

export enum IssueSeverity {
  CRITICAL = "critical",
  HIGH = "high",
  MEDIUM = "medium",
  LOW = "low",
  INFORMATIONAL = "informational",
}

export enum IssueType {
  VISUAL_REGRESSION = "visual_regression",
  ELEMENT_NOT_FOUND = "element_not_found",
  STATE_MISMATCH = "state_mismatch",
  TIMEOUT = "timeout",
  ASSERTION_FAILED = "assertion_failed",
  NAVIGATION_ERROR = "navigation_error",
  SCRIPT_ERROR = "script_error",
  PERFORMANCE = "performance",
  ACCESSIBILITY = "accessibility",
  OTHER = "other",
}

export enum IssueStatus {
  OPEN = "open",
  IN_PROGRESS = "in_progress",
  RESOLVED = "resolved",
  WONT_FIX = "wont_fix",
  DUPLICATE = "duplicate",
  CANNOT_REPRODUCE = "cannot_reproduce",
}

export enum IssueSource {
  AUTOMATION = "automation",
  AI_ANALYSIS = "ai_analysis",
  VISUAL_REGRESSION = "visual_regression",
  USER_REPORTED = "user_reported",
}

export enum ScreenshotType {
  ERROR = "error",
  SUCCESS = "success",
  MANUAL = "manual",
  PERIODIC = "periodic",
  ACTION_RESULT = "action_result",
  STATE_VERIFICATION = "state_verification",
  BEFORE_ACTION = "before_action",
  AFTER_ACTION = "after_action",
  DIFF_BASELINE = "diff_baseline",
  DIFF_COMPARISON = "diff_comparison",
}

export enum TreeNodeType {
  WORKFLOW = "workflow",
  ACTION = "action",
  TRANSITION = "transition",
}

export enum TreeEventType {
  WORKFLOW_STARTED = "workflow_started",
  WORKFLOW_COMPLETED = "workflow_completed",
  WORKFLOW_FAILED = "workflow_failed",
  ACTION_STARTED = "action_started",
  ACTION_COMPLETED = "action_completed",
  ACTION_FAILED = "action_failed",
  TRANSITION_STARTED = "transition_started",
  TRANSITION_COMPLETED = "transition_completed",
  TRANSITION_FAILED = "transition_failed",
}

export enum TreeNodeStatus {
  PENDING = "pending",
  RUNNING = "running",
  SUCCESS = "success",
  FAILED = "failed",
}

export interface RunnerMetadata {
  /** Version of the Qontinui runner */
  runner_version: string;
  /** Operating system (e.g., 'Windows 11', 'macOS 14') */
  os: string;
  /** Machine hostname */
  hostname: string;
  /** Screen resolution (e.g., '1920x1080') */
  screen_resolution?: string | null;
  /** CPU model/info */
  cpu_info?: string | null;
  /** Total memory in MB */
  memory_mb?: number | null;
  /** Additional runner-specific metadata */
  extra?: Record<string, any>;
}

export interface WorkflowMetadata {
  /** Unique workflow identifier */
  workflow_id: string;
  /** Human-readable workflow name */
  workflow_name: string;
  /** Workflow version string */
  workflow_version?: string | null;
  /** Total number of states in the workflow */
  total_states?: number | null;
  /** Total number of transitions in the workflow */
  total_transitions?: number | null;
  /** Workflow tags */
  tags?: string[];
  /** Workflow description */
  description?: string | null;
  /** Initial active states when workflow starts */
  initial_state_ids?: string[];
}

export interface MatchLocation {
  /** X coordinate (pixels) */
  x: number;
  /** Y coordinate (pixels) */
  y: number;
  /** Width of matched region */
  width?: number | null;
  /** Height of matched region */
  height?: number | null;
}

export interface ScreenshotAnnotation {
  /** Annotation type: 'box', 'circle', 'arrow', 'text' */
  type: string;
  /** X coordinate */
  x: number;
  /** Y coordinate */
  y: number;
  /** Width (for box/circle) */
  width?: number | null;
  /** Height (for box/circle) */
  height?: number | null;
  /** Text label for annotation */
  label?: string | null;
  /** Color (CSS format, e.g., '#ff0000') */
  color?: string | null;
}

export interface ExecutionStats {
  /** Total number of actions executed */
  total_actions: number;
  /** Actions that succeeded */
  successful_actions: number;
  /** Actions that failed */
  failed_actions: number;
  /** Actions that timed out */
  timeout_actions: number;
  /** Actions that were skipped */
  skipped_actions: number;
  /** Total execution time in ms */
  total_duration_ms: number;
  /** Average action duration in ms */
  avg_action_duration_ms?: number | null;
}

export interface CoverageData {
  /** Overall coverage percentage */
  coverage_percentage: number;
  /** Number of states visited */
  states_covered: number;
  /** Total states in workflow */
  total_states: number;
  /** Number of transitions executed */
  transitions_covered: number;
  /** Total transitions in workflow */
  total_transitions: number;
  /** List of unvisited state IDs */
  uncovered_states?: string[];
  /** List of unexecuted transition IDs */
  uncovered_transitions?: string[];
  /** Map of state ID to visit count */
  state_visit_counts?: Record<string, any>;
  /** Map of transition ID to execution count */
  transition_execution_counts?: Record<string, any>;
}

export interface ReliabilityStats {
  /** Total number of executions */
  total_executions: number;
  /** Successful executions */
  successful_executions: number;
  /** Failed executions */
  failed_executions: number;
  /** Success rate percentage */
  success_rate: number;
  /** Average duration in ms */
  avg_duration_ms: number;
  /** Median duration in ms */
  median_duration_ms: number;
  /** 95th percentile duration in ms */
  p95_duration_ms: number;
  /** Breakdown of failure types and counts */
  failure_modes?: Record<string, any>[];
}

export interface TransitionReliability {
  /** Total number of executions */
  total_executions: number;
  /** Successful executions */
  successful_executions: number;
  /** Failed executions */
  failed_executions: number;
  /** Success rate percentage */
  success_rate: number;
  /** Average duration in ms */
  avg_duration_ms: number;
  /** Median duration in ms */
  median_duration_ms: number;
  /** 95th percentile duration in ms */
  p95_duration_ms: number;
  /** Breakdown of failure types and counts */
  failure_modes?: Record<string, any>[];
  /** Name of the transition */
  transition_name: string;
  /** Source state */
  from_state: string;
  /** Destination state */
  to_state: string;
}

export interface ExecutionRunCreate {
  /** Project this run belongs to */
  project_id: string;
  /** Type of execution run */
  run_type: RunType;
  /** Human-readable name for this run */
  run_name: string;
  /** Optional description */
  description?: string | null;
  /** Information about the runner environment */
  runner_metadata: RunnerMetadata;
  /** Information about the workflow being executed */
  workflow_metadata?: WorkflowMetadata | null;
  /** Snapshot of execution configuration */
  configuration?: Record<string, any>;
}

export interface ExecutionRunResponse {
  /** Unique run identifier */
  run_id: string;
  /** Project ID */
  project_id: string;
  /** Type of run */
  run_type: RunType;
  /** Run name */
  run_name: string;
  /** Current status */
  status: RunStatus;
  /** When the run started (UTC) */
  started_at: string;
  /** When the run ended (UTC) */
  ended_at?: any | null;
  /** Total duration in seconds */
  duration_seconds?: number | null;
}

export interface ExecutionRunDetail {
  /** Unique run identifier */
  run_id: string;
  /** Project ID */
  project_id: string;
  /** Type of run */
  run_type: RunType;
  /** Run name */
  run_name: string;
  /** Current status */
  status: RunStatus;
  /** When the run started (UTC) */
  started_at: string;
  /** When the run ended (UTC) */
  ended_at?: any | null;
  /** Total duration in seconds */
  duration_seconds?: number | null;
  /** Run description */
  description?: string | null;
  /** Runner metadata */
  runner_metadata: Record<string, any>;
  /** Workflow metadata */
  workflow_metadata?: Record<string, any> | null;
  /** Configuration snapshot */
  configuration: Record<string, any>;
  /** Execution statistics */
  stats?: ExecutionStats | null;
  /** Coverage data */
  coverage_data?: CoverageData | null;
  /** Error message if failed */
  error_message?: string | null;
  /** Record creation time (UTC) */
  created_at: string;
  /** Last update time (UTC) */
  updated_at?: any | null;
  /** User who created */
  created_by?: Record<string, any> | null;
}

export interface ExecutionRunComplete {
  /** Final status (completed, failed, timeout, cancelled) */
  status: RunStatus;
  /** When the run ended (UTC) */
  ended_at: string;
  /** Final execution statistics */
  stats: ExecutionStats;
  /** Final coverage data */
  coverage?: CoverageData | null;
  /** Optional execution summary */
  summary?: string | null;
  /** Error message if failed */
  error_message?: string | null;
}

export interface ExecutionRunCompleteResponse {
  /** Run ID */
  run_id: string;
  /** Final status */
  status: RunStatus;
  /** Start time (UTC) */
  started_at: string;
  /** End time (UTC) */
  ended_at: string;
  /** Total duration in seconds */
  duration_seconds: number;
  /** Final statistics */
  stats: ExecutionStats;
  /** Coverage data */
  coverage?: CoverageData | null;
}

export interface ExecutionRunListResponse {
  /** List of runs */
  runs: ExecutionRunResponse[];
  /** Total number of runs */
  total: number;
  /** Items per page */
  limit: number;
  /** Items skipped */
  offset: number;
  /** Whether more items exist */
  has_more: boolean;
}

export interface ActionExecutionCreate {
  /** Order within the run */
  sequence_number: number;
  /** Type of action */
  action_type: ActionType;
  /** Human-readable action name */
  action_name: string;
  /** Execution status */
  status: ActionStatus;
  /** When action started (UTC) */
  started_at: string;
  /** When action completed (UTC) */
  completed_at: string;
  /** Duration in milliseconds */
  duration_ms: number;
  /** State before action */
  from_state?: string | null;
  /** Expected state after action */
  to_state?: string | null;
  /** Currently active states */
  active_states?: string[] | null;
  /** Pattern ID used */
  pattern_id?: string | null;
  /** Pattern name */
  pattern_name?: string | null;
  /** Match confidence (0-1) */
  confidence_score?: number | null;
  /** Where the element was found */
  match_location?: MatchLocation | null;
  /** Error message if failed */
  error_message?: string | null;
  /** Type of error */
  error_type?: ErrorType | null;
  /** Stack trace if available */
  error_stack?: string | null;
  /** Associated screenshot ID */
  screenshot_id?: string | null;
  /** Parent action ID (for nested actions) */
  parent_action_id?: string | null;
  /** Action input parameters */
  input_data?: Record<string, any>;
  /** Action output/results */
  output_data?: Record<string, any>;
  /** Additional metadata */
  metadata?: Record<string, any>;
}

export interface ActionExecutionBatchCreate {
  /** List of actions to report */
  actions: ActionExecutionCreate[];
}

export interface ActionExecutionResponse {
  /** Number of actions recorded */
  recorded: number;
  /** Run ID */
  run_id: string;
  /** IDs of created actions */
  action_ids?: string[];
}

export interface ActionExecutionDetail {
  /** Action execution ID */
  id: string;
  /** Run ID */
  run_id: string;
  /** Sequence number */
  sequence_number: number;
  /** Action type */
  action_type: ActionType;
  /** Action name */
  action_name: string;
  /** Status */
  status: ActionStatus;
  /** Start time (UTC) */
  started_at: string;
  /** Completion time (UTC) */
  completed_at?: any | null;
  /** Duration in ms */
  duration_ms?: number | null;
  /** Source state */
  from_state?: string | null;
  /** Target state */
  to_state?: string | null;
  /** Actual state reached */
  actual_state?: string | null;
  /** Error message */
  error_message?: string | null;
  /** Error type */
  error_type?: string | null;
  /** Input data */
  input_data?: Record<string, any>;
  /** Output data */
  output_data?: Record<string, any>;
  /** Metadata */
  metadata?: Record<string, any>;
  /** Record creation time (UTC) */
  created_at: string;
}

export interface ActionExecutionListResponse {
  /** List of actions */
  actions: ActionExecutionDetail[];
  /** Total count */
  total: number;
  /** Items per page */
  limit: number;
  /** Items skipped */
  offset: number;
  /** Whether more items exist */
  has_more: boolean;
}

export interface ExecutionScreenshotCreate {
  /** Screenshot ID (client-generated UUID) */
  screenshot_id: string;
  /** Order within the run */
  sequence_number: number;
  /** Type of screenshot */
  screenshot_type: ScreenshotType;
  /** When screenshot was taken (UTC) */
  timestamp: string;
  /** Image width in pixels */
  width: number;
  /** Image height in pixels */
  height: number;
  /** Associated action sequence number */
  action_sequence_number?: number | null;
  /** State when screenshot was taken */
  state?: string | null;
  /** Active states at capture time */
  active_states?: string[] | null;
  /** Visual annotations on the screenshot */
  annotations?: ScreenshotAnnotation[];
  /** Additional screenshot metadata */
  metadata?: Record<string, any>;
}

export interface ExecutionScreenshotResponse {
  /** Screenshot ID */
  screenshot_id: string;
  /** Run ID */
  run_id: string;
  /** URL to full image */
  image_url: string;
  /** URL to thumbnail */
  thumbnail_url?: string | null;
  /** Upload time (UTC) */
  uploaded_at: string;
  /** File size in bytes */
  file_size_bytes: number;
}

export interface ExecutionScreenshotDetail {
  /** Screenshot ID */
  id: string;
  /** Run ID */
  run_id: string;
  /** Associated action ID */
  action_execution_id?: string | null;
  /** Sequence number */
  sequence_number: number;
  /** Screenshot type */
  screenshot_type: ScreenshotType;
  /** Storage path */
  storage_path: string;
  /** Public URL */
  image_url?: string | null;
  /** Thumbnail URL */
  thumbnail_url?: string | null;
  /** Width in pixels */
  width: number;
  /** Height in pixels */
  height: number;
  /** File size */
  file_size_bytes?: number | null;
  /** State name */
  state_name?: string | null;
  /** Perceptual hash for diffs */
  perceptual_hash?: string | null;
  /** Capture time (UTC) */
  captured_at: string;
  /** Metadata */
  metadata?: Record<string, any>;
  /** Record creation time (UTC) */
  created_at: string;
}

export interface ExecutionScreenshotListResponse {
  /** List of screenshots */
  screenshots: ExecutionScreenshotDetail[];
  /** Total count */
  total: number;
  /** Items per page */
  limit: number;
  /** Items skipped */
  offset: number;
  /** Whether more items exist */
  has_more: boolean;
}

export interface ExecutionIssueCreate {
  /** Brief issue title */
  title: string;
  /** Detailed issue description */
  description: string;
  /** Issue severity */
  severity: IssueSeverity;
  /** Type of issue */
  issue_type: IssueType;
  /** Related action sequence number */
  action_sequence_number?: number | null;
  /** State where issue occurred */
  state?: string | null;
  /** Associated screenshot IDs */
  screenshot_ids?: string[];
  /** Steps to reproduce */
  reproduction_steps?: string[];
  /** Expected behavior */
  expected_behavior?: string | null;
  /** Actual behavior observed */
  actual_behavior?: string | null;
  /** Additional issue metadata */
  metadata?: Record<string, any>;
}

export interface ExecutionIssueBatchCreate {
  /** List of issues to report */
  issues: ExecutionIssueCreate[];
}

export interface ExecutionIssueResponse {
  /** Number of issues recorded */
  recorded: number;
  /** Run ID */
  run_id: string;
  /** IDs of created issues */
  issue_ids?: string[];
}

export interface ExecutionIssueDetail {
  /** Issue ID */
  id: string;
  /** Run ID */
  run_id: string;
  /** Associated action ID */
  action_execution_id?: string | null;
  /** Assigned user ID */
  assigned_to_user_id?: string | null;
  /** Issue type */
  issue_type: IssueType;
  /** Severity */
  severity: IssueSeverity;
  /** Current status */
  status: IssueStatus;
  /** How issue was detected */
  source: IssueSource;
  /** Issue title */
  title: string;
  /** Issue description */
  description: string;
  /** State name */
  state_name?: string | null;
  /** Screenshot IDs */
  screenshot_ids?: string[];
  /** Reproduction steps */
  reproduction_steps?: string[];
  /** Error details */
  error_details?: Record<string, any>;
  /** Metadata */
  metadata?: Record<string, any>;
  /** Resolution notes */
  resolution_notes?: string | null;
  /** Creation time (UTC) */
  created_at: string;
  /** Last update time (UTC) */
  updated_at: string;
}

export interface ExecutionIssueUpdate {
  /** New status */
  status?: IssueStatus | null;
  /** New severity */
  severity?: IssueSeverity | null;
  /** Assign to user */
  assigned_to_user_id?: string | null;
  /** Resolution notes */
  resolution_notes?: string | null;
}

export interface ExecutionIssueListResponse {
  /** List of issues */
  issues: ExecutionIssueDetail[];
  /** Total count */
  total: number;
  /** Items per page */
  limit: number;
  /** Items skipped */
  offset: number;
  /** Whether more items exist */
  has_more: boolean;
}

export interface ExecutionIssueSummary {
  /** Total issues */
  total?: number;
  /** Count by severity */
  by_severity?: Record<string, any>;
  /** Count by type */
  by_type?: Record<string, any>;
  /** Count by status */
  by_status?: Record<string, any>;
  /** Critical issues */
  critical_count?: number;
  /** High priority issues */
  high_count?: number;
  /** Open issues */
  open_count?: number;
  /** Resolved issues */
  resolved_count?: number;
}

export interface ExecutionTreeEventCreate {
  /** Type of tree event */
  event_type: TreeEventType;
  /** The tree node this event is about */
  node: TreeNode;
  /** Path from root to this node */
  path?: PathElement[];
  /** When event occurred (Unix epoch) */
  timestamp: number;
  /** Sequence number for ordering */
  sequence?: number;
}

export interface ExecutionTreeEventBatchCreate {
  /** List of tree events to store */
  events: ExecutionTreeEventCreate[];
}

export interface ExecutionTreeEventResponse {
  /** Event ID */
  id: string;
  /** Run ID */
  run_id: string;
  /** Event type */
  event_type: TreeEventType;
  /** Node ID */
  node_id: string;
  /** Node type */
  node_type: NodeType;
  /** Node name */
  node_name: string;
  /** Parent node ID */
  parent_node_id?: string | null;
  /** Path to node */
  path?: PathElement[];
  /** Sequence number */
  sequence: number;
  /** Event timestamp (Unix epoch) */
  event_timestamp: number;
  /** Node start timestamp */
  node_start_timestamp?: number | null;
  /** Node end timestamp */
  node_end_timestamp?: number | null;
  /** Duration in milliseconds */
  duration_ms?: number | null;
  /** Node status */
  status: NodeStatus;
  /** Error message if failed */
  error_message?: string | null;
  /** Active states before event */
  active_states_before?: string[];
  /** Active states after event */
  active_states_after?: string[];
  /** Whether states changed */
  states_changed?: boolean;
  /** Node metadata */
  metadata?: NodeMetadata | null;
  /** Record creation time (UTC) */
  created_at: string;
}

export interface ExecutionTreeEventListResponse {
  /** List of tree events */
  events: ExecutionTreeEventResponse[];
  /** Total count */
  total: number;
  /** Items per page */
  limit: number;
  /** Items skipped */
  offset: number;
  /** Whether more items exist */
  has_more: boolean;
}

export interface ExecutionTreeResponse {
  /** Run ID */
  run_id: string;
  /** Root nodes of the tree */
  root_nodes?: DisplayNode[];
  /** Total number of events */
  total_events: number;
  /** Workflow name */
  workflow_name?: string | null;
  /** Overall execution status */
  status: NodeStatus;
  /** Total duration in ms */
  duration_ms?: number | null;
  /** Initial active states */
  initial_state_ids?: string[];
  /** State ID to name mapping */
  state_name_map?: Record<string, any>;
}

/**
 * Re-export tree event core types from tree_events.ts for full structures.
 * These are referenced by ExecutionTreeEventResponse and ExecutionTreeResponse.
 */
// import type { TreeNode, PathElement, DisplayNode, NodeMetadata } from './tree_events';
