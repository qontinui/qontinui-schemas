/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { TicketSource } from './TicketSource';
import type { TicketState } from './TicketState';

/**
 * A ticket fetched from an external provider.
 *
 * Identity is carried by `(source, external_id)` — `external_id` is the
 * provider-specific identifier (GitHub issue number, Linear identifier, Jira
 * key) rather than a runner-assigned UUID.
 */
export interface Ticket {
  /**
   * Assignee username / handle, if any.
   */
  assignee?: string | null;
  /**
   * Ticket body / description (Markdown for GitHub/Linear, wiki-markup on Jira).
   */
  body: string;
  /**
   * ISO 8601 timestamp when the ticket was created.
   */
  created_at: string;
  /**
   * Provider-assigned external ID (issue number, ticket key, etc.).
   */
  external_id: string;
  /**
   * Labels applied to the ticket by the provider.
   */
  labels?: string[];
  source: TicketSource;
  state: TicketState;
  /**
   * Ticket title / summary.
   */
  title: string;
  /**
   * ISO 8601 timestamp when the ticket was last updated.
   */
  updated_at: string;
  /**
   * Canonical URL to the ticket in the provider's UI.
   */
  url: string;
  [k: string]: unknown;
}
