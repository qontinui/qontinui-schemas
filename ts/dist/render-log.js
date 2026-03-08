/**
 * Shared Render Log Types
 *
 * These types define the render logging schema used across:
 * - qontinui-runner (Tauri desktop app) - Component-level logging
 * - qontinui-web (Next.js web app) - Full DOM snapshot logging
 *
 * The runner reads both its own logs and web's logs for verification
 * in AI feedback loops.
 *
 * NOTE: Field names use snake_case to match Rust/Python JSON serialization.
 * This ensures compatibility across all services.
 */
// =============================================================================
// Type Guards
// =============================================================================
/**
 * Check if a render log entry is a component log.
 */
export function isComponentRenderLog(entry) {
    return entry.type === "component";
}
/**
 * Check if a render log entry is a DOM snapshot log.
 */
export function isDomSnapshotRenderLog(entry) {
    return entry.type === "dom_snapshot";
}
//# sourceMappingURL=render-log.js.map