/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { SpecProvenance } from "./SpecProvenance";

/**
 * The server-side effect of an [`Operation`]. `Assumed` by construction; the
 * `assumption` field records the best-practice default the generator applied.
 */
export interface OperationEffect {
  /**
   * The best-practice default applied when `confidence == Assumed`
   * (e.g. `"persists + returns created row (REST 201 default)"`).
   */
  assumption?: string | null;
  confidence: SpecProvenance;
  credibility?: number | null;
  provenance?: string | null;
  [k: string]: unknown;
}
