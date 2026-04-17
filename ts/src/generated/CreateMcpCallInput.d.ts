/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Input shape for recording an MCP call to the `mcp_calls` table.
 *
 * `arguments` / `resolved_arguments` / `response` / `extractions` /
 * `assertions` are serialized-JSON strings rather than `serde_json::Value`
 * because the DB layer stores them as `TEXT` / `JSONB` strings and does not
 * round-trip through a `Value`.
 */
export interface CreateMcpCallInput {
  /**
   * JSON-serialized arguments as originally submitted.
   */
  arguments?: string | null;
  /**
   * JSON-serialized assertion results.
   */
  assertions?: string | null;
  /**
   * Wall-clock duration in milliseconds.
   */
  durationMs: number;
  /**
   * Error message if the call failed.
   */
  errorMessage?: string | null;
  /**
   * JSON-serialized extractions (variables pulled from the response).
   */
  extractions?: string | null;
  /**
   * JSON-serialized arguments after variable resolution.
   */
  resolvedArguments?: string | null;
  /**
   * JSON-serialized response body.
   */
  response?: string | null;
  /**
   * Response type tag (see [`McpToolCallResult::response_type`]).
   */
  responseType: string;
  serverId: string;
  serverName?: string | null;
  stepId: string;
  stepName?: string | null;
  /**
   * Whether the call succeeded.
   */
  success: boolean;
  taskRunId: string;
  toolName: string;
  [k: string]: unknown;
}
