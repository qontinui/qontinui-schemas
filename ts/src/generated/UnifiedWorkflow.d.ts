/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { CanonicalStep } from './CanonicalStep';
import type { CheckType } from './CheckType';
import type { CommandMode } from './CommandMode';
import type { CommandStep } from './CommandStep';
import type { CommandStepPhase } from './CommandStepPhase';
import type { HealthCheckUrl } from './HealthCheckUrl';
import type { LogSourceSelection } from './LogSourceSelection';
import type { ModelOverrideConfig } from './ModelOverrideConfig';
import type { PlaywrightExecutionMode } from './PlaywrightExecutionMode';
import type { PromptStep } from './PromptStep';
import type { PromptStepPhase } from './PromptStepPhase';
import type { RetryPolicy } from './RetryPolicy';
import type { RetrySpec } from './RetrySpec';
import type { RoutingRule } from './RoutingRule';
import type { StageCondition } from './StageCondition';
import type { StageInput } from './StageInput';
import type { StageOutput } from './StageOutput';
import type { TestType } from './TestType';
import type { UiBridgeAction } from './UiBridgeAction';
import type { UiBridgeAssertType } from './UiBridgeAssertType';
import type { UiBridgeComparisonMode } from './UiBridgeComparisonMode';
import type { UiBridgeSeverity } from './UiBridgeSeverity';
import type { UiBridgeStep } from './UiBridgeStep';
import type { UiBridgeStepPhase } from './UiBridgeStepPhase';
import type { UnifiedStep } from './UnifiedStep';
import type { VerificationCategoryKind } from './VerificationCategoryKind';
import type { WorkflowArchitecture } from './WorkflowArchitecture';
import type { WorkflowStage } from './WorkflowStage';
import type { WorkflowStep } from './WorkflowStep';
import type { WorkflowStepPhase } from './WorkflowStepPhase';

/**
 * A unified workflow with steps organized by phase.
 *
 * The "frame" carries all non-step metadata — iteration caps, provider/model
 * selection, log-source routing, health checks, stage list, generator
 * outputs (dependency graph, cost annotations, quality report, acceptance
 * criteria), and timestamps. Step arrays remain opaque until the Wave 4
 * typed-step migration lands.
 */
