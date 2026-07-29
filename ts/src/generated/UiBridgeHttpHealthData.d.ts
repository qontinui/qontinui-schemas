/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Inner `data` payload of the UI-Bridge SDK HTTP health envelope.
 *
 * Emitted by the SDK's relay-transport handler for `GET /health`
 * (`@qontinui/ui-bridge/server` `handleRelayRoute`, `nextjs.ts`): the
 * `responsive` + `lastHeartbeat` fields plus a spread of the relay's
 * `TransportDiagnostics`. Only `responsive` and `lastHeartbeat` are modeled
 * explicitly here — the diagnostics spread (pendingCommandCount, connectedTabs,
 * buildId, …) is intentionally left open (no `additionalProperties: false`,
 * which schemars omits by default) so the diagnostics surface can evolve
 * without a schemas release while the two stable health signals stay typed.
 */
export interface UiBridgeHttpHealthData {
  /**
   * Max heartbeat timestamp (ms since epoch) across all tabs; `0` when none.
   */
  lastHeartbeat: number;
  /**
   * True when at least one connected tab has a fresh heartbeat.
   */
  responsive: boolean;
  [k: string]: unknown;
}
