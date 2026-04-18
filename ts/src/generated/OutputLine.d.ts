/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { OutputStream } from './OutputStream';

/**
 * A single line of output from a managed process.
 */
export interface OutputLine {
  /**
   * The line content
   */
  line: string;
  stream: OutputStream;
  /**
   * ISO 8601 timestamp
   */
  timestamp: string;
}
