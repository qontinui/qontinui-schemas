/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { EntityField } from './EntityField';
import type { Relationship } from './Relationship';
import type { SpecProvenance } from './SpecProvenance';

/**
 * A domain entity inferred from rendered data shapes.
 */
export interface Entity {
  confidence: SpecProvenance;
  /**
   * Optional deduction strength ∈ [0,1]. Present only on `Inferred` nodes;
   * meaningless (and omitted) on `Observed` / `Assumed`.
   */
  credibility?: number | null;
  fields?: EntityField[];
  name: string;
  /**
   * Free-form evidence string (e.g. `"detail view + list view both render it"`).
   */
  provenance?: string | null;
  relationships?: Relationship[];
  [k: string]: unknown;
}
