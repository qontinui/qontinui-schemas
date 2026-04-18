/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ParserType } from './ParserType';

/**
 * Configuration for a managed process.
 *
 * Persisted in `settings.json` under `managed_processes`. Surfaced to the
 * frontend through the `get_process_configs` command and to MCP consumers
 * through the `processes` endpoint.
 */
export interface ProcessConfig {
  /**
   * Command arguments (e.g., ["run", "dev"])
   */
  args: string[];
  /**
   * Start when runner launches
   */
  autoStart: boolean;
  /**
   * Ring buffer max lines (default 2000)
   */
  bufferSize: number;
  /**
   * Build command arguments (e.g., ["build"], ["run", "build"]).
   */
  buildArgs: string[];
  /**
   * Build command to run before restarting (e.g., "cargo", "npm").
   */
  buildCommand?: string | null;
  /**
   * Category (e.g., "backend", "frontend")
   */
  category: string;
  /**
   * Command to execute (e.g., "python", "npm", "cargo")
   */
  command: string;
  /**
   * Working directory (absolute path)
   */
  cwd: string;
  /**
   * Whether this is a dev-mode-only service (not started in production
   * builds).
   */
  devOnly: boolean;
  /**
   * Whether this config is enabled
   */
  enabled: boolean;
  /**
   * Extra environment variables
   */
  env: {
    [k: string]: string;
  };
  /**
   * Port to check for health (optional)
   */
  healthPort?: number | null;
  /**
   * Unique identifier (UUID)
   */
  id: string;
  /**
   * Regex patterns for errors to ignore (matched against error message and
   * raw entry).
   */
  ignorePatterns: string[];
  /**
   * Human-readable name (e.g., "FastAPI Backend")
   */
  name: string;
  /**
   * Parser type for error detection
   */
  parser?: ParserType & string;
  /**
   * Whether rebuild and AI fix features are enabled for this process.
   */
  rebuildEnabled: boolean;
  /**
   * Startup group for ordered startup (lower groups start first, default 0).
   * Processes in the same group start together. The runner waits for health
   * ports in each group to be ready before starting the next group.
   */
  startGroup: number;
}
