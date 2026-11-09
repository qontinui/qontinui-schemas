/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { TerminalLayoutPage } from "./TerminalLayoutPage";
import type { TerminalLayoutRunner } from "./TerminalLayoutRunner";
import type { TerminalLayoutSession } from "./TerminalLayoutSession";

/**
 * One runner's terminal tabs and the sessions open in each, read at one
 * instant: the body of the runner's `GET /terminal-layout`.
 *
 * qontinui-runner serializes it. qontinui-web relays one GET to the runner,
 * validates the body with the generated Python model and keeps it as a saved
 * tab list on the `/sessions` page for 14 days (plan
 * `2026-11-08-save-per-tab-runner-session-lists-on-sessions-page-kept-14-days`).
 * One definition, here, serves both sides.
 *
 * Two wire rules hold for this type and the three it contains
 * ([`TerminalLayoutRunner`], [`TerminalLayoutPage`],
 * [`TerminalLayoutSession`]):
 *
 * - **null means unknown or not applicable, as each field says, and is never
 *   a stand-in such as `false`, `""` or `-1`.** An unknown finished state is
 *   `null`, not `false`; an unknown account is `null`, not `""`; an unzoned
 *   tile is `null`, not `-1`. The optional fields are always serialized,
 *   `null` included, so the wire states the null instead of leaving the key
 *   out, and a missing key also reads as `null`.
 * - **No `deny_unknown_fields`**, neither serde's nor schemars', on purpose.
 *   The generated models then ignore keys they do not know, so a newer runner
 *   that adds a field never makes an older web refuse a capture. Do not add
 *   it.
 */
export interface TerminalLayout {
  /**
   * When the runner built this snapshot: an RFC 3339 timestamp on the
   * runner's own clock.
   */
  capturedAt: string;
  /**
   * Whether the runner's webview has registered its tab list (labels and
   * left-to-right order) with the runner backend since this runner process
   * started.
   *
   * `false` means every page's `label` and `order` is `null` because nothing
   * was registered yet, not because the tabs have no names.
   */
  labelsRegistered: boolean;
  /**
   * The runner's terminal pages (tabs): registered pages first, in their
   * registered order, then any page the webview never registered.
   */
  pages: TerminalLayoutPage[];
  /**
   * The runner instance that answered, as it reports itself.
   */
  runner: TerminalLayoutRunner;
  [k: string]: unknown;
}
