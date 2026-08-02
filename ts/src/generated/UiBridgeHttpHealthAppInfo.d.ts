/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * UI-Bridge metadata block included in the health envelope for the app
 * discovery scanner. Spread of `config.appInfo` plus a fixed `capabilities`
 * list. Present only when the server is configured with `appInfo` (the
 * qontinui-web proxy always is — `route.ts` sets `appId: "qontinui-web"`).
 */
export interface UiBridgeHttpHealthAppInfo {
  /**
   * Stable app identifier (e.g. `"qontinui-web"`).
   */
  appId: string;
  /**
   * Human-readable app name.
   */
  appName: string;
  /**
   * App surface type (e.g. `"web"`).
   */
  appType: string;
  /**
   * Advertised UI-Bridge capabilities (e.g. `["control","renderLog","debug"]`).
   */
  capabilities: string[];
  /**
   * UI framework (e.g. `"nextjs"`).
   */
  framework: string;
  [k: string]: unknown;
}
