/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ProcessState } from './ProcessState';

/**
 * Status summary of a managed process.
 *
 * Derived from runtime state each time it is requested; never persisted.
 */
export interface ProcessStatus {
  category: string;
  /**
   * Number of errors detected from this process
   */
  errorCount: number;
  /**
   * Whether this process has a build command configured
   */
  hasBuildCommand: boolean;
  id: string;
  name: string;
  pid?: number | null;
  /**
   * Whether the health port is responding
   */
  portHealthy?: boolean | null;
  /**
   * Number of times this process has been restarted
   */
  restartCount: number;
  state: ProcessState;
  /**
   * Uptime in seconds (None if not running)
   */
  uptimeSecs?: number | null;
  [k: string]: unknown;
}
