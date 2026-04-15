/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { A11yAction } from './A11yAction';
import type { RetrySpec } from './RetrySpec';
import type { VerificationCategoryKind } from './VerificationCategoryKind';

/**
 * Interact with native UI elements via the accessibility layer (UIA/AT-SPI/AX).
 *
 * Wire tag: `"native_accessibility"`.
 */
export interface NativeAccessibilityStep {
  /**
   * Action to perform.
   */
  action?: A11yAction & string;
  /**
   * Whether to clear existing text before typing.
   */
  clear_first?: boolean | null;
  /**
   * Acceptance criterion IDs verified by this step.
   */
  criterion_ids?: string[];
  /**
   * IDs of other steps that must complete first.
   */
  depends_on?: string[];
  /**
   * Extractions published to subsequent steps.
   */
  extract?: {
    [k: string]: string;
  };
  /**
   * If `Some(true)`, a console-error signal from the UI fails this step.
   */
  fail_on_console_errors?: boolean | null;
  /**
   * Unique identifier for the step.
   */
  id: string;
  /**
   * Include hidden elements in `capture`.
   */
  include_hidden?: boolean | null;
  /**
   * Named input bindings evaluated at step entry.
   */
  inputs?: {
    [k: string]: string;
  };
  /**
   * Only include interactive elements (for `query` and `ai_context`).
   */
  interactive_only?: boolean | null;
  /**
   * Maximum tree depth for `capture`.
   */
  max_depth?: number | null;
  /**
   * Maximum elements for `ai_context` action (default: 50).
   */
  max_elements?: number | null;
  /**
   * Display name for the step.
   */
  name: string;
  /**
   * Label filter for `query` action.
   */
  query_label?: string | null;
  /**
   * Role filter for `query` action (e.g. `"button"`, `"textbox"`).
   */
  query_role?: string | null;
  /**
   * Element ref ID for click/type/focus (e.g. `"@e3"`).
   */
  ref_id?: string | null;
  /**
   * Whether this step is required (default: `true` on consumer side).
   */
  required?: boolean | null;
  /**
   * Per-step retry configuration.
   */
  retry?: RetrySpec | null;
  /**
   * Provenance of this step when generated from a skill template.
   *
   * Typed as `serde_json::Value` here to avoid pulling the `skill`
   * dependency chain into this module; the TS side re-imports the typed
   * `SkillOrigin` after regeneration.
   */
  skill_origin?: {
    [k: string]: unknown;
  };
  /**
   * Connection target: `"Desktop"`, window title, or `"pid:1234"`.
   */
  target?: string | null;
  /**
   * Text to type (for `type` action).
   */
  text?: string | null;
  /**
   * Timeout in seconds.
   */
  timeout_seconds?: number | null;
  /**
   * Verification depth category.
   */
  verification_category?: VerificationCategoryKind | null;
  [k: string]: unknown;
}