export interface UnifiedWorkflow {
  /**
   * Acceptance criteria from the specification agent (opaque JSON blob).
   *
   * Used by the canvas panel manager to show a live requirements tracker.
   */
  acceptanceCriteria?: {
    [k: string]: unknown;
  };
  /**
   * Agentic phase steps (polymorphic JSON array).
   */
  agenticSteps: UnifiedStep[];
  /**
   * Whether the AI semantic review actually ran successfully during
   * generation.
   *
   * When `false`, the workflow passed through the pipeline without AI
   * verification (e.g., all verification iterations failed at
   * infrastructure level).
   */
  aiReviewed: boolean;
  /**
   * Whether to pause for human approval after each agentic phase.
   */
  approvalGate: boolean;
  /**
   * Whether to auto-include contexts based on task mentions (default:
   * `true`).
   */
  autoIncludeContexts: boolean;
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
  completionPromptsFirst: boolean;
  /**
   * Completion phase steps (polymorphic JSON array) — runs once after the
   * verification loop exits.
   */
  completionSteps: UnifiedStep[];
  /**
   * Per-constraint overrides: map of `constraint_id` to enabled (`true`) /
   * disabled (`false`).
   *
   * Applied to the constraint engine at execution time, after loading
   * builtins and config.
   */
  constraintOverrides?: {
    [k: string]: boolean;
  };
  /**
   * Manually added context IDs.
   */
  contextIds?: string[];
  /**
   * Cost annotations computed during generation (opaque JSON blob).
   */
  costAnnotations?: {
    [k: string]: unknown;
  };
  /**
   * ISO 8601 timestamp of creation.
   */
  createdAt: string;
  /**
   * Dependency graph computed during generation (opaque JSON blob).
   */
  dependencyGraph?: {
    [k: string]: unknown;
  };
  /**
   * Description of what this workflow does.
   */
  description: string;
  /**
   * Disabled context IDs (excluded from auto-include).
   */
  disabledContextIds?: string[];
  /**
   * Whether to run a completion sweep after verification passes.
   *
   * The sweep reviews all completed work for gaps before proceeding to
   * completion.
   */
  enableSweep: boolean;
  /**
   * When `true`, the pipeline will stop execution if accumulated token
   * usage exceeds the token budget. Disabled by default — only logs
   * warnings.
   */
  enforceTokenBudget: boolean;
  /**
   * Flow control configuration as a JSON string (e.g., concurrency limits,
   * queue behavior).
   */
  flowControlJson?: string | null;
  /**
   * Task run ID that generated this workflow (for meta-workflow tracking).
   */
  generatedByTaskRunId?: string | null;
  /**
   * Whether to automatically include health check steps before
   * verification.
   *
   * When enabled and `health_check_urls` is non-empty, health check steps
   * are prepended to verification steps to verify configured servers are
   * running.
   */
  healthCheckEnabled: boolean;
  /**
   * URLs to health check before verification (user-configurable).
   *
   * Each entry specifies a URL to check, expected status, and timeout.
   * If empty, no health checks are performed even if `health_check_enabled`
   * is true.
   */
  healthCheckUrls?: HealthCheckUrl[];
  /**
   * Whether HTN (Hierarchical Task Network) planning is enabled for this
   * workflow.
   *
   * When `true`, the loop attempts structured plan-based fixes before
   * falling back to AI agentic sessions.
   */
  htnEnabled: boolean;
  /**
   * Path to a serialized state machine JSON file for HTN planning.
   *
   * When `None` and HTN is enabled, defaults to the bundled
   * `data/runner_state_machine.json`.
   */
  htnStateMachinePath?: string | null;
  /**
   * UI Bridge URL for HTN planning (e.g., `"http://localhost:1420"`).
   *
   * When set, the HTN planner connects to UI Bridge for querying element
   * state. If `None`, HTN runs in plan-only mode without GUI execution.
   */
  htnUiBridgeUrl?: string | null;
  /**
   * Unique identifier (UUID v4).
   */
  id: string;
  /**
   * Whether this workflow is marked as a favorite for quick access.
   */
  isFavorite: boolean;
  logSourceSelection?: LogSourceSelection;
  /**
   * Whether to automatically include a `log_watch` step before verification.
   *
   * When enabled (default), a `log_watch` step is prepended to
   * verification steps to detect runtime errors in backend/frontend logs.
   */
  logWatchEnabled: boolean;
  /**
   * Maximum number of CI-triggered auto-resumes before requiring human
   * intervention. Used by the PR watcher integration. `0` = disabled.
   * Default: `10`.
   */
  maxCiAutoResumes: number;
  /**
   * Maximum consecutive non-improving fix attempts before escalating.
   *
   * When the verification check count does not improve across this many
   * iterations, the loop exits with `fix_attempts_exhausted`. `0` =
   * disabled. Default: `3`.
   */
  maxFixAttempts: number;
  /**
   * Maximum iterations for the agentic phase.
   *
   * `None` means no iteration cap — the loop terminates on success,
   * explicit stop, or fix-attempt exhaustion.
   */
  maxIterations?: number | null;
  /**
   * Maximum number of sweep iterations (default: `5`).
   */
  maxSweepIterations: number;
  /**
   * Model override.
   */
  model?: string | null;
  /**
   * Per-phase model overrides.
   */
  modelOverrides?: {
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
  multiAgentMode: boolean;
  /**
   * Display name.
   */
  name: string;
  /**
   * Per-phase timeout configuration as a JSON string.
   */
  phaseTimeoutsJson?: string | null;
  /**
   * Whether to automatically include a pre-flight environment check at the
   * start of setup.
   *
   * When enabled (default), a shell command step runs to verify disk
   * space, Node.js/npm, Python/Poetry, Rust/Cargo, and Git availability.
   * Uses the global setting from Settings if not explicitly set per
   * workflow.
   */
  preflightCheckEnabled: boolean;
  /**
   * Custom developer prompt template for this workflow.
   *
   * When set, this template is used instead of the global default when
   * running the workflow. Supports variables: `{{SESSION_ID}}`,
   * `{{ITERATION}}`, `{{MAX_ITERATIONS}}`, `{{GOAL}}`,
   * `{{EXECUTION_STEPS}}`, `{{WORKSPACE_ESCAPED}}`.
   */
  promptTemplate?: string | null;
  /**
   * AI provider override.
   */
  provider?: string | null;
  /**
   * Quality report from the revision phase (opaque JSON blob).
   */
  qualityReport?: {
    [k: string]: unknown;
  };
  /**
   * Whether to enable reflection mode during agentic iterations.
   *
   * When `true`, the AI investigates root causes before fixing failures.
   * Default: `true` for user-created workflows.
   */
  reflectionMode: boolean;
  /**
   * Policy for automatic git rollback when the workflow fails.
   *
   * Values: `"none"` (default), `"last_good"`, `"clean"`.
   */
  rollbackPolicy?: string | null;
  /**
   * Per-workflow security profile override.
   *
   * When set, overrides the default security profile from settings for
   * this workflow. Values: `"permissive"`, `"standard"`, `"strict"`,
   * or `"custom"`. If `None`, uses the default from Settings > Security.
   */
  securityProfile?: string | null;
  /**
   * Setup phase steps (polymorphic JSON array; see module docs).
   */
  setupSteps: UnifiedStep[];
  /**
   * Skip AI summary generation at the end (default: `false`, meaning the
   * AI summary is generated).
   */
  skipAiSummary: boolean;
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
  stopOnFailure: boolean;
  /**
   * Restrict working directory resolution to the workspace boundary.
   *
   * When `true`, steps cannot resolve paths outside the workspace root.
   * Default: `false` (permissive, current behavior).
   */
  strictCwd: boolean;
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
  targetedErrorIds?: number[];
  /**
   * Optional inactivity timeout in seconds for AI sessions.
   *
   * - `None` (default): no timeout, runs until completion or manual stop.
   * - `Some(N)`: kill AI session after `N` seconds of no output.
   *
   * Takes precedence over the global AI settings timeout.
   */
  timeoutSeconds?: number | null;
  /**
   * Tags for per-execution tool whitelisting.
   *
   * When non-empty, only skills matching at least one tag are included in
   * the AI prompt context, reducing prompt bloat.
   */
  toolTags?: string[];
  /**
   * Run the workflow in an isolated git worktree.
   *
   * When `true`, a new branch and worktree are created before execution.
   * Changes stay on the worktree branch and can be merged back after
   * review. Default: `false`.
   */
  useWorktree: boolean;
  /**
   * Verification phase steps (polymorphic JSON array).
   */
  verificationSteps: UnifiedStep[];
  /**
   * Workflow execution architecture override.
   *
   * When set, forces the workflow to use a specific execution architecture
   * instead of the default Traditional loop. When `None`, the system
   * infers the best architecture based on workflow complexity.
   */
  workflowArchitecture?: WorkflowArchitecture | null;
  [k: string]: unknown;
}
