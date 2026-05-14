/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Per-field diff between an assertion's expected criteria and a
 * candidate element's actual value.
 */
export interface FieldDiff {
  /**
   * Actual value from the bridge element.
   */
  actual: {
    [k: string]: unknown;
  };
  /**
   * Expected value from the IR assertion.
   */
  expected: {
    [k: string]: unknown;
  };
  /**
   * Field name (`role`, `text`, `textContains`, etc.).
   */
  field: string;
  /**
   * Field-level similarity score (0.0..=1.0).
   */
  similarity: number;
}
