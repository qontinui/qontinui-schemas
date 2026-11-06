/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Provenance of a [`ScheduledTask`] that the runner reconciles from a
 * fleet-wide declaration (plan
 * `2026-10-29-recurring-agents-declared-in-coord-reconciled-by-runner`, D5).
 *
 * `(source, tenant_id, agent)` is the managed key: the runner holds at most
 * one task per key. What the reconciler compares to decide whether a task is
 * current is `digest`, never `version` — a catalog fix in coord's code moves
 * no row version, and a deleted-and-recreated agent restarts its version at 1.
 */
export interface ManagedBy {
  /**
   * The recurring agent's name — its join key across coord's status, run
   * requests and this managed task (coord forbids renames).
   */
  agent: string;
  /**
   * sha256 (64 lowercase hex) of the resolved spec coord served when this
   * task was applied. `None` = applied from a declaration that carried no
   * digest.
   */
  digest?: string | null;
  /**
   * Which declaration store the task came from — [`MANAGED_SOURCE_COORD`].
   * A plain string rather than an enum, so a stored task from a newer
   * source still loads; the consumer decides what it recognises.
   */
  source: string;
  /**
   * The coord tenant (UUID) whose declaration this task applies. A device
   * bound to several tenants holds one managed task per tenant and agent.
   */
  tenantId: string;
  /**
   * The declaration's row version when applied. History and display only.
   */
  version: number;
}
