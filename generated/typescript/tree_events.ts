/**
 * Auto-generated TypeScript types from qontinui-schemas
 * DO NOT EDIT - regenerate with: poetry run python scripts/generate_typescript.py
 */

export enum NodeType {
  WORKFLOW = "workflow",
  ACTION = "action",
  TRANSITION = "transition",
}

export enum NodeStatus {
  PENDING = "pending",
  RUNNING = "running",
  SUCCESS = "success",
  FAILED = "failed",
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

export enum ActionType {
  FIND = "FIND",
  FIND_STATE_IMAGE = "FIND_STATE_IMAGE",
  EXISTS = "EXISTS",
  VANISH = "VANISH",
  CLICK = "CLICK",
  DOUBLE_CLICK = "DOUBLE_CLICK",
  RIGHT_CLICK = "RIGHT_CLICK",
  DRAG = "DRAG",
  SCROLL = "SCROLL",
  MOUSE_MOVE = "MOUSE_MOVE",
  TYPE = "TYPE",
  KEY_PRESS = "KEY_PRESS",
  HOTKEY = "HOTKEY",
  IF = "IF",
  LOOP = "LOOP",
  SWITCH = "SWITCH",
  TRY_CATCH = "TRY_CATCH",
  BREAK = "BREAK",
  CONTINUE = "CONTINUE",
  GO_TO_STATE = "GO_TO_STATE",
  WAIT = "WAIT",
  RUN_WORKFLOW = "RUN_WORKFLOW",
  SCREENSHOT = "SCREENSHOT",
  CODE_BLOCK = "CODE_BLOCK",
  SHELL = "SHELL",
  CUSTOM = "CUSTOM",
}

export interface MatchLocation {
  x: number;
  y: number;
  /** Width of matched region */
  w?: number | null;
  /** Height of matched region */
  h?: number | null;
}

export interface TopMatch {
  confidence: number;
  location: MatchLocation;
  dimensions?: MatchLocation | null;
}

export interface RuntimeData {
  typed_text?: string | null;
  character_count?: number | null;
  image_id?: string | null;
  found?: boolean | null;
  confidence?: number | null;
  location?: MatchLocation | null;
  dimensions?: MatchLocation | null;
  match_method?: string | null;
  top_matches?: TopMatch[] | null;
  clicked_at?: MatchLocation | null;
  button?: string | null;
  target_type?: string | null;
  source_states?: string[] | null;
  target_states?: string[] | null;
  targets_reached?: string[] | null;
  transitions_executed?: string[] | null;
  already_at_target?: boolean | null;
  condition_passed?: boolean | null;
  branch_taken?: string | null;
  workflow_name?: string | null;
  workflow_status?: string | null;
}

export interface StateContext {
  active_before?: string[];
  active_after?: string[];
  changed?: boolean;
  activated?: string[];
  deactivated?: string[];
}

export interface TimingInfo {
  /** ISO 8601 timestamp */
  start_time: string;
  /** ISO 8601 timestamp */
  end_time?: string | null;
  duration_ms?: number | null;
}

export interface Outcome {
  success: boolean;
  error?: string | null;
  retry_count?: number;
}

export interface NodeMetadata {
  config?: Record<string, any> | null;
  /** Whether this action can have child nodes */
  is_expandable?: boolean;
  /** Whether this action is displayed inline */
  is_inline?: boolean;
  runtime?: RuntimeData | null;
  state_context?: StateContext | null;
  timing?: TimingInfo | null;
  outcome?: Outcome | null;
  screenshot_reference?: string | null;
  visual_debug_reference?: string | null;
}

export interface TreeNode {
  /** Unique identifier for this node */
  id: string;
  /** Type of node (workflow, action, transition) */
  node_type: NodeType;
  /** Display name for this node */
  name: string;
  /** When this node was created */
  timestamp: number;
  /** When this node completed */
  end_timestamp?: number | null;
  /** Duration in seconds */
  duration?: number | null;
  /** ID of parent node, null for root */
  parent_id?: string | null;
  /** Current execution status */
  status: NodeStatus;
  metadata?: NodeMetadata;
  /** Error message if failed */
  error?: string | null;
}

export interface PathElement {
  /** Unique identifier for this path element */
  id: string;
  /** Display name of this element */
  name: string;
  /** Type of this element */
  node_type: NodeType;
}

export interface TreeEvent {
  /** Event type identifier */
  type?: string;
  /** Specific tree event type */
  event_type: TreeEventType;
  /** The node this event is about */
  node: TreeNode;
  /** Path from root to this node */
  path?: PathElement[];
  /** When this event was emitted (Unix epoch) */
  timestamp: number;
  /** Sequence number for ordering */
  sequence?: number;
}

export interface DisplayNode {
  id: string;
  node_type: NodeType;
  name: string;
  timestamp: number;
  end_timestamp?: number | null;
  duration?: number | null;
  status: NodeStatus;
  metadata?: NodeMetadata;
  error?: string | null;
  children?: DisplayNode[];
  is_expanded?: boolean;
  /** Nesting level (0 for root) */
  level?: number;
}

export interface TreeEventCreate {
  event_type: TreeEventType;
  node: TreeNode;
  path?: PathElement[];
  timestamp: number;
  sequence?: number;
}

export interface TreeEventResponse {
  id: string;
  run_id: string;
  event_type: TreeEventType;
  node_id: string;
  node_type: NodeType;
  node_name: string;
  parent_node_id: string | null;
  path: PathElement[];
  sequence: number;
  event_timestamp: number;
  status: NodeStatus;
  error_message: string | null;
  metadata: NodeMetadata | null;
  created_at: string;
}

export interface TreeEventListResponse {
  events: TreeEventResponse[];
  total: number;
  limit: number;
  offset: number;
  has_more: boolean;
}

export interface ExecutionTreeResponse {
  run_id: string;
  root_nodes: DisplayNode[];
  total_events: number;
  workflow_name: string | null;
  status: NodeStatus;
  duration_ms: number | null;
  /** Initial active states when workflow started */
  initial_state_ids?: string[];
}
