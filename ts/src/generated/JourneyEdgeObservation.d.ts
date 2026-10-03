/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { JourneyNode } from "./JourneyNode";
import type { JourneyTrigger } from "./JourneyTrigger";

/**
 * One observed edge of the journey graph — a row of
 * `project.journey_edge_observations`.
 *
 * Invariants (checked by [`JourneyEdgeObservation::validate`]; the first also
 * by the migration's CHECK): `to_node` is `None` iff `outcome` is
 * [`EdgeOutcome::ToNodeUnobserved`]; `no_change` joins nodes with equal
 * keys (a `changed` edge may also join equal keys).
 *
 * The ledger's `timeline` column is deliberately unrepresented until
 * Phase 4; readers select explicit columns rather than `*`.
 */
export interface JourneyEdgeObservation {
  /**
   * The registered app id.
   */
  appId: string;
  /**
   * The SDK's self-reported version (`SdkAppInfo.version`, semver or git
   * short-SHA); `None` = not reported, never "no version".
   */
  appVersion?: string | null;
  /**
   * The configuration the action was taken from.
   */
  fromNode: JourneyNode;
  /**
   * Row id (UUID).
   */
  id: string;
  /**
   * When the row was withdrawn (ISO 8601); `None` = live.
   */
  invalidatedAt?: string | null;
  /**
   * Who withdrew the row.
   */
  invalidatedBy?: string | null;
  /**
   * Why the row was withdrawn.
   */
  invalidatedReason?: string | null;
  /**
   * Token grouping one bulk withdrawal, as on `co_occurrence_observations`.
   */
  invalidationToken?: string | null;
  /**
   * When the edge CLOSED (to-snapshot seen, or diff returned), ISO 8601.
   */
  observedAt: string;
  /**
   * How the edge closed.
   */
  outcome:
    "changed" | "no_change" | "error" | "settle_timeout" | "to_node_unobserved";
  /**
   * Agent session / task-run / recording-session id; `None` = not reported.
   */
  runId?: string | null;
  /**
   * Who produced the edge.
   */
  runKind: "agent_action" | "explorer" | "passive_session";
  /**
   * The runner's own `buildId`.
   */
  runnerBuildId: string;
  /**
   * Same meaning as `co_occurrence_observations.runner_instance`.
   */
  runnerInstance: string;
  /**
   * The configuration observed after it; `None` iff
   * `outcome == to_node_unobserved`.
   */
  toNode?: JourneyNode | null;
  /**
   * What caused the edge.
   */
  trigger: JourneyTrigger;
}
