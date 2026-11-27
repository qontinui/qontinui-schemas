/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { CommandOutput } from "./CommandOutput";
import type { McpConnectionRef } from "./McpConnectionRef";

/**
 * Type of task to schedule.
 *
 * Internally tagged by `task_type`: the variant fields are inlined alongside
 * the discriminator rather than nested under a `value` key.
 */
export type ScheduledTaskType =
  | {
      /**
       * Optional path to a workflow config file.
       */
      config_path?: string | null;
      /**
       * Optional monitor index to target.
       */
      monitor_index?: number | null;
      task_type: "Workflow";
      /**
       * If set, run a unified workflow by ID instead of a legacy workflow
       * by name.
       */
      workflow_id?: string | null;
      /**
       * Display name (also used to look up legacy workflows).
       */
      workflow_name: string;
      [k: string]: unknown;
    }
  | {
      /**
       * Optional override for `max_sessions`. `None` uses the prompt's own
       * setting.
       */
      max_sessions?: number | null;
      /**
       * ID of the prompt to run.
       */
      prompt_id: string;
      task_type: "Prompt";
      [k: string]: unknown;
    }
  | {
      /**
       * Whether to check the findings queue before running.
       */
      check_findings: boolean;
      /**
       * Force a run even if no findings are present.
       */
      force_run: boolean;
      task_type: "AutoFix";
      [k: string]: unknown;
    }
  | {
      task_type: "Watcher";
      /**
       * ID of the watcher definition in PostgreSQL.
       */
      watcher_id: string;
      [k: string]: unknown;
    }
  | {
      /**
       * Seconds between successive captures.
       */
      capture_interval_secs: number;
      /**
       * Whether to also trigger a capture on window focus change.
       */
      capture_on_focus_change: boolean;
      /**
       * Optional monitor index to capture.
       */
      monitor_index?: number | null;
      task_type: "BackgroundCapture";
      [k: string]: unknown;
    }
  | {
      /**
       * Allowed tool names (e.g. `["Bash", "Read", "Write", "Edit"]`).
       * Empty = use the runner's default tool allow-list.
       */
      allowed_tools?: string[];
      /**
       * Optional reasoning-effort level for the spawned session: one of
       * `low`, `medium`, `high`, `xhigh` or `max`. `None` leaves the level
       * to the runner. A plain string rather than an enum, so a stored
       * task carrying a level this crate does not list still loads; the
       * consumer validates it.
       */
      effort?: string | null;
      /**
       * Optional kebab-case name of a spawn-effort profile (e.g.
       * `research`, `implementation`) that the runner resolves to an
       * effort level and an ultracode setting. `None` leaves the choice
       * to the runner.
       */
      effort_profile?: string | null;
      /**
       * Hard cap on Claude turns per run (safety bound). `None` =
       * runner's default cap (typically 50).
       */
      max_turns?: number | null;
      /**
       * Optional MCP connection references resolved at dispatch time
       * against the runner's existing MCP config. Empty = inherit
       * whatever the runner currently has configured.
       */
      mcp_connections?: McpConnectionRef[];
      /**
       * Optional model override. `None` = runner default
       * (typically `claude-sonnet-4-6`).
       */
      model?: string | null;
      /**
       * The prompt content to send to the Claude session.
       */
      prompt: string;
      task_type: "RemoteAgent";
      /**
       * Wall-clock timeout in seconds. `None` = runner default
       * (typically 600s = 10 min).
       */
      timeout_seconds?: number | null;
      /**
       * Optional ultracode switch (standing dynamic-workflow orchestration)
       * for the spawned session. `Some(true)` turns it on, `Some(false)`
       * keeps it off, and `None` leaves the choice to the runner.
       */
      ultracode?: boolean | null;
      /**
       * Working directory for the spawned session. `None` means the
       * runner's project root. Stored as a string (not `PathBuf`) to keep
       * this crate JSON-Schema-friendly.
       */
      working_directory?: string | null;
      [k: string]: unknown;
    }
  | {
      /**
       * Shell command line, run via the platform shell (`sh -c`; on
       * Windows `bash -c`, then `cmd /C` as the fallback).
       */
      command: string;
      task_type: "Script";
      /**
       * Wall-clock timeout in seconds. `None` (or 0) = runner default
       * (600s = 10 min).
       */
      timeout_seconds?: number | null;
      /**
       * Working directory for the command. `None` = the runner's project
       * root.
       */
      working_directory?: string | null;
      [k: string]: unknown;
    }
  | {
      /**
       * Arguments passed to `program`, one element per argv entry, with no
       * shell splitting or expansion. Empty = no arguments.
       */
      args?: string[];
      /**
       * Launch the child detached from the runner, so it outlives the
       * runner's process-lifetime grouping, and record the run as
       * launched rather than waiting for its exit (so `output` and
       * `timeout_seconds` do not apply). Defaults to `false`.
       *
       * **Linux systemd caveat:** a runner run as a systemd unit with the
       * default `KillMode=control-group` kills a detached child on a
       * service stop or restart. `setsid` leaves the runner's session and
       * process group, not the unit's cgroup, so there the child does NOT
       * outlive the runner's service.
       */
      detach: boolean;
      /**
       * What happens to the child's output. Defaults to
       * `Tail { bytes: 2048 }` when absent.
       */
      output?: CommandOutput & {};
      /**
       * The program to exec: a bare name resolved on the runner's `PATH`,
       * or an absolute path. Must be non-empty.
       */
      program: string;
      task_type: "Command";
      /**
       * Wall-clock timeout in seconds. `None` = 3600 s (one hour; see
       * [`COMMAND_DEFAULT_TIMEOUT_SECONDS`]). On timeout the runner kills
       * the child's whole process tree, not only the direct child.
       */
      timeout_seconds?: number | null;
      /**
       * Working directory for the child. `None` = the runner's project
       * root. Stored as a string (not `PathBuf`) to keep this crate
       * JSON-Schema-friendly.
       */
      working_directory?: string | null;
      [k: string]: unknown;
    };
