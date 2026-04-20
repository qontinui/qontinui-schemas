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
  alreadyAtTarget?: boolean | null;
  branchTaken?: string | null;
  button?: string | null;
  characterCount?: number | null;
  clickedAt?: MatchLocation | null;
  conditionPassed?: boolean | null;
  confidence?: number | null;
  dimensions?: MatchLocation | null;
  found?: boolean | null;
  imageId?: string | null;
  location?: MatchLocation | null;
  matchMethod?: string | null;
  sourceStates?: string[] | null;
  targetStates?: string[] | null;
  targetType?: string | null;
  targetsReached?: string[] | null;
  topMatches?: TopMatch[] | null;
  transitionsExecuted?: string[] | null;
  typedText?: string | null;
  workflowName?: string | null;
  workflowStatus?: string | null;
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
  activeAfter: string[];
  activeBefore: string[];
  changed: boolean;
  deactivated: string[];
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
  durationMs?: number | null;
  /**
   * ISO 8601 timestamp. `None` while the event is still in flight.
   */
  endTime?: string | null;
  /**
   * ISO 8601 timestamp.
   */
  startTime: string;
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
  retryCount: number;
  success: boolean;
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
  isExpandable: boolean;
  /**
   * Whether this action is displayed inline.
   */
  isInline: boolean;
  outcome?: Outcome | null;
  runtime?: RuntimeData | null;
  /**
   * Screenshot reference (path or URL).
   */
  screenshotReference?: string | null;
  stateContext?: StateContext | null;
  timing?: TimingInfo | null;
  /**
   * Visual-debug image reference (path or URL).
   */
  visualDebugReference?: string | null;
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
  endTimestamp?: number | null;
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
  nodeType: NodeType;
  /**
   * ID of parent node, `None` for root.
   */
  parentId?: string | null;
  status: NodeStatus;
  /**
   * When this node was created (Unix epoch seconds).
   */
  timestamp: number;
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
  nodeType: NodeType;
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
  eventType: TreeEventType;
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
  endTimestamp?: number | null;
  error?: string | null;
  id: string;
  /**
   * Whether this node should be expanded in the UI (default: true).
   */
  isExpanded: boolean;
  /**
   * Nesting level in the tree (0 for root, 1 for first-level children).
   */
  level: number;
  metadata?: NodeMetadata & {};
  name: string;
  nodeType: NodeType;
  status: NodeStatus;
  timestamp: number;
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
  eventType: TreeEventType;
  node: TreeNode;
  path: PathElement[];
  sequence: number;
  timestamp: number;
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
  createdAt: string;
  errorMessage?: string | null;
  eventTimestamp: number;
  eventType: TreeEventType;
  /**
   * UUID as string (wire-format — see crate-level docs).
   */
  id: string;
  metadata?: NodeMetadata | null;
  nodeId: string;
  nodeName: string;
  nodeType: NodeType;
  parentNodeId?: string | null;
  path: PathElement[];
  /**
   * Run UUID as string.
   */
  runId: string;
  sequence: number;
  status: NodeStatus;
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
  hasMore: boolean;
  limit: number;
  offset: number;
  total: number;
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
  durationMs?: number | null;
  /**
   * Initial active states when the workflow started.
   */
  initialStateIds: string[];
  rootNodes: DisplayNode[];
  /**
   * Run UUID as string.
   */
  runId: string;
  /**
   * Mapping of state IDs to display names.
   */
  stateNameMap: {
    [k: string]: string;
  };
  status: NodeStatus;
  totalEvents: number;
  workflowName?: string | null;
}

export type { ActionType, DisplayNode, ExecutionTreeResponse, MatchLocation, NodeMetadata, NodeStatus, NodeType, Outcome, PathElement, RuntimeData, StateContext, TimingInfo, TopMatch, TreeEvent, TreeEventCreate, TreeEventListResponse, TreeEventResponse, TreeEventType, TreeNode };
