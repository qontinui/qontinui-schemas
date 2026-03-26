/**
 * Structured Action Plan Types
 *
 * Defines the schema for LLM-generated UI Bridge action plans.
 * Instead of natural language instructions interpreted by a second LLM call,
 * action plans let the agentic-phase LLM directly specify typed UI actions
 * that map to UI Bridge's control API.
 *
 * Inspired by Skyvern's structured action protocol: each action carries
 * reasoning, confidence, and typed parameters so that execution is
 * deterministic and auditable.
 */

// =============================================================================
// Action Types (mirrors UI Bridge StandardAction + extensions)
// =============================================================================

/**
 * Action types that can appear in an action plan.
 * Maps directly to UI Bridge's StandardAction type with additions
 * for navigation and waiting.
 */
export type PlannedActionType =
  | "click"
  | "doubleClick"
  | "rightClick"
  | "type"
  | "clear"
  | "select"
  | "check"
  | "uncheck"
  | "toggle"
  | "hover"
  | "focus"
  | "scroll"
  | "scrollIntoView"
  | "setValue"
  | "drag"
  | "submit"
  | "sendKeys"
  | "autocomplete"
  | "navigate"
  | "wait";

// =============================================================================
// Element Resolution
// =============================================================================

/**
 * How to find the target element for an action.
 * Supports multiple resolution strategies with fallback.
 */
export interface ElementTarget {
  /** Direct element ID from a prior snapshot (fastest resolution) */
  elementId?: string;
  /** data-testid attribute value */
  testId?: string;
  /** Natural language description for fuzzy search (fallback) */
  searchText?: string;
  /** Element type hint to narrow search (e.g., "button", "input") */
  elementType?: string;
  /** CSS selector */
  selector?: string;
}

// =============================================================================
// Action Plan Entry
// =============================================================================

/**
 * A single action in an action plan.
 *
 * The LLM produces an array of these, each mapping to one UI Bridge
 * control action. The reasoning and confidence fields enable auditing
 * and confidence-gated execution.
 */
export interface PlannedAction {
  /** Action type to execute */
  action: PlannedActionType;

  /** How to find the target element */
  target: ElementTarget;

  /** LLM's reasoning for this action (audit trail) */
  reasoning?: string;

  /** LLM's confidence that this is the right action (0.0–1.0) */
  confidence: number;

  /**
   * Action-specific parameters.
   * - type/setValue: { text: string }
   * - select: { value: string } or { label: string }
   * - scroll: { direction: "up"|"down"|"left"|"right" }
   * - drag: { targetPosition: { x: number, y: number } }
   * - sendKeys: { keys: string }
   * - navigate: { url: string }
   * - wait: { ms: number }
   */
  params?: Record<string, unknown>;

  /**
   * Separates the generic intent from specific data.
   * Enables action plan caching: the query stays stable across runs,
   * only the answer changes per user context.
   *
   * Example:
   *   userDetailQuery: "What email should be entered?"
   *   userDetailAnswer: "test@example.com"
   */
  userDetailQuery?: string;
  userDetailAnswer?: string;
}

// =============================================================================
// Action Plan
// =============================================================================

/**
 * A complete action plan: an ordered sequence of UI actions
 * produced by the agentic-phase LLM.
 */
export interface ActionPlan {
  /** Ordered list of actions to execute */
  actions: PlannedAction[];

  /** High-level goal this plan achieves */
  goal: string;

  /**
   * Minimum confidence threshold. Actions below this confidence
   * are skipped (or trigger verification) rather than executed.
   * Default: 0.5
   */
  confidenceThreshold?: number;

  /** Whether to stop on first action failure (default: true) */
  stopOnFailure?: boolean;
}

// =============================================================================
// Action Plan Result
// =============================================================================

/** Result of executing a single planned action */
export interface PlannedActionResult {
  /** Index of the action in the plan */
  index: number;

  /** Whether the action succeeded */
  success: boolean;

  /** The action that was executed */
  action: PlannedActionType;

  /** Element ID that was resolved and acted upon */
  resolvedElementId?: string;

  /** Error message if the action failed */
  error?: string;

  /** Whether the action was skipped due to low confidence */
  skippedLowConfidence?: boolean;

  /** Duration in milliseconds */
  durationMs: number;

  /** Post-action element state (if available) */
  elementState?: Record<string, unknown>;
}

/** Aggregated result of executing a full action plan */
export interface ActionPlanResult {
  /** Whether all executed actions succeeded */
  success: boolean;

  /** The goal from the action plan */
  goal?: string;

  /** Per-action results */
  results: PlannedActionResult[];

  /** Count of actions executed (excludes skipped) */
  executedCount: number;

  /** Count of actions skipped due to low confidence */
  skippedCount: number;

  /** Count of actions that failed */
  failedCount: number;

  /** Total duration in milliseconds */
  totalDurationMs: number;

  /** Whether this plan was stored in the cache for future reuse */
  cached?: boolean;
}

/**
 * Extended action plan request with caching fields.
 * Used when calling the endpoint directly (not via workflow steps).
 */
export interface ActionPlanExecuteRequest extends ActionPlan {
  /** Page URL for cache keying */
  pageUrl?: string;
  /** Element snapshot for cache fingerprinting (array of {id, type, role, label}) */
  elementSnapshot?: Array<{ id?: string; type?: string; role?: string; label?: string }>;
}
