//! Journey-twin wire types — the OBSERVED user path through an app.
//!
//! These types implement the frozen "Edge-observation contract" of plan
//! `2026-09-20-ui-bridge-represents-the-users-path-and-the-passage-of-time`
//! (Phase 0 froze it; Phase 1 implements it here, in the qontinui-web
//! migration, and in the runner producer).
//!
//! The journey twin is NOT a new state model. Nodes are named in the existing
//! IR vocabulary ([`crate::ir::IrState`] ids scoped by their page spec), and
//! every agent UI action through the runner closes one
//! [`JourneyEdgeObservation`] — a row of `project.journey_edge_observations`.
//! Every interactive affordance that was seen at a node and never activated is
//! a [`FrontierEntry`] — a row of `project.journey_frontier`. The graph is
//! derived on read from those rows; nothing here clusters or infers.
//!
//! ## Wire-format notes
//!
//! - Structs serialize as `camelCase`; unit enums as `snake_case` strings,
//!   exactly the values the migration's CHECK constraints list.
//! - Every enum is `#[schemars(inline)]` (as [`crate::ir::IrEffect`] is), so
//!   the generated schemas carry the closed value sets inline and the runner's
//!   `schema_export.rs` need register only the structs — a `$ref` to an
//!   unregistered enum would render as an undeclared TypeScript name.
//! - Optional fields follow the crate convention
//!   (`#[serde(default, skip_serializing_if = "Option::is_none")]`), so an
//!   absent key and `null` both read as "not reported" — EXCEPT on
//!   [`JourneyNode`], whose optional fields always serialize (as `null` when
//!   absent). Node identity is [`JourneyNode::key`] / `node_key`, never jsonb
//!   equality.
//! - [`crate::ir::IrEffect`] is reused for declared effects; there is no
//!   journey-specific effect enum.
//!
//! ## Privacy — what holds, exactly
//!
//! - [`JourneyTrigger`] is CLOSED: `#[serde(deny_unknown_fields)]`, and it has
//!   no field for a typed text or value, so a caller cannot smuggle one into
//!   the `trigger` column. This keeps agent-action capture inside the
//!   structural redaction line of plan
//!   `2026-07-20-ui-bridge-structural-redaction-enforcement` by construction.
//! - No field of a row carries a raw URL path or typed text. Every free
//!   string field in these types (`actionType`, `targetFingerprint`,
//!   `affordanceFingerprint`, roles, spec/state ids, `pageLabel`, run and
//!   build ids) is an IDENTIFIER the runner derives from structure, from an
//!   app-declared name, or from its own state — never user input. A producer
//!   that puts user-entered content into one of them is violating this
//!   contract.
//! - [`JourneyNode::pathname_template`] is a framework ROUTE PATTERN
//!   (`/agents/[id]`), never the concrete path (`/agents/42`,
//!   `/search/<term>`, `/users/<email>`), which can embed user input. The
//!   concrete path has no field; `pageLabel` (an app-declared page
//!   identifier) stands in for it.
//! - The `pageLabel` guarantee is a PRODUCER OBLIGATION, not something this
//!   crate can check: the producer fills it only from
//!   `page.pageContext.meta.tabId`, `activeTab` or `page.pageContext.name`
//!   (slugged), and NEVER from `page.pathname` in any form — raw or slugged.
//!   [`JourneyNode::validate`]'s rejection of a `/` is only a TRIPWIRE for an
//!   unslugged path leaking in; a slugged path (`/search/secret` →
//!   `search-secret`) passes it.
//!
//! PRIVACY: the ledger's `timeline` column is deliberately NOT represented
//! here. Phase 4 of the plan owns it and must add it as a closed
//! (`deny_unknown_fields`) type with no value/text slot — never as an open
//! `serde_json::Value`, which would make a typed value representable again.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::ir::IrEffect;

/// A violation of the edge-observation contract that the Rust type system
/// alone cannot rule out.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum JourneyContractError {
    /// `toNode` is null but the outcome is not `to_node_unobserved`, or the
    /// outcome is `to_node_unobserved` but a `toNode` is present. The two are
    /// the same fact and must agree (the migration's CHECK enforces the same).
    #[error(
        "toNode is {to_node} but outcome is {}; toNode must be null iff outcome is to_node_unobserved",
        .outcome.as_str()
    )]
    ToNodeOutcomeMismatch {
        /// `"null"` or `"present"`.
        to_node: &'static str,
        /// The outcome the observation carried.
        outcome: EdgeOutcome,
    },
    /// A [`JourneyNode`] is not in the canonical form [`JourneyNode::new`]
    /// produces (state ids unsorted or duplicated, or `modelled` disagreeing
    /// with its derivation). Two non-canonical nodes for one configuration
    /// would split a graph vertex in two.
    #[error("{which} is not canonical: {reason}")]
    NonCanonicalNode {
        /// Which node: `"fromNode"`, `"toNode"` or `"node"`.
        which: &'static str,
        /// What is wrong with it.
        reason: &'static str,
    },
    /// The outcome is `no_change` but the endpoints' keys differ. (The
    /// converse is NOT an error: a `changed` edge may link equal keys — an
    /// unmodelled `/runs/[id]` → `/runs/[id]`, or a DOM change within the same
    /// IR states.)
    #[error("outcome is no_change but fromNode key {from_key:?} and toNode key {to_key:?} differ")]
    NoChangeAcrossDifferentNodes {
        /// `fromNode.key()`.
        from_key: String,
        /// `toNode.key()`.
        to_key: String,
    },
    /// A frontier row's `nodeKey` is not `node.key()`.
    #[error("nodeKey {node_key:?} does not match node.key() {expected:?}")]
    NodeKeyMismatch {
        /// The stored key.
        node_key: String,
        /// The key the node derives.
        expected: String,
    },
}

