/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { IrElementCriteria } from "./IrElementCriteria";
import type { IrWaitSpec } from "./IrWaitSpec";
import type { LegacyProcessStep } from "./LegacyProcessStep";
import type { LegacyTransition } from "./LegacyTransition";

export interface LegacyStateMachineState {
  description: string;
  elements: unknown[];
  id: string;
  isInitial: boolean;
  name: string;
  transitions: LegacyTransition[];
  [k: string]: unknown;
}
