/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { UIBridgeCapturedToast } from './UIBridgeCapturedToast';

/**
 * Toast snapshot context.
 */
export interface UIBridgeToastContext {
  active?: UIBridgeCapturedToast[];
  recent?: UIBridgeCapturedToast[];
  totalCaptured: number;
}
