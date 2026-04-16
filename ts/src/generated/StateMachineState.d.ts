/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { DomainKnowledge } from './DomainKnowledge';

/**
 * A UI state, defined by which elements are present on screen.
 *
 * States are discovered via co-occurrence analysis of DOM snapshots.
 */
export interface StateMachineState {
  /**
   * Acceptance criteria for verifying the state is reached.
   */
  acceptance_criteria: string[];
  /**
   * Confidence score (0.0–1.0) from the discovery pass.
   */
  confidence: number;
  /**
   * Parent config ID.
   */
  config_id: string;
  /**
   * ISO 8601 timestamp of creation.
   */
  created_at: string;
  /**
   * Optional description.
   */
  description?: string | null;
  /**
   * Domain knowledge entries attached to this state.
   */
  domain_knowledge: DomainKnowledge[];
  /**
   * Element IDs that must be present for this state to be active.
   */
  element_ids: string[];
  /**
   * Free-form metadata bag.
   */
  extra_metadata: {
    [k: string]: unknown;
  };
  /**
   * Unique database identifier (UUID).
   */
  id: string;
  /**
   * Display name.
   */
  name: string;
  /**
   * Render IDs that contributed to discovering this state.
   */
  render_ids: string[];
  /**
   * Stable logical state ID (distinct from `id`; set by the user or
   * generator).
   */
  state_id: string;
  /**
   * ISO 8601 timestamp of last modification.
   */
  updated_at: string;
  [k: string]: unknown;
}
