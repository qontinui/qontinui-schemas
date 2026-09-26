/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Result of one memory-compression pass.
 */
export interface RawCompressionResultPayload {
  compressed_categories: string[];
  compressed_tokens: number;
  items_summarized: number;
  original_tokens: number;
  summary_entries_created: number;
  timestamp: string;
  [k: string]: unknown;
}
