/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { UnknownCode } from "./UnknownCode";

/**
 * The payload of an `unknown` observation: a typed code an agent branches
 * on, plus prose for a human.
 */
export interface UnknownInfo {
  code: UnknownCode;
  /**
   * Human-readable explanation. Never parsed; [`Self::code`] is the
   * machine-readable reason.
   */
  detail: string;
  [k: string]: unknown;
}
