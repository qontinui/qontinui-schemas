/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { McpToolInfo } from './McpToolInfo';
import type { McpToolInputSchema } from './McpToolInputSchema';

/**
 * Status of a single MCP server, as reported by the client manager.
 *
 * Derived from runtime connection state; never persisted. `tools` is
 * populated only when `connected == true`.
 */
export interface McpServerStatus {
  /**
   * Whether the client currently holds a live connection.
   */
  connected: boolean;
  /**
   * Last connection / tool-call error, if any.
   */
  error?: string | null;
  /**
   * ISO 8601 timestamp of the most recent connection attempt.
   */
  lastConnectAttempt?: string | null;
  /**
   * ISO 8601 timestamp of the most recent successful connection.
   */
  lastConnected?: string | null;
  /**
   * ID of the server this status refers to.
   */
  serverId: string;
  /**
   * Available tools — `Some(…)` when connected, `None` otherwise.
   */
  tools?: McpToolInfo[] | null;
  [k: string]: unknown;
}
