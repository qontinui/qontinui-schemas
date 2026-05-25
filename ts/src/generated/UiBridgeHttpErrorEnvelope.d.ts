/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Canonical error envelope for the qontinui-web Next.js UI-Bridge proxy.
 *
 * The catch-all route at
 * `qontinui-web/frontend/src/app/api/ui-bridge/[...path]/route.ts`
 * short-circuits requests the SDK would otherwise 404 on — and the
 * browser-required routes that can't respond without a live SDK client —
 * to a structured HTTP 503 with this body
 * (`route.ts::noBrowserResponse`):
 *
 * ```json
 * { "success": false, "code": "NO_BROWSER_CONNECTED",
 *   "message": "<path> requires a browser SDK client" }
 * ```
 *
 * Modeled as a *generic* error envelope rather than a one-off
 * `NoBrowserConnected` type: the `{success:false, code, message}` shape is
 * the canonical structured-error contract for the whole proxy surface (not
 * just the no-browser path), so a single reusable type is the more scalable
 * + cleaner home. `code` stays a free-form `String` — the discriminator set
 * (currently just `NO_BROWSER_CONNECTED`) is expected to grow, and pinning
 * it to an enum here would force a schemas release on every new proxy error
 * code. `success` is always `false` on this envelope (the success path uses
 * [`UiBridgeHttpHealthEnvelope`] / the SDK's own envelopes).
 */
export interface UiBridgeHttpErrorEnvelope {
  /**
   * Machine-readable error discriminator (e.g. `"NO_BROWSER_CONNECTED"`).
   */
  code: string;
  /**
   * Human-readable explanation (e.g. `"<path> requires a browser SDK client"`).
   */
  message: string;
  /**
   * Always `false` — this envelope is only emitted on the error path.
   */
  success: boolean;
  [k: string]: unknown;
}
