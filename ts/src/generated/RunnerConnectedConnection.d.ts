/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Wire-format runner shape carried inside `runner_connected.connection`.
 *
 * This is intentionally a partial Runner payload (snake_case, only the
 * fields the Python emitter actually populates in
 * `RunnerEventPublisher.publish_runner_connected`). It is NOT
 * `qontinui_types::runner::Runner` — that shape requires `derived_status`,
 * `capabilities`, `created_at` etc., which the connection payload doesn't
 * include. Consumers reading this should refetch the canonical Runner row
 * via REST after seeing `runner_connected`.
 */
export interface RunnerConnectedConnection {
  connected_at?: string | null;
  /**
   * Always present as `null` on the wire when the runner is still
   * connected. Python ships this field unconditionally.
   */
  disconnected_at?: string | null;
  duration_seconds?: number | null;
  id: string;
  ip_address?: string | null;
  project_id?: string | null;
  runner_name?: string | null;
  runner_port?: number | null;
  ws_connected: boolean;
  [k: string]: unknown;
}
