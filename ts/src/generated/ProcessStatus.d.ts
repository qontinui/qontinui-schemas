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
  error_count: number;
  /**
   * Whether this process has a build command configured
   */
  has_build_command: boolean;
  id: string;
  name: string;
  pid?: number | null;
  /**
   * Whether the health port is responding
   */
  port_healthy?: boolean | null;
  /**
   * Number of times this process has been restarted
   */
  restart_count: number;
  state: ProcessState;
  /**
   * Uptime in seconds (None if not running)
   */
  uptime_secs?: number | null;
  [k: string]: unknown;
}