// ---------------------------------------------------------------------------
// Node
// ---------------------------------------------------------------------------

/// A vertex of the journey graph: one observed configuration of one page.
///
/// Identity is a deterministic predicate, not a judgment: a node is
/// `(spec_id, sorted active IR state ids)` within an app. When the page has
/// no spec, or no state was classified present, the node is UNMODELLED and is
/// identified by its route template (else its page label) — it still takes
/// part in reachability and is counted as unmodelled in coverage.
///
/// Build one with [`JourneyNode::new`], which sorts and dedups `state_ids`
/// and derives `modelled`; [`JourneyNode::key`] is the canonical string form
/// stored in `journey_frontier.node_key`.
///
/// Node identity is `key()` / `node_key` — NEVER jsonb equality of the
/// serialized node (which also carries non-identity detail such as
/// `pageLabel` on a modelled node).
/// Unlike the crate's other optional fields, this struct's optional fields
/// always serialize, as `null` when absent, so every stored node has the same
/// key set.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct JourneyNode {
    /// The page spec the snapshot matched; `None` = no spec for this page.
    /// Load-bearing: some state ids are declared on more than one page.
    /// Must not contain `#` or start with `unmodelled:` (see [`JourneyNode::key`]).
    #[serde(default)]
    pub spec_id: Option<String>,
    /// Ids of the IR states classified present — deduped, in BYTE-WISE
    /// (UTF-8) ascending order, i.e. Rust's `str` ordering. Producers in
    /// other languages must sort by UTF-8 bytes (not UTF-16 code units, not
    /// a locale collation) or their keys will not match. No id may contain
    /// `,` (see [`JourneyNode::key`]).
    pub state_ids: Vec<String>,
    /// True iff `spec_id` is present AND `state_ids` is non-empty.
    pub modelled: bool,
    /// The framework route PATTERN the snapshot's page was served under
    /// (e.g. `/admin/coord/runs/[id]`), when known. Never a concrete URL path:
    /// a concrete path can embed user input (`/search/<term>`,
    /// `/users/<email>`). Not heuristically validated — the producer must
    /// fill it from the router's pattern, not from `location.pathname`.
    #[serde(default)]
    pub pathname_template: Option<String>,
    /// An APP-DECLARED page identifier — never a URL path, never user input.
    ///
    /// PRODUCER OBLIGATION (this is the privacy guarantee): fill it only from
    /// the snapshot's `page.pageContext.meta.tabId`, then `activeTab`, then
    /// `page.pageContext.name`, and NEVER from `page.pathname` in any form,
    /// raw or slugged. (The runner's existing `resolve_page_label` in
    /// `state_discovery/capture.rs` has a fourth fallback that slugs
    /// `page.pathname`; the journey producer must not use that fallback.)
    /// `pageContext.name` is a free-form display name (e.g. `"Import /
    /// Export"`), so the producer SLUGS it before storing — which leaves a
    /// `/` meaning only one thing: a leaked, unslugged path.
    ///
    /// [`JourneyNode::validate`] rejects a `/` as a TRIPWIRE for that leak.
    /// It is not the guarantee: a slugged path contains no `/` and passes.
    #[serde(default)]
    pub page_label: Option<String>,
}

impl JourneyNode {
    /// Build a node in canonical form: `state_ids` sorted and deduped,
    /// `modelled` derived as `spec_id.is_some() && !state_ids.is_empty()`.
    pub fn new(
        spec_id: Option<String>,
        state_ids: impl IntoIterator<Item = String>,
        pathname_template: Option<String>,
        page_label: Option<String>,
    ) -> Self {
        let mut state_ids: Vec<String> = state_ids.into_iter().collect();
        state_ids.sort();
        state_ids.dedup();
        let modelled = spec_id.is_some() && !state_ids.is_empty();
        Self {
            spec_id,
            state_ids,
            modelled,
            pathname_template,
            page_label,
        }
    }

    /// The canonical string identity of this node.
    ///
    /// - modelled: `"<specId>#<stateIds joined by ','>"`
    /// - unmodelled: `"unmodelled:<pathnameTemplate ?? pageLabel ?? 'unknown'>"`
    ///
    /// Precondition for injectivity: the node passes [`JourneyNode::validate`]
    /// — in particular `spec_id` contains no `#` and does not start with
    /// `unmodelled:`, and no state id contains `,`. Otherwise two distinct
    /// nodes could render the same key.
    pub fn key(&self) -> String {
        match (&self.spec_id, self.modelled) {
            (Some(spec_id), true) => format!("{spec_id}#{}", self.state_ids.join(",")),
            _ => format!(
                "unmodelled:{}",
                self.pathname_template
                    .as_deref()
                    .or(self.page_label.as_deref())
                    .unwrap_or("unknown")
            ),
        }
    }

    /// Check the node is in the form [`JourneyNode::new`] produces. Needed
    /// for nodes that arrived over the wire rather than through `new`.
    pub fn validate(&self) -> Result<(), JourneyContractError> {
        self.validate_as("node")
    }

