/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { RunnerCrash } from './RunnerCrash';
import type { RunnerStatus } from './RunnerStatus';
import type { RunnerUiError } from './RunnerUiError';

/**
 * Canonical runner entity. One row per registered runner; the same shape is
 * served to every consumer (mobile, web, runner UI).
 *
 * Replaces the prior split between `runners` (fleet/heartbeat),
 * `runner_connections` (transient WS sessions), and per-consumer projection
 * types like `RunnerConnection` / `ServerRunner` / `RegisteredRunner` /
 * `WebIntegrationStatus`.
 */
export interface Runner {
  /**
   * Free-form capability tags advertised by the runner
   * (e.g., `["python", "playwright", "vision"]`).
   */
  capabilities: string[];
  /**
   * ISO 8601 timestamp when the runner was first registered.
   */
  createdAt: string;
  derivedStatus: RunnerStatus;
  /**
   * Reported hostname of the machine the runner is running on.
   */
  hostname?: string | null;
  /**
   * Runner identifier (UUID as a string).
   */
  id: string;
  /**
   * Reported network address the runner is reachable on.
   */
  ipAddress?: string | null;
  /**
   * ISO 8601 timestamp of the most recent heartbeat received from the
   * runner, if any.
   */
  lastHeartbeat?: string | null;
  /**
   * Human-readable runner name.
   */
  name: string;
  /**
   * Operating system identifier (e.g., `"windows"`, `"macos"`, `"linux"`).
   */
  os?: string | null;
  /**
   * Operating system version string.
   */
  osVersion?: string | null;
  /**
   * Reported port the runner's HTTP API is listening on.
   */
  port?: number | null;
  /**
   * Most recently captured crash dump metadata, if any.
   */
  recentCrash?: RunnerCrash | null;
  /**
   * Most recently reported structured UI error, if any.
   */
  uiError?: RunnerUiError | null;
  /**
   * Owning user identifier (UUID as a string).
   */
  userId: string;
  /**
   * Whether the server currently holds an open WebSocket from the runner.
   */
  wsConnected: boolean;
}
