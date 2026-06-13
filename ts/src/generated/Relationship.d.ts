/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { SpecProvenance } from './SpecProvenance';

/**
 * A relationship from one entity to another.
 */
export interface Relationship {
  confidence: SpecProvenance;
  credibility?: number | null;
  /**
   * `"one-to-one"`, `"one-to-many"`, `"many-to-one"`, `"many-to-many"`.
   */
  kind: string;
  provenance?: string | null;
  /**
   * Target entity name.
   */
  to: string;
  [k: string]: unknown;
}
