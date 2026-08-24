/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Failure modes for agent text units. Mirrors the tagged-enum shape of
 * [`crate::apps::AppError`] and [`crate::agent_commands::AgentCommandError`],
 * so the qontinui-web layer surfaces a rejected unit the same way it surfaces
 * a rejected app registration.
 *
 * One variant per raise site in qontinui-web's `AgentTextUnitValidationError`,
 * so a Rust caller can render exactly the refusal the store would give.
 */
export type AgentTextUnitError =
  | {
      name: string;
      reason: "invalid-name";
      [k: string]: unknown;
    }
  | {
      name: string;
      reason: "reserved-name";
      [k: string]: unknown;
    }
  | {
      kind: string;
      reason: "invalid-kind";
      [k: string]: unknown;
    }
  | {
      path: string;
      reason: "invalid-file-path";
      [k: string]: unknown;
    }
  | {
      path: string;
      reason: "reserved-file-path";
      [k: string]: unknown;
    }
  | {
      reason: "empty-file-set";
      [k: string]: unknown;
    }
  | {
      count: number;
      max: number;
      reason: "too-many-files";
      [k: string]: unknown;
    }
  | {
      path: string;
      reason: "blank-file";
      [k: string]: unknown;
    }
  | {
      bytes: number;
      max: number;
      path: string;
      reason: "file-too-large";
      [k: string]: unknown;
    }
  | {
      bytes: number;
      max: number;
      reason: "unit-too-large";
      [k: string]: unknown;
    }
  | {
      entrypoint: string;
      kind: string;
      name: string;
      reason: "missing-entrypoint";
      [k: string]: unknown;
    }
  | {
      name: string;
      reason: "underscore-not-invocable";
      [k: string]: unknown;
    };
