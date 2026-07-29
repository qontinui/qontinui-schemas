/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Lightweight reference to an MCP connection for [`ScheduledTaskType::RemoteAgent`].
 *
 * `name` is the MCP server name as registered in the runner's MCP config.
 * `url` is an optional override; when omitted the runner resolves the URL
 * from its existing MCP config at dispatch time.
 */
export interface McpConnectionRef {
  /**
   * MCP server name as registered in the runner's MCP config.
   */
  name: string;
  /**
   * Optional URL override; falls back to the runner's MCP config when
   * `None`.
   */
  url?: string | null;
}
