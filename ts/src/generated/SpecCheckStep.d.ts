/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { RetrySpec } from "./RetrySpec";
import type { VerificationCategoryKind } from "./VerificationCategoryKind";

/**
 * Evaluate a page spec against the live UI of a registered app.
 *
 * Wire tag: `"spec_check"`.
 *
 * Not [`crate::spec_check::SpecCheckStepConfig`]: that is the camelCase,
 * `deny_unknown_fields` policy config with no app id. This is the step as the
 * workflow generator and the runner's `ExecutionStepConfig` carry it.
 */
export interface SpecCheckStep {
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
   * App id whose specs root resolves `spec_check_page_id`. Required at run
   * time (the handler refuses a missing one); optional on the wire.
   */
  specCheckAppId?: string | null;
  /**
   * Snapshot-fetch error kinds that fail the step (lower snake_case, e.g.
   * `"not_connected"`, `"timeout"`).
   */
  specCheckFailOn?: string[] | null;
  /**
   * Fail the step when the app is unreachable (default `true` on the
   * consumer side).
   */
  specCheckFailWhenNoApp?: boolean | null;
  /**
   * Fail the step when the page has no spec (default `true` on the
   * consumer side).
   */
  specCheckFailWhenNoSpec?: boolean | null;
  /**
   * Page id whose spec is evaluated.
   */
  specCheckPageId?: string | null;
  /**
   * AND-conjunct policy, carried verbatim; the handler reconstitutes the
   * typed policy.
   */
  specCheckPolicy?: {
    [k: string]: unknown;
  };
  /**
   * Verification depth category.
   */
  verificationCategory?: VerificationCategoryKind | null;
  [k: string]: unknown;
}
