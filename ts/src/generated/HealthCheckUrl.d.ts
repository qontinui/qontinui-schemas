/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Configuration for a single health check URL.
 *
 * A workflow can have zero or more of these; they run before verification to
 * confirm required services are up.
 */
export interface HealthCheckUrl {
  /**
   * Expected HTTP status code (default: `200`).
   */
  expected_status: number;
  /**
   * Whether failure should stop the workflow (default: `true`).
   */
  is_critical: boolean;
  /**
   * Display name for the health check (e.g., `"Backend Server"`).
   */
  name: string;
  /**
   * Timeout in seconds (default: `30`).
   */
  timeout_seconds: number;
  /**
   * URL to check (e.g., `"http://localhost:8000/health"`).
   */
  url: string;
  [k: string]: unknown;
}
