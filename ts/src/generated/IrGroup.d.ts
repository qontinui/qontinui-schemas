/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { IrAssertion } from "./IrAssertion";
import type { IrAssertionTarget } from "./IrAssertionTarget";

/**
 * Mirror of TS `IRGroup`. Synthesis-produced groups (NOT derived from IR
 * states). Mirrors the legacy `SpecGroup` shape so the projection can pass
 * it through to legacy `groups[]` without loss.
 */
export interface IrGroup {
  assertions: IrAssertion[];
  category: string;
  description: string;
  id: string;
  name: string;
  /**
   * Provenance of the group itself (typically `"ai-generated"` for
   * synthesis output).
   */
  source?: string | null;
  /**
   * Free-form tags. Synthesis emits e.g.
   * `["workflow-generated", "acceptance-criteria"]`.
   */
  tags?: string[] | null;
  [k: string]: unknown;
}
