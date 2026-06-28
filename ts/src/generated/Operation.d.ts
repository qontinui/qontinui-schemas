/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { OperationEffect } from "./OperationEffect";
import type { OperationInput } from "./OperationInput";
import type { SpecProvenance } from "./SpecProvenance";
import type { ValidationRule } from "./ValidationRule";

/**
 * An operation the frontend exposes. Existence (`confidence`) is typically
 * `Observed`; the server-side [`OperationEffect`] is `Assumed` by construction
 * (the frontend cannot reveal what persists server-side).
 */
export interface Operation {
  confidence: SpecProvenance;
  credibility?: number | null;
  /**
   * The (largely assumed) server-side effect.
   */
  effect?: OperationEffect | null;
  /**
   * Target entity, when the operation acts on one.
   */
  entity?: string | null;
  inputs?: OperationInput[];
  name: string;
  provenance?: string | null;
  /**
   * `"create"`, `"read"`, `"update"`, `"delete"`, `"custom"`.
   */
  verb: string;
  [k: string]: unknown;
}
