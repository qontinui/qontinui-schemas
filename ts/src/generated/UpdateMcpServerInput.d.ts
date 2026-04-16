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
 * Request body for updating an MCP server configuration. Every field is
 * optional — fields left as `None` are preserved from the existing row.
 *
 * **Secret surface**: as with [`McpServerConfig`], the nested transport
 * configs carry secrets.
 */
export interface UpdateMcpServerInput {
  auto_start?: boolean | null;
  description?: string | null;
  enabled?: boolean | null;
  http_config?: HttpConfig | null;
  name?: string | null;
  stdio_config?: StdioConfig | null;
  timeout_seconds?: number | null;
  transport?: McpTransport | null;
  [k: string]: unknown;
}
