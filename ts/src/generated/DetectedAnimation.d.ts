/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * One CSS animation or transition, as the SDK's idle signals and the
 * timeline report it (`idle/detected-animation.ts`).
 */
export interface DetectedAnimation {
  /**
   * Duration (ms) when known — the elapsed time at end, or the effect's
   * timing.
   */
  durationMs?: number | null;
  /**
   * Whether the animation repeats forever.
   */
  isInfinite: boolean;
  /**
   * `animationName`, or `transition:<property>` for a CSS transition.
   */
  name: string;
  /**
   * `ElementFingerprintData.hash` of the animated element, when it could
   * be computed.
   */
  targetElement?: string | null;
  [k: string]: unknown;
}
