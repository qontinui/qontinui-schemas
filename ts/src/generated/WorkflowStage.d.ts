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
import type { WorkflowStep } from './WorkflowStep';
import type { WorkflowStepPhase } from './WorkflowStepPhase';

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
export interface WorkflowStage {
  /**
   * Agentic phase steps for this stage.
   */
  agenticSteps: UnifiedStep[];
  /**
   * Whether to pause for human approval after each agentic phase.
   */
  approvalGate: boolean;
  /**
   * When true, run completion prompt steps BEFORE automation steps.
   * Default (`false`) runs automation first, then prompts.
   */
  completionPromptsFirst: boolean;
  /**
   * Completion phase steps for this stage.
   */
  completionSteps: UnifiedStep[];
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
  maxIterations?: number | null;
  /**
   * Model override for this stage.
   */
  model?: string | null;
  /**
   * Per-phase model overrides for this stage.
   */
  modelOverrides?: {
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
  retryPolicy?: RetryPolicy | null;
  /**
   * Setup phase steps for this stage (polymorphic; see module docs).
   */
  setupSteps: UnifiedStep[];
  /**
   * Optional inactivity timeout in seconds for this stage's AI sessions.
   */
  timeoutSeconds?: number | null;
  /**
   * Verification phase steps for this stage.
   */
  verificationSteps: UnifiedStep[];
}
