/**
 * Unified State Machine types for the Qontinui ecosystem.
 *
 * These types are the single source of truth for state machine data structures
 * used by both qontinui-web and qontinui-runner. They use snake_case to match
 * the API/database layer.
 *
 * Architecture:
 * - Config: A named state machine configuration (collection of states + transitions)
 * - State: A UI state defined by which elements are present
 * - Transition: A named action sequence that moves between states
 * - Action: A single UI interaction (click, type, navigate, etc.)
 *
 * Runtime navigation (pathfinding through states) is handled by the qontinui
 * Python library using multistate. The TypeScript pathfinding types here are
 * for the graph editor's path preview only.
 */

// =============================================================================
// Action Types
// =============================================================================

/**
 * All UI Bridge SDK standard actions + workflow-level actions (wait, navigate).
 *
 * Parameterized actions: click, doubleClick, rightClick, type, select, scroll, drag
 * No-param actions: clear, focus, blur, hover, check, uncheck, toggle, setValue, submit, reset
 * Workflow-level actions: wait, navigate
 */
export type StandardActionType =
  | "click"
  | "doubleClick"
  | "rightClick"
  | "type"
  | "clear"
  | "select"
  | "focus"
  | "blur"
  | "hover"
  | "scroll"
  | "check"
  | "uncheck"
  | "toggle"
  | "setValue"
  | "drag"
  | "submit"
  | "reset"
  | "wait"
  | "navigate";

/**
 * A single action within a transition.
 * Each action targets an element and performs an interaction.
 */
export interface TransitionAction {
  type: StandardActionType;
  /** Target element ID (used by most element actions) */
  target?: string;
  /** Text to type (type action) */
  text?: string;
  /** Clear before typing (type action) */
  clear_first?: boolean;
  /** Keystroke delay in ms (type action) */
  type_delay?: number;
  /** Value to select or set (select / setValue actions) */
  value?: string | string[];
  /** Select by label instead of value (select action) */
  select_by_label?: boolean;
  /** URL to navigate to (navigate action) */
  url?: string;
  /** Delay in milliseconds (wait action) */
  delay_ms?: number;
  /** Scroll direction (scroll action) */
  scroll_direction?: "up" | "down" | "left" | "right";
  /** Scroll amount in pixels (scroll action) */
  scroll_amount?: number;
  /** Drag target element ID or selector (drag action) */
  drag_target?: string;
  /** Drag target position {x, y} or named position (drag action) */
  drag_target_position?: string;
  /** Number of intermediate mousemove steps (drag action) */
  drag_steps?: number;
  /** Hold delay before first move in ms (drag action) */
  drag_hold_delay?: number;
  /** Dispatch HTML5 drag events alongside mouse events (drag action) */
  drag_html5?: boolean;
  /** Mouse button: left, right, middle (click / doubleClick / rightClick) */
  button?: "left" | "right" | "middle";
  /** Click position relative to element (click / doubleClick / rightClick) */
  position?: { x: number; y: number };
}

// =============================================================================
// Domain Knowledge
// =============================================================================

export interface DomainKnowledge {
  id: string;
  title: string;
  content: string;
  tags: string[];
}

// =============================================================================
// State Machine Config
// =============================================================================

/**
 * A state machine configuration — a named collection of states and transitions
 * that describe the navigable structure of an application's UI.
 */
export interface StateMachineConfig {
  id: string;
  name: string;
  description?: string | null;
  render_count: number;
  element_count: number;
  include_html_ids: boolean;
  created_at: string;
  updated_at: string;
}

export interface StateMachineConfigCreate {
  name: string;
  description?: string;
}

export interface StateMachineConfigUpdate {
  name?: string;
  description?: string;
}

/**
 * A config with all its states and transitions loaded.
 * Used when the full state machine needs to be displayed or exported.
 */
export interface StateMachineConfigFull extends StateMachineConfig {
  states: StateMachineState[];
  transitions: StateMachineTransition[];
}

// =============================================================================
// State
// =============================================================================

/**
 * A UI state, defined by which elements are present on screen.
 * States are discovered via co-occurrence analysis of DOM snapshots.
 */
export interface StateMachineState {
  id: string;
  config_id: string;
  state_id: string;
  name: string;
  description?: string | null;
  element_ids: string[];
  render_ids: string[];
  confidence: number;
  acceptance_criteria: string[];
  extra_metadata: Record<string, unknown>;
  domain_knowledge: DomainKnowledge[];
  created_at: string;
  updated_at: string;
}

export interface StateMachineStateCreate {
  state_id?: string;
  name: string;
  description?: string;
  element_ids?: string[];
  render_ids?: string[];
  confidence?: number;
  acceptance_criteria?: string[];
  extra_metadata?: Record<string, unknown>;
  domain_knowledge?: DomainKnowledge[];
}

export interface StateMachineStateUpdate {
  name?: string;
  description?: string;
  element_ids?: string[];
  render_ids?: string[];
  confidence?: number;
  acceptance_criteria?: string[];
  extra_metadata?: Record<string, unknown>;
  domain_knowledge?: DomainKnowledge[];
}

// =============================================================================
// Transition
// =============================================================================

