/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { HelperAnswerSchema } from "./HelperAnswerSchema";
import type { HelperTaskKind } from "./HelperTaskKind";
import type { HelperTaskPayload } from "./HelperTaskPayload";
import type { HelperTaskSource } from "./HelperTaskSource";
import type { HelperTaskStatus } from "./HelperTaskStatus";
import type { HelperVerdict } from "./HelperVerdict";

/**
 * A human-judgment micro-task emitted by a runner for a helper to answer.
 *
 * The broker holds the task `Open` until [`required_votes`](HelperTask::required_votes)
 * [`HelperAnswer`]s are collected (or it `expires_at`), then resolves the
 * verdict and routes it back through [`source`](HelperTask::source).
 */
export interface HelperTask {
  answerSchema: HelperAnswerSchema;
  /**
   * Application the task pertains to.
   */
  appId: string;
  /**
   * ISO 8601 timestamp (UTC) when the task was created.
   */
  createdAt: string;
  /**
   * ISO 8601 timestamp (UTC) after which the task can no longer be answered.
   * Absent means the task does not expire.
   */
  expiresAt?: string | null;
  /**
   * Task ID (UUID v4 string).
   */
  id: string;
  kind: HelperTaskKind;
  payload: HelperTaskPayload;
  /**
   * Human-readable question shown to the helper (e.g. "Does this login
   * screen look right?").
   */
  prompt: string;
  /**
   * Number of helper answers required before the task is `Answered`.
   */
  requiredVotes: number;
  source: HelperTaskSource;
  status: HelperTaskStatus;
  /**
   * Tenant that owns the task (coord tenant id).
   */
  tenantId: string;
  [k: string]: unknown;
}
