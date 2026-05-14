/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { IrElementCriteria } from './IrElementCriteria';
import type { IrWaitSpec } from './IrWaitSpec';
import type { LegacyProcessStep } from './LegacyProcessStep';

export interface LegacyTransition {
  activateStates: string[];
  deactivateStates: string[];
  id: string;
  name: string;
  process: LegacyProcessStep[];
  staysVisible: boolean;
  [k: string]: unknown;
}
