/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Provenance of a cached answer.
 *
 * Every key always serializes: `storedAt: null` states "the cache holds no
 * storage time for this entry", which is different from a producer that
 * said nothing.
 */
export interface CacheProvenance {
  /**
   * Whether this answer was served from the cache.
   */
  hit: boolean;
  /**
   * The inputs the cache key was derived from (e.g. `frame_sha256`,
   * `prompt_version`), so a reader can judge what a hit is keyed on.
   */
  keyInputs: string[];
  /**
   * When the served entry was stored. `null` on a miss (nothing was
   * served from storage) or when the cache keeps no timestamp.
   */
  storedAt: string | null;
  [k: string]: unknown;
}
