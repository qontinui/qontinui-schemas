/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

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
 */
export type CanonicalStep =
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
