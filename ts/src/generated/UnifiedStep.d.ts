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
import type { PlaywrightExecutionMode } from './PlaywrightExecutionMode';
import type { PromptStep } from './PromptStep';
import type { PromptStepPhase } from './PromptStepPhase';
import type { RetrySpec } from './RetrySpec';
import type { TestType } from './TestType';
import type { UiBridgeAction } from './UiBridgeAction';
import type { UiBridgeAssertType } from './UiBridgeAssertType';
import type { UiBridgeComparisonMode } from './UiBridgeComparisonMode';
import type { UiBridgeSeverity } from './UiBridgeSeverity';
import type { UiBridgeStep } from './UiBridgeStep';
import type { UiBridgeStepPhase } from './UiBridgeStepPhase';
import type { VerificationCategoryKind } from './VerificationCategoryKind';
import type { WorkflowStep } from './WorkflowStep';
import type { WorkflowStepPhase } from './WorkflowStepPhase';

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
export type UnifiedStep =
  | CanonicalStep
  | {
      [k: string]: unknown;
    };