/**
 * A transition between states, consisting of one or more actions.
 * Transitions define the edges of the state machine graph.
 */
export interface StateMachineTransition {
  id: string;
  config_id: string;
  transition_id: string;
  name: string;
  from_states: string[];
  activate_states: string[];
  exit_states: string[];
  actions: TransitionAction[];
  path_cost: number;
  stays_visible: boolean;
  extra_metadata: Record<string, unknown>;
  created_at: string;
  updated_at: string;
}

export interface StateMachineTransitionCreate {
  name: string;
  from_states: string[];
  activate_states: string[];
  exit_states: string[];
  actions: TransitionAction[];
  path_cost?: number;
  stays_visible?: boolean;
  extra_metadata?: Record<string, unknown>;
}

export interface StateMachineTransitionUpdate {
  name?: string;
  from_states?: string[];
  activate_states?: string[];
  exit_states?: string[];
  actions?: TransitionAction[];
  path_cost?: number;
  stays_visible?: boolean;
  extra_metadata?: Record<string, unknown>;
}

// =============================================================================
// Pathfinding (Graph Editor Visualization)
// =============================================================================

/**
 * These pathfinding types are for the graph editor's path preview feature.
 * Runtime navigation uses the qontinui Python library (multistate) directly.
 */

export interface PathfindingRequest {
  from_states: string[];
  target_states: string[];
}

export interface PathfindingStep {
  transition_id: string;
  transition_name: string;
  from_states: string[];
  activate_states: string[];
  exit_states: string[];
  path_cost: number;
}

export interface PathfindingResult {
  found: boolean;
  steps: PathfindingStep[];
  total_cost: number;
  error?: string;
}

// =============================================================================
// Execution Results (Runtime)
// =============================================================================

/**
 * Result of executing a state transition at runtime.
 * Returned by the runner's Tauri commands or qontinui Python subprocess.
 */
export interface TransitionExecutionResult {
  success: boolean;
  transition_id: string;
  active_states: string[];
  error?: string;
}

/**
 * Result of navigating to one or more target states.
 * Navigation uses multistate pathfinding to determine the optimal path.
 */
export interface NavigationResult {
  success: boolean;
  path: string[];
  active_states: string[];
  target_state?: string;
  results?: NavigationResult[];
  error?: string;
}

/**
 * Result of querying currently active states in the state machine.
 */
export interface ActiveStatesResult {
  success: boolean;
  active_states: string[];
  current_state?: string | null;
  state_history?: string[];
  error?: string;
}

/**
 * Information about a single available transition from the current state.
 */
export interface TransitionInfo {
  id: string;
  from_state: string;
  to_state: string | null;
  workflows: string[];
}

/**
 * Result of querying available transitions from the current state.
 */
export interface AvailableTransitionsResult {
  success: boolean;
  transitions: TransitionInfo[];
  current_state?: string;
  message?: string;
  error?: string;
}

// =============================================================================
// Initial States
// =============================================================================

/**
 * Source of initial states configuration.
 *
 * - "defaults": States with is_initial=true in the state machine definition
 * - "workflow": Initial states defined on the workflow (initialStateIds)
 * - "override": Session-only override from the runner UI
 */
export type InitialStatesSource = "defaults" | "workflow" | "override";

export interface ResolvedInitialStates {
  stateIds: string[];
  source: InitialStatesSource;
  states?: Array<{ id: string; name: string }>;
  workflowId?: string;
}

export interface ResolvedInitialStatesResult {
  success: boolean;
  stateIds: string[];
  source: InitialStatesSource;
  states: Array<{ id: string; name: string }>;
  workflowId: string;
  error?: string;
}

// =============================================================================
// Discovery
// =============================================================================

export type DiscoveryStrategy = "auto" | "fingerprint";

// =============================================================================
// Graph Display Types (ReactFlow)
// =============================================================================

/**
 * Data passed to a state node in the ReactFlow graph editor.
 */
export interface StateNodeData {
  stateId: string;
  name: string;
  elementCount: number;
  confidence: number;
  elementIds: string[];
  description: string | null;
  isBlocking: boolean;
  isSelected: boolean;
  isInitial: boolean;
  outgoingCount?: number;
  incomingCount?: number;
  isDropTarget?: boolean;
  onStartElementDrag?: (stateId: string, elementId: string) => void;
  /** Optional map of element ID → base64 thumbnail image (data URL or raw base64 PNG). */
  elementThumbnails?: Record<string, string>;
}

/**
 * Data passed to a transition edge in the ReactFlow graph editor.
 */
export interface TransitionEdgeData {
  transitionId: string;
  name: string;
  pathCost: number;
  actionCount: number;
  actionTypes: StandardActionType[];
  isHighlighted: boolean;
  staysVisible: boolean;
  firstActionTarget?: string;
}

// =============================================================================
// Export/Import
// =============================================================================

/**
 * Format for exporting a state machine config to JSON.
 * Compatible with UIBridgeRuntime.from_dict() in the qontinui library.
 */
export interface StateMachineExportFormat {
  states: Record<string, Record<string, unknown>>;
  transitions: Record<string, Record<string, unknown>>;
  config: Record<string, unknown>;
}
