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
  autoFix?: boolean | null;
  /**
   * Branch selector for repository-targeted steps.
   */
  branch?: string | null;
  /**
   * Saved check-group ID.
   */
  checkGroupId?: string | null;
  /**
   * Saved check definition ID.
   */
  checkId?: string | null;
  /**
   * Kind of deterministic check (for `check` / `check_group` modes).
   */
  checkType?: CheckType | null;
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
  configPath?: string | null;
  /**
   * Acceptance criterion IDs verified by this step.
   */
  criterionIds?: string[];
  /**
   * IDs of other steps that must complete first.
   */
  dependsOn?: string[];
  /**
   * Execution mode for Playwright tests.
   */
  executionMode?: PlaywrightExecutionMode | null;
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
   * Whether non-zero exit status fails the step.
   */
  failOnError?: boolean | null;
  /**
   * Fail the step on warnings in addition to errors.
   */
  failOnWarning?: boolean | null;
  /**
   * Saved fused-script ID.
   */
  fusedScriptId?: string | null;
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
  runOnSubsequentIterations?: boolean | null;
  /**
   * Inline script contents.
   */
  scriptContent?: string | null;
  /**
   * Saved script ID.
   */
  scriptId?: string | null;
  /**
   * Saved shell command template ID.
   */
  shellCommandId?: string | null;
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
   * Target URL for navigation-style tests.
   */
  targetUrl?: string | null;
  /**
   * Saved test ID.
   */
  testId?: string | null;
  /**
   * Test runner kind.
   */
  testType?: TestType | null;
  /**
   * Timeout in seconds.
   */
  timeoutSeconds?: number | null;
  /**
   * Tool identifier (e.g., `eslint`, `ruff`).
   */
  tool?: string | null;
  /**
   * Verification depth category.
   */
  verificationCategory?: VerificationCategoryKind | null;
  /**
   * Whether the caller waits for the workflow to complete.
   */
  waitForCompletion?: boolean | null;
  /**
   * Name of a workflow to invoke.
   */
  workflowName?: string | null;
  /**
   * Working directory for the command.
   */
  workingDirectory?: string | null;
  [k: string]: unknown;
}
