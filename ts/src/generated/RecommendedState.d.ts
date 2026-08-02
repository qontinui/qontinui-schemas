/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { SpecCheckConfidence } from "./SpecCheckConfidence";

/**
 * The single state the matcher recommends the caller treat as "current".
 */
export interface RecommendedState {
  confidence: SpecCheckConfidence;
  /**
   * Free-form explanation (e.g. "highest match rate among all states").
   */
  reason: string;
  /**
   * The recommended state's identifier.
   */
  stateId: string;
}
