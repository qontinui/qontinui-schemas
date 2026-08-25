/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { McpToolInputSchema } from "./McpToolInputSchema";

/**
 * Single tool exposed by an MCP server, as returned by `tools/list`.
 */
export interface McpToolInfo {
  /**
   * Tool description shown to users.
   */
  description?: string | null;
  /**
   * Input-argument schema. Wire field is `"inputSchema"` per MCP spec.
   */
  inputSchema: McpToolInputSchema;
  /**
   * Tool name (the argument passed back on `tools/call`).
   */
  name: string;
}
