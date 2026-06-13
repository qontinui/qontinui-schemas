/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { SpecProvenance } from './SpecProvenance';

/**
 * A client-side validation rule observed on an input.
 */
export interface ValidationRule {
  confidence: SpecProvenance;
  credibility?: number | null;
  provenance?: string | null;
  /**
   * The rule expression as observed (e.g. `"> 0"`, `"email"`, `"maxLength 80"`).
   */
  rule: string;
  [k: string]: unknown;
}
