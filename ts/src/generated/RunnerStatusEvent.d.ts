/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { RunnerConnectedConnection } from './RunnerConnectedConnection';

/**
 * Discriminated union of every event the Python backend pushes to the web
 * client over the `/api/v1/runners/status` WebSocket.
 *
 * Internally tagged on `"type"` matching the Python wire format. Variant
 * renames (`runner.woke` keeps the dot rather than going to snake_case)
 * preserve the literal strings the emitter sends.
 *
 * Source of truth: the `publish_*` methods in
 * `qontinui-web/backend/app/services/runner/event_publisher.py` and the
 * `initial_state` send in `runner_status_ws.py`.
 */
export type RunnerStatusEvent =
  | {
      runners: unknown[];
      type: "initial_state";
      [k: string]: unknown;
    }
  | {
      connection: RunnerConnectedConnection;
      timestamp: string;
      type: "runner_connected";
      [k: string]: unknown;
    }
  | {
      runner_id: string;
      timestamp: string;
      type: "runner_disconnected";
      [k: string]: unknown;
    }
  | {
      runner_id: string;
      runner_name: string;
      timestamp: string;
      type: "runner_name_updated";
      [k: string]: unknown;
    }
  | {
      runner_id: string;
      runner_port: number;
      timestamp: string;
      type: "runner_port_updated";
      [k: string]: unknown;
    }
  | {
      intent_id?: string | null;
      reason?: string | null;
      runner_id: string;
      task_id?: string | null;
      timestamp: string;
      type: "runner.woke";
      [k: string]: unknown;
    }
  | {
      error: string;
      type: "error";
      [k: string]: unknown;
    };