    fn validate_as(&self, which: &'static str) -> Result<(), JourneyContractError> {
        if let Some(spec_id) = &self.spec_id {
            if spec_id.contains('#') {
                return Err(JourneyContractError::NonCanonicalNode {
                    which,
                    reason: "specId must not contain '#' (the key's spec/state separator)",
                });
            }
            if spec_id.starts_with("unmodelled:") {
                return Err(JourneyContractError::NonCanonicalNode {
                    which,
                    reason: "specId must not start with 'unmodelled:' (the unmodelled key prefix)",
                });
            }
        }
        if self.page_label.as_deref().is_some_and(|l| l.contains('/')) {
            return Err(JourneyContractError::NonCanonicalNode {
                which,
                reason: "pageLabel contains '/': tripwire for an unslugged URL path leaking in (the producer must never derive pageLabel from page.pathname)",
            });
        }
        if self.state_ids.iter().any(|id| id.contains(',')) {
            return Err(JourneyContractError::NonCanonicalNode {
                which,
                reason: "stateIds must not contain ',' (the key's state separator)",
            });
        }
        if self.state_ids.windows(2).any(|w| w[0] >= w[1]) {
            return Err(JourneyContractError::NonCanonicalNode {
                which,
                reason: "stateIds must be sorted ascending with no duplicates",
            });
        }
        let derived = self.spec_id.is_some() && !self.state_ids.is_empty();
        if self.modelled != derived {
            return Err(JourneyContractError::NonCanonicalNode {
                which,
                reason: "modelled must equal (specId is present AND stateIds is non-empty)",
            });
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Trigger
// ---------------------------------------------------------------------------

/// How the transition was initiated. `affordance` is an activated element or
/// component action; the others are the SDK `NavigationTracker`'s
/// `NavigationTrigger` values (`initial` is the raw "arrived by deep link"
/// signal).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[schemars(inline)]
#[serde(rename_all = "snake_case")]
pub enum NavigationTriggerKind {
    /// An element or component action was activated.
    Affordance,
    /// History push.
    Push,
    /// History replace.
    Replace,
    /// History pop (back/forward).
    Pop,
    /// Initial load — not an in-app navigation.
    Initial,
    /// Hash change.
    Hash,
}

impl NavigationTriggerKind {
    /// The wire string.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Affordance => "affordance",
            Self::Push => "push",
            Self::Replace => "replace",
            Self::Pop => "pop",
            Self::Initial => "initial",
            Self::Hash => "hash",
        }
    }
}

/// Which runner action choke point captured the edge. Every transport an
/// agent can act through is one of these; an edge from a transport not listed
/// here is a producer defect, not a new variant to tolerate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[schemars(inline)]
#[serde(rename_all = "snake_case")]
pub enum ChokePoint {
    /// `/ui-bridge/control/element/{id}/action`.
    ElementAction,
    /// `/ui-bridge/control/batch-actions`, `/control/actions/batch`.
    BatchAction,
    /// `/control/component/{id}/action/{action_id}`.
    ComponentAction,
    /// The SDK/WebSocket element action.
    SdkElementAction,
    /// Any execute-with-diff route (runner routes and the SDK twin).
    ExecuteWithDiff,
}

impl ChokePoint {
    /// The wire string.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ElementAction => "element_action",
            Self::BatchAction => "batch_action",
            Self::ComponentAction => "component_action",
            Self::SdkElementAction => "sdk_element_action",
            Self::ExecuteWithDiff => "execute_with_diff",
        }
    }
}

/// What caused an edge — structurally only.
///
/// `deny_unknown_fields` and the deliberate absence of any text/value field
/// are the privacy control: a caller cannot smuggle a typed value into the
/// ledger, because no such key deserializes and no such field exists to set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct JourneyTrigger {
    /// The action verb (`click`, `type`, `select`, a component action id, …).
    pub action_type: String,
    /// Structural fingerprint of the target element; `None` = not resolved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_fingerprint: Option<String>,
    /// ARIA role of the target; `None` = not reported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_role: Option<String>,
    /// The affordance's declared effect; `None` = undeclared (NOT "safe").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub declared_effect: Option<IrEffect>,
    /// How the transition was initiated.
    pub navigation_trigger: NavigationTriggerKind,
    /// Which runner choke point captured it.
    pub choke_point: ChokePoint,
}

// ---------------------------------------------------------------------------
// Edge observation
// ---------------------------------------------------------------------------

/// Who produced the edge (`journey_edge_observations.run_kind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[schemars(inline)]
#[serde(rename_all = "snake_case")]
pub enum RunKind {
    /// An agent acting through the runner's UI Bridge routes.
    AgentAction,
    /// The safe-activation explorer.
    Explorer,
    /// Passive capture of a human session.
    PassiveSession,
}

impl RunKind {
    /// The wire string.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AgentAction => "agent_action",
            Self::Explorer => "explorer",
            Self::PassiveSession => "passive_session",
        }
    }
}

/// How the edge closed (`journey_edge_observations.outcome`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[schemars(inline)]
#[serde(rename_all = "snake_case")]
pub enum EdgeOutcome {
    /// The configuration changed.
    Changed,
    /// The action landed and the configuration did not change — a dead
    /// click is data, not an absent row.
    NoChange,
    /// The action errored.
    Error,
    /// The page did not settle within the budget.
    SettleTimeout,
    /// A second action arrived before any snapshot closed this edge, so the
    /// destination is unknown. The only outcome with a null `toNode`.
    ToNodeUnobserved,
}

impl EdgeOutcome {
    /// The wire string.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Changed => "changed",
            Self::NoChange => "no_change",
            Self::Error => "error",
            Self::SettleTimeout => "settle_timeout",
            Self::ToNodeUnobserved => "to_node_unobserved",
        }
    }
}

