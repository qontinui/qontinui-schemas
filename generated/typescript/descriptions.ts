/**
 * Auto-generated TypeScript types from qontinui-schemas
 * DO NOT EDIT - regenerate with: poetry run python scripts/generate_typescript.py
 */

export interface StateDescription {
  /** Brief description of the state (1-2 sentences) */
  summary: string;
  /** UI elements that should be visible in this state (e.g., 'Login button', 'Username field') */
  expected_elements?: string[] | null;
  /** UI elements that should NOT be visible in this state - helps detect error dialogs or wrong states */
  unexpected_elements?: string[] | null;
  /** Business context - what the user is trying to accomplish when in this state */
  user_goal?: string | null;
  /** Custom hints for AI verification - specific things to check or look for */
  verification_prompt?: string | null;
}

export interface ActionDescription {
  /** What this action is supposed to accomplish (e.g., 'Click the submit button to proceed') */
  intent: string;
  /** Conditions that should be true before this action executes */
  preconditions?: string[] | null;
  /** Conditions that should be true after this action completes successfully */
  postconditions?: string[] | null;
  /** Known ways this action can fail (e.g., 'Button may be disabled', 'Network timeout') */
  failure_modes?: string[] | null;
}

export interface TransitionDescription {
  /** What this transition accomplishes (e.g., 'Navigate from login to dashboard') */
  intent: string;
  /** Conditions that must be true before this transition can occur */
  preconditions?: string[] | null;
  /** Conditions that should be true after this transition completes */
  postconditions?: string[] | null;
  /** Known ways this transition can fail (e.g., 'Authentication error', 'Server unavailable') */
  failure_modes?: string[] | null;
  /** Typical duration of this transition in milliseconds - helps detect performance issues */
  expected_duration_ms?: number | null;
}

export interface WorkflowDescription {
  /** Overall goal of the workflow (e.g., 'Complete user registration process') */
  purpose: string;
  /** How to determine if the workflow succeeded (e.g., 'User sees welcome message') */
  success_criteria: string;
  /** Human-readable summary of the workflow steps for quick understanding */
  steps_summary?: string[] | null;
  /** Why this workflow exists and its importance in the application */
  business_context?: string | null;
}
