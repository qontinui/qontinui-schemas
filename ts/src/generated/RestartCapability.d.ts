/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { RestartPathKind } from "./RestartPathKind";
import type { RestartUnsupportedCode } from "./RestartUnsupportedCode";

/**
 * Pre-start verdict on whether the configured between-iterations mode can
 * restart the target.
 */
export interface RestartCapability {
  /**
   * Why the mode cannot run against this target. `Some` when `!supported`.
   */
  code?: RestartUnsupportedCode | null;
  /**
   * Instance-manager slot id of the target, when `path` is
   * `InstanceManager`.
   */
  instanceId?: string | null;
  /**
   * The mechanism that will restart the target. `Some` when `supported`.
   */
  path?: RestartPathKind | null;
  /**
   * Human-readable sentence explaining `code`. `Some` when `!supported`.
   */
  reason?: string | null;
  /**
   * Whether the configured mode can restart the target.
   */
  supported: boolean;
  /**
   * Port of the target runner the verdict was computed for.
   */
  targetPort: number;
}
