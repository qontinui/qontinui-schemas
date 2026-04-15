/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { CheckType } from './CheckType';
import type { CommandMode } from './CommandMode';
import type { CommandStepPhase } from './CommandStepPhase';
import type { PlaywrightExecutionMode } from './PlaywrightExecutionMode';
import type { RetrySpec } from './RetrySpec';
import type { TestType } from './TestType';
import type { VerificationCategoryKind } from './VerificationCategoryKind';

/**
 * Shell commands, checks, check groups, and tests.
 *
 * A single variant covers all command-like steps; the specific sub-kind is
 * carried by [`CommandMode`] and the matching `*_id` / `*_type` fields.
 */
export interface CommandStep {
  /**
   * Whether to auto-fix during the check.
   */
  auto_fix?: boolean | null;
  /**
   * Branch selector for repository-targeted steps.
   */
  branch?: string | null;
  /**
   * Saved check-group ID.
   */
  check_group_id?: string | null;
  /**
   * Saved check definition ID.
   */
  check_id?: string | null;
  /**
   * Kind of deterministic check (for `check` / `check_group` modes).
   */
  check_type?: CheckType | null;
  /**
   * Inline code body (e.g., Python snippet).
   */
  code?: string | null;
  /**
   * Shell command line (for `shell` mode).
   */
  command?: string | null;
  /**
   * Path to the check's config file.
   */
  config_path?: string | null;
  /**
   * Acceptance criterion IDs verified by this step.
   */
  criterion_ids?: string[];
  /**
   * IDs of other steps that must complete first.
   */
  depends_on?: string[];
  /**
   * Execution mode for Playwright tests.
   */
  execution_mode?: PlaywrightExecutionMode | null;
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
   * Whether non-zero exit status fails the step.
   */
  fail_on_error?: boolean | null;
  /**
   * Fail the step on warnings in addition to errors.
   */
  fail_on_warning?: boolean | null;
  /**
   * Saved fused-script ID.
   */
  fused_script_id?: string | null;
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
   * Execution mode — which sub-kind of command step this is.
   */
  mode?: CommandMode | null;
  /**
   * Display name for the step.
   */
  name: string;
  phase: CommandStepPhase;
  /**
   * Repository selector for repository-targeted steps.
   */
  repository?: string | null;
  /**
   * Whether this step is required (default: `true` on consumer side).
   */
  required?: boolean | null;
  /**
   * Per-step retry configuration.
   */
  retry?: RetrySpec | null;
  /**
   * Re-run this step on every verification-agentic iteration.
   */
  run_on_subsequent_iterations?: boolean | null;
  /**
   * Inline script contents.
   */
  script_content?: string | null;
  /**
   * Saved script ID.
   */
  script_id?: string | null;
  /**
   * Saved shell command template ID.
   */
  shell_command_id?: string | null;
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
   * Target URL for navigation-style tests.
   */
  target_url?: string | null;
  /**
   * Saved test ID.
   */
  test_id?: string | null;
  /**
   * Test runner kind.
   */
  test_type?: TestType | null;
  /**
   * Timeout in seconds.
   */
  timeout_seconds?: number | null;
  /**
   * Tool identifier (e.g., `eslint`, `ruff`).
   */
  tool?: string | null;
  /**
   * Verification depth category.
   */
  verification_category?: VerificationCategoryKind | null;
  /**
   * Whether the caller waits for the workflow to complete.
   */
  wait_for_completion?: boolean | null;
  /**
   * Name of a workflow to invoke.
   */
  workflow_name?: string | null;
  /**
   * Working directory for the command.
   */
  working_directory?: string | null;
  [k: string]: unknown;
}
