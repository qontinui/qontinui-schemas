/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { RetrySpec } from './RetrySpec';
import type { VerificationCategoryKind } from './VerificationCategoryKind';

/**
 * Run a saved workflow inline with input variable substitution.
 *
 * Wire tag: `"workflow_ref"`.
 */
export interface WorkflowRefStep {
  /**
   * Acceptance criterion IDs verified by this step.
   */
  criterionIds?: string[];
  /**
   * IDs of other steps that must complete first.
   */
  dependsOn?: string[];
  /**
   * Extractions published to subsequent steps.
   */
  extract?: {
    [k: string]: string;
  };
  /**
   * If `Some(true)`, a console-error signal from the UI fails this step.
   */
  failOnConsoleErrors?: boolean | null;
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
  /**
   * Whether to inherit model overrides from the parent context (default: `true`).
   */
  refInheritModelOverrides?: boolean | null;
  /**
   * Input variables substituted into the child workflow's prompt.
   */
  refWorkflowInputs?: {
    [k: string]: string;
  } | null;
  /**
   * Display name of the referenced workflow (denormalized for UI).
   */
  refWorkflowName?: string | null;
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
  skillOrigin?: {
    [k: string]: unknown;
  };
  /**
   * Timeout in seconds.
   */
  timeoutSeconds?: number | null;
  /**
   * Verification depth category.
   */
  verificationCategory?: VerificationCategoryKind | null;
  /**
   * ID of the saved workflow to run.
   */
  workflowId?: string | null;
  [k: string]: unknown;
}
