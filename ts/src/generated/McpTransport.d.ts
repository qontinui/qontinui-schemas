/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Transport type for an MCP server connection.
 *
 * Serialized as lowercase to match the existing DB column values
 * (`"stdio"` / `"http"`). Defaults to `Stdio` to preserve the pre-extraction
 * default from the runner.
 */
export type McpTransport = "stdio" | "http";
