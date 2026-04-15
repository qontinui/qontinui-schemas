/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Types of nodes in the execution tree.
 */
type NodeType = "workflow" | "action" | "transition";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Execution status of a tree node.
 */
type NodeStatus = "pending" | "running" | "success" | "failed";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Types of tree events emitted during execution.
 */
type TreeEventType =
  | "workflow_started"
  | "workflow_completed"
  | "workflow_failed"
  | "action_started"
  | "action_completed"
  | "action_failed"
  | "transition_started"
  | "transition_completed"
  | "transition_failed";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Types of actions in the automation system.
 *
 * Corresponds to the action types defined in the qontinui-schemas config
 * models. Variants use SCREAMING_SNAKE_CASE on the wire to match the Python
 * enum values.
 */
type ActionType =
  | "FIND"
  | "FIND_STATE_IMAGE"
  | "EXISTS"
  | "VANISH"
  | "CLICK"
  | "DOUBLE_CLICK"
  | "RIGHT_CLICK"
  | "DRAG"
  | "SCROLL"
  | "MOUSE_MOVE"
  | "TYPE"
  | "KEY_PRESS"
  | "HOTKEY"
  | "IF"
  | "LOOP"
  | "SWITCH"
  | "TRY_CATCH"
  | "BREAK"
  | "CONTINUE"
  | "GO_TO_STATE"
  | "WAIT"
  | "RUN_WORKFLOW"
  | "SCREENSHOT"
  | "CODE_BLOCK"
  | "SHELL"
  | "CUSTOM";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Location of a pattern match on screen.
 */
