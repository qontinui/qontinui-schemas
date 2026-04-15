/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { RetrySpec } from './RetrySpec';
import type { VerificationCategoryKind } from './VerificationCategoryKind';
import type { WorkflowFixupMode } from './WorkflowFixupMode';

/**
 * Run deterministic Rust fixups on a workflow JSON file.
 *
 * Wire tag: `"workflow_fixup"`.
 */
export interface WorkflowFixupStep {
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
   * Path to criteria JSON file (for `validate_criteria` mode).
   */
  fixup_criteria_path?: string | null;
  /**
   * Path to the workflow JSON file to fix (supports `{{artifact_dir}}`).
   */
  fixup_input_path?: string | null;
  /**
   * Fixup mode.
   */
  fixup_mode?: WorkflowFixupMode | null;
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
