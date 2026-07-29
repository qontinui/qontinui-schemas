/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { MemoryRow } from "./MemoryRow";

/**
 * `GET /coord/memory/:name` response — latest row + the 10
 * most-recent versions (DESC). The first entry of `history` is the
 * same row as `latest`; subsequent entries are prior versions
 * (including tombstones, so the dashboard can render "deleted at v3
 * then restored at v4").
 *
 * `latest` is flattened on the wire — the response object's top-level
 * fields are the `MemoryRow` columns plus a `history` array.
 */
export interface MemoryWithHistory {
  content: string;
  description?: string | null;
  history: MemoryRow[];
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
