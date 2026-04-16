/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { CriterionOverride } from './CriterionOverride';

/**
 * Collection of overrides for a task run.
 */
export interface OverrideCollection {
  /**
   * All recorded overrides.
   */
  overrides?: CriterionOverride[];
  [k: string]: unknown;
}
