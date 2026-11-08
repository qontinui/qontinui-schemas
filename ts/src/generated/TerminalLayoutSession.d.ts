/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * One terminal on a [`TerminalLayoutPage`], joined to the session running in
 * it.
 *
 * The session fields (`claudeSessionId`, `accountLabel`, `sessionName`,
 * `zoneIndex`, `finished`, `finishedAt`, `finishReason`) come from the
 * terminal's CONFIRMED open session record only. A terminal with none, such
 * as a plain shell, is still listed, with all of them `null`. `finished` is
 * the runner's own finished marker; coord's view of the session is not part
 * of this type.
 *
 * See [`TerminalLayout`] for the null and forward-compatibility rules.
 */
export interface TerminalLayoutSession {
  /**
   * The Claude account the session runs under; `null` when unknown (the
   * runner's own `"unknown"` label is served as `null`).
   */
  accountLabel: string | null;
  /**
   * The id of the terminal's confirmed session (the runner's
   * `claude_session_id`; a UUID for a Claude Code session).
   */
  claudeSessionId: string | null;
  /**
   * Why the session was marked finished, as recorded with the mark.
   */
  finishReason: string | null;
  /**
   * The runner's finished marker for the session: `true` when it was
   * marked finished, `false` when its confirmed record carries no finished
   * mark, `null` when unknown.
   */
  finished: boolean | null;
  /**
   * When the session was marked finished, as an RFC 3339 timestamp; `null`
   * when it was not marked finished or the time is unknown.
   */
  finishedAt: string | null;
  /**
   * Whether the terminal's process is still running.
   */
  isAlive: boolean;
  /**
   * The session's display name.
   */
  sessionName: string | null;
  /**
   * The name the runner spawned the terminal under, where it has one.
   */
  spawnName: string | null;
  /**
   * The runner terminal id, the same value as [`TerminalInfo::id`].
   */
  terminalId: string;
  /**
   * The terminal's title. It is set by the program in the terminal (OSC 0/2
   * escapes in its output), so it is session content. `null` when the
   * terminal has no title, or when it was withheld (`titleWithheld`).
   */
  title: string | null;
  /**
   * `true` when the runner's egress gate withheld `title` because the
   * session's tenant does not let terminal content leave the machine (its
   * terminal-stream switch is off); `title` is then `null`. `false` when
   * nothing was withheld.
   */
  titleWithheld: boolean;
  /**
   * The grid zone the session's tile occupies; `null` when the tile is
   * unzoned or its zone is unknown. Unsigned, so the runner's `-1` sentinel
   * cannot reach the wire: it is served as `null`.
   */
  zoneIndex: number | null;
  [k: string]: unknown;
}
