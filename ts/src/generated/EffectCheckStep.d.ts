/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { RetrySpec } from "./RetrySpec";
import type { VerificationCategoryKind } from "./VerificationCategoryKind";

/**
 * Perform an action on an element and classify the observed effect against
 * its effect signature (effect calculus).
 *
 * Wire tag: `"effect_check"`.
 */
export interface EffectCheckStep {
  /**
   * Acceptance criterion IDs verified by this step.
   */
  criterionIds?: string[];
  /**
   * IDs of other steps that must complete first.
   */
  dependsOn?: string[];
  /**
   * Action to perform (e.g. `"click"`, `"type"`). Required at run time.
   */
  effectCheckAction?: string | null;
  /**
   * Element id to act on. Required at run time; optional on the wire.
   */
  effectCheckElementId?: string | null;
  /**
   * Expected outcome: `"Confirmed"`, `"Surprise"`, `"Failure"`,
   * `"Contradiction"` or `"Partial"`.
   */
  effectCheckExpectedOutcome?: string | null;
  /**
   * Action-specific params forwarded verbatim to the SDK action endpoint.
   */
  effectCheckParams?: {
    [k: string]: unknown;
  };
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
   * Verification depth category.
   */
  verificationCategory?: VerificationCategoryKind | null;
  [k: string]: unknown;
}
