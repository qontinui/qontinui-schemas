/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { HealthLevel } from "./HealthLevel";

/**
 * One green/amber/red answer to "is it broken?", plus the plain-English
 * reason behind it.
 */
export interface HealthLite {
  /**
   * Names of the processes driving an amber/red verdict.
   */
  failingProcesses: string[];
  /**
   * The traffic light.
   */
  level?: HealthLevel & string;
  /**
   * Why — a sentence a non-developer can read
   * ("The website still starts up fine").
   */
  reason: string;
}