interface MatchLocation {
  /**
   * Height of the matched region (optional — point matches omit it).
   */
  h?: number | null;
  /**
   * Width of the matched region (optional — point matches omit it).
   */
  w?: number | null;
  x: number;
  y: number;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * A single match result with confidence and location.
 */
interface TopMatch {
  confidence: number;
  dimensions?: MatchLocation | null;
  location: MatchLocation;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Runtime execution data captured during action execution.
 *
 * Different fields are populated depending on the action type. All fields
 * are optional; the wire schema allows additional runtime fields via a
 * `flatten`-d map.
 */
interface RuntimeData {
  already_at_target?: boolean | null;
  branch_taken?: string | null;
  button?: string | null;
  character_count?: number | null;
  clicked_at?: MatchLocation | null;
  condition_passed?: boolean | null;
  confidence?: number | null;
  dimensions?: MatchLocation | null;
  found?: boolean | null;
  image_id?: string | null;
  location?: MatchLocation | null;
  match_method?: string | null;
  source_states?: string[] | null;
  target_states?: string[] | null;
  target_type?: string | null;
  targets_reached?: string[] | null;
  top_matches?: TopMatch[] | null;
  transitions_executed?: string[] | null;
  typed_text?: string | null;
  workflow_name?: string | null;
  workflow_status?: string | null;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * State-machine context captured before/after an action.
 */
interface StateContext {
  activated: string[];
  active_after: string[];
  active_before: string[];
  changed: boolean;
  deactivated: string[];
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Precise timing information for an event. Times are ISO 8601 strings (see
 * crate-level docs for the rationale — the types crate is wire-only and
 * doesn't depend on a chrono version).
 */
interface TimingInfo {
  duration_ms?: number | null;
  /**
   * ISO 8601 timestamp. `None` while the event is still in flight.
   */
  end_time?: string | null;
  /**
   * ISO 8601 timestamp.
   */
  start_time: string;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Execution outcome of an action.
 */
interface Outcome {
  error?: string | null;
  retry_count: number;
  success: boolean;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Metadata for a tree node — action configuration, runtime data, state
 * context, and timing. All fields are optional because different node
 * types populate different fields.
 */
interface NodeMetadata {
  /**
   * Action configuration (JSON object — shape varies by `ActionType`).
   */
  config?: {
    [k: string]: unknown;
  } | null;
  /**
   * Whether this action can have child nodes.
   */
  is_expandable: boolean;
  /**
   * Whether this action is displayed inline.
   */
  is_inline: boolean;
  outcome?: Outcome | null;
  runtime?: RuntimeData | null;
  /**
   * Screenshot reference (path or URL).
   */
  screenshot_reference?: string | null;
  state_context?: StateContext | null;
  timing?: TimingInfo | null;
  /**
   * Visual-debug image reference (path or URL).
   */
  visual_debug_reference?: string | null;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * A node in the execution tree — a single workflow, action, or transition
 * in the execution hierarchy.
 */
interface TreeNode {
  /**
   * Duration in seconds.
   */
  duration?: number | null;
  /**
   * When this node completed (Unix epoch seconds).
   */
  end_timestamp?: number | null;
  /**
   * Error message if `status == Failed`.
   */
  error?: string | null;
  /**
   * Unique identifier for this node.
   */
  id: string;
  metadata?: NodeMetadata & {};
  /**
   * Display name for this node.
   */
  name: string;
  node_type: NodeType;
  /**
   * ID of parent node, `None` for root.
   */
  parent_id?: string | null;
  status: NodeStatus;
  /**
   * When this node was created (Unix epoch seconds).
   */
  timestamp: number;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Element in a tree path (for breadcrumb navigation).
 */
interface PathElement {
  id: string;
  name: string;
  node_type: NodeType;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * A tree event emitted during execution.
 *
 * Primary event type for execution logging. Carries the event type, the
 * affected node with full metadata, the path from root to this node, and
 * a sequence number for ordering.
 */
interface TreeEvent {
  event_type: TreeEventType;
  node: TreeNode;
  /**
   * Path from root to this node (breadcrumb).
   */
  path: PathElement[];
  /**
   * Sequence number for ordering.
   */
  sequence: number;
  /**
   * When this event was emitted (Unix epoch seconds).
   */
  timestamp: number;
  /**
   * Event type identifier — always `"tree_event"` on the wire.
   */
  type: string;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Display node structure used by the frontend — extended version of
 * `TreeNode` with tree-rendering properties. NOT persisted; constructed
 * from `TreeNode` data for UI display.
 */
interface DisplayNode {
  /**
   * Child nodes in the tree.
   */
  children: DisplayNode[];
  duration?: number | null;
  end_timestamp?: number | null;
  error?: string | null;
  id: string;
  /**
   * Whether this node should be expanded in the UI (default: true).
   */
  is_expanded: boolean;
  /**
   * Nesting level in the tree (0 for root, 1 for first-level children).
   */
  level: number;
  metadata?: NodeMetadata & {};
  name: string;
  node_type: NodeType;
  status: NodeStatus;
  timestamp: number;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Request to store a tree event.
 */
interface TreeEventCreate {
  event_type: TreeEventType;
  node: TreeNode;
  path: PathElement[];
  sequence: number;
  timestamp: number;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Response for a stored tree event.
 */
interface TreeEventResponse {
  /**
   * ISO 8601 timestamp.
   */
  created_at: string;
  error_message?: string | null;
  event_timestamp: number;
  event_type: TreeEventType;
  /**
   * UUID as string (wire-format — see crate-level docs).
   */
  id: string;
  metadata?: NodeMetadata | null;
  node_id: string;
  node_name: string;
  node_type: NodeType;
  parent_node_id?: string | null;
  path: PathElement[];
  /**
   * Run UUID as string.
   */
  run_id: string;
  sequence: number;
  status: NodeStatus;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Paginated list of tree events.
 */
interface TreeEventListResponse {
  events: TreeEventResponse[];
  has_more: boolean;
  limit: number;
  offset: number;
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
 * Full execution tree reconstructed from events.
 */
interface ExecutionTreeResponse {
  duration_ms?: number | null;
  /**
   * Initial active states when the workflow started.
   */
  initial_state_ids: string[];
  root_nodes: DisplayNode[];
  /**
   * Run UUID as string.
   */
  run_id: string;
  /**
   * Mapping of state IDs to display names.
   */
  state_name_map: {
    [k: string]: string;
  };
  status: NodeStatus;
  total_events: number;
  workflow_name?: string | null;
  [k: string]: unknown;
}

export type { ActionType, DisplayNode, ExecutionTreeResponse, MatchLocation, NodeMetadata, NodeStatus, NodeType, Outcome, PathElement, RuntimeData, StateContext, TimingInfo, TopMatch, TreeEvent, TreeEventCreate, TreeEventListResponse, TreeEventResponse, TreeEventType, TreeNode };
