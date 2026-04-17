/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { HttpConfig } from './HttpConfig';
import type { McpTransport } from './McpTransport';
import type { StdioConfig } from './StdioConfig';

/**
 * Request body for creating a new MCP server configuration.
 *
 * **Secret surface**: as with [`McpServerConfig`], the nested transport
 * configs carry secrets.
 */
export interface CreateMcpServerInput {
  /**
   * Override for the default `auto_start = false`.
   */
  autoStart?: boolean | null;
  /**
   * Optional description.
   */
  description?: string | null;
  /**
   * Override for the default `enabled = true`.
   */
  enabled?: boolean | null;
  /**
   * HTTP config (required when `transport == Http`).
   */
  httpConfig?: HttpConfig | null;
  /**
   * Display name.
   */
  name: string;
  /**
   * Stdio config (required when `transport == Stdio`).
   */
  stdioConfig?: StdioConfig | null;
  /**
   * Override for the default `timeout_seconds = 30`.
   */
  timeoutSeconds?: number | null;
  transport: McpTransport;
  [k: string]: unknown;
}
