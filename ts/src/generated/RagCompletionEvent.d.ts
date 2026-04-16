/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { EmbeddingResultItem } from './EmbeddingResultItem';

/**
 * Completion event emitted when RAG processing finishes.
 */
export interface RagCompletionEvent {
  failed: number;
  project_id: string;
  results: EmbeddingResultItem[];
  success: boolean;
  successful: number;
  total_processed: number;
  web_sync_error?: string | null;
  /**
   * Whether sync to the web backend succeeded.
   */
  web_sync_success?: boolean | null;
  [k: string]: unknown;
}