/// One observed edge of the journey graph — a row of
/// `project.journey_edge_observations`.
///
/// Invariants (checked by [`JourneyEdgeObservation::validate`]; the first also
/// by the migration's CHECK): `to_node` is `None` iff `outcome` is
/// [`EdgeOutcome::ToNodeUnobserved`]; `no_change` joins nodes with equal
/// keys (a `changed` edge may also join equal keys).
///
/// The ledger's `timeline` column is deliberately unrepresented until
/// Phase 4; readers select explicit columns rather than `*`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct JourneyEdgeObservation {
    /// Row id (UUID).
    pub id: String,
    /// When the edge CLOSED (to-snapshot seen, or diff returned), ISO 8601.
    pub observed_at: String,
    /// The registered app id.
    pub app_id: String,
    /// The SDK's self-reported version (`SdkAppInfo.version`, semver or git
    /// short-SHA); `None` = not reported, never "no version".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_version: Option<String>,
    /// The runner's own `buildId`.
    pub runner_build_id: String,
    /// Same meaning as `co_occurrence_observations.runner_instance`.
    pub runner_instance: String,
    /// Agent session / task-run / recording-session id; `None` = not reported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    /// Who produced the edge.
    pub run_kind: RunKind,
    /// The configuration the action was taken from.
    pub from_node: JourneyNode,
    /// The configuration observed after it; `None` iff
    /// `outcome == to_node_unobserved`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_node: Option<JourneyNode>,
    /// What caused the edge.
    pub trigger: JourneyTrigger,
    /// How the edge closed.
    pub outcome: EdgeOutcome,
    // PRIVACY: the `timeline` column is not represented here — see the
    // module doc. Phase 4 adds it as a closed type with no value slot.
    /// When the row was withdrawn (ISO 8601); `None` = live.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invalidated_at: Option<String>,
    /// Why the row was withdrawn.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invalidated_reason: Option<String>,
    /// Who withdrew the row.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invalidated_by: Option<String>,
    /// Token grouping one bulk withdrawal, as on `co_occurrence_observations`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invalidation_token: Option<String>,
}

