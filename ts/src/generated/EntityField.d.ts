/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { SpecProvenance } from "./SpecProvenance";

/**
 * A field of an [`Entity`].
 */
export interface EntityField {
  confidence: SpecProvenance;
  credibility?: number | null;
  name: string;
  provenance?: string | null;
  /**
   * Coarse semantic type: `"string"`, `"money"`, `"enum"`, `"date"`,
   * `"bool"`, `"number"`, `"reference"`, … Free-form so comprehension can
   * introduce new types without a schema bump.
   */
  type: string;
  /**
   * Enumerated values when `field_type == "enum"`.
   */
  values?: string[];
  [k: string]: unknown;
}
