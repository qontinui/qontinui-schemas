/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * A comment on a ticket.
 */
export interface TicketComment {
  /**
   * Comment author's username / handle.
   */
  author: string;
  /**
   * Comment body (Markdown for GitHub/Linear, wiki-markup on Jira).
   */
  body: string;
  /**
   * ISO 8601 timestamp when the comment was created.
   */
  createdAt: string;
  /**
   * Provider-assigned comment ID.
   */
  id: string;
}