impl JourneyEdgeObservation {
    /// Check the contract invariants the type cannot express: `toNode` is
    /// null iff `outcome` is `to_node_unobserved`; both nodes are in
    /// canonical form; and a `no_change` edge joins nodes with equal
    /// [`JourneyNode::key`]s. A producer calls this before writing a row; a
    /// reader may call it on a row it did not write.
    pub fn validate(&self) -> Result<(), JourneyContractError> {
        let unobserved = self.outcome == EdgeOutcome::ToNodeUnobserved;
        match (&self.to_node, unobserved) {
            (None, false) => {
                return Err(JourneyContractError::ToNodeOutcomeMismatch {
                    to_node: "null",
                    outcome: self.outcome,
                })
            }
            (Some(_), true) => {
                return Err(JourneyContractError::ToNodeOutcomeMismatch {
                    to_node: "present",
                    outcome: self.outcome,
                })
            }
            _ => {}
        }
        self.from_node.validate_as("fromNode")?;
        if let Some(to) = &self.to_node {
            to.validate_as("toNode")?;
            if self.outcome == EdgeOutcome::NoChange {
                let from_key = self.from_node.key();
                let to_key = to.key();
                if from_key != to_key {
                    return Err(JourneyContractError::NoChangeAcrossDifferentNodes {
                        from_key,
                        to_key,
                    });
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Frontier
// ---------------------------------------------------------------------------

/// Why a seen affordance has not been activated from its node
/// (`journey_frontier.reason`). An open frontier is what keeps "unexplored"
/// distinguishable from "unreachable".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[schemars(inline)]
#[serde(rename_all = "snake_case")]
pub enum FrontierReason {
    /// Seen, eligible, simply not activated yet.
    NotYetActivated,
    /// No declared effect and not an explicit navigation role/type — the
    /// explorer never guesses an effect.
    EffectUndeclared,
    /// Declared `write` — never activated by the explorer.
    EffectWrite,
    /// Declared `destructive` — never activated by the explorer.
    EffectDestructive,
    /// The exploration budget ran out first.
    BudgetExhausted,
    /// Activation was attempted and failed.
    ActivationFailed,
}

impl FrontierReason {
    /// The wire string.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NotYetActivated => "not_yet_activated",
            Self::EffectUndeclared => "effect_undeclared",
            Self::EffectWrite => "effect_write",
            Self::EffectDestructive => "effect_destructive",
            Self::BudgetExhausted => "budget_exhausted",
            Self::ActivationFailed => "activation_failed",
        }
    }
}

/// An interactive affordance seen at a node and not yet the trigger of an
/// edge from it — a row of `project.journey_frontier`, keyed
/// `(app_id, node_key, affordance_fingerprint)`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct FrontierEntry {
    /// The registered app id.
    pub app_id: String,
    /// `node.key()` — stored so the primary key is a plain text column.
    pub node_key: String,
    /// The node the affordance was seen at.
    pub node: JourneyNode,
    /// Structural fingerprint of the affordance.
    pub affordance_fingerprint: String,
    /// ARIA role of the affordance; `None` = not reported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub affordance_role: Option<String>,
    /// The affordance's declared effect; `None` = undeclared.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub declared_effect: Option<IrEffect>,
    /// Why it is on the frontier.
    pub reason: FrontierReason,
    /// First time it was seen at this node (ISO 8601).
    pub first_seen_at: String,
    /// Most recent time it was seen at this node (ISO 8601).
    pub last_seen_at: String,
    /// The run that most recently saw it; `None` = not reported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_seen_run_id: Option<String>,
}

impl FrontierEntry {
    /// Check `nodeKey == node.key()` and that the node is canonical.
    pub fn validate(&self) -> Result<(), JourneyContractError> {
        self.node.validate_as("node")?;
        let expected = self.node.key();
        if self.node_key != expected {
            return Err(JourneyContractError::NodeKeyMismatch {
                node_key: self.node_key.clone(),
                expected,
            });
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Ledger health
// ---------------------------------------------------------------------------

/// Whether the edge ledger is actually being written. A graph that stopped
/// growing must never read as a finished one, so every journey read repeats
/// this state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[schemars(inline)]
#[serde(rename_all = "snake_case")]
pub enum LedgerState {
    /// The schema probe found the tables and writes are succeeding.
    Writing,
    /// The journey tables do not exist in this database (e.g. an embedded DB
    /// that predates them).
    SchemaAbsent,
    /// The tables exist but writes are failing.
    WriteFailing,
}

impl LedgerState {
    /// The wire string.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Writing => "writing",
            Self::SchemaAbsent => "schema_absent",
            Self::WriteFailing => "write_failing",
        }
    }
}

/// Body of `GET /apps/{app_id}/journey/health`, and the ledger block every
/// journey query response repeats.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct JourneyLedgerHealth {
    /// The ledger's state.
    pub state: LedgerState,
    /// Human-readable cause, e.g. `"embedded DB predates the journey tables"`.
    pub detail: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn modelled_node() -> JourneyNode {
        JourneyNode::new(
            Some("coord-runners".into()),
            ["b".to_string(), "a".to_string(), "b".to_string()],
            Some("/admin/coord/runners".into()),
            Some("runners-tab".into()),
        )
    }

    fn other_node() -> JourneyNode {
        JourneyNode::new(
            Some("coord-lands".into()),
            ["lands-list".to_string()],
            Some("/admin/coord/lands".into()),
            None,
        )
    }

    fn trigger() -> JourneyTrigger {
        JourneyTrigger {
            action_type: "click".into(),
            target_fingerprint: Some("fp-1".into()),
            target_role: Some("link".into()),
            declared_effect: Some(IrEffect::Read),
            navigation_trigger: NavigationTriggerKind::Affordance,
            choke_point: ChokePoint::ElementAction,
        }
    }

    fn edge(to_node: Option<JourneyNode>, outcome: EdgeOutcome) -> JourneyEdgeObservation {
        JourneyEdgeObservation {
            id: "00000000-0000-4000-8000-000000000001".into(),
            observed_at: "2026-09-30T12:00:00Z".into(),
            app_id: "qontinui-web".into(),
            app_version: Some("abc1234".into()),
            runner_build_id: "build-1".into(),
            runner_instance: "primary".into(),
            run_id: Some("session-1".into()),
            run_kind: RunKind::AgentAction,
            from_node: modelled_node(),
            to_node,
            trigger: trigger(),
            outcome,
            invalidated_at: None,
            invalidated_reason: None,
            invalidated_by: None,
            invalidation_token: None,
        }
    }

    fn round_trip<T>(value: &T) -> T
    where
        T: Serialize + for<'de> Deserialize<'de>,
    {
        let text = serde_json::to_string(value).expect("serialize");
        serde_json::from_str(&text).expect("deserialize")
    }

    // --- JourneyNode --------------------------------------------------------

    #[test]
    fn new_sorts_and_dedups_state_ids() {
        let node = JourneyNode::new(
            Some("s".into()),
            ["z", "a", "m", "a", "z"].map(String::from),
            None,
            None,
        );
        assert_eq!(node.state_ids, vec!["a", "m", "z"]);
        assert!(node.validate().is_ok());
    }

    #[test]
    fn modelled_requires_spec_and_states() {
        assert!(JourneyNode::new(Some("s".into()), ["a".to_string()], None, None).modelled);
        assert!(!JourneyNode::new(Some("s".into()), Vec::<String>::new(), None, None).modelled);
        assert!(!JourneyNode::new(None, ["a".to_string()], None, None).modelled);
        assert!(!JourneyNode::new(None, Vec::<String>::new(), None, None).modelled);
    }

    #[test]
    fn key_modelled() {
        assert_eq!(modelled_node().key(), "coord-runners#a,b");
    }

    #[test]
    fn key_unmodelled_prefers_template() {
        let node = JourneyNode::new(
            None,
            Vec::<String>::new(),
            Some("/runs/[id]".into()),
            Some("run-detail".into()),
        );
        assert_eq!(node.key(), "unmodelled:/runs/[id]");
    }

    #[test]
    fn key_unmodelled_with_page_label_only() {
        let node = JourneyNode::new(
            Some("spec-with-no-state-present".into()),
            Vec::<String>::new(),
            None,
            Some("run-detail".into()),
        );
        assert_eq!(node.key(), "unmodelled:run-detail");
    }

    #[test]
    fn page_label_with_a_slash_is_rejected() {
        // Tripwire: an unslugged URL path leaking into pageLabel. A SLUGGED
        // path would pass — the guarantee is the producer's obligation.
        for label in ["/users/someone@example.com", "search/term"] {
            let node = JourneyNode::new(None, Vec::<String>::new(), None, Some(label.into()));
            assert_eq!(
                node.validate(),
                Err(JourneyContractError::NonCanonicalNode {
                    which: "node",
                    reason: "pageLabel contains '/': tripwire for an unslugged URL path leaking in (the producer must never derive pageLabel from page.pathname)",
                }),
                "{label}"
            );
        }
        // And it is rejected inside an edge, too.
        let mut e = edge(Some(other_node()), EdgeOutcome::Changed);
        e.to_node.as_mut().unwrap().page_label = Some("/search/secret".into());
        assert!(matches!(
            e.validate(),
            Err(JourneyContractError::NonCanonicalNode {
                which: "toNode",
                ..
            })
        ));
    }

    #[test]
    fn node_has_no_concrete_path_field() {
        // The published schema must offer no concrete-path property at all.
        let schema = serde_json::to_value(schemars::schema_for!(JourneyNode)).unwrap();
        let props = schema["properties"].as_object().unwrap();
        assert!(!props.contains_key("pathname"));
        assert!(props.contains_key("pageLabel"));
    }

    #[test]
    fn key_unmodelled_with_neither() {
        let node = JourneyNode::new(None, ["orphan".to_string()], None, None);
        assert_eq!(node.key(), "unmodelled:unknown");
    }

    #[test]
    fn node_wire_is_camel_case() {
        let value = serde_json::to_value(modelled_node()).unwrap();
        assert_eq!(
            value,
            json!({
                "specId": "coord-runners",
                "stateIds": ["a", "b"],
                "modelled": true,
                "pathnameTemplate": "/admin/coord/runners",
                "pageLabel": "runners-tab",
            })
        );
    }

    #[test]
    fn non_canonical_node_is_rejected() {
        let mut unsorted = modelled_node();
        unsorted.state_ids = vec!["b".into(), "a".into()];
        assert!(matches!(
            unsorted.validate(),
            Err(JourneyContractError::NonCanonicalNode { .. })
        ));

        let mut duplicated = modelled_node();
        duplicated.state_ids = vec!["a".into(), "a".into()];
        assert!(duplicated.validate().is_err());

        let mut lying = modelled_node();
        lying.modelled = false;
        assert!(lying.validate().is_err());
    }

    // --- JourneyTrigger -----------------------------------------------------

    #[test]
    fn trigger_rejects_value_and_text_keys() {
        let base = serde_json::to_value(trigger()).unwrap();
        for smuggled in ["value", "text"] {
            let mut payload = base.clone();
            payload[smuggled] = json!("hunter2");
            let err = serde_json::from_value::<JourneyTrigger>(payload)
                .expect_err("a typed value must not deserialize into a trigger");
            assert!(
                err.to_string().contains("unknown field"),
                "{smuggled}: {err}"
            );
        }
    }

    #[test]
    fn trigger_schema_forbids_additional_properties() {
        let schema = serde_json::to_value(schemars::schema_for!(JourneyTrigger)).unwrap();
        assert_eq!(schema["additionalProperties"], json!(false));
        let props = schema["properties"].as_object().unwrap();
        assert!(!props.contains_key("value") && !props.contains_key("text"));
    }

    #[test]
    fn trigger_wire_is_camel_case() {
        let value = serde_json::to_value(trigger()).unwrap();
        assert_eq!(
            value,
            json!({
                "actionType": "click",
                "targetFingerprint": "fp-1",
                "targetRole": "link",
                "declaredEffect": "read",
                "navigationTrigger": "affordance",
                "chokePoint": "element_action",
            })
        );
    }

    // --- Edge invariant -----------------------------------------------------

    #[test]
    fn edge_with_to_node_and_observed_outcome_is_valid() {
        assert_eq!(
            edge(Some(other_node()), EdgeOutcome::Changed).validate(),
            Ok(())
        );
        assert_eq!(
            edge(Some(modelled_node()), EdgeOutcome::NoChange).validate(),
            Ok(())
        );
        for outcome in [EdgeOutcome::Error, EdgeOutcome::SettleTimeout] {
            assert_eq!(edge(Some(modelled_node()), outcome).validate(), Ok(()));
            assert_eq!(edge(Some(other_node()), outcome).validate(), Ok(()));
        }
        assert_eq!(edge(None, EdgeOutcome::ToNodeUnobserved).validate(), Ok(()));
    }

    #[test]
    fn edge_null_to_node_requires_unobserved_outcome() {
        assert_eq!(
            edge(None, EdgeOutcome::Changed).validate(),
            Err(JourneyContractError::ToNodeOutcomeMismatch {
                to_node: "null",
                outcome: EdgeOutcome::Changed,
            })
        );
    }

    #[test]
    fn edge_unobserved_outcome_requires_null_to_node() {
        assert_eq!(
            edge(Some(modelled_node()), EdgeOutcome::ToNodeUnobserved).validate(),
            Err(JourneyContractError::ToNodeOutcomeMismatch {
                to_node: "present",
                outcome: EdgeOutcome::ToNodeUnobserved,
            })
        );
    }

    #[test]
    fn edge_rejects_non_canonical_nodes() {
        let mut bad = edge(Some(other_node()), EdgeOutcome::Changed);
        bad.from_node.state_ids.reverse();
        assert!(matches!(
            bad.validate(),
            Err(JourneyContractError::NonCanonicalNode {
                which: "fromNode",
                ..
            })
        ));
        let mut bad = edge(Some(other_node()), EdgeOutcome::Changed);
        bad.to_node.as_mut().unwrap().modelled = false;
        assert!(matches!(
            bad.validate(),
            Err(JourneyContractError::NonCanonicalNode {
                which: "toNode",
                ..
            })
        ));
    }

    // --- Enum wire strings --------------------------------------------------

    fn assert_wire<T>(pairs: &[(T, &str)], as_str: fn(T) -> &'static str)
    where
        T: Copy + Serialize + for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug,
    {
        for (variant, wire) in pairs {
            assert_eq!(serde_json::to_value(variant).unwrap(), json!(wire));
            assert_eq!(serde_json::from_value::<T>(json!(wire)).unwrap(), *variant);
            assert_eq!(as_str(*variant), *wire);
        }
    }

    #[test]
    fn navigation_trigger_wire_strings() {
        use NavigationTriggerKind::*;
        assert_wire(
            &[
                (Affordance, "affordance"),
                (Push, "push"),
                (Replace, "replace"),
                (Pop, "pop"),
                (Initial, "initial"),
                (Hash, "hash"),
            ],
            NavigationTriggerKind::as_str,
        );
    }

    #[test]
    fn choke_point_wire_strings() {
        use ChokePoint::*;
        assert_wire(
            &[
                (ElementAction, "element_action"),
                (BatchAction, "batch_action"),
                (ComponentAction, "component_action"),
                (SdkElementAction, "sdk_element_action"),
                (ExecuteWithDiff, "execute_with_diff"),
            ],
            ChokePoint::as_str,
        );
    }

    #[test]
    fn run_kind_wire_strings() {
        use RunKind::*;
        assert_wire(
            &[
                (AgentAction, "agent_action"),
                (Explorer, "explorer"),
                (PassiveSession, "passive_session"),
            ],
            RunKind::as_str,
        );
    }

    #[test]
    fn edge_outcome_wire_strings() {
        use EdgeOutcome::*;
        assert_wire(
            &[
                (Changed, "changed"),
                (NoChange, "no_change"),
                (Error, "error"),
                (SettleTimeout, "settle_timeout"),
                (ToNodeUnobserved, "to_node_unobserved"),
            ],
            EdgeOutcome::as_str,
        );
    }

    #[test]
    fn frontier_reason_wire_strings() {
        use FrontierReason::*;
        assert_wire(
            &[
                (NotYetActivated, "not_yet_activated"),
                (EffectUndeclared, "effect_undeclared"),
                (EffectWrite, "effect_write"),
                (EffectDestructive, "effect_destructive"),
                (BudgetExhausted, "budget_exhausted"),
                (ActivationFailed, "activation_failed"),
            ],
            FrontierReason::as_str,
        );
    }

    #[test]
    fn ledger_state_wire_strings() {
        use LedgerState::*;
        assert_wire(
            &[
                (Writing, "writing"),
                (SchemaAbsent, "schema_absent"),
                (WriteFailing, "write_failing"),
            ],
            LedgerState::as_str,
        );
    }

    // --- Round trips --------------------------------------------------------

    #[test]
    fn round_trip_every_type() {
        let node = modelled_node();
        assert_eq!(round_trip(&node), node);
        let unmodelled = JourneyNode::new(None, Vec::<String>::new(), None, None);
        assert_eq!(round_trip(&unmodelled), unmodelled);

        let t = trigger();
        assert_eq!(round_trip(&t), t);
        let bare = JourneyTrigger {
            action_type: "type".into(),
            target_fingerprint: None,
            target_role: None,
            declared_effect: None,
            navigation_trigger: NavigationTriggerKind::Initial,
            choke_point: ChokePoint::SdkElementAction,
        };
        assert_eq!(round_trip(&bare), bare);

        for e in [
            edge(Some(other_node()), EdgeOutcome::Changed),
            edge(None, EdgeOutcome::ToNodeUnobserved),
        ] {
            assert_eq!(round_trip(&e), e);
        }
        let mut withdrawn = edge(Some(modelled_node()), EdgeOutcome::NoChange);
        withdrawn.invalidated_at = Some("2026-09-30T13:00:00Z".into());
        withdrawn.invalidated_reason = Some("bad run".into());
        withdrawn.invalidated_by = Some("operator".into());
        withdrawn.invalidation_token = Some("tok".into());
        assert_eq!(round_trip(&withdrawn), withdrawn);

        let frontier = FrontierEntry {
            app_id: "qontinui-web".into(),
            node_key: node.key(),
            node: node.clone(),
            affordance_fingerprint: "fp-delete".into(),
            affordance_role: Some("button".into()),
            declared_effect: Some(IrEffect::Destructive),
            reason: FrontierReason::EffectDestructive,
            first_seen_at: "2026-09-30T12:00:00Z".into(),
            last_seen_at: "2026-09-30T12:05:00Z".into(),
            last_seen_run_id: Some("session-1".into()),
        };
        assert_eq!(frontier.validate(), Ok(()));
        assert_eq!(round_trip(&frontier), frontier);

        let health = JourneyLedgerHealth {
            state: LedgerState::SchemaAbsent,
            detail: "embedded DB predates the journey tables".into(),
        };
        assert_eq!(round_trip(&health), health);
        assert_eq!(
            serde_json::to_value(&health).unwrap(),
            json!({"state": "schema_absent", "detail": "embedded DB predates the journey tables"})
        );

        for outcome in [
            EdgeOutcome::Changed,
            EdgeOutcome::NoChange,
            EdgeOutcome::Error,
            EdgeOutcome::SettleTimeout,
            EdgeOutcome::ToNodeUnobserved,
        ] {
            assert_eq!(round_trip(&outcome), outcome);
        }
    }

    #[test]
    fn edge_null_and_absent_to_node_read_the_same() {
        let mut value = serde_json::to_value(edge(None, EdgeOutcome::ToNodeUnobserved)).unwrap();
        assert!(value.get("toNode").is_none());
        value["toNode"] = serde_json::Value::Null;
        let parsed: JourneyEdgeObservation = serde_json::from_value(value).unwrap();
        assert_eq!(parsed.to_node, None);
        assert_eq!(parsed.validate(), Ok(()));
    }

    #[test]
    fn no_change_with_differing_keys_is_rejected() {
        assert_eq!(
            edge(Some(other_node()), EdgeOutcome::NoChange).validate(),
            Err(JourneyContractError::NoChangeAcrossDifferentNodes {
                from_key: "coord-runners#a,b".into(),
                to_key: "coord-lands#lands-list".into(),
            })
        );
        let msg = edge(Some(other_node()), EdgeOutcome::NoChange)
            .validate()
            .unwrap_err()
            .to_string();
        assert!(msg.starts_with("outcome is no_change "), "{msg}");
    }

    #[test]
    fn changed_with_equal_keys_is_valid() {
        // A DOM change within the same IR states: same key, still `changed`.
        let mut to = modelled_node();
        to.page_label = Some("runners-tab-2".into());
        assert_eq!(edge(Some(to), EdgeOutcome::Changed).validate(), Ok(()));
        // An unmodelled /runs/[id] -> /runs/[id] (a different run).
        let run = JourneyNode::new(None, Vec::<String>::new(), Some("/runs/[id]".into()), None);
        let mut e = edge(Some(run.clone()), EdgeOutcome::Changed);
        e.from_node = run;
        assert_eq!(e.validate(), Ok(()));
    }

    #[test]
    fn error_messages_render_wire_strings() {
        let err = edge(None, EdgeOutcome::SettleTimeout)
            .validate()
            .unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("outcome is settle_timeout;"), "{msg}");
        assert!(!msg.contains("SettleTimeout"), "{msg}");
    }

    #[test]
    fn key_ambiguous_spec_ids_are_rejected() {
        for spec_id in ["a#b", "unmodelled:/x"] {
            let node = JourneyNode::new(Some(spec_id.into()), ["s".to_string()], None, None);
            assert!(
                matches!(
                    node.validate(),
                    Err(JourneyContractError::NonCanonicalNode { .. })
                ),
                "{spec_id} must be rejected"
            );
        }
        // Rejected even when unmodelled: the spec id is still stored.
        let node = JourneyNode::new(Some("a#b".into()), Vec::<String>::new(), None, None);
        assert!(node.validate().is_err());
    }

    #[test]
    fn key_ambiguous_state_ids_are_rejected() {
        // Without the rule, {"a,b"} and {"a","b"} would share the key "s#a,b".
        let joined = JourneyNode::new(Some("s".into()), ["a,b".to_string()], None, None);
        let split = JourneyNode::new(Some("s".into()), ["a", "b"].map(String::from), None, None);
        assert_eq!(joined.key(), split.key());
        assert!(matches!(
            joined.validate(),
            Err(JourneyContractError::NonCanonicalNode { .. })
        ));
        assert_eq!(split.validate(), Ok(()));
    }

    #[test]
    fn node_optional_fields_serialize_as_null() {
        let node = JourneyNode::new(None, Vec::<String>::new(), None, None);
        assert_eq!(
            serde_json::to_value(&node).unwrap(),
            json!({
                "specId": null,
                "stateIds": [],
                "modelled": false,
                "pathnameTemplate": null,
                "pageLabel": null,
            })
        );
        // Absent keys still read as None.
        let parsed: JourneyNode =
            serde_json::from_value(json!({"stateIds": [], "modelled": false})).unwrap();
        assert_eq!(parsed, node);
    }

    #[test]
    fn state_id_order_is_bytewise() {
        // 'Z' (0x5A) < 'a' (0x61) < 'é' (0xC3 0xA9): byte order, not collation.
        let node = JourneyNode::new(
            Some("s".into()),
            ["é", "a", "Z"].map(String::from),
            None,
            None,
        );
        assert_eq!(node.state_ids, vec!["Z", "a", "é"]);
    }

    #[test]
    fn struct_schemas_have_no_ref_to_a_journey_enum() {
        const ENUMS: [&str; 7] = [
            "NavigationTriggerKind",
            "ChokePoint",
            "RunKind",
            "EdgeOutcome",
            "FrontierReason",
            "LedgerState",
            "IrEffect",
        ];
        let schemas = [
            (
                "JourneyEdgeObservation",
                schemars::schema_for!(JourneyEdgeObservation),
            ),
            ("JourneyTrigger", schemars::schema_for!(JourneyTrigger)),
            ("FrontierEntry", schemars::schema_for!(FrontierEntry)),
            (
                "JourneyLedgerHealth",
                schemars::schema_for!(JourneyLedgerHealth),
            ),
        ];
        for (name, schema) in schemas {
            let text = serde_json::to_string(&schema).unwrap();
            for e in ENUMS {
                assert!(
                    !text.contains(&format!("\"#/$defs/{e}\"")),
                    "{name} schema references {e} by $ref: {text}"
                );
            }
        }
        // Control: the probe's spelling does match a real $ref (the nested
        // struct JourneyNode is referenced, not inlined), so a zero above is
        // not vacuous.
        let frontier = serde_json::to_string(&schemars::schema_for!(FrontierEntry)).unwrap();
        assert!(frontier.contains("\"#/$defs/JourneyNode\""), "{frontier}");
        // The inlined closed sets are still present.
        let edge = serde_json::to_string(&schemars::schema_for!(JourneyEdgeObservation)).unwrap();
        assert!(edge.contains("\"to_node_unobserved\"") && edge.contains("\"passive_session\""));
        assert!(
            !edge.contains("\"timeline\""),
            "timeline must not be representable"
        );
    }

    #[test]
    fn frontier_node_key_must_match_node() {
        let node = modelled_node();
        let entry = FrontierEntry {
            app_id: "a".into(),
            node_key: "coord-runners#b,a".into(),
            node,
            affordance_fingerprint: "fp".into(),
            affordance_role: None,
            declared_effect: None,
            reason: FrontierReason::EffectUndeclared,
            first_seen_at: "2026-09-30T12:00:00Z".into(),
            last_seen_at: "2026-09-30T12:00:00Z".into(),
            last_seen_run_id: None,
        };
        assert_eq!(
            entry.validate(),
            Err(JourneyContractError::NodeKeyMismatch {
                node_key: "coord-runners#b,a".into(),
                expected: "coord-runners#a,b".into(),
            })
        );
    }
}
