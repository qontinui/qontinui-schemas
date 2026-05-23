/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * One row in `coord.memories`. Returned by `GET /coord/memory/:name`
 * (as the `latest` field of `MemoryWithHistory` + each `history` entry)
 * and by `GET /coord/memory/:name/version/:version`.
 */
export interface MemoryRow {
  content: string;
  description?: string | null;
  is_tombstone: boolean;
  memory_id: string;
  name: string;
  type?: string | null;
  version: number;
  /**
   * ISO 8601 timestamp (RFC 3339).
   */
  written_at: string;
  written_by_agent?: string | null;
  written_by_device?: string | null;
  [k: string]: unknown;
}
