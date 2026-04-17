/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Result from the export pipeline.
 *
 * Tracks what was exported and any errors that occurred.
 * Mirrors `rag.models.ExportResult`.
 */
export interface ExportResult {
  errors: string[];
  export_path: string;
  /**
   * ISO 8601 UTC timestamp (`Z` suffix).
   */
  export_timestamp?: string | null;
  exported_count: number;
  failed_count: number;
  /**
   * `"json"`, `"csv"`, `"parquet"`, etc.
   */
  format: string;
  skipped_count: number;
  success: boolean;
  warnings: string[];
  [k: string]: unknown;
}
