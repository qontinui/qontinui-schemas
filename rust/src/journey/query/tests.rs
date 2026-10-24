use super::*;
use crate::journey::{ChokePoint, JourneyContractError};
use chrono::TimeZone;
use serde_json::json;

const SPEC: &str = "fx";

fn node(name: &str) -> JourneyNode {
    JourneyNode::new(Some(SPEC.into()), [name.to_string()], None, None)
}

fn key(name: &str) -> String {
    node(name).key()
}

fn trigger(kind: NavigationTriggerKind, fingerprint: &str) -> JourneyTrigger {
    if kind == NavigationTriggerKind::Affordance {
        JourneyTrigger {
            action_type: "click".into(),
            target_fingerprint: Some(fingerprint.into()),
            target_role: Some("link".into()),
            declared_effect: None,
            navigation_trigger: kind,
            choke_point: ChokePoint::ElementAction,
        }
    } else {
        JourneyTrigger {
            action_type: "navigate".into(),
            target_fingerprint: None,
            target_role: None,
            declared_effect: None,
            navigation_trigger: kind,
            choke_point: ChokePoint::Navigation,
        }
    }
}

struct E {
    id: &'static str,
    from: &'static str,
    to: Option<&'static str>,
    kind: NavigationTriggerKind,
    outcome: EdgeOutcome,
    at: &'static str,
}

fn e(id: &'static str, from: &'static str, to: &'static str) -> E {
    E {
        id,
        from,
        to: Some(to),
        kind: NavigationTriggerKind::Affordance,
        outcome: EdgeOutcome::Changed,
        at: "2026-10-01T00:00:00Z",
    }
}

impl E {
    fn kind(mut self, kind: NavigationTriggerKind) -> Self {
        self.kind = kind;
        self
    }
    fn outcome(mut self, outcome: EdgeOutcome) -> Self {
        self.outcome = outcome;
        if outcome == EdgeOutcome::ToNodeUnobserved {
            self.to = None;
        }
        self
    }
    fn at(mut self, at: &'static str) -> Self {
        self.at = at;
        self
    }
    fn build(&self) -> JourneyEdgeObservation {
        JourneyEdgeObservation {
            id: self.id.into(),
            observed_at: self.at.into(),
            app_id: "app".into(),
            app_version: Some("1.0.0".into()),
            runner_build_id: "build".into(),
            runner_instance: "primary".into(),
            run_id: Some("run-1".into()),
            run_kind: RunKind::AgentAction,
            from_node: node(self.from),
            to_node: self.to.map(node),
            trigger: trigger(
                self.kind,
                &format!("fp-{}-{}", self.from, self.to.unwrap_or("?")),
            ),
            outcome: self.outcome,
            invalidated_at: None,
            invalidated_reason: None,
            invalidated_by: None,
            invalidation_token: None,
        }
    }
}

fn edges(list: &[E]) -> Vec<JourneyEdgeObservation> {
    list.iter().map(E::build).collect()
}

fn frontier(at: &str, fingerprint: &str, reason: FrontierReason) -> FrontierEntry {
    let n = node(at);
    FrontierEntry {
        app_id: "app".into(),
        node_key: n.key(),
        node: n,
        affordance_fingerprint: fingerprint.into(),
        affordance_role: Some("button".into()),
        declared_effect: None,
        reason,
        first_seen_at: "2026-10-01T00:00:00Z".into(),
        last_seen_at: "2026-10-01T00:00:00Z".into(),
        last_seen_run_id: None,
    }
}

fn no_filter() -> JourneyFilter {
    JourneyFilter::default()
}

fn writing() -> JourneyLedgerHealth {
    ledger_in(LedgerState::Writing)
}

fn ledger_in(state: LedgerState) -> JourneyLedgerHealth {
    JourneyLedgerHealth {
        state,
        detail: format!("test ledger: {}", state.as_str()),
    }
}

// --- Acceptance 3: an empty ledger is a measured zero, not a bare [] -------

#[test]
fn empty_ledger_reports_zero_coverage_and_unobserved_start() {
    let g = build_graph(&[], &[], &writing(), &no_filter());
    assert_eq!(g.coverage.edges_observed, 0);
    assert_eq!(g.coverage.nodes_observed, 0);
    assert_eq!(g.coverage.oldest_edge_at, None);
    assert!(g.coverage.app_versions.is_empty());
    assert_eq!(
        g.coverage.frontier_by_reason.len(),
        FrontierReason::ALL.len()
    );
    assert!(g.coverage.frontier_by_reason.values().all(|&n| n == 0));

    let r = reachable_from(&g, &key("a"));
    assert!(!r.from_observed);
    assert!(r.reachable.is_empty());
    assert_eq!(r.coverage.edges_observed, 0);

    let s = steps_to(&g, &key("a"), &key("b"));
    assert!(!s.from_observed && !s.to_observed);
    assert_eq!(s.steps, None);

    // The coverage is ON the wire: every key present, nulls stated.
    let v = serde_json::to_value(&r).unwrap();
    assert_eq!(v["coverage"]["edgesObserved"], json!(0));
    assert_eq!(v["coverage"]["oldestEdgeAt"], json!(null));
    assert_eq!(v["fromObserved"], json!(false));
    assert_eq!(
        v["filterEcho"],
        json!({"since": null, "appVersion": null, "runKind": null})
    );
}

// --- Acceptance 2: unexplored is not unreachable ----------------------------

#[test]
fn target_behind_a_frontier_open_node_exposes_that_frontier() {
    // A -> B by affordance. C was observed (arrived at by a typed URL), but no
    // affordance edge reaches it. B has open frontier entries.
    let ledger = edges(&[
        e("1", "a", "b"),
        e("2", "a", "c").kind(NavigationTriggerKind::Push),
    ]);
    let open = [
        frontier("b", "fp-b-more", FrontierReason::EffectUndeclared),
        frontier("b", "fp-b-next", FrontierReason::NotYetActivated),
    ];
    let g = build_graph(&ledger, &open, &writing(), &no_filter());
    let s = steps_to(&g, &key("a"), &key("c"));
    assert!(s.from_observed && s.to_observed);
    assert_eq!(s.steps, None);
    assert!(s.path.is_empty());
    assert!(
        !s.frontier.is_settled(&s.coverage, true),
        "a negative over an open frontier is unknown"
    );
    assert_eq!(s.frontier.open_entries, 2);
    assert_eq!(s.frontier.nodes.len(), 1);
    let b = &s.frontier.nodes[0];
    assert_eq!(b.node_key, key("b"));
    let named: Vec<&str> = b
        .open_by_reason
        .values()
        .flatten()
        .map(|f| f.affordance_fingerprint.as_str())
        .collect();
    assert_eq!(named, ["fp-b-more", "fp-b-next"]);
    assert_eq!(s.frontier.by_reason["not_yet_activated"], 1);
    assert_eq!(s.frontier.by_reason["effect_undeclared"], 1);
    assert_eq!(s.frontier.by_reason["effect_write"], 0);
    // The envelope inputs the runner builds `unknown` from — not `absent`.
    assert_eq!(
        s.frontier.unmeasured(&s.coverage, true),
        [("frontier", 2, "frontier_open")]
    );
    assert_eq!(
        s.frontier
            .unsettled_because(&s.coverage, s.from_observed && s.to_observed),
        Some("frontier_open")
    );

    // reachable_from tells the same story.
    let r = reachable_from(&g, &key("a"));
    assert_eq!(r.steps_to(&key("c")), None);
    assert!(!r.frontier.is_settled(&r.coverage, true));

    // Closing B's frontier by exploring it — the explored affordance led to
    // C — flips the answer to a measured path.
    let explored = edges(&[
        e("1", "a", "b"),
        e("2", "a", "c").kind(NavigationTriggerKind::Push),
        e("3", "b", "c"),
    ]);
    let g = build_graph(&explored, &[], &writing(), &no_filter());
    let s = steps_to(&g, &key("a"), &key("c"));
    assert_eq!(s.steps, Some(2));
    let hops: Vec<(&str, &str)> = s
        .path
        .iter()
        .map(|e| (e.from_key.as_str(), e.to_key.as_str()))
        .collect();
    assert_eq!(hops, [("fx#a", "fx#b"), ("fx#b", "fx#c")]);
    assert!(s.frontier.is_settled(&s.coverage, true));

    // Closing it with NO edge to C settles the negative: now it is absent.
    let g = build_graph(&ledger, &[], &writing(), &no_filter());
    let s = steps_to(&g, &key("a"), &key("c"));
    assert_eq!(s.steps, None);
    assert!(s.frontier.is_settled(&s.coverage, true));
    assert!(s.frontier.unmeasured(&s.coverage, true).is_empty());
    assert_eq!(s.frontier.unsettled_because(&s.coverage, true), None);
}

#[test]
fn an_unobserved_departure_keeps_a_negative_unknown() {
    let ledger = edges(&[
        e("1", "a", "b"),
        e("2", "b", "?").outcome(EdgeOutcome::ToNodeUnobserved),
        e("3", "z", "c").kind(NavigationTriggerKind::Push),
    ]);
    let g = build_graph(&ledger, &[], &writing(), &no_filter());
    assert_eq!(g.coverage.edges_observed, 3);
    assert_eq!(g.coverage.edges_to_node_unobserved, 1);
    // Not a graph edge, but an observation of B.
    assert!(g
        .edges
        .iter()
        .all(|e| e.outcome != EdgeOutcome::ToNodeUnobserved));
    assert_eq!(g.node(&key("b")).unwrap().unobserved_departures, 1);

    let s = steps_to(&g, &key("a"), &key("c"));
    assert_eq!(s.steps, None);
    assert!(!s.frontier.is_settled(&s.coverage, true));
    assert_eq!(s.frontier.unobserved_departures, 1);
    assert_eq!(s.frontier.nodes[0].node_key, key("b"));
    assert_eq!(
        s.frontier.unmeasured(&s.coverage, true),
        [("unobserved_departures", 1, "to_node_unobserved")]
    );
    assert_eq!(
        s.frontier.unsettled_because(&s.coverage, true),
        Some("to_node_unobserved")
    );

    let d = dead_ends(&g);
    assert_eq!(
        d.unexplored,
        vec![JourneyUnexploredNode {
            node_key: key("b"),
            open_by_reason: BTreeMap::new(),
            unobserved_departures: 1,
            unmeasured_departures: JourneyUnmeasuredDepartures::default(),
            unconfirmed_destination: false,
            unsettled_by_graph: false,
        }]
    );
    assert!(d.dead_ends.iter().all(|x| x.node_key != key("b")));
}

// --- Acceptance 5: same input in any order -> byte-identical output ----------

/// A small deterministic Fisher-Yates (no `rand` dependency).
fn shuffle<T>(items: &mut [T], mut seed: u64) {
    for i in (1..items.len()).rev() {
        seed = seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let j = (seed >> 33) as usize % (i + 1);
        items.swap(i, j);
    }
}

fn render_everything(edges: &[JourneyEdgeObservation], open: &[FrontierEntry]) -> String {
    let filter = JourneyFilter {
        since: Some(Utc.with_ymd_and_hms(2026, 9, 1, 0, 0, 0).unwrap()),
        app_version: None,
        run_kind: None,
    };
    let g = build_graph(edges, open, &writing(), &filter);
    let mut out = serde_json::to_string(&g).unwrap();
    for n in &g.nodes {
        out.push_str(&serde_json::to_string(&reachable_from(&g, &n.key)).unwrap());
        for m in &g.nodes {
            out.push_str(&serde_json::to_string(&steps_to(&g, &n.key, &m.key)).unwrap());
        }
    }
    out.push_str(&serde_json::to_string(&dead_ends(&g)).unwrap());
    out.push_str(&serde_json::to_string(&requires_prior_knowledge(&g)).unwrap());
    out
}

#[test]
fn output_is_byte_identical_under_any_input_order() {
    let mut ledger = edges(&[
        e("1", "a", "b"),
        e("2", "a", "c"),
        e("3", "b", "d"),
        e("4", "c", "d"),
        e("5", "d", "a").kind(NavigationTriggerKind::Pop),
        e("6", "a", "b").at("2026-10-02T00:00:00Z"),
        e("7", "c", "?").outcome(EdgeOutcome::ToNodeUnobserved),
        e("8", "x", "e").kind(NavigationTriggerKind::Push),
        e("9", "e", "e").outcome(EdgeOutcome::NoChange),
        e("10", "b", "b").at("2026-08-01T00:00:00Z"),
    ]);
    // Same key, different non-identity detail: the representative must not
    // depend on which row came first.
    ledger[1].to_node.as_mut().unwrap().page_label = Some("zz".into());
    ledger[3].from_node.page_label = Some("aa".into());
    let mut open = vec![
        frontier("c", "fp-1", FrontierReason::EffectWrite),
        frontier("d", "fp-2", FrontierReason::NotYetActivated),
        frontier("d", "fp-0", FrontierReason::BudgetExhausted),
    ];
    let reference = render_everything(&ledger, &open);
    for seed in 1..=10u64 {
        shuffle(&mut ledger, seed);
        shuffle(&mut open, seed.wrapping_mul(31));
        assert_eq!(
            render_everything(&ledger, &open),
            reference,
            "seed {seed} changed the output"
        );
    }
}

// --- Graph construction ------------------------------------------------------

