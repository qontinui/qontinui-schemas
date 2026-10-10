/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { CapabilityState } from "./CapabilityState";

/**
 * How one account's CLI state is kept apart from another's.
 *
 * Wire shape (internally tagged with `"kind"`):
 * - `{ "kind": "env_var", "name": "CLAUDE_CONFIG_DIR" }`
 * - `{ "kind": "home_dir" }` / `{ "kind": "none" }` / `{ "kind": "unknown" }`
 */
export type AccountIsolation =
  | {
      kind: "env_var";
      /**
       * The variable's name (`"CLAUDE_CONFIG_DIR"`).
       */
      name: string;
      [k: string]: unknown;
    }
  | {
      kind: "home_dir";
      [k: string]: unknown;
    }
  | {
      kind: "none";
      [k: string]: unknown;
    }
  | {
      kind: "unknown";
      [k: string]: unknown;
    };
/**
 * One way a CLI can be authenticated.
 *
 * Wire shape (internally tagged with `"kind"`):
 * - `{ "kind": "cli_login", "args": ["login"] }`
 * - `{ "kind": "api_key_env", "vars": ["OPENAI_API_KEY"] }`
 *
 * No `unknown` arm: an unknown auth story is an empty
 * [`CliProfile::auth`] list.
 */
export type AuthMethod =
  | {
      /**
       * Arguments after the program (`["login"]`, or `["/login"]` typed in
       * the TUI).
       */
      args: string[];
      kind: "cli_login";
      [k: string]: unknown;
    }
  | {
      kind: "api_key_env";
      /**
       * Variable names, in the CLI's precedence order.
       */
      vars: string[];
      [k: string]: unknown;
    };
/**
 * How the CLI is launched without per-tool approval prompts.
 *
 * Wire shape (internally tagged with `"kind"`):
 * - `{ "kind": "flags", "argv": ["--permission-mode", "bypassPermissions"],
 *   "detect": ["--dangerously-skip-permissions", "bypassPermissions"] }`
 * - `{ "kind": "none" }` / `{ "kind": "unknown" }`
 */
export type AutoApprove =
  | {
      /**
       * The flags the runner appends to enable auto-approval.
       */
      argv: string[];
      /**
       * Tokens whose presence on an existing argv means auto-approval is
       * already on — every spelling the CLI accepts, not only the one the
       * runner writes.
       */
      detect: string[];
      kind: "flags";
      [k: string]: unknown;
    }
  | {
      kind: "none";
      [k: string]: unknown;
    }
  | {
      kind: "unknown";
      [k: string]: unknown;
    };
/**
 * How the runner asks a live session to exit cleanly.
 *
 * Wire shape (internally tagged with `"kind"`):
 * - `{ "kind": "typed_command", "text": "/exit" }`
 * - `{ "kind": "signal" }` / `{ "kind": "unknown" }`
 *
 * `Unknown` means the runner must type nothing and report that it could not
 * exit the session gracefully.
 */
export type GracefulExit =
  | {
      kind: "typed_command";
      /**
       * The command text (`"/exit"`).
       */
      text: string;
      [k: string]: unknown;
    }
  | {
      kind: "signal";
      [k: string]: unknown;
    }
  | {
      kind: "unknown";
      [k: string]: unknown;
    };
/**
 * How the runner learns a CLI session's id.
 *
 * Wire shape (internally tagged with `"kind"`):
 * - `{ "kind": "pinned", "flag": "--session-id" }`
 * - `{ "kind": "read_back", "capture": "codex_session_file" }`
 * - `{ "kind": "unknown" }`
 */
export type IdentitySource =
  | {
      /**
       * The flag that takes the id (`"--session-id"`).
       */
      flag: string;
      kind: "pinned";
      [k: string]: unknown;
    }
  | {
      /**
       * Name of the runner capture mechanism that recovers the id
       * (`"codex_session_file"`). The mechanism itself is runner behaviour.
       */
      capture: string;
      kind: "read_back";
      [k: string]: unknown;
    }
  | {
      kind: "unknown";
      [k: string]: unknown;
    };
/**
 * What a restore of a CLI's session can honestly bring back.
 *
 * Serialized in the coord wire spelling the runner's `RestoreTier::wire_str`
 * already uses: `"full"` / `"terminal_only"`.
 *
 * Defaults to `TerminalOnly` — the conservative claim: a profile that has not
 * established resume must not promise the conversation back.
 */
export type RestoreTier = "full" | "terminal_only";
/**
 * How a session is resumed by id.
 *
 * Wire shape (internally tagged with `"kind"`):
 * - `{ "kind": "by_id_argv", "template": ["claude", "--resume", "{id}"] }`
 * - `{ "kind": "none" }` / `{ "kind": "unknown" }`
 */
export type ResumeSpec =
  | {
      kind: "by_id_argv";
      /**
       * The argv template (`["codex", "resume", "{id}"]`).
       */
      template: string[];
      [k: string]: unknown;
    }
  | {
      kind: "none";
      [k: string]: unknown;
    }
  | {
      kind: "unknown";
      [k: string]: unknown;
    };
