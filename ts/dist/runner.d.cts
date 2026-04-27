/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Most recent crash dump metadata, if any.
 */
interface RunnerCrash {
  /**
   * Path to the crash dump file on disk.
   */
  filePath: string;
  /**
   * Source location (file/line) where the panic originated.
   */
  panicLocation: string;
  /**
   * Panic message captured from the runner process.
   */
  panicMessage: string;
  /**
   * ISO 8601 timestamp when the crash was reported.
   */
  reportedAt: string;
  /**
   * Name of the thread that panicked.
   */
  thread: string;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Health/availability state of a runner, computed server-side from the
 * WebSocket-presence and heartbeat-freshness signals.
 */
type RunnerStatus = "healthy" | "degraded" | "offline" | "starting" | "errored";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Structured error reported by the runner that the UI surfaces verbatim.
 */
interface RunnerUiError {
  /**
   * Optional long-form detail (stack trace, stderr, etc.).
   */
  detail?: string | null;
  /**
   * Error category (e.g., `"build_failed"`, `"port_conflict"`).
   */
  kind: string;
  /**
   * Short, user-facing error message.
   */
  message: string;
  /**
   * ISO 8601 timestamp when the error was reported.
   */
  reportedAt: string;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Canonical runner entity. One row per registered runner; the same shape is
 * served to every consumer (mobile, web, runner UI).
 *
 * Replaces the prior split between `runners` (fleet/heartbeat),
 * `runner_connections` (transient WS sessions), and per-consumer projection
 * types like `RunnerConnection` / `ServerRunner` / `RegisteredRunner` /
 * `WebIntegrationStatus`.
 */
interface Runner {
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

export type { Runner, RunnerCrash, RunnerStatus, RunnerUiError };
