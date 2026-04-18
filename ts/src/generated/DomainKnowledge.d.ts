/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * A piece of domain knowledge attached to a state — free-form notes that
 * help the AI reason about what a state represents.
 */
export interface DomainKnowledge {
  /**
   * Full knowledge content (markdown or plain text).
   */
  content: string;
  /**
   * Unique identifier for this knowledge entry.
   */
  id: string;
  /**
   * Tags for filtering and grouping.
   */
  tags: string[];
  /**
   * Short title / headline.
   */
  title: string;
}
