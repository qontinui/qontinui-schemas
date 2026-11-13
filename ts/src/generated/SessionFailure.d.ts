/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * An action a surface may offer for a failure. Serialized `snake_case`.
 */
export type FailureAction =
  "retry" | "login" | "switch_account" | "new_session" | "resume" | "none";

/**
 * One failure of an AI session, whichever lane noticed it.
 *
 * Emitted on the runner's `session-failure` event and readable from its
 * failures route, so a surface can render *why* a session stopped and which
 * actions are available, instead of an exit code or a bare banner. The
 * runner's classifier fills it from a signal and its recovery table chooses
 * [`Self::recovery_policy`]; neither lives in this crate.
 */
export interface SessionFailure {
  /**
   * The account the session was running under, when known.
   */
  account?: string | null;
  /**
   * Actions a surface may offer the user, in display order.
   */
  actions: FailureAction[];
  /**
   * Coarse grouping of [`Self::kind`] for filtering and styling.
   */
  category:
    | "limit"
    | "context"
    | "auth"
    | "provider"
    | "process"
    | "session"
    | "request"
    | "unknown";
  /**
   * Longer human-readable explanation.
   */
  details?: string | null;
  evidence: FailureEvidence;
  /**
   * Unique id of this failure occurrence (UUID minted by the runner).
   */
  id: string;
  /**
   * What went wrong. [`FailureKind::Unknown`] is a real answer, not "fine".
   */
  kind:
    | "rate_limited"
    | "quota_exhausted"
    | "budget_exhausted"
    | "context_exhausted"
    | "auth_required"
    | "access_denied"
    | "overloaded"
    | "transport_lost"
    | "resume_failed"
    | "spawn_failed"
    | "process_exited"
    | "bad_request"
    | "internal_error"
    | "unknown";
  /**
   * The [`CliProfile::id`] of the CLI whose session failed.
   */
  provider: string;
  /**
   * The raw reason the source gave, verbatim (a provider error type, a
   * stderr line, the matched screen phrase), for diagnosis.
   */
  reason?: string | null;
  /**
   * What the runner does about it automatically.
   */
  recoveryPolicy:
    | "never"
    | "backoff_then_retry"
    | "wait_until_reset"
    | "migrate_account"
    | "login_then_resume"
    | "resume_same_id"
    | "handoff_new_session";
  /**
   * When a limit resets (RFC 3339), when the source states it.
   */
  resetAt?: string | null;
  /**
   * How serious the failure is for the session.
   */
  severity: "info" | "warning" | "error";
  /**
   * One-line human-readable summary.
   */
  title: string;
  /**
   * The turn that failed, when the source identifies one.
   */
  turnId?: string | null;
  [k: string]: unknown;
}
/**
 * Where the failure was observed and how sure the runner is of it.
 */
export interface FailureEvidence {
  /**
   * Whether the source states the failure or merely suggests it.
   */
  confidence: "confirmed" | "hint";
  /**
   * The signal source.
   */
  source:
    | "structured_event"
    | "hook"
    | "stderr"
    | "grid_scrape"
    | "exit_status"
    | "transcript"
    | "handshake_timeout";
  [k: string]: unknown;
}
