/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Failure modes for agent-command overrides. Mirrors the tagged-enum shape of
 * [`crate::apps::AppError`], so the qontinui-web layer can surface a rejected
 * override the same way it surfaces a rejected app registration.
 */
export type AgentCommandError =
  | {
      name: string;
      reason: "invalid-name";
      [k: string]: unknown;
    }
  | {
      name: string;
      reason: "reserved-name";
      [k: string]: unknown;
    };
