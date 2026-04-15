/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Log source selection for a workflow.
 *
 * - `"default"` / `"ai"` / `"all"`: Use the corresponding [`LogSourceMode`].
 * - `{ "profile_id": "..." }`: Use a specific profile.
 *
 * Serialized as an untagged union: a bare string for the mode variants, or an
 * object with a `profile_id` key for the profile variant.
 */
export type LogSourceSelection =
  | ("default" | "ai" | "all")
  | {
      /**
       * ID of the log-source profile to use.
       */
      profile_id: string;
      [k: string]: unknown;
    };
