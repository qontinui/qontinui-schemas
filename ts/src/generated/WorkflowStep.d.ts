/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { RetrySpec } from './RetrySpec';
import type { VerificationCategoryKind } from './VerificationCategoryKind';
import type { WorkflowStepPhase } from './WorkflowStepPhase';

/**
 * Run a saved workflow inline (composition).
 */
export interface WorkflowStep {
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
   * Display name for the step.
   */
  name: string;
  phase: WorkflowStepPhase;
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
  /**
   * ID of the saved workflow to run.
   */
  workflow_id: string;
  /**
   * Display name of the saved workflow (denormalized for UI).
   */
  workflow_name: string;
  [k: string]: unknown;
}
