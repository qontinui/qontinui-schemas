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
  duration_ms: number;
  /**
   * Error message if the call failed.
   */
  error_message?: string | null;
  /**
   * JSON-serialized extractions (variables pulled from the response).
   */
  extractions?: string | null;
  /**
   * JSON-serialized arguments after variable resolution.
   */
  resolved_arguments?: string | null;
  /**
   * JSON-serialized response body.
   */
  response?: string | null;
  /**
   * Response type tag (see [`McpToolCallResult::response_type`]).
   */
  response_type: string;
  server_id: string;
  server_name?: string | null;
  step_id: string;
  step_name?: string | null;
  /**
   * Whether the call succeeded.
   */
  success: boolean;
  task_run_id: string;
  tool_name: string;
  [k: string]: unknown;
}
