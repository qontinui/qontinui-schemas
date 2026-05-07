/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Wire envelope for `ui-bridge-response` events emitted by the React
 * frontend in response to a [`UiBridgeRequestEnvelope`].
 *
 * The runner-side dispatcher in
 * `mcp::ui_bridge::request::handle_ui_bridge_response` matches `request_id`
 * to a pending oneshot sender and forwards the inner `data` to the awaiting
 * caller; `success`, `error`, `hint`, and `timestamp` are surfaced via
 * `wrap_ipc_result`'s F2 two-tier envelope flattening.
 *
 * `data`, `error`, and `hint` are all optional. The hint sibling carries
 * closest-match / recovery hints (set by frontend handlers like
 * `useControlEvents` for typo recovery on element-not-found and
 * action-not-allowed) and stays a sibling of `error` — the success/error
 * envelope shape is unchanged.
 */
export interface UiBridgeResponseEnvelope {
  /**
   * Inner data payload on success. Frontend handlers set this to whatever
   * shape the operation returns (e.g. an `elements` array for
   * `get_elements`, a discovery report for `discover`, etc.).
   */
  data?: {
    [k: string]: unknown;
  };
  /**
   * Error message on failure. Mirrors the inner-failure path of
   * `wrap_ipc_result`'s F2 two-tier envelope flattening.
   */
  error?: string | null;
  /**
   * Optional closest-match / recovery hint payload. Sibling field to
   * `error` (does NOT replace the success/error envelope shape). Used by
   * `useControlEvents` for element-not-found / action-not-allowed typo
   * recovery and by Rust `page.rs` for eval-rejected workaround guidance.
   */
  hint?: {
    [k: string]: unknown;
  };
  /**
   * Echoes the request's [`UiBridgeRequestEnvelope::request_id`].
   */
  requestId: string;
  /**
   * Whether the frontend handler succeeded.
   */
  success: boolean;
  /**
   * Frontend-side timestamp (ms since Unix epoch) of when the response was
   * produced.
   */
  timestamp: number;
  /**
   * Echoes the request's [`UiBridgeRequestEnvelope::request_type`].
   * Serializes as `type` to match the wire.
   */
  type: string;
  [k: string]: unknown;
}
