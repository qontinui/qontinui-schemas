/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Failure modes for app-registry operations. Mirrors the
 * `responses::SpecError` tagged-enum pattern in
 * `qontinui-runner/src-tauri/src/spec_api/responses.rs`.
 */
export type AppError =
  | {
      appId: string;
      reason: "not-registered";
      [k: string]: unknown;
    }
  | {
      appId: string;
      reason: "invalid-app-id";
      [k: string]: unknown;
    }
  | {
      reason: "invalid-repo-root";
      repoRoot: string;
      [k: string]: unknown;
    }
  | {
      appId: string;
      reason: "already-registered";
      [k: string]: unknown;
    };
