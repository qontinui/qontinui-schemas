/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Round-trip provenance linking a task back to what produced it.
 *
 * Carries the originating [`crate::findings`] / page / spec-check context so a
 * collected verdict can be folded back into the source flow. Spec-check exposes
 * `match_rate` / classification (not a "confidence" scalar), so the rate field
 * is named [`match_rate`](HelperTaskSource::match_rate) accordingly.
 */
export interface HelperTaskSource {
  /**
   * The finding this task was raised from, if any.
   */
  findingId?: string | null;
  /**
   * The spec-check match rate that triggered the task, if any — the
   * `[0.0, 1.0]` structural match score, **not** a confidence scalar.
   */
  matchRate?: number | null;
  /**
   * The page / spec the screenshot was captured from, if any.
   */
  pageId?: string | null;
  [k: string]: unknown;
}
