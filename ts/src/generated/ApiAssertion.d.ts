/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ApiAssertionOperator } from './ApiAssertionOperator';
import type { ApiAssertionType } from './ApiAssertionType';

/**
 * A single assertion evaluated against an API response.
 */
export interface ApiAssertion {
  /**
   * Expected value. The TS source allows either a string or number, so
   * this field stays as `serde_json::Value` on the wire.
   */
  expected: {
    [k: string]: unknown;
  };
  /**
   * Header name for `header` assertions.
   */
  headerName?: string | null;
  /**
   * JSONPath for `json_path` assertions.
   */
  jsonPath?: string | null;
  /**
   * Comparison operator; defaults to `equals` on the consumer side.
   */
  operator?: ApiAssertionOperator | null;
  type: ApiAssertionType;
}
