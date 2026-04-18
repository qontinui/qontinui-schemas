/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { DomainKnowledge } from './DomainKnowledge';

/**
 * Payload for creating a new state within a config.
 */
export interface StateMachineStateCreate {
  /**
   * Acceptance criteria.
   */
  acceptance_criteria?: string[] | null;
  /**
   * Initial confidence score.
   */
  confidence?: number | null;
  /**
   * Optional description.
   */
  description?: string | null;
  /**
   * Domain knowledge entries.
   */
  domain_knowledge?: DomainKnowledge[] | null;
  /**
   * Element IDs that define this state.
   */
  element_ids?: string[] | null;
  /**
   * Free-form metadata bag.
   */
  extra_metadata?: {
    [k: string]: unknown;
  } | null;
  /**
   * Display name.
   */
  name: string;
  /**
   * Render IDs associated with this state.
   */
  render_ids?: string[] | null;
  /**
   * Optional stable logical state ID (generated if omitted).
   */
  state_id?: string | null;
}
