/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * One row in `coord.git_ops`. Returned by `GET /coord/git-ops/:op_id`,
 * `GET /coord/git-ops/list`, and `GET /coord/git-ops/by-session/:id`.
 */
export interface GitOpRecord {
  branch?: string | null;
  device_id: string;
  message?: string | null;
  metadata: unknown;
  op_id: string;
  op_kind: string;
  /**
   * ISO 8601 timestamp (RFC 3339).
   */
  recorded_at: string;
  repo: string;
  session_id: string;
  sha?: string | null;
  tenant_id?: string | null;
  [k: string]: unknown;
}
