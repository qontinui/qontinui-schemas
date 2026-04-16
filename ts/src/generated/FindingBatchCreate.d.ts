/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { FindingActionType } from './FindingActionType';
import type { FindingCategory } from './FindingCategory';
import type { FindingCodeContext } from './FindingCodeContext';
import type { FindingCreate } from './FindingCreate';
import type { FindingSeverity } from './FindingSeverity';
import type { FindingUserInput } from './FindingUserInput';

/**
 * Request schema for batch finding creation.
 *
 * Allows creating multiple findings in a single request. The Python side
 * enforces `1 <= len(findings) <= 50`; validators on the Rust side are
 * intentionally omitted to keep this a pure wire-format layer.
 */
export interface FindingBatchCreate {
  /**
   * Findings to create (1–50 items on the Python side).
   */
  findings: FindingCreate[];
  [k: string]: unknown;
}
