/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * A lightweight reference to a state by id and display name.
 *
 * Used in [`ResolvedInitialStates`] and [`ResolvedInitialStatesResult`] to
 * let the UI render human-readable lists without a separate lookup.
 */
export interface InitialStateRef {
  /**
   * State ID.
   */
  id: string;
  /**
   * Display name.
   */
  name: string;
}
