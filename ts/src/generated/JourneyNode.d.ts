/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * A vertex of the journey graph: one observed configuration of one page.
 *
 * Identity is a deterministic predicate, not a judgment: a node is
 * `(spec_id, sorted active IR state ids)` within an app. When the page has
 * no spec, or no state was classified present, the node is UNMODELLED and is
 * identified by its route template (else its page label) — it still takes
 * part in reachability and is counted as unmodelled in coverage.
 *
 * Build one with [`JourneyNode::new`], which sorts and dedups `state_ids`
 * and derives `modelled`; [`JourneyNode::key`] is the canonical string form
 * stored in `journey_frontier.node_key`.
 *
 * Node identity is `key()` / `node_key` — NEVER jsonb equality of the
 * serialized node (which also carries non-identity detail such as
 * `pageLabel` on a modelled node).
 * Unlike the crate's other optional fields, this struct's optional fields
 * always serialize, as `null` when absent, so every stored node has the same
 * key set.
 */
export interface JourneyNode {
  /**
   * True iff `spec_id` is present AND `state_ids` is non-empty.
   */
  modelled: boolean;
  /**
   * An APP-DECLARED page identifier — never a URL path, never user input.
   *
   * PRODUCER OBLIGATION (this is the privacy guarantee): fill it only from
   * the snapshot's `page.pageContext.meta.tabId`, then `activeTab`, then
   * `page.pageContext.name`, and NEVER from `page.pathname` in any form,
   * raw or slugged. (The runner's existing `resolve_page_label` in
   * `state_discovery/capture.rs` has a fourth fallback that slugs
   * `page.pathname`; the journey producer must not use that fallback.)
   * `pageContext.name` is a free-form display name (e.g. `"Import /
   * Export"`), so the producer SLUGS it before storing — which leaves a
   * `/` meaning only one thing: a leaked, unslugged path.
   *
   * [`JourneyNode::validate`] rejects a `/` as a TRIPWIRE for that leak.
   * It is not the guarantee: a slugged path contains no `/` and passes.
   */
  pageLabel: string | null;
  /**
   * The framework route PATTERN the snapshot's page was served under
   * (e.g. `/admin/coord/runs/[id]`), when known. Never a concrete URL path:
   * a concrete path can embed user input (`/search/<term>`,
   * `/users/<email>`). Not heuristically validated — the producer must
   * fill it from the router's pattern, not from `location.pathname`.
   */
  pathnameTemplate: string | null;
  /**
   * The page spec the snapshot matched; `None` = no spec for this page.
   * Load-bearing: some state ids are declared on more than one page.
   * Must not contain `#` or start with `unmodelled:` (see [`JourneyNode::key`]).
   */
  specId: string | null;
  /**
   * Ids of the IR states classified present — deduped, in BYTE-WISE
   * (UTF-8) ascending order, i.e. Rust's `str` ordering. Producers in
   * other languages must sort by UTF-8 bytes (not UTF-16 code units, not
   * a locale collation) or their keys will not match. No id may contain
   * `,` (see [`JourneyNode::key`]).
   */
  stateIds: string[];
}
