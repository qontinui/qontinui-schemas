/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { NextActionKind } from "./NextActionKind";

/**
 * The typed next step of a [`Refusal`]. Decoding is lenient toward newer
 * producers (module docs, "Forward compatibility").
 */
export interface NextAction {
  kind: NextActionKind;
  /**
   * For [`NextActionKind::RetryLater`]: the earliest useful retry, in whole
   * seconds. Absent when no delay is known.
   */
  retry_after_s?: number | null;
  /**
   * What the action applies to: the command to run, the page to open, the
   * setting to set, the gate to wait for. Its meaning is fixed by `kind`.
   */
  target?: string | null;
  /**
   * READER-SIDE: the raw `kind` when this reader did not recognise it
   * (`kind` is then [`NextActionKind::Unrecognised`]). Producers must not
   * emit it.
   */
  unrecognised_kind?: string | null;
  [k: string]: unknown;
}
