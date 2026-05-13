/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { IrCrossRef } from './IrCrossRef';
import type { IrElementCriteria } from './IrElementCriteria';
import type { IrMetadata } from './IrMetadata';
import type { IrProvenance } from './IrProvenance';
import type { IrTransitionAction } from './IrTransitionAction';
import type { IrWaitSpec } from './IrWaitSpec';

export interface IrTransition {
  actions: IrTransitionAction[];
  activateStates: string[];
  bidirectional?: boolean | null;
  crossRefs?: IrCrossRef[] | null;
  description?: string | null;
  /**
   * "read" | "write" | "destructive"
   */
  effect?: string | null;
  exitStates?: string[] | null;
  fromStates: string[];
  id: string;
  metadata?: IrMetadata | null;
  name: string;
  pathCost?: number | null;
  provenance?: IrProvenance | null;
  [k: string]: unknown;
}
