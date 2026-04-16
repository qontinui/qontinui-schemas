/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Persisted MCP call record as read back from the `mcp_calls` table.
 *
 * Same shape as [`CreateMcpCallInput`] plus the row id and creation
 * timestamp.
 */
export interface McpCallRecord {
  arguments?: string | null;
  assertions?: string | null;
  /**
   * ISO 8601 creation timestamp.
   */
  created_at: string;
  duration_ms: number;
  error_message?: string | null;
  extractions?: string | null;
  id: string;
  resolved_arguments?: string | null;
  response?: string | null;
  response_type: string;
  server_id: string;
  server_name?: string | null;
  step_id: string;
  step_name?: string | null;
  success: boolean;
  task_run_id: string;
  tool_name: string;
  [k: string]: unknown;
}
