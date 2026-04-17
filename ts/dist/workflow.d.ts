/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Log source selection for a workflow.
 *
 * - `"default"` / `"ai"` / `"all"`: Use the corresponding [`LogSourceMode`].
 * - `{ "profile_id": "..." }`: Use a specific profile.
 *
 * Serialized as an untagged union: a bare string for the mode variants, or an
 * object with a `profile_id` key for the profile variant.
 */
type LogSourceSelection =
  | ("default" | "ai" | "all")
  | {
      /**
       * ID of the log-source profile to use.
       */
      profile_id: string;
      [k: string]: unknown;
    };

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Configuration for a single health check URL.
 *
 * A workflow can have zero or more of these; they run before verification to
 * confirm required services are up.
 */
interface HealthCheckUrl {
  /**
   * Expected HTTP status code (default: `200`).
   */
  expected_status: number;
  /**
   * Whether failure should stop the workflow (default: `true`).
   */
  is_critical: boolean;
  /**
   * Display name for the health check (e.g., `"Backend Server"`).
   */
  name: string;
  /**
   * Timeout in seconds (default: `30`).
   */
  timeout_seconds: number;
  /**
   * URL to check (e.g., `"http://localhost:8000/health"`).
   */
  url: string;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * A conditional routing rule that selects model/provider based on runtime
 * context.
 *
 * Rules are evaluated in order; the first matching rule wins. Condition
 * syntax: `"<variable> <op> <value>"` where:
 * - Variables: `verification_failures`, `iteration`, `stage_index`
 * - Operators: `>=`, `>`, `<=`, `<`, `==`, `!=`
 */
interface RoutingRule {
  /**
   * Condition expression, e.g. `"verification_failures >= 2"`.
   */
  condition: string;
  /**
   * Max tokens override when this rule matches.
   */
  max_tokens?: number | null;
  /**
   * Model to use when this rule matches.
   */
  model?: string | null;
  /**
   * Provider to use when this rule matches.
   */
  provider?: string | null;
  /**
   * Temperature override when this rule matches.
   */
  temperature?: number | null;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Per-phase model override configuration.
 *
 * Each phase can independently specify a provider and/or model, along with
 * optional temperature, max_tokens, fallback config, and conditional routing
 * rules.
 */
interface ModelOverrideConfig {
  /**
   * Fallback model if the primary fails with a retryable error.
   */
  fallback_model?: string | null;
  /**
   * Fallback provider if the primary fails with a retryable error.
   */
  fallback_provider?: string | null;
  /**
   * Max output tokens override for this phase.
   */
  max_tokens?: number | null;
  /**
   * Model override for this phase.
   */
  model?: string | null;
  /**
   * Provider override for this phase.
   */
  provider?: string | null;
  /**
   * Conditional routing rules evaluated at runtime. First matching rule
   * wins; unmatched falls back to this config's static fields.
   */
  routing_rules?: RoutingRule[] | null;
  /**
   * Temperature override for this phase (`0.0`–`1.0`).
   */
  temperature?: number | null;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Condition for conditional stage execution.
 *
 * When attached to a [`WorkflowStage`], the stage is skipped if the condition
 * evaluates to "should skip". All condition fields are optional and combine
 * with AND semantics — all specified conditions must be met for the stage to
 * run.
 */
interface StageCondition {
  /**
   * Run this stage only if the previous stage had this outcome.
   *
   * - `"passed"`: run only if previous stage verification passed
   * - `"failed"`: run only if previous stage verification failed
   * - `"any"`: always run regardless of previous outcome (default behavior)
   */
  if_previous?: string | null;
  /**
   * Skip this stage if the total number of failed stages so far is below
   * this threshold. Useful for "recovery" stages that only run when
   * multiple prior stages have failed.
   */
  min_failures?: number | null;
  /**
   * Run this stage only after this many loop iterations have occurred
   * (across all stages). Useful for "escalation" stages that only kick in
   * after initial attempts have failed.
   */
  min_iteration?: number | null;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Retry policy for a step or stage.
 */
interface RetryPolicy {
  /**
   * Whether to use exponential backoff.
   */
  backoff: boolean;
  /**
   * Number of retry attempts (`0` = no retries).
   */
  count: number;
  /**
   * Delay between retries in milliseconds.
   */
  delay_ms: number;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * An output declared by a stage, available to subsequent stages.
 */
interface StageOutput {
  /**
   * Human-readable description.
   */
  description: string;
  /**
   * Unique key for this output (e.g. `"api_url"`, `"auth_token"`).
   */
  key: string;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * An input required by a stage, referencing a prior stage's output.
 */
interface StageInput {
  /**
   * Which stage provides this input (stage id). If omitted, searches all
   * prior stages.
   */
  from_stage?: string | null;
  /**
   * The key to bind this input to (matches a [`StageOutput::key`] from a
   * prior stage).
   */
  key: string;
  /**
   * Whether this input is required (default: `true`). Missing required
   * inputs are Critical findings.
   */
  required: boolean;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Kinds of deterministic check surfaced by `CommandStep` when
 * `mode = "check"` or `mode = "check_group"`.
 */
type CheckType =
  | "lint"
  | "format"
  | "typecheck"
  | "analyze"
  | "security"
  | "custom_command"
  | "http_status"
  | "ai_review"
  | "ci_cd";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Execution mode for a [`CommandStep`].
 */
type CommandMode = "shell" | "check" | "check_group" | "test";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Phases in which a [`CommandStep`] may appear.
 */
type CommandStepPhase = "setup" | "verification" | "completion";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Whether a Playwright test executes independently or as part of a chain.
 */
type PlaywrightExecutionMode = "independent" | "chained";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Per-step retry configuration carried by [`BaseStepFields::retry`].
 *
 * Distinct from the workflow frame's `RetryPolicy` — that one also carries a
 * `backoff` flag, this per-step form is the older, simpler shape that step
 * DTOs inherited from the TS `BaseStep` interface.
 */
interface RetrySpec {
  /**
   * Number of retry attempts (`0` = no retries).
   */
  count: number;
  /**
   * Delay between retries in milliseconds.
   */
  delay_ms: number;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Test-runner kinds surfaced by `CommandStep` when `mode = "test"`.
 */
type TestType = "playwright" | "qontinui_vision" | "python" | "repository" | "custom_command";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Verification-depth category for a step.
 *
 * Mirrors the TS `VerificationCategory` literal union. Kept local to this
 * module because it is only referenced from [`BaseStepFields`].
 */
type VerificationCategoryKind =
  | "existence"
  | "uniqueness"
  | "referential_integrity"
  | "semantic_correctness"
  | "runtime_behavior";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Shell commands, checks, check groups, and tests.
 *
 * A single variant covers all command-like steps; the specific sub-kind is
 * carried by [`CommandMode`] and the matching `*_id` / `*_type` fields.
 */
interface CommandStep {
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

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Phases in which a [`PromptStep`] may appear.
 *
 * Prompt steps are the only variant that may appear in the agentic phase.
 */
type PromptStepPhase = "setup" | "verification" | "agentic" | "completion";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * AI task instructions (prompt).
 */
interface PromptStep {
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

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * UI Bridge action kind.
 */
type UiBridgeAction =
  | "navigate"
  | "execute"
  | "assert"
  | "snapshot"
  | "compare"
  | "snapshot_assert"
  | "action_plan";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Kinds of assertion supported by `assert` actions.
 */
type UiBridgeAssertType = "exists" | "text_equals" | "contains" | "visible" | "enabled";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Comparison mode for `compare` / `snapshot_assert` actions.
 */
type UiBridgeComparisonMode = "structural" | "visual" | "both";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Severity threshold for `compare` / `snapshot_assert` actions.
 */
type UiBridgeSeverity = "critical" | "major" | "minor" | "info";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Phases in which a [`UiBridgeStep`] may appear.
 *
 * UI-bridge interactions only run in deterministic phases — never inside
 * the agentic loop (where the AI drives steps directly via prompts).
 */
type UiBridgeStepPhase = "setup" | "verification" | "completion";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * UI Bridge SDK interaction — navigate, execute, assert, snapshot, compare.
 */
interface UiBridgeStep {
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

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Phases in which a [`WorkflowStep`] may appear.
 */
type WorkflowStepPhase = "setup" | "verification" | "completion";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Run a saved workflow inline (composition).
 */
interface WorkflowStep {
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

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Discriminated union over the four canonical step variants.
 *
 * Wire format is a flat object with a `"type"` discriminator — serde's
 * internal tagging merges the inner struct's fields (including the flattened
 * [`BaseStepFields`]) up into the top-level object. Example:
 *
 * ```text
 * {"type":"command","id":"s1","name":"build","phase":"setup","mode":"shell","command":"cargo build"}
 * ```
 *
 * Consumers that want a strict 4-variant typed view should use
 * [`CanonicalStep`]. Consumers that need to tolerate runner-specific step
 * types (e.g. `gate`, `screenshot`, `playwright`, `state`, `action`,
 * `log_watch`, and others dispatched by the runner but absent from the
 * wire-contract surface) should use [`UnifiedStep`], which preserves
 * unknown payloads verbatim as `serde_json::Value`.
 *
 * Variant sizes are similar (~200–672 bytes each); the asymmetry reflects
 * real differences in step-field cardinality and doesn't warrant boxing.
 */
type CanonicalStep =
  | (CommandStep & {
      type: "command";
    })
  | (PromptStep & {
      type: "prompt";
    })
  | (UiBridgeStep & {
      type: "ui_bridge";
    })
  | (WorkflowStep & {
      type: "workflow";
    });

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * A workflow step, preferring typed canonical variants and falling back to an
 * opaque [`serde_json::Value`] for runner-specific types not yet part of the
 * wire contract.
 *
 * Serialization is transparent: a [`CanonicalStep`] serializes with its flat
 * `"type"`-tagged shape; [`UnifiedStep::Other`] serializes the wrapped value
 * as-is. Deserialization tries the canonical shape first; any payload that
 * does not match (unknown `"type"`, missing fields, or missing discriminator)
 * is preserved as [`UnifiedStep::Other`].
 *
 * This catch-all is what makes the type *robust* on the wire: a runner can
 * emit a `{"type":"gate", ...}` step and a consumer using [`UnifiedStep`]
 * will round-trip it losslessly even though `gate` is not in the canonical
 * set.
 *
 * ## Layout note (`#[allow(large_enum_variant)]`)
 *
 * `Canonical` carries a [`CanonicalStep`] (~672 bytes) while `Other` carries
 * a [`serde_json::Value`] (~32 bytes). The size asymmetry is intentional and
 * the enum is not held in bulk by any hot path today — `UnifiedWorkflow.*_steps`
 * remains `Vec<serde_json::Value>`. Boxing `Canonical` would save stack space
 * in hypothetical dense `Vec<UnifiedStep>` consumers at the cost of an extra
 * heap allocation per step everywhere else and noisier pattern-matching.
 */
type UnifiedStep =
  | CanonicalStep
  | {
      [k: string]: unknown;
    };

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * A workflow stage — a self-contained unit of execution with its own
 * setup / verification / agentic / completion steps and verification-agentic
 * loop.
 *
 * Multi-stage workflows execute stages sequentially. Each stage gets its own
 * verification-agentic loop, and later stages see full output from all prior
 * stages. Step arrays are opaque `serde_json::Value` payloads pending the
 * Wave 4 typed-step migration.
 */
interface WorkflowStage {
  /**
   * Agentic phase steps for this stage.
   */
  agentic_steps: UnifiedStep[];
  /**
   * Whether to pause for human approval after each agentic phase.
   */
  approval_gate: boolean;
  /**
   * When true, run completion prompt steps BEFORE automation steps.
   * Default (`false`) runs automation first, then prompts.
   */
  completion_prompts_first: boolean;
  /**
   * Completion phase steps for this stage.
   */
  completion_steps: UnifiedStep[];
  /**
   * Optional condition for stage execution. When set, the stage is
   * evaluated against this condition before running. If the condition is
   * not met, the stage is skipped.
   */
  condition?: StageCondition | null;
  /**
   * Description of what this stage does.
   */
  description: string;
  /**
   * Unique identifier (UUID v4).
   */
  id: string;
  /**
   * Inputs required from prior stages.
   */
  inputs?: StageInput[] | null;
  /**
   * Maximum iterations for this stage's verification-agentic loop.
   *
   * `None` (omitted in JSON) means no iteration cap — the loop terminates
   * on success, explicit stop, or fix-attempt exhaustion.
   */
  max_iterations?: number | null;
  /**
   * Model override for this stage.
   */
  model?: string | null;
  /**
   * Per-phase model overrides for this stage.
   */
  model_overrides?: {
    [k: string]: ModelOverrideConfig;
  };
  /**
   * Display name for this stage.
   */
  name: string;
  /**
   * Declared outputs that this stage produces for downstream stages.
   */
  outputs?: StageOutput[] | null;
  /**
   * AI provider override for this stage.
   */
  provider?: string | null;
  /**
   * Retry policy for this stage (overrides per-step defaults).
   */
  retry_policy?: RetryPolicy | null;
  /**
   * Setup phase steps for this stage (polymorphic; see module docs).
   */
  setup_steps: UnifiedStep[];
  /**
   * Optional inactivity timeout in seconds for this stage's AI sessions.
   */
  timeout_seconds?: number | null;
  /**
   * Verification phase steps for this stage.
   */
  verification_steps: UnifiedStep[];
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * The workflow execution architecture to use.
 *
 * This is a first-class workflow architecture option, allowing direct
 * comparison between traditional deterministic verification and agentic
 * verification approaches. Mirrors the runner-side enum in
 * `crate::agentic_verification`.
 */
type WorkflowArchitecture = "traditional" | "agentic_verification" | "multi_agent_pipeline";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * A unified workflow with steps organized by phase.
 *
 * The "frame" carries all non-step metadata — iteration caps, provider/model
 * selection, log-source routing, health checks, stage list, generator
 * outputs (dependency graph, cost annotations, quality report, acceptance
 * criteria), and timestamps. Step arrays remain opaque until the Wave 4
 * typed-step migration lands.
 */
interface UnifiedWorkflow {
  /**
   * Acceptance criteria from the specification agent (opaque JSON blob).
   *
   * Used by the canvas panel manager to show a live requirements tracker.
   */
  acceptance_criteria?: {
    [k: string]: unknown;
  };
  /**
   * Agentic phase steps (polymorphic JSON array).
   */
  agentic_steps: UnifiedStep[];
  /**
   * Whether the AI semantic review actually ran successfully during
   * generation.
   *
   * When `false`, the workflow passed through the pipeline without AI
   * verification (e.g., all verification iterations failed at
   * infrastructure level).
   */
  ai_reviewed: boolean;
  /**
   * Whether to pause for human approval after each agentic phase.
   */
  approval_gate: boolean;
  /**
   * Whether to auto-include contexts based on task mentions (default:
   * `true`).
   */
  auto_include_contexts: boolean;
  /**
   * Category for organization.
   */
  category: string;
  /**
   * When `true`, run completion prompt steps BEFORE automation steps.
   *
   * Used by meta-workflows so the AI hardener runs before
   * `save_workflow_artifact`. Default (`false`) runs automation first,
   * then prompts.
   */
  completion_prompts_first: boolean;
  /**
   * Completion phase steps (polymorphic JSON array) — runs once after the
   * verification loop exits.
   */
  completion_steps: UnifiedStep[];
  /**
   * Per-constraint overrides: map of `constraint_id` to enabled (`true`) /
   * disabled (`false`).
   *
   * Applied to the constraint engine at execution time, after loading
   * builtins and config.
   */
  constraint_overrides?: {
    [k: string]: boolean;
  };
  /**
   * Manually added context IDs.
   */
  context_ids?: string[];
  /**
   * Cost annotations computed during generation (opaque JSON blob).
   */
  cost_annotations?: {
    [k: string]: unknown;
  };
  /**
   * ISO 8601 timestamp of creation.
   */
  created_at: string;
  /**
   * Dependency graph computed during generation (opaque JSON blob).
   */
  dependency_graph?: {
    [k: string]: unknown;
  };
  /**
   * Description of what this workflow does.
   */
  description: string;
  /**
   * Disabled context IDs (excluded from auto-include).
   */
  disabled_context_ids?: string[];
  /**
   * Whether to run a completion sweep after verification passes.
   *
   * The sweep reviews all completed work for gaps before proceeding to
   * completion.
   */
  enable_sweep: boolean;
  /**
   * When `true`, the pipeline will stop execution if accumulated token
   * usage exceeds the token budget. Disabled by default — only logs
   * warnings.
   */
  enforce_token_budget: boolean;
  /**
   * Flow control configuration as a JSON string (e.g., concurrency limits,
   * queue behavior).
   */
  flow_control_json?: string | null;
  /**
   * Task run ID that generated this workflow (for meta-workflow tracking).
   */
  generated_by_task_run_id?: string | null;
  /**
   * Whether to automatically include health check steps before
   * verification.
   *
   * When enabled and `health_check_urls` is non-empty, health check steps
   * are prepended to verification steps to verify configured servers are
   * running.
   */
  health_check_enabled: boolean;
  /**
   * URLs to health check before verification (user-configurable).
   *
   * Each entry specifies a URL to check, expected status, and timeout.
   * If empty, no health checks are performed even if `health_check_enabled`
   * is true.
   */
  health_check_urls?: HealthCheckUrl[];
  /**
   * Whether HTN (Hierarchical Task Network) planning is enabled for this
   * workflow.
   *
   * When `true`, the loop attempts structured plan-based fixes before
   * falling back to AI agentic sessions.
   */
  htn_enabled: boolean;
  /**
   * Path to a serialized state machine JSON file for HTN planning.
   *
   * When `None` and HTN is enabled, defaults to the bundled
   * `data/runner_state_machine.json`.
   */
  htn_state_machine_path?: string | null;
  /**
   * UI Bridge URL for HTN planning (e.g., `"http://localhost:1420"`).
   *
   * When set, the HTN planner connects to UI Bridge for querying element
   * state. If `None`, HTN runs in plan-only mode without GUI execution.
   */
  htn_ui_bridge_url?: string | null;
  /**
   * Unique identifier (UUID v4).
   */
  id: string;
  /**
   * Whether this workflow is marked as a favorite for quick access.
   */
  is_favorite: boolean;
  log_source_selection?: LogSourceSelection;
  /**
   * Whether to automatically include a `log_watch` step before verification.
   *
   * When enabled (default), a `log_watch` step is prepended to
   * verification steps to detect runtime errors in backend/frontend logs.
   */
  log_watch_enabled: boolean;
  /**
   * Maximum number of CI-triggered auto-resumes before requiring human
   * intervention. Used by the PR watcher integration. `0` = disabled.
   * Default: `10`.
   */
  max_ci_auto_resumes: number;
  /**
   * Maximum consecutive non-improving fix attempts before escalating.
   *
   * When the verification check count does not improve across this many
   * iterations, the loop exits with `fix_attempts_exhausted`. `0` =
   * disabled. Default: `3`.
   */
  max_fix_attempts: number;
  /**
   * Maximum iterations for the agentic phase.
   *
   * `None` means no iteration cap — the loop terminates on success,
   * explicit stop, or fix-attempt exhaustion.
   */
  max_iterations?: number | null;
  /**
   * Maximum number of sweep iterations (default: `5`).
   */
  max_sweep_iterations: number;
  /**
   * Model override.
   */
  model?: string | null;
  /**
   * Per-phase model overrides.
   */
  model_overrides?: {
    [k: string]: ModelOverrideConfig;
  };
  /**
   * ISO 8601 timestamp of last modification (serialized as `"modified_at"`
   * to match the frontend).
   */
  modified_at: string;
  /**
   * Enable multi-agent fixer mode for the agentic phase.
   *
   * When `true`, verification failures are triaged and fixed by
   * specialized agents (quick-fix for lint/compilation, feature-fix for
   * missing functionality). Default: `true`.
   */
  multi_agent_mode: boolean;
  /**
   * Display name.
   */
  name: string;
  /**
   * Per-phase timeout configuration as a JSON string.
   */
  phase_timeouts_json?: string | null;
  /**
   * Whether to automatically include a pre-flight environment check at the
   * start of setup.
   *
   * When enabled (default), a shell command step runs to verify disk
   * space, Node.js/npm, Python/Poetry, Rust/Cargo, and Git availability.
   * Uses the global setting from Settings if not explicitly set per
   * workflow.
   */
  preflight_check_enabled: boolean;
  /**
   * Custom developer prompt template for this workflow.
   *
   * When set, this template is used instead of the global default when
   * running the workflow. Supports variables: `{{SESSION_ID}}`,
   * `{{ITERATION}}`, `{{MAX_ITERATIONS}}`, `{{GOAL}}`,
   * `{{EXECUTION_STEPS}}`, `{{WORKSPACE_ESCAPED}}`.
   */
  prompt_template?: string | null;
  /**
   * AI provider override.
   */
  provider?: string | null;
  /**
   * Quality report from the revision phase (opaque JSON blob).
   */
  quality_report?: {
    [k: string]: unknown;
  };
  /**
   * Whether to enable reflection mode during agentic iterations.
   *
   * When `true`, the AI investigates root causes before fixing failures.
   * Default: `true` for user-created workflows.
   */
  reflection_mode: boolean;
  /**
   * Policy for automatic git rollback when the workflow fails.
   *
   * Values: `"none"` (default), `"last_good"`, `"clean"`.
   */
  rollback_policy?: string | null;
  /**
   * Per-workflow security profile override.
   *
   * When set, overrides the default security profile from settings for
   * this workflow. Values: `"permissive"`, `"standard"`, `"strict"`,
   * or `"custom"`. If `None`, uses the default from Settings > Security.
   */
  security_profile?: string | null;
  /**
   * Setup phase steps (polymorphic JSON array; see module docs).
   */
  setup_steps: UnifiedStep[];
  /**
   * Skip AI summary generation at the end (default: `false`, meaning the
   * AI summary is generated).
   */
  skip_ai_summary: boolean;
  /**
   * Optional stages for multi-stage workflows.
   *
   * When non-empty, the workflow executes stages sequentially instead of
   * using top-level steps. Each stage has its own
   * setup / verification / agentic / completion steps and loop.
   */
  stages?: WorkflowStage[];
  /**
   * Whether to stop execution if a stage fails verification.
   *
   * Default: `false` (autonomous mode — continue to the next stage even
   * if the previous failed).
   */
  stop_on_failure: boolean;
  /**
   * Restrict working directory resolution to the workspace boundary.
   *
   * When `true`, steps cannot resolve paths outside the workspace root.
   * Default: `false` (permissive, current behavior).
   */
  strict_cwd: boolean;
  /**
   * Tags for filtering.
   */
  tags: string[];
  /**
   * Error IDs targeted by this workflow (for auto-resolution on success).
   *
   * When the workflow completes successfully, these errors will be marked
   * as resolved. Used by error-fix workflows generated from the Error
   * Monitor.
   */
  targeted_error_ids?: number[];
  /**
   * Optional inactivity timeout in seconds for AI sessions.
   *
   * - `None` (default): no timeout, runs until completion or manual stop.
   * - `Some(N)`: kill AI session after `N` seconds of no output.
   *
   * Takes precedence over the global AI settings timeout.
   */
  timeout_seconds?: number | null;
  /**
   * Tags for per-execution tool whitelisting.
   *
   * When non-empty, only skills matching at least one tag are included in
   * the AI prompt context, reducing prompt bloat.
   */
  tool_tags?: string[];
  /**
   * Run the workflow in an isolated git worktree.
   *
   * When `true`, a new branch and worktree are created before execution.
   * Changes stay on the worktree branch and can be merged back after
   * review. Default: `false`.
   */
  use_worktree: boolean;
  /**
   * Verification phase steps (polymorphic JSON array).
   */
  verification_steps: UnifiedStep[];
  /**
   * Workflow execution architecture override.
   *
   * When set, forces the workflow to use a specific execution architecture
   * instead of the default Traditional loop. When `None`, the system
   * infers the best architecture based on workflow complexity.
   */
  workflow_architecture?: WorkflowArchitecture | null;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Shared fields common to every canonical step variant.
 *
 * Flattened into each step struct via `#[serde(flatten)]` so the wire shape
 * stays flat (no nested `"base": { … }` envelope).
 */
interface BaseStepFields {
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

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * HTTP methods accepted by API-request command steps.
 *
 * Serialized uppercase (`GET`, `POST`, …) to match HTTP convention.
 */
type HttpMethod = "GET" | "POST" | "PUT" | "PATCH" | "DELETE";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Content-Type values for an API request body.
 */
type ApiContentType = "application/json" | "application/x-www-form-urlencoded" | "text/plain" | "none";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Comparison operators supported by API assertions.
 */
type ApiAssertionOperator = "equals" | "contains" | "matches" | "greater_than" | "less_than";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Assertion kinds supported on API responses.
 */
type ApiAssertionType = "status_code" | "json_path" | "header" | "body_contains" | "response_time";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * A single assertion evaluated against an API response.
 */
interface ApiAssertion {
  /**
   * Expected value. The TS source allows either a string or number, so
   * this field stays as `serde_json::Value` on the wire.
   */
  expected: {
    [k: string]: unknown;
  };
  /**
   * Header name for `header` assertions.
   */
  header_name?: string | null;
  /**
   * JSONPath for `json_path` assertions.
   */
  json_path?: string | null;
  /**
   * Comparison operator; defaults to `equals` on the consumer side.
   */
  operator?: ApiAssertionOperator | null;
  type: ApiAssertionType;
  [k: string]: unknown;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Extract a named variable from an API response via JSONPath.
 */
interface ApiVariableExtraction {
  /**
   * Default value if the path does not resolve.
   */
  default_value?: string | null;
  /**
   * JSONPath expression used to extract the value.
   */
  json_path: string;
  /**
   * Variable name to bind the extracted value to.
   */
  variable_name: string;
  [k: string]: unknown;
}

/**
 * Skill Types
 *
 * A skill is a named, parameterized template that produces pre-configured
 * workflow step(s) when instantiated. Skills sit between raw step types
 * and full workflows:
 *
 *   Raw Step Types  (command, prompt, ui_bridge, workflow)  ← execution primitives
 *        ↑ instantiates
 *   Skills          ("Lint Project", "API Health Check")    ← named capability templates
 *        ↑ composes into
 *   Workflows       (multi-phase verification-agentic loops) ← orchestration
 *
 * Skills are purely a configuration-time abstraction — they produce steps,
 * they do NOT add new runtime behavior.
 */

type SkillCategory = "code-quality" | "testing" | "monitoring" | "ai-task" | "deployment" | "composition" | "custom";
interface SkillAuthor {
    name: string;
    email?: string;
    url?: string;
}
interface SkillParameterOption {
    label: string;
    value: string;
}
interface SkillParameter {
    name: string;
    type: "string" | "number" | "boolean" | "select";
    label: string;
    description: string;
    required: boolean;
    default?: unknown;
    options?: SkillParameterOption[];
    placeholder?: string;
    min?: number;
    max?: number;
    pattern?: string;
    depends_on?: {
        param: string;
        value: unknown;
    };
}
interface SingleStepTemplate {
    kind: "single_step";
    step: Record<string, unknown>;
}
interface MultiStepTemplate {
    kind: "multi_step";
    steps: Record<string, unknown>[];
}
interface CompositionTemplate {
    kind: "composition";
    skill_refs: SkillRef[];
}
interface SkillRef {
    skill_id: string;
    parameter_overrides?: Record<string, unknown>;
}
type SkillTemplate = SingleStepTemplate | MultiStepTemplate | CompositionTemplate;
interface SkillDefinition {
    id: string;
    name: string;
    slug: string;
    description: string;
    category: SkillCategory;
    tags: string[];
    icon: string;
    color: string;
    allowed_phases: WorkflowPhase[];
    parameters: SkillParameter[];
    template: SkillTemplate;
    source: "builtin" | "user" | "community";
    version?: string;
    author?: SkillAuthor;
    checksum?: string;
    depends_on?: string[];
    usage_count?: number;
    approval_status?: "pending" | "approved" | "rejected";
    forked_from?: string;
}
interface SkillOrigin {
    skill_id: string;
    skill_slug: string;
    parameter_values: Record<string, unknown>;
}
interface SkillExportManifest {
    version: string;
    exported_at: string;
    app_version: string;
    content_type: "skills";
    skill_count: number;
    checksum?: string;
}
interface SkillExport {
    manifest: SkillExportManifest;
    skills: SkillDefinition[];
}
interface SkillImportResult {
    imported: number;
    skipped: number;
    overwritten: number;
    errors: string[];
}

/**
 * Structured Action Plan Types
 *
 * Defines the schema for LLM-generated UI Bridge action plans.
 * Instead of natural language instructions interpreted by a second LLM call,
 * action plans let the agentic-phase LLM directly specify typed UI actions
 * that map to UI Bridge's control API.
 *
 * Inspired by Skyvern's structured action protocol: each action carries
 * reasoning, confidence, and typed parameters so that execution is
 * deterministic and auditable.
 */
/**
 * Action types that can appear in an action plan.
 * Maps directly to UI Bridge's StandardAction type with additions
 * for navigation and waiting.
 */
type PlannedActionType = "click" | "doubleClick" | "rightClick" | "type" | "clear" | "select" | "check" | "uncheck" | "toggle" | "hover" | "focus" | "scroll" | "scrollIntoView" | "setValue" | "drag" | "submit" | "sendKeys" | "autocomplete" | "navigate" | "wait";
/**
 * How to find the target element for an action.
 * Supports multiple resolution strategies with fallback.
 */
interface ElementTarget {
    /** Direct element ID from a prior snapshot (fastest resolution) */
    elementId?: string;
    /** data-testid attribute value */
    testId?: string;
    /** Natural language description for fuzzy search (fallback) */
    searchText?: string;
    /** Element type hint to narrow search (e.g., "button", "input") */
    elementType?: string;
    /** CSS selector */
    selector?: string;
}
/**
 * A single action in an action plan.
 *
 * The LLM produces an array of these, each mapping to one UI Bridge
 * control action. The reasoning and confidence fields enable auditing
 * and confidence-gated execution.
 */
interface PlannedAction {
    /** Action type to execute */
    action: PlannedActionType;
    /** How to find the target element */
    target: ElementTarget;
    /** LLM's reasoning for this action (audit trail) */
    reasoning?: string;
    /** LLM's confidence that this is the right action (0.0–1.0) */
    confidence: number;
    /**
     * Action-specific parameters.
     * - type/setValue: { text: string }
     * - select: { value: string } or { label: string }
     * - scroll: { direction: "up"|"down"|"left"|"right" }
     * - drag: { targetPosition: { x: number, y: number } }
     * - sendKeys: { keys: string }
     * - navigate: { url: string }
     * - wait: { ms: number }
     */
    params?: Record<string, unknown>;
    /**
     * Separates the generic intent from specific data.
     * Enables action plan caching: the query stays stable across runs,
     * only the answer changes per user context.
     *
     * Example:
     *   userDetailQuery: "What email should be entered?"
     *   userDetailAnswer: "test@example.com"
     */
    userDetailQuery?: string;
    userDetailAnswer?: string;
}
/**
 * A complete action plan: an ordered sequence of UI actions
 * produced by the agentic-phase LLM.
 */
interface ActionPlan {
    /** Ordered list of actions to execute */
    actions: PlannedAction[];
    /** High-level goal this plan achieves */
    goal: string;
    /**
     * Minimum confidence threshold. Actions below this confidence
     * are skipped (or trigger verification) rather than executed.
     * Default: 0.5
     */
    confidenceThreshold?: number;
    /** Whether to stop on first action failure (default: true) */
    stopOnFailure?: boolean;
}
/** Result of executing a single planned action */
interface PlannedActionResult {
    /** Index of the action in the plan */
    index: number;
    /** Whether the action succeeded */
    success: boolean;
    /** The action that was executed */
    action: PlannedActionType;
    /** Element ID that was resolved and acted upon */
    resolvedElementId?: string;
    /** Error message if the action failed */
    error?: string;
    /** Whether the action was skipped due to low confidence */
    skippedLowConfidence?: boolean;
    /** Duration in milliseconds */
    durationMs: number;
    /** Post-action element state (if available) */
    elementState?: Record<string, unknown>;
}
/** Aggregated result of executing a full action plan */
interface ActionPlanResult {
    /** Whether all executed actions succeeded */
    success: boolean;
    /** The goal from the action plan */
    goal?: string;
    /** Per-action results */
    results: PlannedActionResult[];
    /** Count of actions executed (excludes skipped) */
    executedCount: number;
    /** Count of actions skipped due to low confidence */
    skippedCount: number;
    /** Count of actions that failed */
    failedCount: number;
    /** Total duration in milliseconds */
    totalDurationMs: number;
    /** Whether this plan was stored in the cache for future reuse */
    cached?: boolean;
}
/**
 * Extended action plan request with caching fields.
 * Used when calling the endpoint directly (not via workflow steps).
 */
interface ActionPlanExecuteRequest extends ActionPlan {
    /** Page URL for cache keying */
    pageUrl?: string;
    /** Element snapshot for cache fingerprinting (array of {id, type, role, label}) */
    elementSnapshot?: Array<{
        id?: string;
        type?: string;
        role?: string;
        label?: string;
    }>;
}

/**
 * Workflow Types
 *
 * Canonical type definitions for the unified Workflow Builder system.
 * Shared across qontinui-runner and qontinui-web.
 *
 * All automation is organized into four phases: Setup, Verification, Agentic, Completion.
 *
 * Execution Order:
 *   Setup (once) -> [Verification <-> Agentic]* -> Completion (once)
 *
 * Step Types (4 core types):
 *   command   - Shell commands, checks, check groups, tests
 *   ui_bridge - UI Bridge SDK interactions (navigate, execute, assert, snapshot)
 *   prompt    - AI task instructions
 *   workflow  - Run a saved workflow inline (composition)
 */

type ModelOverrides = {
    setup?: ModelOverrideConfig;
    agentic?: ModelOverrideConfig;
    completion?: ModelOverrideConfig;
    verification?: ModelOverrideConfig;
    investigation?: ModelOverrideConfig;
    summary?: ModelOverrideConfig;
    generation?: ModelOverrideConfig;
};
type WorkflowPhase = "setup" | "verification" | "agentic" | "completion";

type BaseStep = BaseStepFields;
type StepTypeName = "command" | "ui_bridge" | "prompt" | "workflow" | "native_accessibility";
type SetupStep = CommandStep | PromptStep | UiBridgeStep | WorkflowStep;
type VerificationStep = CommandStep | PromptStep | UiBridgeStep | WorkflowStep;
type AgenticStep = PromptStep;
type CompletionStep = CommandStep | PromptStep | UiBridgeStep | WorkflowStep;

type VerificationCategory = VerificationCategoryKind;
interface DependencyNode {
    id: string;
    label: string;
    type: string;
    phase: WorkflowPhase;
    is_referenced: boolean;
    cost_category?: string;
}
interface DependencyEdge {
    source: string;
    target: string;
    label?: string;
    edge_type: "explicit_depends_on" | "implicit_reference" | "setup_provides";
}
interface DependencyGraph {
    nodes: DependencyNode[];
    edges: DependencyEdge[];
}
type CostCategory = "network" | "ai_call" | "setup" | "ui_interaction";
interface StepCost {
    step_id: string;
    name: string;
    estimated_ms: number;
    category: CostCategory;
    has_side_effects: boolean;
}
interface CostAnnotations {
    steps: StepCost[];
    total_estimated_ms: number;
}
type QualityFindingSeverity = "critical" | "warning" | "info";
type QualityFindingCategory = "verification_gap" | "missing_criterion" | "unnecessary_step" | "weak_retry" | "required_flag_violation" | "retry_inconsistency" | "data_contract_violation" | "false_positive_risk";
interface QualityFinding {
    finding_id: string;
    step_id?: string;
    severity: QualityFindingSeverity;
    category: QualityFindingCategory;
    description: string;
    suggested_fix?: string;
}
interface CoverageMatrix {
    criteria_to_steps: Record<string, string[]>;
    steps_to_criteria: Record<string, string[]>;
    uncovered_criteria: string[];
    unlinked_steps: string[];
}
interface QualityReport {
    findings: QualityFinding[];
    score: number;
    pass: boolean;
    coverage_matrix?: CoverageMatrix;
}

interface WorkflowExportManifest {
    version: string;
    exported_at: string;
    app_version: string;
    content_type: "unified_workflow";
}
interface WorkflowExport {
    manifest: WorkflowExportManifest;
    workflow: UnifiedWorkflow;
}
interface WorkflowImportResult {
    workflow: UnifiedWorkflow;
    overwritten: boolean;
    original_id: string | null;
}
interface WorkflowFeatures {
    hasSetup: boolean;
    hasVerification: boolean;
    hasAgentic: boolean;
    hasCompletion: boolean;
    hasUiBridge: boolean;
    showIterationSettings: boolean;
    hasAiPrompts: boolean;
}
interface StepTypeInfo {
    type: string;
    label: string;
    description: string;
    icon: string;
    color: string;
    phase: WorkflowPhase;
}
declare const STEP_TYPES: Record<WorkflowPhase, StepTypeInfo[]>;
declare const PHASE_INFO: Record<WorkflowPhase, {
    label: string;
    description: string;
    color: string;
}>;
declare const DEFAULT_SUMMARY_PROMPT = "Write a one-paragraph summary of all the tasks completed in this workflow. Include what was accomplished, whether the stated goal was achieved, any issues encountered and how they were resolved, and remaining work if the goal was not fully achieved. Be concise but comprehensive.";

export { type ActionPlan, type ActionPlanExecuteRequest, type ActionPlanResult, type AgenticStep, type ApiAssertion, type ApiContentType, type ApiVariableExtraction, type BaseStep, type BaseStepFields, type CanonicalStep, type CheckType, type CommandMode, type CommandStep, type CompletionStep, type CompositionTemplate, type CostAnnotations, type CostCategory, type CoverageMatrix, DEFAULT_SUMMARY_PROMPT, type DependencyEdge, type DependencyGraph, type DependencyNode, type ElementTarget, type HealthCheckUrl, type HttpMethod, type LogSourceSelection, type ModelOverrideConfig, type ModelOverrides, type MultiStepTemplate, PHASE_INFO, type PlannedAction, type PlannedActionResult, type PlannedActionType, type PlaywrightExecutionMode, type PromptStep, type QualityFinding, type QualityFindingCategory, type QualityFindingSeverity, type QualityReport, type RetryPolicy, type RetrySpec, type RoutingRule, STEP_TYPES, type SetupStep, type SingleStepTemplate, type SkillAuthor, type SkillCategory, type SkillDefinition, type SkillExport, type SkillExportManifest, type SkillImportResult, type SkillOrigin, type SkillParameter, type SkillParameterOption, type SkillRef, type SkillTemplate, type StageCondition, type StageInput, type StageOutput, type StepCost, type StepTypeInfo, type StepTypeName, type TestType, type UiBridgeAction, type UiBridgeStep, type UnifiedStep, type UnifiedWorkflow, type VerificationCategory, type VerificationCategoryKind, type VerificationStep, type WorkflowArchitecture, type WorkflowExport, type WorkflowExportManifest, type WorkflowFeatures, type WorkflowImportResult, type WorkflowPhase, type WorkflowStage, type WorkflowStep };
