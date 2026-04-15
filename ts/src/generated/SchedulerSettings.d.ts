/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Global scheduler settings.
 */
export interface SchedulerSettings {
  /**
   * Default `auto_fix_on_failure` value for newly created tasks.
   */
  default_auto_fix_on_failure: boolean;
  /**
   * Whether the scheduler is enabled globally.
   */
  enabled: boolean;
  /**
   * Maximum number of scheduled tasks allowed to run concurrently.
   */
  max_concurrent: number;
  /**
   * Timezone for schedule interpretation (IANA name). `None` = local time.
   */
  timezone?: string | null;
  [k: string]: unknown;
}