/**
 * The machine-readable protocol a CLI offers besides its TUI.
 *
 * Wire shape (internally tagged with `"kind"` so an arm can gain data, e.g.
 * an adapter command for `acp`, without a breaking change):
 * `{ "kind": "claude_stream_json" }`, `{ "kind": "codex_app_server" }`,
 * `{ "kind": "acp" }`, `{ "kind": "none" }`, `{ "kind": "unknown" }`.
 */
export type StructuredLane =
  | {
      kind: "claude_stream_json";
      [k: string]: unknown;
    }
  | {
      kind: "codex_app_server";
      [k: string]: unknown;
    }
  | {
      kind: "acp";
      [k: string]: unknown;
    }
  | {
      kind: "none";
      [k: string]: unknown;
    }
  | {
      kind: "unknown";
      [k: string]: unknown;
    };
/**
 * Where the CLI writes its conversation transcript.
 *
 * Wire shape (internally tagged with `"kind"`):
 * - `{ "kind": "jsonl_under_config_dir", "subdir": "projects" }`
 * - `{ "kind": "session_file_glob", "glob": ".codex/sessions/** /rollout-*.jsonl" }`
 * - `{ "kind": "none" }` / `{ "kind": "unknown" }`
 */
export type TranscriptSpec =
  | {
      kind: "jsonl_under_config_dir";
      /**
       * Subdirectory of the config dir holding transcripts (`"projects"`).
       */
      subdir: string;
      [k: string]: unknown;
    }
  | {
      /**
       * The glob (`".codex/sessions/** /rollout-*.jsonl"`).
       */
      glob: string;
      kind: "session_file_glob";
      [k: string]: unknown;
    }
  | {
      kind: "none";
      [k: string]: unknown;
    }
  | {
      kind: "unknown";
      [k: string]: unknown;
    };
/**
 * Whether the CLI stops on a folder-trust prompt at first launch in a
 * directory.
 *
 * Wire shape (internally tagged with `"kind"`):
 * - `{ "kind": "prompt", "markers": ["Do you trust the files in this folder?"], "accept": "1" }`
 * - `{ "kind": "none" }` / `{ "kind": "unknown" }`
 */
export type TrustDialog =
  | {
      /**
       * The keystroke text that accepts it (sent followed by Enter).
       */
      accept: string;
      kind: "prompt";
      /**
       * Case-insensitive substrings that identify the prompt on the
       * ANSI-stripped screen.
       */
      markers: string[];
      [k: string]: unknown;
    }
  | {
      kind: "none";
      [k: string]: unknown;
    }
  | {
      kind: "unknown";
      [k: string]: unknown;
    };

/**
 * Everything the runner needs to know about one AI CLI's session behaviour.
 *
 * One instance per CLI (`claude`, `codex`, …), built by the runner as static
 * data and served to the frontend (`GET /terminals/cli-profiles`, Tauri
 * command `terminal_cli_profiles`) so no consumer holds a second copy.
 *
 * Every fact that can be unverified defaults to its `Unknown` arm; see the
 * module docs. Scraped facts ([`Self::handshake`], [`Self::usage_limit_phrases`],
 * [`TrustDialog::Prompt`]) break when the CLI changes its screens, which is
 * why [`Self::verified_version`] records the CLI version they were verified
 * against.
 */
