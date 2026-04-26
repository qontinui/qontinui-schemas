/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { McpConnectionRef } from './McpConnectionRef';

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
       * Working directory for the spawned session. `None` means the
       * runner's project root. Stored as a string (not `PathBuf`) to keep
       * this crate JSON-Schema-friendly.
       */
      working_directory?: string | null;
      [k: string]: unknown;
    };
