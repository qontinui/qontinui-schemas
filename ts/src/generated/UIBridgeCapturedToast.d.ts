/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * A captured toast/notification entry.
 */
export interface UIBridgeCapturedToast {
  appearedAt: number;
  dismissedAt?: number | null;
  durationMs: number;
  id: string;
  /**
   * Severity level. One of: info|success|warning|error|loading|unknown.
   */
  level: string;
  message: string;
  visible: boolean;
}
