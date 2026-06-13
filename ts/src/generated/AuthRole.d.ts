/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { SpecProvenance } from './SpecProvenance';

/**
 * A role surfaced by the auth model (e.g. inferred from nav items only some
 * sessions see).
 */
export interface AuthRole {
  confidence: SpecProvenance;
  credibility?: number | null;
  name: string;
  provenance?: string | null;
  [k: string]: unknown;
}
