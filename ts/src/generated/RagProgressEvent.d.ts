/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { RagProcessingStatus } from './RagProcessingStatus';

/**
 * Progress event emitted during RAG processing.
 */
export interface RagProgressEvent {
  elements_processed?: number | null;
  error?: string | null;
  /**
   * Human-readable status message.
   */
  message: string;
  /**
   * Progress percentage (0-100).
   */
  percent?: number | null;
  project_id: string;
  status: RagProcessingStatus;
  total_elements?: number | null;
  [k: string]: unknown;
}
