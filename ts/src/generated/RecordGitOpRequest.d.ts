/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * `POST /coord/git-ops/record` body. `repo` + `op_kind` are the only
 * required fields; the rest is optional metadata.
 *
 * `device_id` / `session_id` are NOT in this body — coord resolves the
 * tenant from the `X-Qontinui-Tenant-Id` header and reads device/session
 * from the request wrapper (mirroring the federation-report surface).
 */
export interface RecordGitOpRequest {
  /**
   * Affected branch (null for stash/tag ops).
   */
  branch?: string | null;
  /**
   * Commit message or op description.
   */
  message?: string | null;
  /**
   * Extensible metadata (files_changed, remote, ahead_count, …).
   */
  metadata?: {
    [k: string]: unknown;
  };
  /**
   * One of: `commit | push | checkout | branch_create | merge |
   * rebase | reset | stash | tag` (free-form on the wire).
   */
  op_kind: string;
  /**
   * Repo basename (origin remote URL basename, falling back to the
   * working-dir basename for remote-less clones).
   */
  repo: string;
  /**
   * Resulting commit SHA (null for branch_create/stash).
   */
  sha?: string | null;
  [k: string]: unknown;
}