#[test]
fn identical_rows_aggregate_into_one_edge() {
    let ledger = edges(&[
        e("1", "a", "b").at("2026-10-03T00:00:00Z"),
        e("2", "a", "b").at("2026-10-01T00:00:00+02:00"),
    ]);
    let g = build_graph(&ledger, &[], &writing(), &no_filter());
    assert_eq!(g.edges.len(), 1);
    assert_eq!(g.edges[0].observations, 2);
    assert_eq!(g.edges[0].first_observed_at, "2026-09-30T22:00:00Z");
    assert_eq!(g.edges[0].last_observed_at, "2026-10-03T00:00:00Z");
    assert_eq!(
        g.coverage.oldest_edge_at.as_deref(),
        Some("2026-09-30T22:00:00Z")
    );
    assert_eq!(g.coverage.runs, 1);
}

#[test]
fn filter_narrows_edges_echoes_itself_and_never_hides_the_frontier() {
    let mut ledger = edges(&[
        e("1", "a", "b").at("2026-09-01T00:00:00Z"),
        e("2", "b", "c").at("2026-10-05T00:00:00Z"),
        e("3", "c", "d").at("2026-10-06T00:00:00Z"),
    ]);
    ledger[2].app_version = None;
    ledger[1].run_kind = RunKind::Explorer;
    let open = [frontier("a", "fp", FrontierReason::NotYetActivated)];

    let since = JourneyFilter {
        since: Some(Utc.with_ymd_and_hms(2026, 10, 1, 0, 0, 0).unwrap()),
        ..JourneyFilter::default()
    };
    let g = build_graph(&ledger, &open, &writing(), &since);
    assert_eq!(g.coverage.edges_observed, 2);
    assert_eq!(g.coverage.edges_filtered_out, 1);
    assert_eq!(
        g.coverage.app_versions,
        vec![None, Some("1.0.0".to_string())]
    );
    // The frontier at A survives the filter that removed A's only edge.
    assert_eq!(g.node(&key("a")).unwrap().frontier.len(), 1);
    assert_eq!(g.coverage.frontier_open, 1);
    assert_eq!(
        serde_json::to_value(&g.filter_echo).unwrap(),
        json!({"since": "2026-10-01T00:00:00Z", "appVersion": null, "runKind": null})
    );

    let version = JourneyFilter {
        app_version: Some("1.0.0".into()),
        ..JourneyFilter::default()
    };
    let g = build_graph(&ledger, &[], &writing(), &version);
    assert_eq!(
        g.coverage.edges_observed, 2,
        "an unreported version never matches"
    );

    let kind = JourneyFilter {
        run_kind: Some(RunKind::Explorer),
        ..JourneyFilter::default()
    };
    let g = build_graph(&ledger, &[], &writing(), &kind);
    assert_eq!(g.coverage.edges_observed, 1);
    assert_eq!(g.filter_echo.run_kind, Some(RunKind::Explorer));
}

#[test]
fn withdrawn_rows_are_excluded_and_broken_rows_are_listed() {
    let mut ledger = edges(&[
        e("ok", "a", "b"),
        e("withdrawn", "a", "c"),
        e("bad-time", "a", "d").at("yesterday"),
        e("bad-contract", "a", "e"),
    ]);
    ledger[1].invalidated_at = Some("2026-10-02T00:00:00Z".into());
    ledger[3].to_node = None; // changed with a null toNode
    let mut bad_frontier = frontier("a", "fp", FrontierReason::NotYetActivated);
    bad_frontier.node_key = "fx#wrong".into();

    let g = build_graph(&ledger, &[bad_frontier], &writing(), &no_filter());
    assert_eq!(g.coverage.edges_observed, 1);
    assert_eq!(g.coverage.edges_invalidated, 1);
    assert_eq!(g.coverage.rows_rejected, 3);
    let rows: Vec<(JourneyRejectedRowKind, &str)> = g
        .rejected
        .iter()
        .map(|r| (r.kind, r.row.as_str()))
        .collect();
    assert_eq!(
        rows,
        [
            (JourneyRejectedRowKind::Edge, "bad-contract"),
            (JourneyRejectedRowKind::Edge, "bad-time"),
            (JourneyRejectedRowKind::Frontier, "fx#wrong fp"),
        ]
    );
    assert!(g.rejected[1].reason.contains("RFC 3339"));
    assert!(
        g.node(&key("c")).is_none(),
        "a withdrawn edge observes nothing"
    );
}

#[test]
fn representative_node_is_the_smallest() {
    let mut ledger = edges(&[e("1", "a", "b"), e("2", "c", "b")]);
    ledger[0].to_node.as_mut().unwrap().page_label = Some("second".into());
    ledger[1].to_node.as_mut().unwrap().page_label = Some("first".into());
    let g = build_graph(&ledger, &[], &writing(), &no_filter());
    // None < Some, so the frontier-free... both carry a label: "first" wins.
    assert_eq!(
        g.node(&key("b")).unwrap().node.page_label.as_deref(),
        Some("first")
    );
}

#[test]
fn unmodelled_nodes_are_counted() {
    let mut ledger = edges(&[e("1", "a", "b")]);
    ledger[0].to_node = Some(JourneyNode::new(
        None,
        Vec::<String>::new(),
        Some("/runs/[id]".into()),
        None,
    ));
    let g = build_graph(&ledger, &[], &writing(), &no_filter());
    assert_eq!(g.coverage.nodes_observed, 2);
    assert_eq!(g.coverage.nodes_unmodelled, 1);
    assert!(g.node("unmodelled:/runs/[id]").is_some());
}

// --- steps_to ----------------------------------------------------------------

#[test]
fn shortest_path_tie_break_takes_the_smallest_next_key() {
    // Two shortest paths a->c->d and a->b->d: b < c, so via b. Edges listed
    // with c first so input order cannot be what decides.
    let ledger = edges(&[
        e("1", "a", "c"),
        e("2", "c", "d"),
        e("3", "a", "b"),
        e("4", "b", "d"),
        e("5", "a", "a"), // a self-loop is not a step
    ]);
    let g = build_graph(&ledger, &[], &writing(), &no_filter());
    let s = steps_to(&g, &key("a"), &key("d"));
    assert_eq!(s.steps, Some(2));
    let via: Vec<&str> = s.path.iter().map(|e| e.to_key.as_str()).collect();
    assert_eq!(via, ["fx#b", "fx#d"]);

    let zero = steps_to(&g, &key("a"), &key("a"));
    assert_eq!(zero.steps, Some(0));
    assert!(zero.path.is_empty());
}

#[test]
fn only_affordance_changed_edges_are_steps() {
    let ledger = edges(&[
        e("1", "a", "b").kind(NavigationTriggerKind::Push),
        e("2", "a", "c").outcome(EdgeOutcome::Error),
        e("3", "a", "d").outcome(EdgeOutcome::SettleTimeout),
        e("4", "a", "e").kind(NavigationTriggerKind::Pop),
    ]);
    let g = build_graph(&ledger, &[], &writing(), &no_filter());
    let r = reachable_from(&g, &key("a"));
    assert_eq!(
        r.reachable,
        vec![JourneyReachedNode {
            node_key: key("a"),
            steps: 0
        }]
    );
    // An error / settle_timeout edge to another node is not a step, but it
    // is not a negative either: "d is not reachable" stays unknown.
    assert!(!r.frontier.is_settled(&r.coverage, true));
    assert_eq!(
        r.frontier.unmeasured(&r.coverage, true),
        [
            ("settle_timeout_departures", 1, "no_event_before_timeout"),
            ("errored_departures", 1, "input_missing"),
        ]
    );
}

// --- dead_ends ---------------------------------------------------------------

#[test]
fn dead_ends_back_only_and_unexplored() {
    let ledger = edges(&[
        e("1", "a", "b"),
        e("2", "b", "a").kind(NavigationTriggerKind::Pop), // b: back-only
        e("3", "a", "c"),
        e("4", "c", "c"), // c: a self-loop is not an exit
        e("5", "a", "d"), // d: has an open frontier
        e("6", "d", "a").kind(NavigationTriggerKind::Push),
    ]);
    let open = [frontier("d", "fp-d", FrontierReason::EffectDestructive)];
    let g = build_graph(&ledger, &open, &writing(), &no_filter());
    let d = dead_ends(&g);
    assert_eq!(
        d.dead_ends,
        vec![
            JourneyDeadEnd {
                node_key: key("b"),
                back_only: Some(true)
            },
            JourneyDeadEnd {
                node_key: key("c"),
                back_only: Some(false)
            },
        ]
    );
    assert_eq!(d.unexplored.len(), 1);
    assert_eq!(d.unexplored[0].node_key, key("d"));
    assert_eq!(d.unexplored[0].open_by_reason["effect_destructive"], 1);
    assert_eq!(d.frontier.open_entries, 1);
}

// --- requires_prior_knowledge ------------------------------------------------

#[test]
fn prior_knowledge_needs_url_only_arrivals() {
    let ledger = edges(&[
        e("1", "a", "b"),                                      // b: clicked into
        e("2", "a", "c").kind(NavigationTriggerKind::Push),    // c: URL only
        e("3", "c", "c").kind(NavigationTriggerKind::Initial), // c: reload
        e("4", "a", "d").kind(NavigationTriggerKind::Hash),    // d: URL + click
        e("5", "b", "d").outcome(EdgeOutcome::Error),          // d: errored click (disqualifies)
        e("6", "a", "f").kind(NavigationTriggerKind::Replace), // f: URL + back
        e("7", "b", "f").kind(NavigationTriggerKind::Pop),
    ]);
    let g = build_graph(&ledger, &[], &writing(), &no_filter());
    let p = requires_prior_knowledge(&g);
    // c qualifies; but the errored b -> d click's real destination is
    // unknown (it may have been c), so c is unsettled, not listed.
    assert!(p.nodes.is_empty(), "{:?}", p.nodes);
    assert_eq!(p.unsettled.len(), 1);
    assert_eq!(p.unsettled[0].node_key, key("c"));
    assert_eq!(
        p.unsettled[0].arrivals,
        BTreeMap::from([("initial".to_string(), 1), ("push".to_string(), 1)])
    );
    assert_eq!(p.unsettled[0].unknown_destination_arrivals, 1);
    // The same click closing `changed`: c is listed (d stays disqualified).
    let mut landed = ledger.clone();
    landed[4].outcome = EdgeOutcome::Changed;
    let p2 = requires_prior_knowledge(&build_graph(&landed, &[], &writing(), &no_filter()));
    assert_eq!(
        p2.nodes,
        vec![JourneyPriorKnowledgeNode {
            node_key: key("c"),
            arrivals: BTreeMap::from([("initial".to_string(), 1), ("push".to_string(), 1)]),
        }]
    );
    // The errored b -> d click is an unmeasured departure from b.
    assert_eq!(
        p.frontier.unmeasured(&p.coverage, true),
        [("errored_departures", 1, "input_missing")]
    );
}

// --- Schemas -----------------------------------------------------------------

#[test]
fn result_schemas_inline_every_enum() {
    let schemas = [
        ("JourneyGraph", schemars::schema_for!(JourneyGraph)),
        ("JourneyReachable", schemars::schema_for!(JourneyReachable)),
        ("JourneySteps", schemars::schema_for!(JourneySteps)),
        ("JourneyDeadEnds", schemars::schema_for!(JourneyDeadEnds)),
        (
            "JourneyPriorKnowledge",
            schemars::schema_for!(JourneyPriorKnowledge),
        ),
    ];
    for (name, schema) in schemas {
        let text = serde_json::to_string(&schema).unwrap();
        for e in [
            "RunKind",
            "EdgeOutcome",
            "FrontierReason",
            "NavigationTriggerKind",
            "ChokePoint",
            "IrEffect",
            "LedgerState",
            "JourneyRejectedRowKind",
        ] {
            assert!(
                !text.contains(&format!("\"#/$defs/{e}\"")),
                "{name} references {e} by $ref"
            );
        }
        // Control: structs ARE referenced, so the probe is not vacuous.
        assert!(text.contains("\"#/$defs/JourneyFilter\""), "{name}: {text}");
    }
    let filter = serde_json::to_value(schemars::schema_for!(JourneyFilter)).unwrap();
    assert_eq!(filter["additionalProperties"], json!(false));
}

