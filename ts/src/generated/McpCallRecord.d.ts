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
  createdAt: string;
  durationMs: number;
  errorMessage?: string | null;
  extractions?: string | null;
  id: string;
  resolvedArguments?: string | null;
  response?: string | null;
  responseType: string;
  serverId: string;
  serverName?: string | null;
  stepId: string;
  stepName?: string | null;
  success: boolean;
  taskRunId: string;
  toolName: string;
  [k: string]: unknown;
}
