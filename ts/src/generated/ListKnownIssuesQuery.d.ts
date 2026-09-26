/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Query parameters for listing known issues.
 */
export interface ListKnownIssuesQuery {
  category?: string | null;
  scope_type?: string | null;
  scope_value?: string | null;
  severity?: string | null;
  /**
   * Convenience: equivalent to scope_type=spec + scope_value=<id>
   */
  spec_id?: string | null;
  status?: string | null;
  [k: string]: unknown;
}
