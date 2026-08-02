/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * `POST /coord/memory/:name/restore` body.
 */
export interface MemoryRestoreRequest {
  version: number;
  written_by_agent?: string | null;
  written_by_device?: string | null;
  [k: string]: unknown;
}
