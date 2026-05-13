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
import type { IrStateCondition } from './IrStateCondition';

export interface IrState {
  blocking?: boolean | null;
  conditions?: IrStateCondition[] | null;
  crossRefs?: IrCrossRef[] | null;
  description?: string | null;
  elementIds?: string[] | null;
  excludedElements?: IrElementCriteria[] | null;
  group?: string | null;
  id: string;
  incomingTransitions?: string[] | null;
  isInitial?: boolean | null;
  isTerminal?: boolean | null;
  metadata?: IrMetadata | null;
  name: string;
  pathCost?: number | null;
  precondition?: string | null;
  provenance?: IrProvenance | null;
  requiredElements: IrElementCriteria[];
  [k: string]: unknown;
}
