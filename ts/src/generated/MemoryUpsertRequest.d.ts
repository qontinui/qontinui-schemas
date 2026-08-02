/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * `POST /coord/memory/upsert` body. `name` + `content` are the only
 * required fields; the rest is optional metadata.
 *
 * UUID fields (`written_by_agent`, `written_by_device`) are `String` on
 * the wire; coord parses them with `Uuid::parse_str` and returns 400
 * on parse failure.
 */
export interface MemoryUpsertRequest {
  content: string;
  description?: string | null;
  name: string;
  type?: string | null;
  written_by_agent?: string | null;
  written_by_device?: string | null;
  [k: string]: unknown;
}
