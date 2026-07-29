/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Per-page validation summary surfaced on `GET /spec/list`. §5.12 G2.
 */
export interface SpecValidation {
  /**
   * State IDs whose distinctness signal is empty (a state with no
   * assertions that distinguish it from any other state).
   */
  degenerateStateIds?: string[];
  /**
   * Pairs of state IDs that share an identical normalized distinctness
   * signal. Each inner array is sorted lexicographically so the same
   * pair always serializes the same way regardless of detection order.
   */
  indistinguishableStatePairs?: [string, string][];
  /**
   * Page identifier.
   */
  pageId: string;
}
