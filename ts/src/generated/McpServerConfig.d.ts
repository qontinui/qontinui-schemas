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
 * Full configuration for a registered MCP server.
 *
 * Persisted in the `mcp_servers` Postgres table and surfaced to the frontend
 * through the `mcp_*` Tauri commands and the MCP `mcp-servers` HTTP
 * endpoint. The `transport` field selects which of `stdio_config` /
 * `http_config` is meaningful; the other is expected to be `None`.
 *
 * **Secret surface**: the nested `stdio_config` / `http_config` carry secrets
 * — see their own docs and the module-level security note.
 */
export interface McpServerConfig {
  /**
   * Auto-connect when the runner launches. Default: `false`.
   */
  auto_start: boolean;
  /**
   * Serialized JSON list of tools cached from the last successful
   * connection. Stored as a string for DB portability.
   */
  cached_tools?: string | null;
  /**
   * ISO 8601 creation timestamp.
   */
  created_at: string;
  /**
   * Optional human-readable description.
   */
  description?: string | null;
  /**
   * Whether this server is enabled. Disabled servers won't be connected
   * even if `auto_start` is true. Default: `true`.
   */
  enabled: boolean;
  /**
   * HTTP-transport settings. Expected to be `Some` iff
   * `transport == McpTransport::Http`.
   */
  http_config?: HttpConfig | null;
  /**
   * Unique identifier (UUID).
   */
  id: string;
  /**
   * Display name.
   */
  name: string;
  /**
   * Stdio-transport settings. Expected to be `Some` iff
   * `transport == McpTransport::Stdio`.
   */
  stdio_config?: StdioConfig | null;
  /**
   * Per-request connection / tool-call timeout in seconds. Default: `30`.
   */
  timeout_seconds: number;
  /**
   * ISO 8601 timestamp of when `cached_tools` was populated.
   */
  tools_cached_at?: string | null;
  transport: McpTransport;
  /**
   * ISO 8601 last-update timestamp.
   */
  updated_at: string;
  [k: string]: unknown;
}
