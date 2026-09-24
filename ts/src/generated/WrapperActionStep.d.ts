/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { RetrySpec } from "./RetrySpec";
import type { VerificationCategoryKind } from "./VerificationCategoryKind";

/**
 * Dispatch a typed action through an installed wrapper.
 *
 * Wire tag: `"wrapper_action"`.
 *
 * Serializes with the Builder's keys (`wrapperId`, `actionId`, `params`,
 * `resultVariable` — what the runner frontend writes) and also accepts the
 * runner's `ExecutionStepConfig` names (`wrapper_action_id`, `wrapper_params`,
 * `wrapper_result_variable`).
 */
export interface WrapperActionStep {
  /**
   * Id of the action the wrapper exposes.
   */
  actionId?: string | null;
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
   * Unique identifier for the step. Absent or `null` reads as `""`.
   */
  id: string;
  /**
   * Named input bindings evaluated at step entry.
   */
  inputs?: {
    [k: string]: string;
  };
  /**
   * Display name for the step. Absent or `null` reads as `""`.
   */
  name: string;
  /**
   * Params passed to the action; values may carry `{{ variable }}`
   * templates resolved at run time.
   */
  params?: {
    [k: string]: unknown;
  };
  /**
   * Whether this step is required (default: `true` on consumer side).
   */
  required?: boolean | null;
  /**
   * Workflow variable the dispatch result is written to (empty / absent =
   * not stored).
   */
  resultVariable?: string | null;
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
   * Verification depth category.
   */
  verificationCategory?: VerificationCategoryKind | null;
  /**
   * Id of the installed wrapper to dispatch through.
   */
  wrapperId?: string | null;
  [k: string]: unknown;
}
