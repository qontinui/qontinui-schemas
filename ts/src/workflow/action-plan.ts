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

// The RESPONSE side is generated from the runner's
// `src-tauri/src/ui_bridge_action_plan.rs` (what the action-plan endpoint
// serializes); do not edit by hand. `action` is the request's action string
// echoed back, so it is a plain `string` there, and the defaulted
// `skippedLowConfidence` / `cached` are always present on the wire.
//
// The REQUEST side above stays hand-authored: the runner's request structs are
// deserialize-only with `#[serde(default)]` on every optional field, and the
// codegen promotes a defaulted field to required — correct for a value the
// runner emits, wrong for one a caller (or an LLM) writes.

/** Result of executing a single planned action */
export type { PlannedActionResult } from "../generated/PlannedActionResult";

/** Aggregated result of executing a full action plan */
export type { ActionPlanResult } from "../generated/ActionPlanResult";

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