export interface CliProfile {
  /**
   * How one account's CLI state is kept apart from another's.
   */
  accountIsolation?: AccountIsolation & {};
  /**
   * The ways the CLI can be authenticated. Empty means none are recorded —
   * UNKNOWN, not "needs no authentication".
   */
  auth: AuthMethod[];
  /**
   * How the CLI is launched without per-tool approval prompts, and how an
   * existing argv is recognised as already doing so.
   */
  autoApprove?: AutoApprove & {};
  /**
   * Human-readable name for launch menus and zone labels (`"Claude Code"`).
   */
  displayName: string;
  /**
   * How the runner asks a live session to exit cleanly.
   */
  gracefulExit?: GracefulExit & {};
  /**
   * On-screen markers that decide whether a resume succeeded or failed.
   */
  handshake?: HandshakePatterns & {};
  /**
   * Stable provider id (`"claude"`, `"codex"`). The same string the runner
   * stores as a terminal session record's `provider`.
   */
  id: string;
  /**
   * How the runner learns the CLI's session id.
   */
  identity?: IdentitySource & {};
  /**
   * Per-OS install commands, shown beside a disabled launch entry when the
   * CLI's binary is absent.
   */
  install?: InstallCommands & {};
  /**
   * Free-form caveats a reader should see beside the profile (e.g. "macOS:
   * same code path, unverified on hardware").
   */
  notes?: string[];
  /**
   * Arguments that make the structured lane hand each tool approval to the
   * runner as a typed permission request instead of deciding it itself
   * (`["--permission-prompt-tool", "stdio"]`), verified against
   * [`Self::verified_version`]. Rendered only for a launch that asks to be
   * prompted, and never beside an auto-approve flag. Empty means no such
   * arguments are known, so a prompted structured launch is not offered.
   */
  permissionPromptArgs?: string[];
  /**
   * Program stems that identify this CLI on an argv head or in a process
   * census, without directory or extension (`["claude"]`). Matching strips
   * the path and a Windows `.exe`/`.cmd` suffix before comparing.
   */
  programs: string[];
  /**
   * Arguments the runner appends to every PTY-hosted (interactive TUI)
   * launch of this CLI, verified against [`Self::verified_version`]
   * (`["--teammate-mode", "in-process"]`, which keeps Claude Code agent
   * teams inside the session instead of splitting the caller's tmux window).
   * Empty means none.
   */
  ptyArgs?: string[];
  /**
   * Whether the structured lane emits a typed rate-limit event.
   */
  rateLimitEvent?: CapabilityState & string;
  /**
   * What a restore of this CLI's session can honestly bring back.
   */
  restoreTier?: RestoreTier & string;
  /**
   * How a session is resumed by id.
   */
  resume?: ResumeSpec & {};
  /**
   * Other spellings the CLI accepts for the resume flag of
   * [`ResumeSpec::ByIdArgv`] (`["-r"]`). Matched only as a whole argv
   * token followed by the id, and only when the argv's program is this
   * CLI: a short flag like `-r` means something else to most programs.
   * Empty means none are known.
   */
  resumeFlagAliases?: string[];
  /**
   * Argv tokens, besides the identity flag, the resume flag and its
   * aliases, by which the user picks the session themselves
   * (`["--continue", "-c"]` — continue the most recent session). Any of
   * them on a launch means the runner must not pin an id of its own. A token
   * matches a whole argv entry case-insensitively; a `--long` token also
   * matches `--long=…`. Empty means none are known.
   */
  sessionChoiceArgs?: string[];
  /**
   * The machine-readable protocol the CLI offers besides its TUI.
   */
  structuredLane?: StructuredLane & {};
  /**
   * Where the CLI writes its conversation transcript.
   */
  transcript?: TranscriptSpec & {};
  /**
   * Whether the CLI stops on a folder-trust prompt at first launch in a
   * directory, and how to recognise and answer it.
   */
  trustDialog?: TrustDialog & {};
  /**
   * Whether the structured lane emits an explicit end-of-turn event.
   */
  turnBoundaryEvent?: CapabilityState & string;
  /**
   * Whether the structured lane delivers a typed permission request (a
   * tool-use approval the runner answers) rather than bypassing approval.
   */
  typedPermission?: CapabilityState & string;
  /**
   * Case-insensitive phrases the CLI paints when an account's usage limit
   * is reached. A grid match is a `Hint`-confidence
   * [`FailureKind::QuotaExhausted`] signal, never a confirmed one. Empty
   * means no phrases are known.
   */
  usageLimitPhrases: string[];
  /**
   * CLI version the scraped facts in this profile were verified against
   * (`"2.1.285"`). `None` means no version was recorded — treat every
   * scraped fact as unverified.
   */
  verifiedVersion?: string | null;
  [k: string]: unknown;
}
/**
 * On-screen markers that decide whether a resume succeeded or failed.
 *
 * Substring lists match the ANSI-stripped screen case-insensitively; regex
 * lists are compiled by BOTH the Rust `regex` crate and JavaScript `RegExp`,
 * so a pattern must stay in the syntax both engines accept (no look-around,
 * no backreferences, no named-group syntax that differs between them).
 *
 * Regex sources are ALWAYS matched case-insensitively, like the substrings:
 * each engine compiles them with its own case-insensitive switch (Rust
 * `RegexBuilder::case_insensitive`, the JavaScript `i` flag). A source must
 * therefore not carry inline flags such as `(?i)`, which JavaScript rejects.
 *
 * All body lists empty means no markers are known, and a resume cannot be
 * verified from the screen.
 */
export interface HandshakePatterns {
  /**
   * Substrings whose presence means the resume failed.
   */
  failure: string[];
  /**
   * Regex sources (matched case-insensitively) whose match means the
   * resume failed.
   */
  failureRegex: string[];
  /**
   * Substrings whose presence means the resumed session came up.
   */
  success: string[];
  /**
   * Regex sources (matched case-insensitively) whose match means the
   * resumed session came up.
   */
  successRegex: string[];
  /**
   * Regex sources (matched case-insensitively) matched against the pane's
   * CURRENT window title only, never its body, whose match means the
   * resumed session came up. A title the CLI sets at launch is evidence the
   * body cannot fake, provided the pattern is anchored to the CLI's own
   * title (the shell titles the window too). Empty means none are known.
   */
  titleRegex?: string[];
  [k: string]: unknown;
}
/**
 * Per-OS install commands for a CLI. `None` for an OS means no command is
 * recorded for it.
 */
export interface InstallCommands {
  /**
   * Install command on Linux (`"npm i -g @openai/codex"`).
   */
  linux?: string | null;
  /**
   * Install command on macOS.
   */
  macos?: string | null;
  /**
   * Install command on Windows.
   */
  windows?: string | null;
  [k: string]: unknown;
}
