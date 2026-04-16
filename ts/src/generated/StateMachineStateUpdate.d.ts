/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { DomainKnowledge } from './DomainKnowledge';

/**
 * Payload for updating an existing state.
 */
export interface StateMachineStateUpdate {
  /**
   * New acceptance criteria.
   */
  acceptance_criteria?: string[] | null;
  /**
   * New confidence score.
   */
  confidence?: number | null;
  /**
   * New description.
   */
  description?: string | null;
  /**
   * New domain knowledge entries.
   */
  domain_knowledge?: DomainKnowledge[] | null;
  /**
   * New element ID set.
   */
  element_ids?: string[] | null;
  /**
   * New metadata bag.
   */
  extra_metadata?: {
    [k: string]: unknown;
  } | null;
  /**
   * New display name.
   */
  name?: string | null;
  /**
   * New render ID set.
   */
  render_ids?: string[] | null;
  [k: string]: unknown;
}
