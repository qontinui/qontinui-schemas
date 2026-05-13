/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { IrElementCriteria } from './IrElementCriteria';
import type { IrWaitSpec } from './IrWaitSpec';

export interface IrTransitionAction {
  params?: unknown;
  target: IrElementCriteria;
  type: string;
  waitAfter?: IrWaitSpec | null;
  [k: string]: unknown;
}
