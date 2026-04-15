/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { PromptStepPhase } from './PromptStepPhase';
import type { RetrySpec } from './RetrySpec';
import type { VerificationCategoryKind } from './VerificationCategoryKind';

/**
 * AI task instructions (prompt).
 */
export interface PromptStep {
  /**
   * Prompt body.
   */
  content: string;
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
   * Named input bindings evaluated at step entry.
   */
  inputs?: {
    [k: string]: string;
  };
  /**
   * Marks this prompt as the summary step at the end of completion.
   */
  is_summary_step?: boolean | null;
  /**
   * Model override.
   */
  model?: string | null;
  /**
   * Display name for the step.
   */
  name: string;
  phase: PromptStepPhase;
  /**
   * Saved prompt ID (when the body is a reference).
   */
  prompt_id?: string | null;
  /**
   * AI provider override.
   */
  provider?: string | null;
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
   * Verification depth category.
   */
  verification_category?: VerificationCategoryKind | null;
  [k: string]: unknown;
}
