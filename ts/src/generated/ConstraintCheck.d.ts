/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * What to check and how.
 *
 * Internally tagged by the `type` field. Variants correspond to the four
 * check kinds implemented by the runner's constraint engine.
 */
export type ConstraintCheck =
  | {
      /**
       * Optional glob to limit which modified files are checked.
       * Default: all modified files.
       */
      file_glob?: string | null;
      /**
       * Regex pattern to search for.
       */
      pattern: string;
      type: "grep_forbidden";
      [k: string]: unknown;
    }
  | {
      /**
       * Optional glob to limit which modified files are checked.
       */
      file_glob?: string | null;
      /**
       * Regex pattern that must be present.
       */
      pattern: string;
      type: "grep_required";
      [k: string]: unknown;
    }
  | {
      /**
       * Allowed directory prefixes (relative to project root).
       * e.g., `["src/", "tests/", "config/"]`.
       */
      allowed_paths: string[];
      type: "file_scope";
      [k: string]: unknown;
    }
  | {
      /**
       * The command to run.
       */
      cmd: string;
      /**
       * Working directory (relative to project root). Default: project root.
       */
      cwd?: string | null;
      /**
       * Timeout in seconds. Default: 30.
       */
      timeout_secs: number;
      type: "command";
      [k: string]: unknown;
    };
