/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * All UI Bridge SDK standard actions plus workflow-level actions
 * (`wait`, `navigate`).
 *
 * Parameterized actions: `click`, `doubleClick`, `rightClick`, `type`,
 * `select`, `scroll`, `drag`.
 * No-param actions: `clear`, `focus`, `blur`, `hover`, `check`, `uncheck`,
 * `toggle`, `setValue`, `submit`, `reset`.
 * Workflow-level actions: `wait`, `navigate`.
 *
 * Serialized as a bare camelCase string matching the TypeScript literal
 * union (e.g., `"doubleClick"`, `"setValue"`).
 */
type StandardActionType =
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

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * An (x, y) pixel offset used by the click-position and related parameters.
 */
interface Point {
  /**
   * Horizontal offset in pixels.
   */
  x: number;
  /**
   * Vertical offset in pixels.
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
 * Scroll direction for the `scroll` action.
 *
 * Serialized as a bare lowercase string.
 */
type ScrollDirection = "up" | "down" | "left" | "right";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Mouse button for click actions.
 *
 * Serialized as a bare lowercase string.
 */
type MouseButton = "left" | "right" | "middle";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * A value for `select` / `setValue` actions: either a single string or a list
 * of strings (for multi-select).
 *
 * Serialized as an untagged union matching the TS `string | string[]`.
 */
type TransitionActionValue = string | string[];

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * A single action within a transition.
 *
 * Each action targets an element and performs an interaction. Most fields are
 * only meaningful for specific action types — see the per-field docs.
 */
interface TransitionAction {
  /**
   * Mouse button: `left`, `right`, `middle`
   * (`click` / `doubleClick` / `rightClick`).
   */
  button?: MouseButton | null;
  /**
   * Clear before typing (`type` action).
   */
  clear_first?: boolean | null;
  /**
   * Delay in milliseconds (`wait` action).
   */
  delay_ms?: number | null;
  /**
   * Hold delay before the first move in milliseconds (`drag` action).
   */
  drag_hold_delay?: number | null;
  /**
   * Dispatch HTML5 drag events alongside mouse events (`drag` action).
   */
  drag_html5?: boolean | null;
  /**
   * Number of intermediate mousemove steps (`drag` action).
   */
  drag_steps?: number | null;
  /**
   * Drag target element ID or selector (`drag` action).
   */
  drag_target?: string | null;
  /**
   * Drag target position (e.g., `{ x, y }` stringified, or a named
   * position) (`drag` action).
   */
  drag_target_position?: string | null;
  /**
   * Click position relative to the element
   * (`click` / `doubleClick` / `rightClick`).
   */
  position?: Point | null;
  /**
   * Scroll amount in pixels (`scroll` action).
   */
  scroll_amount?: number | null;
  /**
   * Scroll direction (`scroll` action).
   */
  scroll_direction?: ScrollDirection | null;
  /**
   * Select by label instead of value (`select` action).
   */
  select_by_label?: boolean | null;
  /**
   * Target element ID (used by most element actions).
   */
  target?: string | null;
  /**
   * Text to type (`type` action).
   */
  text?: string | null;
  type: StandardActionType;
  /**
   * Keystroke delay in milliseconds (`type` action).
   */
  type_delay?: number | null;
  /**
   * URL to navigate to (`navigate` action).
   */
  url?: string | null;
  /**
   * Value to select or set (`select` / `setValue` actions).
   */
  value?: TransitionActionValue | null;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * A piece of domain knowledge attached to a state — free-form notes that
 * help the AI reason about what a state represents.
 */
interface DomainKnowledge {
  /**
   * Full knowledge content (markdown or plain text).
   */
  content: string;
  /**
   * Unique identifier for this knowledge entry.
   */
  id: string;
  /**
   * Tags for filtering and grouping.
   */
  tags: string[];
  /**
   * Short title / headline.
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
 * A state machine configuration — a named collection of states and
 * transitions that describe the navigable structure of an application's UI.
 */
interface StateMachineConfig {
  /**
   * ISO 8601 timestamp of creation.
   */
  created_at: string;
  /**
   * Optional description.
   */
  description?: string | null;
  /**
   * Number of unique elements discovered in this config.
   */
  element_count: number;
  /**
   * Unique identifier (UUID).
   */
  id: string;
  /**
   * Whether to include HTML IDs when generating selectors.
   */
  include_html_ids: boolean;
  /**
   * Display name.
   */
  name: string;
  /**
   * Number of DOM renders collected for this config.
   */
  render_count: number;
  /**
   * ISO 8601 timestamp of last modification.
   */
  updated_at: string;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Payload for creating a new state machine configuration.
 */
interface StateMachineConfigCreate {
  /**
   * Optional description.
   */
  description?: string | null;
  /**
   * Display name.
   */
  name: string;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Payload for updating an existing state machine configuration.
 */
interface StateMachineConfigUpdate {
  /**
   * New description.
   */
  description?: string | null;
  /**
   * New display name.
   */
  name?: string | null;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * A UI state, defined by which elements are present on screen.
 *
 * States are discovered via co-occurrence analysis of DOM snapshots.
 */
interface StateMachineState {
  /**
   * Acceptance criteria for verifying the state is reached.
   */
  acceptance_criteria: string[];
  /**
   * Confidence score (0.0–1.0) from the discovery pass.
   */
  confidence: number;
  /**
   * Parent config ID.
   */
  config_id: string;
  /**
   * ISO 8601 timestamp of creation.
   */
  created_at: string;
  /**
   * Optional description.
   */
  description?: string | null;
  /**
   * Domain knowledge entries attached to this state.
   */
  domain_knowledge: DomainKnowledge[];
  /**
   * Element IDs that must be present for this state to be active.
   */
  element_ids: string[];
  /**
   * Free-form metadata bag.
   */
  extra_metadata: {
    [k: string]: unknown;
  };
  /**
   * Unique database identifier (UUID).
   */
  id: string;
  /**
   * Display name.
   */
  name: string;
  /**
   * Render IDs that contributed to discovering this state.
   */
  render_ids: string[];
  /**
   * Stable logical state ID (distinct from `id`; set by the user or
   * generator).
   */
  state_id: string;
  /**
   * ISO 8601 timestamp of last modification.
   */
  updated_at: string;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * A transition between states, consisting of one or more actions.
 *
 * Transitions define the edges of the state machine graph.
 */
interface StateMachineTransition {
  /**
   * Ordered list of actions executed as part of this transition.
   */
  actions: TransitionAction[];
  /**
   * States activated when this transition fires.
   */
  activate_states: string[];
  /**
   * Parent config ID.
   */
  config_id: string;
  /**
   * ISO 8601 timestamp of creation.
   */
  created_at: string;
  /**
   * States exited when this transition fires.
   */
  exit_states: string[];
  /**
   * Free-form metadata bag.
   */
  extra_metadata: {
    [k: string]: unknown;
  };
  /**
   * States this transition may be taken from.
   */
  from_states: string[];
  /**
   * Unique database identifier (UUID).
   */
  id: string;
  /**
   * Display name.
   */
  name: string;
  /**
   * Cost used by pathfinding to prefer cheaper transitions.
   */
  path_cost: number;
  /**
   * Whether source states stay visible after activation.
   */
  stays_visible: boolean;
  /**
   * Stable logical transition ID (distinct from `id`).
   */
  transition_id: string;
  /**
   * ISO 8601 timestamp of last modification.
   */
  updated_at: string;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * A config with all its states and transitions loaded.
 *
 * Used when the full state machine needs to be displayed or exported. This
 * mirrors the TypeScript `StateMachineConfigFull extends StateMachineConfig`
 * by flattening the base config's fields via `#[serde(flatten)]`.
 */
interface StateMachineConfigFull {
  /**
   * ISO 8601 timestamp of creation.
   */
  created_at: string;
  /**
   * Optional description.
   */
  description?: string | null;
  /**
   * Number of unique elements discovered in this config.
   */
  element_count: number;
  /**
   * Unique identifier (UUID).
   */
  id: string;
  /**
   * Whether to include HTML IDs when generating selectors.
   */
  include_html_ids: boolean;
  /**
   * Display name.
   */
  name: string;
  /**
   * Number of DOM renders collected for this config.
   */
  render_count: number;
  /**
   * All states belonging to this config.
   */
  states: StateMachineState[];
  /**
   * All transitions belonging to this config.
   */
  transitions: StateMachineTransition[];
  /**
   * ISO 8601 timestamp of last modification.
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
 * Payload for creating a new state within a config.
 */
interface StateMachineStateCreate {
  /**
   * Acceptance criteria.
   */
  acceptance_criteria?: string[] | null;
  /**
   * Initial confidence score.
   */
  confidence?: number | null;
  /**
   * Optional description.
   */
  description?: string | null;
  /**
   * Domain knowledge entries.
   */
  domain_knowledge?: DomainKnowledge[] | null;
  /**
   * Element IDs that define this state.
   */
  element_ids?: string[] | null;
  /**
   * Free-form metadata bag.
   */
  extra_metadata?: {
    [k: string]: unknown;
  } | null;
  /**
   * Display name.
   */
  name: string;
  /**
   * Render IDs associated with this state.
   */
  render_ids?: string[] | null;
  /**
   * Optional stable logical state ID (generated if omitted).
   */
  state_id?: string | null;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Payload for updating an existing state.
 */
interface StateMachineStateUpdate {
  /**
   * New acceptance criteria.
   */
  acceptance_criteria?: string[] | null;
  /**
   * New confidence score.
   */
  confidence?: number | null;
  /**
   * New description.
   */
  description?: string | null;
  /**
   * New domain knowledge entries.
   */
  domain_knowledge?: DomainKnowledge[] | null;
  /**
   * New element ID set.
   */
  element_ids?: string[] | null;
  /**
   * New metadata bag.
   */
  extra_metadata?: {
    [k: string]: unknown;
  } | null;
  /**
   * New display name.
   */
  name?: string | null;
  /**
   * New render ID set.
   */
  render_ids?: string[] | null;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Payload for creating a new transition.
 */
interface StateMachineTransitionCreate {
  /**
   * Ordered list of actions executed as part of this transition.
   */
  actions: TransitionAction[];
  /**
   * States activated when this transition fires.
   */
  activate_states: string[];
  /**
   * States exited when this transition fires.
   */
  exit_states: string[];
  /**
   * Free-form metadata bag.
   */
  extra_metadata?: {
    [k: string]: unknown;
  } | null;
  /**
   * States this transition may be taken from.
   */
  from_states: string[];
  /**
   * Display name.
   */
  name: string;
  /**
   * Cost used by pathfinding.
   */
  path_cost?: number | null;
  /**
   * Whether source states stay visible after activation.
   */
  stays_visible?: boolean | null;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Payload for updating an existing transition.
 */
interface StateMachineTransitionUpdate {
  /**
   * New action list.
   */
  actions?: TransitionAction[] | null;
  /**
   * New "activate" states.
   */
  activate_states?: string[] | null;
  /**
   * New "exit" states.
   */
  exit_states?: string[] | null;
  /**
   * New metadata bag.
   */
  extra_metadata?: {
    [k: string]: unknown;
  } | null;
  /**
   * New "from" states.
   */
  from_states?: string[] | null;
  /**
   * New display name.
   */
  name?: string | null;
  /**
   * New path cost.
   */
  path_cost?: number | null;
  /**
   * New stays-visible flag.
   */
  stays_visible?: boolean | null;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Request to compute a path between state sets.
 *
 * The pathfinding types are for the graph editor's path-preview feature.
 * Runtime navigation uses the qontinui Python library (`multistate`)
 * directly.
 */
interface PathfindingRequest {
  /**
   * States assumed to be active at the start of the search.
   */
  from_states: string[];
  /**
   * States that must all be active at the end.
   */
  target_states: string[];
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * A single step on a computed path.
 */
interface PathfindingStep {
  /**
   * States activated by this step.
   */
  activate_states: string[];
  /**
   * States exited by this step.
   */
  exit_states: string[];
  /**
   * States required to be active before this step.
   */
  from_states: string[];
  /**
   * Cost of this step.
   */
  path_cost: number;
  /**
   * Logical transition ID taken in this step.
   */
  transition_id: string;
  /**
   * Display name of the transition.
   */
  transition_name: string;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Result of a pathfinding search.
 */
interface PathfindingResult {
  /**
   * Error message when pathfinding failed.
   */
  error?: string | null;
  /**
   * Whether a valid path was found.
   */
  found: boolean;
  /**
   * Ordered steps on the computed path (empty when `found` is `false`).
   */
  steps: PathfindingStep[];
  /**
   * Sum of `path_cost` across steps.
   */
  total_cost: number;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Result of executing a single state transition at runtime.
 *
 * Returned by the runner's Tauri commands or the qontinui Python subprocess.
 */
interface TransitionExecutionResult {
  /**
   * States that are active after the transition.
   */
  active_states: string[];
  /**
   * Error message when execution failed.
   */
  error?: string | null;
  /**
   * Whether the transition executed successfully.
   */
  success: boolean;
  /**
   * Logical transition ID that was executed.
   */
  transition_id: string;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Result of navigating to one or more target states.
 *
 * Navigation uses multistate pathfinding to determine the optimal path.
 * Nested `results` allow a single top-level navigation to fan out to several
 * targets and report per-target outcomes.
 */
interface NavigationResult {
  /**
   * States active after the navigation completed.
   */
  active_states: string[];
  /**
   * Error message when navigation failed.
   */
  error?: string | null;
  /**
   * Ordered list of state IDs visited.
   */
  path: string[];
  /**
   * Per-target sub-results for fan-out navigation.
   */
  results?: NavigationResult[] | null;
  /**
   * Whether the overall navigation succeeded.
   */
  success: boolean;
  /**
   * Target state for this navigation branch (if applicable).
   */
  target_state?: string | null;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Result of querying currently active states in the state machine.
 */
interface ActiveStatesResult {
  /**
   * State IDs currently active.
   */
  active_states: string[];
  /**
   * The "primary" current state, when one can be singled out.
   */
  current_state?: string | null;
  /**
   * Error message when the query failed.
   */
  error?: string | null;
  /**
   * Ordered history of states the machine has passed through.
   */
  state_history?: string[] | null;
  /**
   * Whether the query succeeded.
   */
  success: boolean;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Information about a single available transition from the current state.
 */
interface TransitionInfo {
  /**
   * Source state ID.
   */
  from_state: string;
  /**
   * Transition ID.
   */
  id: string;
  /**
   * Destination state ID (`null` if the transition stays within the same
   * state set — e.g. a self-transition or a guard-only action).
   */
  to_state: string | null;
  /**
   * Names of workflows attached to this transition.
   */
  workflows: string[];
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Result of querying available transitions from the current state.
 */
interface AvailableTransitionsResult {
  /**
   * The state the transitions are taken from.
   */
  current_state?: string | null;
  /**
   * Error message when the query failed.
   */
  error?: string | null;
  /**
   * Human-readable message (e.g., "no transitions available").
   */
  message?: string | null;
  /**
   * Whether the query succeeded.
   */
  success: boolean;
  /**
   * Transitions available from the current state.
   */
  transitions: TransitionInfo[];
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Source of initial states configuration.
 *
 * - `Defaults`: states with `is_initial=true` in the state machine definition
 * - `Workflow`: initial states defined on the workflow (`initialStateIds`)
 * - `Override`: session-only override from the runner UI
 *
 * Serialized as a bare lowercase string (`"defaults"` / `"workflow"` /
 * `"override"`) to match the TS literal union.
 */
type InitialStatesSource = "defaults" | "workflow" | "override";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * A lightweight reference to a state by id and display name.
 *
 * Used in [`ResolvedInitialStates`] and [`ResolvedInitialStatesResult`] to
 * let the UI render human-readable lists without a separate lookup.
 */
interface InitialStateRef {
  /**
   * State ID.
   */
  id: string;
  /**
   * Display name.
   */
  name: string;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * The resolved set of initial states for a run, along with the source the
 * resolution came from.
 */
interface ResolvedInitialStates {
  source: InitialStatesSource;
  /**
   * Resolved initial state IDs.
   */
  stateIds: string[];
  /**
   * Display-ready references for each state (optional).
   */
  states?: InitialStateRef[] | null;
  /**
   * Workflow ID when `source == Workflow`.
   */
  workflowId?: string | null;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Result envelope for the "resolve initial states" operation.
 *
 * Unlike [`ResolvedInitialStates`], this shape is non-optional: `states` and
 * `workflowId` are always present (possibly empty), and a `success` / `error`
 * pair is provided.
 */
interface ResolvedInitialStatesResult {
  /**
   * Error message when resolution failed.
   */
  error?: string | null;
  source: InitialStatesSource;
  /**
   * Resolved initial state IDs.
   */
  stateIds: string[];
  /**
   * Display-ready references for each state.
   */
  states: InitialStateRef[];
  /**
   * Whether resolution succeeded.
   */
  success: boolean;
  /**
   * Workflow ID (may be empty).
   */
  workflowId: string;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Strategy used by the state-discovery pass.
 *
 * Serialized as a bare lowercase string.
 */
type DiscoveryStrategy = "auto" | "fingerprint";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Data passed to a transition edge in the ReactFlow graph editor.
 */
interface TransitionEdgeData {
  /**
   * Number of actions in this transition.
   */
  actionCount: number;
  /**
   * Distinct action types used.
   */
  actionTypes: StandardActionType[];
  /**
   * Target of the first action, for label rendering.
   */
  firstActionTarget?: string | null;
  /**
   * Whether the edge is highlighted (e.g., part of a path preview).
   */
  isHighlighted: boolean;
  /**
   * Display name.
   */
  name: string;
  /**
   * Path cost.
   */
  pathCost: number;
  /**
   * Whether source states stay visible after activation.
   */
  staysVisible: boolean;
  /**
   * Transition ID this edge represents.
   */
  transitionId: string;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Format for exporting a state machine config to JSON.
 *
 * Compatible with `UIBridgeRuntime.from_dict()` in the qontinui library. The
 * nested maps hold opaque per-state / per-transition / per-config dictionaries
 * because the exporter serializes implementation-specific fields not captured
 * by the DTO types.
 */
interface StateMachineExportFormat {
  /**
   * Config-level payload.
   */
  config: {
    [k: string]: unknown;
  };
  /**
   * State ID → state payload.
   */
  states: {
    [k: string]: {
      [k: string]: unknown;
    };
  };
  /**
   * Transition ID → transition payload.
   */
  transitions: {
    [k: string]: {
      [k: string]: unknown;
    };
  };
}

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
 *
 * Most wire types in this module are generated from Rust (source of truth:
 * qontinui-schemas/rust/src/state_machine.rs). Do not edit those by hand —
 * regenerate via `just generate-types` (or the runner's generate_types.sh).
 * A small number of UI-only sugar types remain hand-authored below.
 */

/**
 * Data passed to a state node in the ReactFlow graph editor.
 *
 * UI-only sugar: mirrors the generated `StateNodeData` but adds the
 * `onStartElementDrag` callback that the graph editor wires up at render time.
 */
interface StateNodeData {
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

export type { ActiveStatesResult, AvailableTransitionsResult, DiscoveryStrategy, DomainKnowledge, InitialStateRef, InitialStatesSource, MouseButton, NavigationResult, PathfindingRequest, PathfindingResult, PathfindingStep, Point, ResolvedInitialStates, ResolvedInitialStatesResult, ScrollDirection, StandardActionType, StateMachineConfig, StateMachineConfigCreate, StateMachineConfigFull, StateMachineConfigUpdate, StateMachineExportFormat, StateMachineState, StateMachineStateCreate, StateMachineStateUpdate, StateMachineTransition, StateMachineTransitionCreate, StateMachineTransitionUpdate, StateNodeData, TransitionAction, TransitionActionValue, TransitionEdgeData, TransitionExecutionResult, TransitionInfo };
