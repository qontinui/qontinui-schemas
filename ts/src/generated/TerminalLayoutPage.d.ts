/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { TerminalLayoutSession } from "./TerminalLayoutSession";

/**
 * One terminal page (a tab of the runner's terminal view) and the sessions
 * in it.
 *
 * See [`TerminalLayout`] for the null and forward-compatibility rules.
 */
export interface TerminalLayoutPage {
  /**
   * The tab label the webview registered for this page; `null` when no
   * label was registered for it.
   */
  label: string | null;
  /**
   * The page's left-to-right position as registered by the webview; `null`
   * when the page was never registered.
   */
  order: number | null;
  /**
   * The page id, the same value as [`TerminalInfo::page_id`].
   */
  pageId: string;
  /**
   * The terminals on this page.
   */
  sessions: TerminalLayoutSession[];
  [k: string]: unknown;
}
