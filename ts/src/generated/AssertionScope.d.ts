/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Filter narrowing which assertions / states a policy conjunct applies
 * to. Empty fields (default) mean "everything"; `assertion_ids` uses an
 * explicit `Option` to distinguish "everything" (`None`) from "none"
 * (`Some([])`).
 */
export interface AssertionScope {
  /**
   * Restrict to a specific set of assertion IDs. `None` = all
   * assertions; `Some([])` = no assertions (a deliberately empty
   * scope — useful for testing policies).
   */
  assertionIds?: string[] | null;
  /**
   * Restrict to these category values. Empty = all categories.
   */
  categories?: string[];
  /**
   * Restrict to these group identifiers. Empty = all groups.
   */
  groups?: string[];
  /**
   * Restrict to these severity values. Empty = all severities.
   */
  severities?: string[];
  /**
   * Restrict to these state IDs. Empty = all states.
   */
  states?: string[];
}
