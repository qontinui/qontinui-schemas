/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { UiBridgeHttpHealthAppInfo } from './UiBridgeHttpHealthAppInfo';
import type { UiBridgeHttpHealthData } from './UiBridgeHttpHealthData';

/**
 * Canonical success envelope for `GET /api/ui-bridge/health`.
 *
 * Distinct from [`UiBridgeResponseEnvelope`]: the HTTP health envelope has
 * no `requestId` / `type`, and carries an app-discovery `uiBridge` block.
 * Wire shape (`nextjs.ts::handleRelayRoute`):
 *
 * ```json
 * { "success": true,
 *   "data": { "responsive": false, "lastHeartbeat": 0, ...diagnostics },
 *   "timestamp": 1713200000000,
 *   "uiBridge": { "appId": "qontinui-web", ..., "capabilities": [...] } }
 * ```
 *
 * `uiBridge` is optional (omitted when the server has no `appInfo`); the
 * other three fields are always present.
 */
export interface UiBridgeHttpHealthEnvelope {
  data: UiBridgeHttpHealthData;
  /**
   * Always `true` on the health path.
   */
  success: boolean;
  /**
   * Server-side timestamp (ms since epoch) of when the response was produced.
   */
  timestamp: number;
  /**
   * App-discovery metadata; present only when the server is configured
   * with `appInfo`.
   */
  uiBridge?: UiBridgeHttpHealthAppInfo | null;
  [k: string]: unknown;
}
