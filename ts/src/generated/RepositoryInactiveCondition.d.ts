/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { RepositoryWatch } from './RepositoryWatch';

/**
 * Condition that requires repositories to have no file modifications for a
 * period. ALL configured repositories must be inactive for the overall
 * condition to be met.
 */
export interface RepositoryInactiveCondition {
  /**
   * Whether this condition is active.
   */
  enabled: boolean;
  /**
   * List of repositories to watch. ALL must be inactive simultaneously.
   */
  repositories: RepositoryWatch[];
  [k: string]: unknown;
}
