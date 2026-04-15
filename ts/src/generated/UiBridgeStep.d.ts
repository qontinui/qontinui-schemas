/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { RetrySpec } from './RetrySpec';
import type { UiBridgeAction } from './UiBridgeAction';
import type { UiBridgeAssertType } from './UiBridgeAssertType';
import type { UiBridgeComparisonMode } from './UiBridgeComparisonMode';
import type { UiBridgeSeverity } from './UiBridgeSeverity';
import type { UiBridgeStepPhase } from './UiBridgeStepPhase';
import type { VerificationCategoryKind } from './VerificationCategoryKind';

/**
 * UI Bridge SDK interaction — navigate, execute, assert, snapshot, compare.
 */
export interface UiBridgeStep {
  action: UiBridgeAction;
  /**
   * Structured action plan (for `action_plan`).
   *
   * Typed as `serde_json::Value` here to avoid pulling the `action-plan`
   * module into this crate; the TS side re-imports the typed `ActionPlan`
   * after regeneration.
   */
  action_plan?: {
    [k: string]: unknown;
  };
  /**
   * Assertion kind (for `assert`).
   */
  assert_type?: UiBridgeAssertType | null;
  /**
   * Comparison mode (for `compare` / `snapshot_assert`).
   */
  comparison_mode?: UiBridgeComparisonMode | null;
  /**
   * Acceptance criterion IDs verified by this step.
   */
  criterion_ids?: string[];
  /**
   * IDs of other steps that must complete first.
   */
  depends_on?: string[];
  /**
   * Expected value for assertions.
   */
  expected?: string | null;
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
   * Free-form instruction text (for `execute`).
   */
  instruction?: string | null;
  /**
   * Display name for the step.
   */
  name: string;
  phase: UiBridgeStepPhase;
  /**
   * Reference snapshot ID (for `compare` / `snapshot_assert`).
   */
  reference_snapshot_id?: string | null;
  /**
   * Whether this step is required (default: `true` on consumer side).
   */
  required?: boolean | null;
  /**
   * Per-step retry configuration.
   */
  retry?: RetrySpec | null;
  /**
   * Severity threshold (for `compare` / `snapshot_assert`).
   */
  severity_threshold?: UiBridgeSeverity | null;
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
   * Target selector or element ID.
   */
  target?: string | null;
  /**
   * Timeout in milliseconds.
   */
  timeout_ms?: number | null;
  /**
   * Snapshot target — `"control"`, `"sdk"`, or `"proxy:PORT"`.
   */
  ui_bridge_snapshot_target?: string | null;
  /**
   * Navigation URL (for `navigate`).
   */
  url?: string | null;
  /**
   * Verification depth category.
   */
  verification_category?: VerificationCategoryKind | null;
  [k: string]: unknown;
}