#[test]
fn filter_round_trips() {
    let f = JourneyFilter {
        since: Some(Utc.with_ymd_and_hms(2026, 10, 1, 12, 30, 0).unwrap()),
        app_version: Some("abc1234".into()),
        run_kind: Some(RunKind::PassiveSession),
    };
    let text = serde_json::to_string(&f).unwrap();
    assert_eq!(
        text,
        r#"{"since":"2026-10-01T12:30:00Z","appVersion":"abc1234","runKind":"passive_session"}"#
    );
    assert_eq!(serde_json::from_str::<JourneyFilter>(&text).unwrap(), f);
    assert!(serde_json::from_str::<JourneyFilter>(r#"{"sinse":null}"#).is_err());
}

// --- Acceptance 4: the ledger state reaches every query response -------------

#[test]
fn ledger_state_propagates_into_every_query_response() {
    let ledger = edges(&[e("1", "a", "b")]);
    for state in [
        LedgerState::Writing,
        LedgerState::SchemaAbsent,
        LedgerState::WriteFailing,
    ] {
        let g = build_graph(&ledger, &[], &ledger_in(state), &no_filter());
        let rendered = [
            serde_json::to_value(&g).unwrap(),
            serde_json::to_value(reachable_from(&g, &key("a"))).unwrap(),
            serde_json::to_value(steps_to(&g, &key("a"), &key("b"))).unwrap(),
            serde_json::to_value(dead_ends(&g)).unwrap(),
            serde_json::to_value(requires_prior_knowledge(&g)).unwrap(),
        ];
        for v in rendered {
            assert_eq!(
                v["coverage"]["ledger"]["state"],
                json!(state.as_str()),
                "{v}"
            );
        }
        // A negative over an explored part is still not `absent` over a
        // ledger that is not being written: is_settled says so itself, and
        // the ledger outranks every other cause.
        let s = steps_to(&g, &key("b"), &key("a"));
        assert_eq!(
            s.frontier.is_settled(&s.coverage, true),
            state == LedgerState::Writing
        );
        let expected = match state {
            LedgerState::Writing => None,
            LedgerState::SchemaAbsent => Some("ledger_schema_absent"),
            LedgerState::WriteFailing => Some("ledger_write_failing"),
        };
        assert_eq!(s.frontier.unsettled_because(&s.coverage, true), expected);
        assert_eq!(g.coverage.ledger_unknown_code(), expected);
        assert_eq!(
            s.frontier.unsettled_because(&s.coverage, false),
            expected.or(Some("no_observations"))
        );
    }
}

/// Every code `unsettled_because` / `unmeasured` / `ledger_unknown_code` can
/// return is a member of the closed `UnknownCode` set. This crate cannot see
/// that enum (it lives in vision-core), so the pin goes through the generated
/// ui-bridge registry mirror, which vision-core's own test pins to
/// `UnknownCode` in both directions.
#[test]
fn every_returned_unknown_code_is_a_registry_code() {
    use crate::ui_bridge_diagnostics::UiBridgeErrorCode;
    use std::str::FromStr;
    for code in [
        "ledger_schema_absent",
        "ledger_write_failing",
        "no_observations",
        "frontier_open",
        "to_node_unobserved",
        "no_event_before_timeout",
        "producer_failed",
        "input_missing",
    ] {
        let registry = format!("UB-OBS-{}", code.to_ascii_uppercase().replace('_', "-"));
        assert!(
            UiBridgeErrorCode::from_str(&registry).is_ok(),
            "{code} has no registry entry {registry}"
        );
    }
}

// --- C1: a departure left out of the navigation set is never a negative -----
//
// A -> B by affordance, then B -> C. Each test removes B -> C from the
// navigation edge set by one path and asserts that "C is not reachable from
// A" stays unknown with that path's own dimension, and that B is unexplored,
// not a dead end.

fn a_b_c(second: E) -> Vec<JourneyEdgeObservation> {
    edges(&[e("1", "a", "b"), second])
}

fn assert_c_unknown_via(g: &JourneyGraph, dimension: &str, code: &str) {
    let s = steps_to(g, &key("a"), &key("c"));
    assert_eq!(s.steps, None);
    assert!(
        !s.frontier.is_settled(&s.coverage, true),
        "{dimension}: must not settle"
    );
    let dims = s.frontier.unmeasured(&s.coverage, true);
    assert!(
        dims.iter()
            .any(|(d, n, c)| *d == dimension && *n == 1 && *c == code),
        "{dimension}: {dims:?}"
    );
    let b = s.frontier.nodes.iter().find(|n| n.node_key == key("b"));
    assert!(
        b.is_some(),
        "{dimension}: B is named as what was not looked at"
    );
    let to_c_observed = s.to_observed;
    assert_eq!(
        s.frontier
            .unsettled_because(&s.coverage, s.from_observed && to_c_observed),
        if to_c_observed {
            Some(code)
        } else {
            Some("no_observations")
        },
        "{dimension}"
    );
    let r = reachable_from(g, &key("a"));
    assert!(
        !r.frontier.is_settled(&r.coverage, true),
        "{dimension}: reachable_from"
    );
    let d = dead_ends(g);
    assert!(
        d.dead_ends.iter().all(|x| x.node_key != key("b")),
        "{dimension}: B is not a dead end"
    );
    assert!(
        d.unexplored.iter().any(|x| x.node_key == key("b")),
        "{dimension}: B is unexplored"
    );
}

#[test]
fn c1a_a_filtered_departure_is_unmeasured_not_absent() {
    let mut ledger = a_b_c(e("2", "b", "c"));
    ledger[1].app_version = Some("2.0.0".into());
    // C is also observed (by URL) so the question is about the path only.
    ledger.extend(edges(&[e("3", "z", "c").kind(NavigationTriggerKind::Push)]));
    let filter = JourneyFilter {
        app_version: Some("1.0.0".into()),
        ..JourneyFilter::default()
    };
    let g = build_graph(&ledger, &[], &writing(), &filter);
    assert_eq!(
        g.node(&key("b"))
            .unwrap()
            .unmeasured_departures
            .filtered_out,
        1
    );
    assert_c_unknown_via(&g, "filtered_departures", "input_missing");

    // A filtered no_change click is a genuine negative: nothing to count.
    let mut ledger = a_b_c(e("2", "b", "b").outcome(EdgeOutcome::NoChange));
    ledger[1].app_version = Some("2.0.0".into());
    let g = build_graph(&ledger, &[], &writing(), &filter);
    assert!(g.node(&key("b")).unwrap().unmeasured_departures.is_zero());
}

#[test]
fn c1b_a_withdrawn_departure_is_unmeasured_not_absent() {
    let mut ledger = a_b_c(e("2", "b", "c"));
    ledger[1].invalidated_at = Some("2026-10-02T00:00:00Z".into());
    let g = build_graph(&ledger, &[], &writing(), &no_filter());
    assert_eq!(
        g.node(&key("b")).unwrap().unmeasured_departures.withdrawn,
        1
    );
    assert_c_unknown_via(&g, "withdrawn_departures", "input_missing");
}

#[test]
fn c1c_a_rejected_frontier_row_counts_as_open() {
    let ledger = edges(&[
        e("1", "a", "b"),
        e("3", "z", "c").kind(NavigationTriggerKind::Push),
    ]);
    let mut bad = frontier("b", "fp-b-next", FrontierReason::NotYetActivated);
    bad.node_key = "fx#not-b".into(); // fails validate(): key != node.key()
    assert!(bad.validate().is_err(), "the probe row must be invalid");
    let g = build_graph(&ledger, &[bad], &writing(), &no_filter());
    assert_eq!(g.coverage.rows_rejected, 1);
    assert_eq!(g.coverage.frontier_open, 0, "it cannot be listed");
    assert_eq!(
        g.node(&key("b"))
            .unwrap()
            .unmeasured_departures
            .rejected_frontier,
        1
    );
    assert_c_unknown_via(&g, "rejected_frontier", "frontier_open");
    let f = frontier_of(&g, Some(&key("b")));
    assert!(f.entries.is_empty());
    assert_eq!(
        f.frontier.unsettled_because(&f.coverage, f.node_observed),
        Some("frontier_open"),
        "an empty listing over a rejected row is not 'nothing left'"
    );
}

#[test]
fn c1c_a_rejected_edge_is_unmeasured_and_an_unplaceable_one_counts_everywhere() {
    let mut ledger = a_b_c(e("2", "b", "c").at("not a time"));
    ledger.extend(edges(&[e("3", "z", "c").kind(NavigationTriggerKind::Push)]));
    let g = build_graph(&ledger, &[], &writing(), &no_filter());
    // Nothing in a rejected row is trusted, its fromNode included: it is
    // placed at no node, and every negative counts it.
    assert!(g.node(&key("b")).unwrap().unmeasured_departures.is_zero());
    assert_eq!(g.unplaced_rejected_rows, 1);
    let s = steps_to(&g, &key("a"), &key("c"));
    assert_eq!(
        s.frontier.unmeasured(&s.coverage, true),
        [("unplaced_rejected_rows", 1, "producer_failed")]
    );
    assert!(dead_ends(&g).dead_ends.is_empty());

    // A rejected row whose fromNode itself is invalid cannot be placed, so
    // every negative counts it.
    let mut ledger = edges(&[e("1", "a", "b"), e("2", "b", "c")]);
    ledger[1].from_node.state_ids = vec!["x,y".into()];
    ledger[1].from_node.spec_id = Some("fx".into());
    assert!(ledger[1].from_node.validate().is_err());
    let g = build_graph(&ledger, &[], &writing(), &no_filter());
    assert_eq!(g.unplaced_rejected_rows, 1);
    let s = steps_to(&g, &key("a"), &key("b"));
    assert_eq!(s.steps, Some(1), "a positive answer still stands");
    let s = steps_to(&g, &key("b"), &key("a"));
    assert_eq!(
        s.frontier.unmeasured(&s.coverage, true),
        [("unplaced_rejected_rows", 1, "producer_failed")]
    );
    assert_eq!(
        s.frontier.unsettled_because(&s.coverage, true),
        Some("producer_failed")
    );
}

#[test]
fn c1d_a_settle_timeout_to_another_node_is_unmeasured_not_a_step() {
    let ledger = a_b_c(e("2", "b", "c").outcome(EdgeOutcome::SettleTimeout));
    let g = build_graph(&ledger, &[], &writing(), &no_filter());
    assert_eq!(
        g.node(&key("b"))
            .unwrap()
            .unmeasured_departures
            .settle_timeout,
        1
    );
    // Not a navigation step...
    assert_eq!(reachable_from(&g, &key("a")).steps_to(&key("c")), None);
    // ...and not a negative.
    assert_c_unknown_via(&g, "settle_timeout_departures", "no_event_before_timeout");

    // A settle_timeout that stayed on the node is unmeasured too (its
    // destination may not have rendered yet): b is not a dead end.
    let ledger = a_b_c(e("2", "b", "b").outcome(EdgeOutcome::SettleTimeout));
    let g = build_graph(&ledger, &[], &writing(), &no_filter());
    assert_eq!(
        g.node(&key("b"))
            .unwrap()
            .unmeasured_departures
            .settle_timeout,
        1
    );
    let d = dead_ends(&g);
    assert!(d.dead_ends.iter().all(|x| x.node_key != key("b")));
    // An `error` self-loop observed the failure: b is a dead end.
    let ledger = a_b_c(e("2", "b", "b").outcome(EdgeOutcome::Error));
    let g = build_graph(&ledger, &[], &writing(), &no_filter());
    assert!(g.node(&key("b")).unwrap().unmeasured_departures.is_zero());
    assert!(dead_ends(&g)
        .dead_ends
        .iter()
        .any(|x| x.node_key == key("b")));
}

// --- W1: an empty ledger is no_observations for every question --------------

fn frontier_of(g: &JourneyGraph, key: Option<&str>) -> JourneyFrontier {
    super::frontier(g, key)
}

#[test]
fn w1_no_edges_is_no_observations_whatever_the_subject() {
    // Frontier rows but no edge at all: the nodes are "observed", the graph
    // is not.
    let open = [frontier("a", "fp-a", FrontierReason::NotYetActivated)];
    for rows in [&[][..], &open[..]] {
        let g = build_graph(&[], rows, &writing(), &no_filter());
        assert_eq!(g.coverage.edges_observed, 0);
        let r = reachable_from(&g, &key("a"));
        assert_eq!(
            r.frontier.unsettled_because(&r.coverage, true),
            Some("no_observations")
        );
        let s = steps_to(&g, &key("a"), &key("b"));
        assert_eq!(
            s.frontier.unsettled_because(&s.coverage, true),
            Some("no_observations")
        );
        let d = dead_ends(&g);
        assert_eq!(
            d.frontier.unsettled_because(&d.coverage, true),
            Some("no_observations")
        );
        let p = requires_prior_knowledge(&g);
        assert_eq!(
            p.frontier.unsettled_because(&p.coverage, true),
            Some("no_observations")
        );
        for k in [None, Some(key("a"))] {
            let f = frontier_of(&g, k.as_deref());
            assert_eq!(
                f.frontier.unsettled_because(&f.coverage, f.node_observed),
                Some("no_observations")
            );
        }
    }
    // The ledger state still outranks it.
    let g = build_graph(
        &[],
        &[],
        &ledger_in(LedgerState::SchemaAbsent),
        &no_filter(),
    );
    let r = reachable_from(&g, &key("a"));
    assert_eq!(
        r.frontier.unsettled_because(&r.coverage, true),
        Some("ledger_schema_absent")
    );
}

#[test]
fn frontier_lists_one_node_or_the_whole_graph() {
    let ledger = edges(&[e("1", "a", "b")]);
    let open = [
        frontier("a", "fp-a", FrontierReason::NotYetActivated),
        frontier("b", "fp-b", FrontierReason::EffectWrite),
    ];
    let g = build_graph(&ledger, &open, &writing(), &no_filter());
    let all = frontier_of(&g, None);
    assert!(all.node_observed);
    assert_eq!(all.entries.len(), 2);
    assert_eq!(all.frontier.open_entries, 2);
    let b = frontier_of(&g, Some(&key("b")));
    assert_eq!(b.entries.len(), 1);
    assert_eq!(b.entries[0].affordance_fingerprint, "fp-b");
    let none = frontier_of(&g, Some(&key("zz")));
    assert!(!none.node_observed && none.entries.is_empty());
    assert_eq!(
        none.frontier
            .unsettled_because(&none.coverage, none.node_observed),
        Some("no_observations")
    );
    // A fully explored node: an empty listing is a real "nothing left".
    let g = build_graph(&ledger, &[], &writing(), &no_filter());
    let a = frontier_of(&g, Some(&key("a")));
    assert_eq!(
        a.frontier.unsettled_because(&a.coverage, a.node_observed),
        None
    );
}

// --- Re-review of 3425cc2a ----------------------------------------------------

/// A row whose `run_kind` differs, for filter tests.
fn passive(mut row: JourneyEdgeObservation) -> JourneyEdgeObservation {
    row.run_kind = RunKind::PassiveSession;
    row
}

fn agent_actions_only() -> JourneyFilter {
    JourneyFilter {
        run_kind: Some(RunKind::AgentAction),
        ..JourneyFilter::default()
    }
}

#[test]
fn rr1_a_rejected_frontier_row_counts_at_its_key_column_too() {
    // a -> c, c -> c (no_change). The rejected frontier row says
    // nodeKey = c, but its node JSON is b.
    let ledger = edges(&[
        e("1", "a", "c"),
        e("2", "c", "c").outcome(EdgeOutcome::NoChange),
    ]);
    let mut bad = frontier("b", "fp-c-next", FrontierReason::NotYetActivated);
    bad.node_key = key("c");
    assert!(bad.validate().is_err());
    let g = build_graph(&ledger, &[bad], &writing(), &no_filter());
    assert_eq!(
        g.node(&key("c"))
            .unwrap()
            .unmeasured_departures
            .rejected_frontier,
        1,
        "counted at the key column"
    );
    // ...and at the node JSON's key, which is off the graph here.
    assert_eq!(g.off_graph_departures.rejected_frontier, 1);
    let d = dead_ends(&g);
    assert!(
        d.dead_ends.iter().all(|x| x.node_key != key("c")),
        "{:?}",
        d.dead_ends
    );
    assert!(d.unexplored.iter().any(|x| x.node_key == key("c")));
    let r = reachable_from(&g, &key("c"));
    assert_eq!(
        r.frontier.unsettled_because(&r.coverage, r.from_observed),
        Some("frontier_open")
    );
}

#[test]
fn rr2_an_excluded_pop_row_keeps_prior_knowledge_unknown() {
    // z -push-> n; m -pop-> n (a passive session); n -> z.
    let mut ledger = edges(&[
        e("1", "z", "n").kind(NavigationTriggerKind::Push),
        e("3", "n", "z"),
    ]);
    ledger.push(passive(
        e("2", "m", "n").kind(NavigationTriggerKind::Pop).build(),
    ));
    let g = build_graph(&ledger, &[], &writing(), &agent_actions_only());
    let p = requires_prior_knowledge(&g);
    // The filtered back-arrival into n could disqualify it: n is unsettled,
    // not listed.
    assert!(p.nodes.is_empty());
    assert_eq!(p.unsettled.len(), 1);
    assert_eq!(p.unsettled[0].node_key, key("n"));
    assert_eq!(p.unsettled[0].arrival_doubt.filtered, 1);
    assert_eq!(g.off_graph_departures.excluded_back_arrivals, 1);
    assert_eq!(
        p.frontier.unmeasured(&p.coverage, true),
        [("excluded_back_arrivals", 1, "input_missing")]
    );
    assert_eq!(
        p.frontier.unsettled_because(&p.coverage, true),
        Some("input_missing")
    );
    // A NAVIGATION question ignores it: a pop is not a navigation step.
    let r = reachable_from(&g, &key("z"));
    assert!(r.frontier.is_settled(&r.coverage, true), "{:?}", r.frontier);
}

#[test]
fn rr2_an_excluded_pop_exit_makes_back_only_unknown() {
    // a -> b; b has no affordance exit; its only back exit was withdrawn.
    let mut ledger = edges(&[
        e("1", "a", "b"),
        e("2", "b", "a").kind(NavigationTriggerKind::Pop),
    ]);
    ledger[1].invalidated_at = Some("2026-10-02T00:00:00Z".into());
    let g = build_graph(&ledger, &[], &writing(), &no_filter());
    assert_eq!(
        g.node(&key("b"))
            .unwrap()
            .unmeasured_departures
            .excluded_back_arrivals,
        1
    );
    let d = dead_ends(&g);
    let b = d
        .dead_ends
        .iter()
        .find(|x| x.node_key == key("b"))
        .expect("b is a dead end");
    assert_eq!(
        b.back_only, None,
        "not false: the back exit was not looked at"
    );
    assert!(d.frontier.unmeasured(&d.coverage, true).contains(&(
        "excluded_back_arrivals",
        1,
        "input_missing"
    )));
    // Kept, it is a measured true.
    let ledger = edges(&[
        e("1", "a", "b"),
        e("2", "b", "a").kind(NavigationTriggerKind::Pop),
    ]);
    let d = dead_ends(&build_graph(&ledger, &[], &writing(), &no_filter()));
    assert_eq!(d.dead_ends[0].back_only, Some(true));
}

#[test]
fn rr3_an_empty_or_non_writing_ledger_never_settles() {
    let empty = build_graph(&[], &[], &writing(), &no_filter());
    let s = steps_to(&empty, &key("a"), &key("b"));
    assert!(!s.frontier.is_settled(&s.coverage, true));
    assert_eq!(
        s.frontier.unmeasured(&s.coverage, true),
        [("no_observations", 1, "no_observations")]
    );
    for (state, code) in [
        (LedgerState::SchemaAbsent, "ledger_schema_absent"),
        (LedgerState::WriteFailing, "ledger_write_failing"),
    ] {
        // Even over a graph that would otherwise settle.
        let ledger = edges(&[e("1", "a", "b")]);
        let g = build_graph(&ledger, &[], &ledger_in(state), &no_filter());
        let s = steps_to(&g, &key("b"), &key("a"));
        assert!(!s.frontier.is_settled(&s.coverage, true), "{code}");
        assert_eq!(
            s.frontier.unmeasured(&s.coverage, true),
            [("ledger", 1, code)]
        );
        assert_eq!(s.frontier.unsettled_because(&s.coverage, true), Some(code));
        let g = build_graph(&[], &[], &ledger_in(state), &no_filter());
        let d = dead_ends(&g);
        assert_eq!(
            d.frontier.unmeasured(&d.coverage, true),
            [
                ("ledger", 1, code),
                ("no_observations", 1, "no_observations")
            ]
        );
    }
}

#[test]
fn rr4_off_graph_departures_reach_graph_wide_questions_only() {
    // a -> b is kept; x -> y is filtered out, and x / y appear nowhere else.
    let mut ledger = edges(&[e("1", "a", "b")]);
    ledger.push(passive(e("2", "x", "y").build()));
    let g = build_graph(&ledger, &[], &writing(), &agent_actions_only());
    assert!(g.node(&key("x")).is_none());
    assert_eq!(g.off_graph_departures.filtered_out, 1);
    for (name, f, cov) in [
        ("dead_ends", dead_ends(&g).frontier, g.coverage.clone()),
        (
            "prior_knowledge",
            requires_prior_knowledge(&g).frontier,
            g.coverage.clone(),
        ),
        (
            "frontier(None)",
            frontier_of(&g, None).frontier,
            g.coverage.clone(),
        ),
    ] {
        assert_eq!(f.off_graph_departures.filtered_out, 1, "{name}");
        assert_eq!(
            f.unmeasured(&cov, true),
            [("filtered_departures", 1, "input_missing")],
            "{name}"
        );
    }
    // A component question about a and b cannot reach x.
    let r = reachable_from(&g, &key("a"));
    assert!(r.frontier.is_settled(&r.coverage, true));
    assert!(r.frontier.off_graph_departures.is_zero());
}

// --- Re-review of d84ab7eb ----------------------------------------------------

#[test]
fn rr5_c1_a_rejected_row_counts_whatever_its_claimed_trigger() {
    // a -> b, d -> c, and a rejected row a -> c claiming `push` while naming
    // an affordance fingerprint (NavigationNamesAffordance). Its trigger
    // cannot be trusted: it may be the click that reaches c.
    let mut ledger = edges(&[e("1", "a", "b"), e("2", "d", "c")]);
    let mut bad = e("3", "a", "c").kind(NavigationTriggerKind::Push).build();
    bad.trigger.target_fingerprint = Some("fp-a-c".into());
    assert!(bad.validate().is_err());
    ledger.push(bad);
    let g = build_graph(&ledger, &[], &writing(), &no_filter());
    assert_eq!(g.unplaced_rejected_rows, 1);
    assert!(g.node(&key("a")).unwrap().unmeasured_departures.is_zero());
    let s = steps_to(&g, &key("a"), &key("c"));
    assert_eq!(s.steps, None);
    assert!(!s.frontier.is_settled(&s.coverage, true));
    assert_eq!(
        s.frontier.unmeasured(&s.coverage, true),
        [("unplaced_rejected_rows", 1, "producer_failed")]
    );
}

#[test]
fn rr5_w1_an_excluded_back_arrival_does_not_unsettle_a_component_question() {
    let mut ledger = edges(&[
        e("1", "a", "b"),
        e("2", "b", "a").kind(NavigationTriggerKind::Pop),
    ]);
    ledger[1].invalidated_at = Some("2026-10-02T00:00:00Z".into());
    let g = build_graph(&ledger, &[], &writing(), &no_filter());
    assert_eq!(
        g.node(&key("b"))
            .unwrap()
            .unmeasured_departures
            .excluded_back_arrivals,
        1
    );
    let r = reachable_from(&g, &key("a"));
    assert!(r.frontier.is_settled(&r.coverage, true), "{:?}", r.frontier);
    assert_eq!(r.frontier.unsettled_because(&r.coverage, true), None);
}

#[test]
fn rr5_w2_an_unsettled_pop_exit_makes_back_only_unknown() {
    for outcome in [EdgeOutcome::Error, EdgeOutcome::SettleTimeout] {
        let ledger = edges(&[
            e("1", "a", "b"),
            e("2", "b", "a")
                .kind(NavigationTriggerKind::Pop)
                .outcome(outcome),
        ]);
        let g = build_graph(&ledger, &[], &writing(), &no_filter());
        assert_eq!(
            g.node(&key("b"))
                .unwrap()
                .unmeasured_departures
                .excluded_back_arrivals,
            1,
            "{outcome:?}"
        );
        let d = dead_ends(&g);
        let b = d
            .dead_ends
            .iter()
            .find(|x| x.node_key == key("b"))
            .expect("b is a dead end");
        assert_eq!(b.back_only, None, "{outcome:?}");
        assert!(d.frontier.unmeasured(&d.coverage, true).contains(&(
            "excluded_back_arrivals",
            1,
            "input_missing"
        )));
    }
    // An unsettled pop that stayed on the node is no exit at all.
    let ledger = edges(&[
        e("1", "a", "b"),
        e("2", "b", "b")
            .kind(NavigationTriggerKind::Pop)
            .outcome(EdgeOutcome::Error),
    ]);
    let d = dead_ends(&build_graph(&ledger, &[], &writing(), &no_filter()));
    let b = d.dead_ends.iter().find(|x| x.node_key == key("b")).unwrap();
    assert_eq!(b.back_only, Some(false));
}

#[test]
fn rr5_w3_a_doubly_placed_rejected_frontier_row_sums_once() {
    let ledger = edges(&[e("1", "a", "c")]);
    let mut bad = frontier("b", "fp-x", FrontierReason::NotYetActivated);
    bad.node_key = key("c");
    let mut bad2 = frontier("a", "fp-y", FrontierReason::NotYetActivated);
    bad2.node_key = key("zz");
    let g = build_graph(&ledger, &[bad, bad2], &writing(), &no_filter());
    assert_eq!(g.rejected_frontier_double_placed, 2);
    // Placed at both keys per node...
    assert_eq!(
        g.node(&key("c"))
            .unwrap()
            .unmeasured_departures
            .rejected_frontier,
        1
    );
    assert_eq!(
        g.node(&key("a"))
            .unwrap()
            .unmeasured_departures
            .rejected_frontier,
        1
    );
    // ...summed once graph-wide.
    let f = frontier_of(&g, None);
    assert_eq!(
        f.frontier.unmeasured_departures.rejected_frontier,
        g.coverage.rows_rejected
    );
    assert_eq!(
        dead_ends(&g)
            .frontier
            .unmeasured_departures
            .rejected_frontier,
        2
    );
}

#[test]
fn rr5_s1_an_unplaced_rejected_pop_row_counts_everywhere() {
    // Its trigger cannot be trusted, so a pop-claiming unplaceable row is a
    // possible click anywhere: component questions count it too.
    let mut ledger = edges(&[
        e("1", "a", "b"),
        e("2", "b", "c").kind(NavigationTriggerKind::Pop),
    ]);
    ledger[1].from_node.state_ids = vec!["x,y".into()];
    let g = build_graph(&ledger, &[], &writing(), &no_filter());
    assert_eq!(g.unplaced_rejected_rows, 1);
    let s = steps_to(&g, &key("b"), &key("a"));
    assert_eq!(
        s.frontier.unmeasured(&s.coverage, true),
        [("unplaced_rejected_rows", 1, "producer_failed")]
    );
}

#[test]
fn rr5_s2_the_gate_names_an_unobserved_subject() {
    let ledger = edges(&[e("1", "a", "b")]);
    let g = build_graph(&ledger, &[], &writing(), &no_filter());
    let s = steps_to(&g, &key("b"), &key("zz"));
    let subject = s.from_observed && s.to_observed;
    assert!(!subject);
    assert!(!s.frontier.is_settled(&s.coverage, subject));
    assert_eq!(
        s.frontier.unmeasured(&s.coverage, subject),
        [("subject_unobserved", 1, "no_observations")]
    );
    // unsettled_because(c, s) is None exactly when is_settled(c, s).
    for subject in [true, false] {
        for g in [
            build_graph(&ledger, &[], &writing(), &no_filter()),
            build_graph(&[], &[], &writing(), &no_filter()),
            build_graph(
                &ledger,
                &[],
                &ledger_in(LedgerState::WriteFailing),
                &no_filter(),
            ),
            build_graph(
                &ledger,
                &[frontier("b", "f", FrontierReason::NotYetActivated)],
                &writing(),
                &no_filter(),
            ),
        ] {
            let s = steps_to(&g, &key("b"), &key("a"));
            assert_eq!(
                s.frontier.unsettled_because(&s.coverage, subject).is_none(),
                s.frontier.is_settled(&s.coverage, subject)
            );
        }
    }
}

#[test]
fn rr5_s5_a_withdrawn_no_change_row_is_a_genuine_negative() {
    let mut ledger = edges(&[
        e("1", "a", "b"),
        e("2", "b", "b").outcome(EdgeOutcome::NoChange),
    ]);
    ledger[1].invalidated_at = Some("2026-10-02T00:00:00Z".into());
    let g = build_graph(&ledger, &[], &writing(), &no_filter());
    assert!(g.node(&key("b")).unwrap().unmeasured_departures.is_zero());
    let s = steps_to(&g, &key("b"), &key("a"));
    assert!(s.frontier.is_settled(&s.coverage, true), "{:?}", s.frontier);
    assert!(dead_ends(&g)
        .dead_ends
        .iter()
        .any(|x| x.node_key == key("b")));
}

// --- Re-review of 8cf1f2fc (the reviewer's probes p1..p7, kept as tests) ------

#[test]
fn rr6_p1_an_unplaceable_rejected_frontier_row_counts_everywhere() {
    // a -> b. An open affordance at b, whose frontier row carries a
    // non-canonical node keyed by what that invalid node renders — a key
    // that names no observed node.
    let ledger = edges(&[e("1", "a", "b")]);
    let mut bad = frontier("b", "fp-open", FrontierReason::NotYetActivated);
    bad.node.modelled = false;
    bad.node_key = bad.node.key();
    assert!(bad.validate().is_err());
    assert!(bad.node.validate().is_err());
    let g = build_graph(&ledger, &[bad], &writing(), &no_filter());
    assert_eq!(g.unplaced_rejected_frontier, 1);
    assert!(g.off_graph_departures.is_zero());
    let s = steps_to(&g, &key("b"), &key("a"));
    assert_eq!(
        s.frontier.unmeasured(&s.coverage, true),
        [("unplaced_rejected_frontier", 1, "frontier_open")]
    );
    let r = reachable_from(&g, &key("b"));
    assert!(!r.frontier.is_settled(&r.coverage, true));
    let f = frontier_of(&g, Some(&key("b")));
    assert!(!f.frontier.is_settled(&f.coverage, f.node_observed));
    assert!(dead_ends(&g).dead_ends.is_empty());

    // The same invalid node under a key column that DOES name an observed
    // node is STILL unplaced: with an invalid node, the row's key column
    // cannot be trusted either.
    let mut bad = frontier("b", "fp-open", FrontierReason::NotYetActivated);
    bad.node.modelled = false;
    assert_eq!(bad.node_key, key("b"));
    let g = build_graph(&ledger, &[bad], &writing(), &no_filter());
    assert_eq!(g.unplaced_rejected_frontier, 1);
    assert!(g.node(&key("b")).unwrap().unmeasured_departures.is_zero());
}

#[test]
fn rr6_p2_no_dead_end_beside_an_unplaced_row_or_a_non_writing_ledger() {
    let mut ledger = edges(&[
        e("1", "a", "b"),
        e("2", "b", "a").kind(NavigationTriggerKind::Pop),
    ]);
    ledger[1].from_node.modelled = false; // b's back exit, unplaceable
    let g = build_graph(&ledger, &[], &writing(), &no_filter());
    assert_eq!(g.unplaced_rejected_rows, 1);
    let d = dead_ends(&g);
    assert!(d.dead_ends.is_empty(), "{:?}", d.dead_ends);
    let b = d
        .unexplored
        .iter()
        .find(|x| x.node_key == key("b"))
        .unwrap();
    assert!(b.unsettled_by_graph);

    // A non-writing ledger: same.
    let ledger = edges(&[e("1", "a", "b")]);
    let g = build_graph(
        &ledger,
        &[],
        &ledger_in(LedgerState::WriteFailing),
        &no_filter(),
    );
    let d = dead_ends(&g);
    assert!(d.dead_ends.is_empty());
    assert!(d
        .unexplored
        .iter()
        .any(|x| x.node_key == key("b") && x.unsettled_by_graph));
    // Writing: b is a dead end, and not "unsettled by graph".
    let d = dead_ends(&build_graph(&ledger, &[], &writing(), &no_filter()));
    assert_eq!(d.dead_ends.len(), 1);
    assert!(d.unexplored.is_empty());
}

#[test]
fn rr6_p3_an_excluded_url_arrival_keeps_prior_knowledge_unknown() {
    // y -> a by click (v1). z -> y by push (v2) is the ONLY arrival at y.
    let mut ledger = edges(&[
        e("1", "y", "a"),
        e("2", "z", "y").kind(NavigationTriggerKind::Push),
    ]);
    ledger[1].app_version = Some("2.0.0".into());
    let unfiltered = requires_prior_knowledge(&build_graph(&ledger, &[], &writing(), &no_filter()));
    assert_eq!(
        unfiltered.nodes.len(),
        1,
        "y needs prior knowledge, unfiltered"
    );
    let filter = JourneyFilter {
        app_version: Some("1.0.0".into()),
        ..JourneyFilter::default()
    };
    let g = build_graph(&ledger, &[], &writing(), &filter);
    let p = requires_prior_knowledge(&g);
    assert!(p.nodes.is_empty());
    assert_eq!(
        p.frontier.unmeasured(&p.coverage, true),
        [("excluded_url_arrivals", 1, "input_missing")]
    );
    let f = frontier_of(&g, None);
    assert!(!f.frontier.is_settled(&f.coverage, true));
    // A navigation question ignores it.
    let r = reachable_from(&g, &key("y"));
    assert!(r.frontier.is_settled(&r.coverage, true));

    // A withdrawn push: same.
    let mut ledger_w = ledger.clone();
    ledger_w[1].app_version = Some("1.0.0".into());
    ledger_w[1].invalidated_at = Some("2026-10-02T00:00:00Z".into());
    let pw = requires_prior_knowledge(&build_graph(&ledger_w, &[], &writing(), &no_filter()));
    assert!(pw.nodes.is_empty());
    assert_eq!(
        pw.frontier.unmeasured(&pw.coverage, true),
        [("excluded_url_arrivals", 1, "input_missing")]
    );
}

#[test]
fn rr6_p4_l3_a_withdrawn_row_with_a_bad_timestamp_is_untrusted() {
    let mut ledger = edges(&[
        e("1", "a", "b"),
        e("2", "a", "c")
            .kind(NavigationTriggerKind::Push)
            .at("garbage"),
    ]);
    let live = build_graph(&ledger, &[], &writing(), &no_filter());
    ledger[1].invalidated_at = Some("2026-10-02T00:00:00Z".into());
    let wd = build_graph(&ledger, &[], &writing(), &no_filter());
    let (l, w) = (
        live.node(&key("a")).unwrap().unmeasured_departures,
        wd.node(&key("a")).unwrap().unmeasured_departures,
    );
    assert!(l.is_zero() && w.is_zero(), "placed at no node");
    assert_eq!(live.unplaced_rejected_rows, 1);
    assert_eq!(
        wd.unplaced_rejected_rows, 1,
        "withdrawn is judged like live"
    );
}

#[test]
fn rr6_p5_l2_duplicate_rejected_frontier_rows_count_once() {
    let ledger = edges(&[e("1", "a", "c")]);
    let mut bad = frontier("a", "fp-x", FrontierReason::NotYetActivated);
    bad.node_key = key("zz");
    let g = build_graph(&ledger, &[bad.clone(), bad], &writing(), &no_filter());
    assert_eq!(g.coverage.rows_rejected, 1);
    let f = frontier_of(&g, None);
    assert_eq!(
        f.frontier.unmeasured_departures.rejected_frontier + f.frontier.unplaced_rejected_frontier,
        g.coverage.rows_rejected
    );
}

#[test]
fn rr6_p6_determinism_with_the_new_counters() {
    let mut ledger = edges(&[
        e("1", "a", "b"),
        e("2", "b", "a")
            .kind(NavigationTriggerKind::Pop)
            .outcome(EdgeOutcome::Error),
        e("3", "a", "c").kind(NavigationTriggerKind::Push),
        e("4", "b", "c"),
        e("5", "c", "d").kind(NavigationTriggerKind::Replace),
    ]);
    ledger[2].trigger.target_fingerprint = Some("x".into()); // rejected
    ledger[3].invalidated_at = Some("2026-10-02T00:00:00Z".into());
    ledger[3].from_node.modelled = false; // withdrawn + invalid -> unplaced
    ledger[4].invalidated_at = Some("2026-10-02T00:00:00Z".into()); // url arrival
    let mut b1 = frontier("b", "fp-x", FrontierReason::NotYetActivated);
    b1.node_key = key("c");
    let mut b2 = frontier("b", "fp-z", FrontierReason::NotYetActivated);
    b2.node.modelled = false;
    b2.node_key = b2.node.key(); // unplaced
    let fr = vec![
        b1,
        b2,
        frontier("a", "fp-y", FrontierReason::NotYetActivated),
    ];
    let base = render_everything(&ledger, &fr);
    for seed in 1..40u64 {
        let mut l = ledger.clone();
        let mut f = fr.clone();
        shuffle(&mut l, seed);
        shuffle(&mut f, seed * 7 + 1);
        assert_eq!(render_everything(&l, &f), base, "seed {seed}");
    }
}

#[test]
fn rr6_p7_f4_an_action_choke_point_claiming_a_url_trigger_is_rejected() {
    let mut ledger = edges(&[e("1", "a", "b"), e("2", "b", "c")]);
    // chokePoint stays element_action, fingerprint kept: a click posing as
    // a typed URL.
    ledger[1].trigger.navigation_trigger = NavigationTriggerKind::Push;
    assert!(matches!(
        ledger[1].validate(),
        Err(JourneyContractError::ActionChokePointNotAffordance { .. })
    ));
    let g = build_graph(&ledger, &[], &writing(), &no_filter());
    assert_eq!(g.coverage.rows_rejected, 1);
    let s = steps_to(&g, &key("a"), &key("c"));
    assert!(!s.frontier.is_settled(&s.coverage, true));
    assert!(s.frontier.unmeasured(&s.coverage, true).contains(&(
        "unplaced_rejected_rows",
        1,
        "producer_failed"
    )));
}

#[test]
fn rr6_l5_an_unplaced_row_claiming_push_counts_everywhere() {
    let mut ledger = edges(&[
        e("1", "a", "b"),
        e("2", "b", "c").kind(NavigationTriggerKind::Push),
    ]);
    ledger[1].from_node.state_ids = vec!["x,y".into()];
    let g = build_graph(&ledger, &[], &writing(), &no_filter());
    assert_eq!(g.unplaced_rejected_rows, 1);
    let s = steps_to(&g, &key("b"), &key("a"));
    assert_eq!(
        s.frontier.unmeasured(&s.coverage, true),
        [("unplaced_rejected_rows", 1, "producer_failed")]
    );
}

#[test]
fn rr6_l5_a_withdrawn_row_claiming_push_that_fails_validate_is_rejected() {
    let mut ledger = edges(&[e("1", "a", "b")]);
    let mut bad = e("2", "a", "c").kind(NavigationTriggerKind::Push).build();
    bad.trigger.target_fingerprint = Some("fp-a-c".into());
    bad.invalidated_at = Some("2026-10-02T00:00:00Z".into());
    assert!(bad.validate().is_err());
    ledger.push(bad);
    let g = build_graph(&ledger, &[], &writing(), &no_filter());
    assert!(g.node(&key("a")).unwrap().unmeasured_departures.is_zero());
    assert_eq!(g.unplaced_rejected_rows, 1);
    let s = steps_to(&g, &key("a"), &key("c"));
    assert!(!s.frontier.is_settled(&s.coverage, true));
}

// --- Re-review of 9bd3a8c2 (probes q1..q5, kept as tests) ----------------------

/// Two live rows with the SAME id and the SAME rejection reason, from
/// different valid nodes a and q.
fn id_collision() -> (
    Vec<JourneyEdgeObservation>,
    JourneyEdgeObservation,
    JourneyEdgeObservation,
) {
    let base = edges(&[e("1", "a", "b"), e("2", "q", "r")]);
    let mut x_a = e("x", "a", "c").build();
    x_a.trigger.navigation_trigger = NavigationTriggerKind::Push; // element_action + push
    let mut x_q = e("x", "q", "c").build();
    x_q.trigger.navigation_trigger = NavigationTriggerKind::Push;
    assert_eq!(
        x_a.validate().unwrap_err().to_string(),
        x_q.validate().unwrap_err().to_string()
    );
    (base, x_a, x_q)
}

#[test]
fn rr7_q1_an_id_collision_places_both_rows_in_any_order() {
    let (base, x_a, x_q) = id_collision();
    let mut l1 = base.clone();
    l1.extend([x_a.clone(), x_q.clone()]);
    let mut l2 = base;
    l2.extend([x_q, x_a]);
    let g1 = build_graph(&l1, &[], &writing(), &no_filter());
    let g2 = build_graph(&l2, &[], &writing(), &no_filter());
    assert_eq!(
        serde_json::to_string(&g1).unwrap(),
        serde_json::to_string(&g2).unwrap(),
        "order-dependent graph"
    );
    assert_eq!(g1.unplaced_rejected_rows, 2);
    // The rejected LIST keeps its (kind, row, reason) identity: one entry.
    assert_eq!(g1.coverage.rows_rejected, 1);
}

#[test]
fn rr7_q1b_an_id_collision_never_settles_the_second_node() {
    for flip in [false, true] {
        let (mut l, x_a, x_q) = id_collision();
        if flip {
            l.extend([x_q, x_a]);
        } else {
            l.extend([x_a, x_q]);
        }
        let g = build_graph(&l, &[], &writing(), &no_filter());
        for from in ["a", "q"] {
            let s = steps_to(&g, &key(from), &key(if from == "a" { "r" } else { "b" }));
            assert!(
                !s.frontier.is_settled(&s.coverage, true),
                "flip={flip} from={from}: {:?}",
                s.frontier.unmeasured(&s.coverage, true)
            );
        }
    }
}

#[test]
fn rr7_l2_an_identical_duplicate_rejected_edge_row_is_placed_once() {
    let mut l = edges(&[e("1", "a", "b")]);
    let mut bad = e("x", "a", "c").build();
    bad.trigger.navigation_trigger = NavigationTriggerKind::Push;
    l.extend([bad.clone(), bad]);
    let g = build_graph(&l, &[], &writing(), &no_filter());
    assert_eq!(g.unplaced_rejected_rows, 1);
}

#[test]
fn rr7_q2_a_frontier_row_with_an_invalid_node_is_always_unplaced() {
    // a -> d (d has no exit), a -> x. A rejected frontier row whose node is
    // d with a leaked pageLabel (invalid); its key column names x.
    let ledger = edges(&[e("1", "a", "d"), e("2", "a", "x")]);
    let mut bad = frontier("d", "fp-open", FrontierReason::NotYetActivated);
    bad.node.page_label = Some("search/sentinel".into());
    assert!(bad.node.validate().is_err());
    assert_eq!(bad.node.key(), key("d"));
    bad.node_key = key("x");
    let g = build_graph(&ledger, &[bad], &writing(), &no_filter());
    assert_eq!(g.unplaced_rejected_frontier, 1);
    assert!(g.node(&key("x")).unwrap().unmeasured_departures.is_zero());
    let r = reachable_from(&g, &key("d"));
    assert!(!r.frontier.is_settled(&r.coverage, true));
    let de = dead_ends(&g);
    assert!(de.dead_ends.is_empty(), "{:?}", de.dead_ends);
}

#[test]
fn rr7_q3_prior_knowledge_does_not_list_a_node_with_a_doubtful_click_arrival() {
    // z -> b by push (kept); a -> b by click, WITHDRAWN (valid).
    let mut ledger = edges(&[
        e("1", "a", "z"),
        e("2", "z", "b").kind(NavigationTriggerKind::Push),
        e("3", "a", "b"),
    ]);
    ledger[2].invalidated_at = Some("2026-10-02T00:00:00Z".into());
    let g = build_graph(&ledger, &[], &writing(), &no_filter());
    let p = requires_prior_knowledge(&g);
    assert!(p.nodes.iter().all(|n| n.node_key != key("b")));
    let b = p.unsettled.iter().find(|n| n.node_key == key("b")).unwrap();
    assert_eq!(b.arrival_doubt.withdrawn, 1);
    assert_eq!(b.arrivals["push"], 1);

    // An untrusted row anywhere, or a click with an unknown destination,
    // unsettles every candidate.
    let mut ledger = edges(&[
        e("1", "a", "z"),
        e("2", "z", "b").kind(NavigationTriggerKind::Push),
        e("3", "a", "?").outcome(EdgeOutcome::ToNodeUnobserved),
    ]);
    let p = requires_prior_knowledge(&build_graph(&ledger, &[], &writing(), &no_filter()));
    assert!(p.nodes.is_empty());
    assert_eq!(p.unsettled[0].unknown_destination_arrivals, 1);
    ledger.pop();
    let mut bad = e("9", "q", "r").build();
    bad.trigger.navigation_trigger = NavigationTriggerKind::Push;
    ledger.push(bad);
    let p = requires_prior_knowledge(&build_graph(&ledger, &[], &writing(), &no_filter()));
    assert!(p.nodes.is_empty());
    assert_eq!(p.unsettled[0].unplaced_rejected_rows, 1);
    // Control: with neither, b is listed.
    ledger.pop();
    let p = requires_prior_knowledge(&build_graph(&ledger, &[], &writing(), &no_filter()));
    assert_eq!(p.nodes.len(), 1);
    assert!(p.unsettled.is_empty());
}

#[test]
fn rr7_q5_determinism_many_seeds_mixed() {
    let mut ledger = edges(&[
        e("1", "a", "b"),
        e("2", "b", "c").kind(NavigationTriggerKind::Push),
        e("3", "c", "a").kind(NavigationTriggerKind::Pop),
        e("4", "c", "d").outcome(EdgeOutcome::SettleTimeout),
        e("5", "d", "d").outcome(EdgeOutcome::NoChange),
        e("6", "a", "e").at("bad"),
        e("7", "e", "f").kind(NavigationTriggerKind::Hash),
    ]);
    ledger[1].invalidated_at = Some("2026-10-03T00:00:00Z".into());
    ledger[6].from_node.modelled = false;
    let (_, x_a, x_q) = id_collision();
    ledger.extend([x_a, x_q]);
    let mut fr1 = frontier("d", "fp1", FrontierReason::EffectWrite);
    fr1.node.page_label = Some("x/y".into());
    let fr = vec![
        fr1.clone(),
        fr1,
        frontier("a", "fp2", FrontierReason::BudgetExhausted),
    ];
    let base = render_everything(&ledger, &fr);
    for seed in 1..200u64 {
        let mut l = ledger.clone();
        let mut f = fr.clone();
        shuffle(&mut l, seed);
        shuffle(&mut f, seed * 13 + 3);
        assert_eq!(render_everything(&l, &f), base, "seed {seed}");
    }
}

// --- Public row-trust and filter helpers ----------------------------------------

#[test]
fn trusted_edge_row_and_keeps_are_the_graphs_own_rules() {
    let ok = e("1", "a", "b").build();
    assert!(trusted_edge_row(&ok).is_ok());
    let bad_time = e("2", "a", "b").at("garbage").build();
    let err = trusted_edge_row(&bad_time).unwrap_err();
    assert!(err.contains("RFC 3339"), "{err}");
    let mut bad_trigger = e("3", "a", "b").build();
    bad_trigger.trigger.navigation_trigger = NavigationTriggerKind::Push;
    let err = trusted_edge_row(&bad_trigger).unwrap_err();
    // The reason is worded exactly as the graph's rejected list words it.
    let g = build_graph(&[bad_trigger], &[], &writing(), &no_filter());
    assert_eq!(g.rejected[0].reason, err);

    let at = trusted_edge_row(&ok).unwrap();
    let f = JourneyFilter {
        app_version: Some("2.0.0".into()),
        ..JourneyFilter::default()
    };
    assert!(!f.keeps(&ok, at));
    assert!(no_filter().keeps(&ok, at));
}

// --- Re-review of 9ec35528 (probes a..d, kept as tests) ------------------------

fn z_push_b() -> Vec<JourneyEdgeObservation> {
    // z -> b by push is the only arrival at b.
    edges(&[
        e("1", "a", "z"),
        e("2", "z", "b").kind(NavigationTriggerKind::Push),
    ])
}

#[test]
fn rr8_a_prior_knowledge_never_settles_over_a_non_writing_ledger() {
    let g = build_graph(
        &z_push_b(),
        &[],
        &ledger_in(LedgerState::WriteFailing),
        &no_filter(),
    );
    let p = requires_prior_knowledge(&g);
    assert!(p.nodes.is_empty(), "{:?}", p.nodes);
    let b = p.unsettled.iter().find(|n| n.node_key == key("b")).unwrap();
    assert!(b.ledger_not_writing);
    assert_eq!(b.open_frontier_entries, 0);
    // Control: writing, b is listed.
    let p = requires_prior_knowledge(&build_graph(&z_push_b(), &[], &writing(), &no_filter()));
    assert_eq!(p.nodes.len(), 1);
}

#[test]
fn rr8_b_prior_knowledge_never_settles_beside_an_open_frontier() {
    // a has an unexplored link; clicking it may lead to b.
    let fr = [frontier(
        "a",
        "fp-unclicked",
        FrontierReason::NotYetActivated,
    )];
    let p = requires_prior_knowledge(&build_graph(&z_push_b(), &fr, &writing(), &no_filter()));
    assert!(p.nodes.is_empty());
    let b = p.unsettled.iter().find(|n| n.node_key == key("b")).unwrap();
    assert_eq!(b.open_frontier_entries, 1);
    assert!(!b.ledger_not_writing);
}

#[test]
fn rr8_c_a_rejected_frontier_row_unsettles_prior_knowledge() {
    // Unplaced (invalid node).
    let mut bad = frontier("a", "fp-x", FrontierReason::NotYetActivated);
    bad.node.page_label = Some("x/y".into());
    let p = requires_prior_knowledge(&build_graph(&z_push_b(), &[bad], &writing(), &no_filter()));
    assert!(p.nodes.is_empty());
    assert_eq!(p.unsettled[0].open_frontier_entries, 1);
    // Placed at a node, and doubly placed: still counted once.
    let mut bad = frontier("a", "fp-y", FrontierReason::NotYetActivated);
    bad.node_key = key("z");
    let p = requires_prior_knowledge(&build_graph(&z_push_b(), &[bad], &writing(), &no_filter()));
    assert!(p.nodes.is_empty());
    assert_eq!(p.unsettled[0].open_frontier_entries, 1);
}

#[test]
fn rr8_d_m7_a_filtered_click_with_no_destination_is_an_unknown_destination_arrival() {
    let mut ledger = edges(&[
        e("1", "a", "z"),
        e("2", "z", "b").kind(NavigationTriggerKind::Push),
        e("3", "a", "b"),
        e("4", "q", "?").outcome(EdgeOutcome::ToNodeUnobserved),
        e("5", "q", "b").kind(NavigationTriggerKind::Pop),
    ]);
    for row in &mut ledger[2..] {
        row.app_version = Some("2.0.0".into());
    }
    let f = JourneyFilter {
        app_version: Some("1.0.0".into()),
        ..JourneyFilter::default()
    };
    let g = build_graph(&ledger, &[], &writing(), &f);
    assert_eq!(g.unknown_destination_arrivals, 1);
    let p = requires_prior_knowledge(&g);
    assert!(p.nodes.is_empty());
    let b = p.unsettled.iter().find(|n| n.node_key == key("b")).unwrap();
    assert_eq!(b.arrival_doubt.filtered, 2);
    assert_eq!(b.unknown_destination_arrivals, 1);
}

#[test]
fn rr8_m2_rejected_frontier_rows_differing_only_in_node_json_are_both_placed() {
    let ledger = edges(&[e("1", "a", "d")]);
    let mut r1 = frontier("d", "fp", FrontierReason::NotYetActivated);
    r1.node.page_label = Some("x/y".into());
    let mut r2 = r1.clone();
    r2.node.page_label = Some("p/q".into());
    assert_eq!(
        r1.validate().unwrap_err().to_string(),
        r2.validate().unwrap_err().to_string()
    );
    let g = build_graph(&ledger, &[r1, r2], &writing(), &no_filter());
    // One entry in the (kind, row, reason) list, two placements.
    assert_eq!(g.coverage.rows_rejected, 1);
    assert_eq!(g.unplaced_rejected_frontier, 2);
    assert_eq!(g.rejected[0].rows, 2);
}

// --- Re-review of 8d48006d ------------------------------------------------------

#[test]
fn rr9_1_a_settle_timeout_click_elsewhere_has_an_unknown_destination() {
    // a -> q closed settle_timeout: q may be an intermediate render, and the
    // click's real destination could be b.
    let ledger = edges(&[
        e("1", "a", "z"),
        e("2", "z", "b").kind(NavigationTriggerKind::Push),
        e("3", "a", "q").outcome(EdgeOutcome::SettleTimeout),
    ]);
    let g = build_graph(&ledger, &[], &writing(), &no_filter());
    assert_eq!(g.unknown_destination_arrivals, 1);
    let p = requires_prior_knowledge(&g);
    assert!(p.nodes.is_empty(), "{:?}", p.nodes);
    let b = p.unsettled.iter().find(|n| n.node_key == key("b")).unwrap();
    assert_eq!(b.unknown_destination_arrivals, 1);

    // The same row, filtered out: still an unknown destination.
    let mut ledger = ledger;
    ledger[2].app_version = Some("2.0.0".into());
    let f = JourneyFilter {
        app_version: Some("1.0.0".into()),
        ..JourneyFilter::default()
    };
    let g = build_graph(&ledger, &[], &writing(), &f);
    assert_eq!(g.unknown_destination_arrivals, 1);
    assert!(g.node(&key("b")).unwrap().arrival_doubt.is_zero());
    assert!(requires_prior_knowledge(&g).nodes.is_empty());

    // A settle_timeout that stayed on its node has an unknown destination
    // too (it may not have rendered yet); an `error` self-loop does not.
    let ledger = edges(&[
        e("1", "a", "z"),
        e("2", "z", "b").kind(NavigationTriggerKind::Push),
        e("3", "a", "a").outcome(EdgeOutcome::SettleTimeout),
    ]);
    let g = build_graph(&ledger, &[], &writing(), &no_filter());
    assert_eq!(g.unknown_destination_arrivals, 1);
    assert!(requires_prior_knowledge(&g).nodes.is_empty());
    let mut err = ledger;
    err[2].outcome = EdgeOutcome::Error;
    let g = build_graph(&err, &[], &writing(), &no_filter());
    assert_eq!(g.unknown_destination_arrivals, 0);
    assert_eq!(requires_prior_knowledge(&g).nodes.len(), 1);
}

// --- Re-review of 6ac846f5 (probes a..d, kept as tests) ------------------------

#[test]
fn rr10_a_a_settle_timeout_only_destination_is_never_a_settled_dead_end() {
    // a <-> b explored; a -> q closed settle_timeout (q may be a spinner).
    let ledger = edges(&[
        e("1", "a", "b"),
        e("2", "b", "a"),
        e("3", "a", "q").outcome(EdgeOutcome::SettleTimeout),
    ]);
    let g = build_graph(&ledger, &[], &writing(), &no_filter());
    assert!(g.node(&key("q")).unwrap().unconfirmed_destination);
    assert!(!g.node(&key("b")).unwrap().unconfirmed_destination);
    let d = dead_ends(&g);
    assert!(
        d.dead_ends.iter().all(|x| x.node_key != key("q")),
        "{:?}",
        d.dead_ends
    );
    let q = d
        .unexplored
        .iter()
        .find(|x| x.node_key == key("q"))
        .unwrap();
    assert!(q.unconfirmed_destination);
    assert!(!q.unsettled_by_graph);

    // A confirming arrival (changed) makes q a real destination again.
    let mut ledger = ledger;
    ledger.extend(edges(&[e("4", "b", "q")]));
    let g = build_graph(&ledger, &[], &writing(), &no_filter());
    assert!(!g.node(&key("q")).unwrap().unconfirmed_destination);
    assert!(dead_ends(&g)
        .dead_ends
        .iter()
        .any(|x| x.node_key == key("q")));
}

#[test]
fn rr10_b_a_settle_timeout_only_destination_never_settles_prior_knowledge() {
    let ledger = edges(&[
        e("1", "a", "b"),
        e("2", "b", "a"),
        e("3", "a", "q")
            .kind(NavigationTriggerKind::Push)
            .outcome(EdgeOutcome::SettleTimeout),
    ]);
    let g = build_graph(&ledger, &[], &writing(), &no_filter());
    let p = requires_prior_knowledge(&g);
    assert!(p.nodes.is_empty(), "{:?}", p.nodes);
    let q = p.unsettled.iter().find(|n| n.node_key == key("q")).unwrap();
    assert!(q.unconfirmed_destination);
    // Control: the push closed `changed`, q is listed.
    let ledger = edges(&[
        e("1", "a", "b"),
        e("2", "b", "a"),
        e("3", "a", "q").kind(NavigationTriggerKind::Push),
    ]);
    let p = requires_prior_knowledge(&build_graph(&ledger, &[], &writing(), &no_filter()));
    assert_eq!(p.nodes.len(), 1);
}

#[test]
fn rr10_c_a_withdrawn_untrusted_row_is_listed_as_rejected() {
    let mut ledger = edges(&[e("1", "a", "b"), e("2", "b", "a")]);
    let mut bad = e("3", "a", "c").at("not a time").build();
    bad.invalidated_at = Some("2026-10-02T00:00:00Z".into());
    ledger.push(bad);
    let g = build_graph(&ledger, &[], &writing(), &no_filter());
    assert_eq!(g.unplaced_rejected_rows, 1);
    assert_eq!(g.coverage.edges_invalidated, 1);
    assert_eq!(g.coverage.rows_rejected, 1);
    assert_eq!(g.rejected[0].row, "3");
    assert!(g.rejected[0].reason.contains("RFC 3339"));
}

#[test]
fn rr10_d_order_independence_with_untrusted_and_settle_timeout() {
    let mut ledger = edges(&[
        e("1", "a", "b"),
        e("2", "b", "a"),
        e("3", "a", "q").outcome(EdgeOutcome::SettleTimeout),
        e("4", "q", "a")
            .kind(NavigationTriggerKind::Pop)
            .outcome(EdgeOutcome::SettleTimeout),
        e("5", "z", "c").at("bad"),
        e("5", "y", "c").at("bad"),
    ]);
    let f = vec![frontier("a", "fp-x", FrontierReason::NotYetActivated)];
    let base = render_everything(&ledger, &f);
    for seed in 1..60u64 {
        shuffle(&mut ledger, seed);
        assert_eq!(render_everything(&ledger, &f), base, "seed {seed}");
    }
}

#[test]
fn rr11_an_error_only_destination_is_unconfirmed() {
    // a <-> b explored; a -> q by click closed `error`: q's observed node may
    // be an error render, not a real destination.
    let ledger = edges(&[
        e("1", "a", "b"),
        e("2", "b", "a"),
        e("3", "a", "q").outcome(EdgeOutcome::Error),
    ]);
    let g = build_graph(&ledger, &[], &writing(), &no_filter());
    assert!(g.node(&key("q")).unwrap().unconfirmed_destination);
    let d = dead_ends(&g);
    assert!(
        d.dead_ends.iter().all(|x| x.node_key != key("q")),
        "{:?}",
        d.dead_ends
    );
    assert!(d
        .unexplored
        .iter()
        .any(|x| x.node_key == key("q") && x.unconfirmed_destination));

    // The same through a URL arrival that closed `error`.
    let ledger = edges(&[
        e("1", "a", "b"),
        e("2", "b", "a"),
        e("3", "a", "q")
            .kind(NavigationTriggerKind::Push)
            .outcome(EdgeOutcome::Error),
    ]);
    let p = requires_prior_knowledge(&build_graph(&ledger, &[], &writing(), &no_filter()));
    assert!(p.nodes.is_empty(), "{:?}", p.nodes);
    assert!(p
        .unsettled
        .iter()
        .any(|n| n.node_key == key("q") && n.unconfirmed_destination));
}

// --- Re-review of 032a25e4 (probes p1..p3, kept as tests) ----------------------

fn with_click_into_q(outcome: EdgeOutcome) -> Vec<JourneyEdgeObservation> {
    // a <-> b explored; a -> q click closed `outcome`; y reached only by an
    // initial load.
    edges(&[
        e("1", "a", "b"),
        e("2", "b", "a"),
        e("3", "a", "q").outcome(outcome),
        e("4", "b", "y").kind(NavigationTriggerKind::Initial),
        e("5", "y", "a"),
    ])
}

#[test]
fn rr12_1_an_error_click_elsewhere_has_an_unknown_destination() {
    for outcome in [EdgeOutcome::Error, EdgeOutcome::SettleTimeout] {
        let g = build_graph(&with_click_into_q(outcome), &[], &writing(), &no_filter());
        assert_eq!(g.unknown_destination_arrivals, 1, "{outcome:?}");
        let p = requires_prior_knowledge(&g);
        assert!(p.nodes.is_empty(), "{outcome:?}: {:?}", p.nodes);
        let y = p.unsettled.iter().find(|n| n.node_key == key("y")).unwrap();
        assert_eq!(y.unknown_destination_arrivals, 1, "{outcome:?}");
    }
    // The excluded path: the same error click, filtered out.
    let mut ledger = with_click_into_q(EdgeOutcome::Error);
    ledger[2].app_version = Some("2.0.0".into());
    let f = JourneyFilter {
        app_version: Some("1.0.0".into()),
        ..JourneyFilter::default()
    };
    let g = build_graph(&ledger, &[], &writing(), &f);
    assert_eq!(g.unknown_destination_arrivals, 1);
    // Control: the click closed `changed`, y is listed.
    let p = requires_prior_knowledge(&build_graph(
        &with_click_into_q(EdgeOutcome::Changed),
        &[],
        &writing(),
        &no_filter(),
    ));
    assert_eq!(p.nodes.len(), 1);
    assert_eq!(p.nodes[0].node_key, key("y"));
}

#[test]
fn rr12_3_a_self_loop_is_not_an_arrival() {
    // z is a start node whose only edge is an erroring self-loop: no exit,
    // no arrival, so it is a dead end by the usual rules.
    let ledger = edges(&[
        e("1", "a", "b"),
        e("2", "b", "a"),
        e("3", "z", "z").outcome(EdgeOutcome::Error),
    ]);
    let g = build_graph(&ledger, &[], &writing(), &no_filter());
    assert!(!g.node(&key("z")).unwrap().unconfirmed_destination);
    let d = dead_ends(&g);
    assert!(
        d.dead_ends.iter().any(|x| x.node_key == key("z")),
        "{:?}",
        d
    );
}

#[test]
fn rr12_3_determinism_with_error_and_self_loops() {
    let mut ledger = edges(&[
        e("1", "a", "b"),
        e("2", "b", "a"),
        e("3", "a", "q").outcome(EdgeOutcome::Error),
        e("3", "a", "q").outcome(EdgeOutcome::Changed),
        e("6", "a", "q").outcome(EdgeOutcome::SettleTimeout),
        e("7", "q", "q").outcome(EdgeOutcome::NoChange),
        e("4", "b", "y")
            .kind(NavigationTriggerKind::Initial)
            .outcome(EdgeOutcome::Error),
        e("5", "y", "a").at("bad"),
    ]);
    let base = render_everything(&ledger, &[]);
    for seed in 1..200u64 {
        shuffle(&mut ledger, seed);
        assert_eq!(render_everything(&ledger, &[]), base, "seed {seed}");
    }
}

#[test]
fn rr12_6_rejected_entries_and_unplaced_rows_count_the_same_rows() {
    // A live and a withdrawn copy of one untrusted row: same (kind, id,
    // reason), different full rows.
    let mut ledger = edges(&[e("1", "a", "b")]);
    let live = e("x", "a", "c").at("not a time").build();
    let mut withdrawn = live.clone();
    withdrawn.invalidated_at = Some("2026-10-02T00:00:00Z".into());
    ledger.extend([live.clone(), withdrawn, live]); // the live copy twice
    let g = build_graph(&ledger, &[], &writing(), &no_filter());
    assert_eq!(g.rejected.len(), 1);
    assert_eq!(g.rejected[0].rows, 2, "an identical duplicate is one row");
    assert_eq!(g.unplaced_rejected_rows, 2);
    let edge_rows: u64 = g
        .rejected
        .iter()
        .filter(|r| r.kind == JourneyRejectedRowKind::Edge)
        .map(|r| r.rows)
        .sum();
    assert_eq!(edge_rows, g.unplaced_rejected_rows);
}

#[test]
fn rr13_a_settle_timeout_self_loop_is_unmeasured_and_an_error_self_loop_is_not() {
    // a -> z click; z's only departure is a click that did not settle with
    // the snapshot still showing z. c is reached by push.
    let ledger = edges(&[
        e("1", "a", "z"),
        e("2", "z", "a").kind(NavigationTriggerKind::Initial),
        e("3", "z", "z").outcome(EdgeOutcome::SettleTimeout),
        e("4", "a", "c").kind(NavigationTriggerKind::Push),
    ]);
    let g = build_graph(&ledger, &[], &writing(), &no_filter());
    assert_eq!(
        g.node(&key("z"))
            .unwrap()
            .unmeasured_departures
            .settle_timeout,
        1
    );
    assert_eq!(g.unknown_destination_arrivals, 1);
    let d = dead_ends(&g);
    assert!(
        d.dead_ends.iter().all(|x| x.node_key != key("z")),
        "{:?}",
        d.dead_ends
    );
    assert!(d.unexplored.iter().any(|x| x.node_key == key("z")));
    let p = requires_prior_knowledge(&g);
    assert!(p.nodes.is_empty(), "{:?}", p.nodes);
    assert!(p
        .unsettled
        .iter()
        .any(|n| n.node_key == key("c") && n.unknown_destination_arrivals == 1));

    // The same self-loop, filtered out: still unmeasured.
    let mut filtered = ledger.clone();
    filtered[2].app_version = Some("2.0.0".into());
    let f = JourneyFilter {
        app_version: Some("1.0.0".into()),
        ..JourneyFilter::default()
    };
    let g = build_graph(&filtered, &[], &writing(), &f);
    assert_eq!(g.unknown_destination_arrivals, 1);
    assert_eq!(
        g.node(&key("z"))
            .unwrap()
            .unmeasured_departures
            .filtered_out,
        1
    );

    // An `error` self-loop observed the failure: a genuine negative, kept or
    // filtered. z is a dead end and c is listed.
    let mut err = ledger.clone();
    err[2].outcome = EdgeOutcome::Error;
    for (g, label) in [
        (build_graph(&err, &[], &writing(), &no_filter()), "kept"),
        (
            {
                let mut e2 = err.clone();
                e2[2].app_version = Some("2.0.0".into());
                build_graph(&e2, &[], &writing(), &f)
            },
            "filtered",
        ),
    ] {
        assert!(
            g.node(&key("z")).unwrap().unmeasured_departures.is_zero(),
            "{label}"
        );
        assert_eq!(g.unknown_destination_arrivals, 0, "{label}");
        assert!(
            dead_ends(&g)
                .dead_ends
                .iter()
                .any(|x| x.node_key == key("z")),
            "{label}"
        );
        let listed: Vec<String> = requires_prior_knowledge(&g)
            .nodes
            .into_iter()
            .map(|n| n.node_key)
            .collect();
        assert!(listed.contains(&key("c")), "{label}: {listed:?}");
    }
}

#[test]
fn rr14_1_an_excluded_url_self_loop_is_never_a_genuine_negative() {
    // z -> a click, a -> b push, z -> z reload (`initial`). Kept, the reload
    // is a URL arrival at z, so z qualifies for prior knowledge.
    for outcome in [EdgeOutcome::Error, EdgeOutcome::NoChange] {
        let ledger = edges(&[
            e("1", "z", "a"),
            e("2", "a", "b").kind(NavigationTriggerKind::Push),
            e("3", "z", "z")
                .kind(NavigationTriggerKind::Initial)
                .outcome(outcome),
        ]);
        let kept = requires_prior_knowledge(&build_graph(&ledger, &[], &writing(), &no_filter()));
        assert!(
            kept.nodes.iter().any(|n| n.node_key == key("z")),
            "{outcome:?}: kept, z qualifies"
        );
        // Filtered and withdrawn: the reload is an excluded URL arrival, so
        // the graph-wide "no other node requires prior knowledge" is unsettled.
        let mut filtered = ledger.clone();
        filtered[2].app_version = Some("2.0.0".into());
        let f = JourneyFilter {
            app_version: Some("1.0.0".into()),
            ..JourneyFilter::default()
        };
        let mut withdrawn = ledger.clone();
        withdrawn[2].invalidated_at = Some("2026-10-02T00:00:00Z".into());
        for (g, label) in [
            (build_graph(&filtered, &[], &writing(), &f), "filtered"),
            (
                build_graph(&withdrawn, &[], &writing(), &no_filter()),
                "withdrawn",
            ),
        ] {
            assert_eq!(
                g.node(&key("z"))
                    .unwrap()
                    .unmeasured_departures
                    .excluded_url_arrivals,
                1,
                "{outcome:?} {label}"
            );
            let p = requires_prior_knowledge(&g);
            assert!(
                !p.frontier.is_settled(&p.coverage, true),
                "{outcome:?} {label}"
            );
            assert!(
                p.frontier.unmeasured(&p.coverage, true).contains(&(
                    "excluded_url_arrivals",
                    1,
                    "input_missing"
                )),
                "{outcome:?} {label}"
            );
        }
    }
}

#[test]
fn rr14_2_a_kept_pop_settle_timeout_self_loop_makes_back_only_unknown() {
    let ledger = edges(&[
        e("1", "a", "z"),
        e("2", "z", "z")
            .kind(NavigationTriggerKind::Pop)
            .outcome(EdgeOutcome::SettleTimeout),
    ]);
    let g = build_graph(&ledger, &[], &writing(), &no_filter());
    assert_eq!(
        g.node(&key("z"))
            .unwrap()
            .unmeasured_departures
            .excluded_back_arrivals,
        1
    );
    let d = dead_ends(&g);
    let z = d
        .dead_ends
        .iter()
        .find(|x| x.node_key == key("z"))
        .expect("z is a dead end");
    assert_eq!(z.back_only, None);
    // Control: an error pop self-loop is a genuine negative, back_only false.
    let mut err = ledger;
    err[1].outcome = EdgeOutcome::Error;
    let d = dead_ends(&build_graph(&err, &[], &writing(), &no_filter()));
    let z = d.dead_ends.iter().find(|x| x.node_key == key("z")).unwrap();
    assert_eq!(z.back_only, Some(false));
}

// --- Invariant: every trusted excluded row is counted or changes nothing ------
//
// For every trusted row in an enumeration of triggers x outcomes x from x to,
// filtered or withdrawn over two base graphs: either keeping the row would
// change no answer (dead_ends listing and back_only, requires_prior_knowledge,
// reachable_from and its steps, the whole frontier), or the answer computed
// WITHOUT it is unsettled. An excluded row that would change an answer
// while the answer reads settled is a false negative.

#[derive(Debug, Clone, PartialEq)]
enum DeadEndClass {
    Dead(Option<bool>),
    Unexplored,
    Absent,
}

fn dead_end_class(d: &JourneyDeadEnds, k: &str) -> DeadEndClass {
    if let Some(x) = d.dead_ends.iter().find(|x| x.node_key == k) {
        return DeadEndClass::Dead(x.back_only);
    }
    if d.unexplored.iter().any(|x| x.node_key == k) {
        return DeadEndClass::Unexplored;
    }
    DeadEndClass::Absent
}

fn excluded_row_violations(base: &[E], probe: E, label: &str, out: &mut Vec<String>) {
    let names = ["s", "a", "b", "n", "m"];
    let mut ledger = edges(base);
    ledger.push(probe.build());
    let last = ledger.len() - 1;
    let kept = build_graph(&ledger, &[], &writing(), &no_filter());
    let mut filtered = ledger.clone();
    filtered[last].app_version = Some("2.0.0".into());
    let f = JourneyFilter {
        app_version: Some("1.0.0".into()),
        ..JourneyFilter::default()
    };
    let mut withdrawn = ledger.clone();
    withdrawn[last].invalidated_at = Some("2026-10-02T00:00:00Z".into());
    for (exc, how) in [
        (build_graph(&filtered, &[], &writing(), &f), "filtered"),
        (
            build_graph(&withdrawn, &[], &writing(), &no_filter()),
            "withdrawn",
        ),
    ] {
        let tag = format!("{label} [{how}]");
        // dead_ends
        let (dk, dx) = (dead_ends(&kept), dead_ends(&exc));
        let wide_settled = dx.frontier.is_settled(&dx.coverage, true);
        for n in names {
            let k = key(n);
            match (dead_end_class(&dx, &k), dead_end_class(&dk, &k)) {
                (DeadEndClass::Dead(bx), DeadEndClass::Dead(bk)) if bx.is_some() && bx != bk => {
                    out.push(format!("{tag}: back_only {n}: excluded {bx:?} kept {bk:?}"));
                }
                (DeadEndClass::Dead(_), DeadEndClass::Absent) => {
                    out.push(format!("{tag}: excluded lists {n} dead, kept has an exit"))
                }
                (DeadEndClass::Absent, DeadEndClass::Dead(_)) if wide_settled => out.push(format!(
                    "{tag}: kept lists {n} dead, excluded silent and settled"
                )),
                _ => {}
            }
        }
        // requires_prior_knowledge
        let (pk, px) = (
            requires_prior_knowledge(&kept),
            requires_prior_knowledge(&exc),
        );
        let psettled = px.frontier.is_settled(&px.coverage, true);
        for n in names {
            let k = key(n);
            let in_k = pk.nodes.iter().any(|x| x.node_key == k);
            let uns_k = pk.unsettled.iter().any(|x| x.node_key == k);
            let in_x = px.nodes.iter().any(|x| x.node_key == k);
            let uns_x = px.unsettled.iter().any(|x| x.node_key == k);
            if in_x && !in_k && !uns_k {
                out.push(format!(
                    "{tag}: prior {n}: excluded settled, kept disqualified"
                ));
            }
            if in_k && !in_x && !uns_x && psettled {
                out.push(format!(
                    "{tag}: prior {n}: kept lists, excluded silent and settled"
                ));
            }
        }
        // reachable_from and steps
        for s in names {
            let (rk, rx) = (
                reachable_from(&kept, &key(s)),
                reachable_from(&exc, &key(s)),
            );
            if !rx.from_observed || !rx.frontier.is_settled(&rx.coverage, true) {
                continue;
            }
            let ks: Vec<_> = rk
                .reachable
                .iter()
                .map(|r| (r.node_key.clone(), r.steps))
                .collect();
            let xs: Vec<_> = rx
                .reachable
                .iter()
                .map(|r| (r.node_key.clone(), r.steps))
                .collect();
            if ks != xs {
                out.push(format!(
                    "{tag}: reachable_from {s}: excluded settled {xs:?} kept {ks:?}"
                ));
            }
        }
        // the whole frontier
        let (fk, fx) = (frontier_of(&kept, None), frontier_of(&exc, None));
        if !fk.frontier.is_settled(&fk.coverage, true) && fx.frontier.is_settled(&fx.coverage, true)
        {
            out.push(format!(
                "{tag}: whole frontier kept unsettled, excluded settled"
            ));
        }
    }
}

fn excluded_row_enumeration_violations() -> (usize, Vec<String>) {
    use EdgeOutcome as O;
    use NavigationTriggerKind as T;
    let bases: [fn() -> Vec<E>; 2] = [
        || vec![e("1", "s", "a").kind(T::Push), e("2", "a", "b")],
        || {
            vec![
                e("1", "s", "a").kind(T::Push),
                e("2", "a", "b"),
                e("3", "b", "a").kind(T::Pop),
                e("4", "s", "s").kind(T::Initial),
            ]
        },
    ];
    let triggers = [
        T::Affordance,
        T::Pop,
        T::Initial,
        T::Push,
        T::Replace,
        T::Hash,
    ];
    let outcomes = [
        O::Changed,
        O::NoChange,
        O::Error,
        O::SettleTimeout,
        O::ToNodeUnobserved,
    ];
    let froms: [&'static str; 4] = ["s", "a", "b", "n"];
    let tos: [&'static str; 6] = ["s", "a", "b", "n", "m", "SELF"];
    let mut out = Vec::new();
    let mut rows = 0;
    for base in bases {
        for t in triggers {
            for o in outcomes {
                for fr in froms {
                    for to in tos {
                        let to: &'static str = if to == "SELF" { fr } else { to };
                        if matches!(o, O::ToNodeUnobserved | O::NoChange) && to != fr {
                            continue;
                        }
                        let p = e("9", fr, to).kind(t).outcome(o);
                        if p.build().validate().is_err() {
                            continue; // trusted rows only
                        }
                        rows += 1;
                        let label = format!("{t:?}/{o:?} {fr}->{}", p.to.unwrap_or("None"));
                        excluded_row_violations(&base(), p, &label, &mut out);
                    }
                }
            }
        }
    }
    out.sort();
    out.dedup();
    (rows, out)
}

#[test]
fn every_trusted_excluded_row_is_counted_or_changes_nothing() {
    let (rows, violations) = excluded_row_enumeration_violations();
    assert!(rows > 200, "the enumeration is not vacuous: {rows} rows");
    assert!(
        violations.is_empty(),
        "{} violation(s):\n{}",
        violations.len(),
        violations.join("\n")
    );
}

#[test]
fn rr15_an_excluded_genuine_negative_on_an_unkept_node_is_counted() {
    // s -push-> a -click-> b; n -> n (a click or back that closed no_change
    // or error on its own node). Kept, n is a dead end. Excluded, nothing
    // else shows n, so the row is counted off the graph.
    for trigger in [
        NavigationTriggerKind::Affordance,
        NavigationTriggerKind::Pop,
    ] {
        for outcome in [EdgeOutcome::NoChange, EdgeOutcome::Error] {
            let ledger = edges(&[
                e("1", "s", "a").kind(NavigationTriggerKind::Push),
                e("2", "a", "b"),
                e("3", "n", "n").kind(trigger).outcome(outcome),
            ]);
            let label = format!("{trigger:?}/{outcome:?}");
            let kept = dead_ends(&build_graph(&ledger, &[], &writing(), &no_filter()));
            assert!(
                kept.dead_ends.iter().any(|x| x.node_key == key("n")),
                "{label}: kept, n is a dead end"
            );
            let mut filtered = ledger.clone();
            filtered[2].app_version = Some("2.0.0".into());
            let f = JourneyFilter {
                app_version: Some("1.0.0".into()),
                ..JourneyFilter::default()
            };
            let mut withdrawn = ledger.clone();
            withdrawn[2].invalidated_at = Some("2026-10-02T00:00:00Z".into());
            for (g, how) in [
                (build_graph(&filtered, &[], &writing(), &f), "filtered"),
                (
                    build_graph(&withdrawn, &[], &writing(), &no_filter()),
                    "withdrawn",
                ),
            ] {
                assert!(g.node(&key("n")).is_none(), "{label} {how}");
                let og = g.off_graph_departures;
                let counted = match (trigger, how) {
                    (NavigationTriggerKind::Pop, _) => og.excluded_back_arrivals,
                    (_, "filtered") => og.filtered_out,
                    _ => og.withdrawn,
                };
                assert_eq!(counted, 1, "{label} {how}: {og:?}");
                let d = dead_ends(&g);
                assert!(!d.frontier.is_settled(&d.coverage, true), "{label} {how}");
            }
            // On a KEPT node the same row is still a genuine negative.
            let mut on_kept = ledger.clone();
            on_kept[2] = e("3", "b", "b").kind(trigger).outcome(outcome).build();
            on_kept[2].app_version = Some("2.0.0".into());
            let f = JourneyFilter {
                app_version: Some("1.0.0".into()),
                ..JourneyFilter::default()
            };
            let g = build_graph(&on_kept, &[], &writing(), &f);
            assert!(g.off_graph_departures.is_zero(), "{label}");
            assert!(
                g.node(&key("b")).unwrap().unmeasured_departures.is_zero(),
                "{label}"
            );
        }
    }
}
