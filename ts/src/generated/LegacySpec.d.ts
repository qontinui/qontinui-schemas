/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { IrElementCriteria } from "./IrElementCriteria";
import type { IrWaitSpec } from "./IrWaitSpec";
import type { LegacyAssertion } from "./LegacyAssertion";
import type { LegacyAssertionTarget } from "./LegacyAssertionTarget";
import type { LegacyGroup } from "./LegacyGroup";
import type { LegacyProcessStep } from "./LegacyProcessStep";
import type { LegacyStateMachine } from "./LegacyStateMachine";
import type { LegacyStateMachineState } from "./LegacyStateMachineState";
import type { LegacyTransition } from "./LegacyTransition";

/**
 * Top-level legacy spec output. Serialized via a `serde_json::Value` step
 * so the projection can apply lexicographic key sorting at the end (matches
 * the TS projection's `sortKeys` pass).
 */
export interface LegacySpec {
  description: string;
  groups: LegacyGroup[];
  metadata: unknown;
  stateMachine: LegacyStateMachine;
  version: string;
  [k: string]: unknown;
}
