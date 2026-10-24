use super::*;
use crate::journey::ChokePoint;
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
        !s.frontier.is_settled(&s.coverage),
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
        s.frontier.unmeasured(&s.coverage),
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
    assert!(!r.frontier.is_settled(&r.coverage));

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
    assert!(s.frontier.is_settled(&s.coverage));

    // Closing it with NO edge to C settles the negative: now it is absent.
    let g = build_graph(&ledger, &[], &writing(), &no_filter());
    let s = steps_to(&g, &key("a"), &key("c"));
    assert_eq!(s.steps, None);
    assert!(s.frontier.is_settled(&s.coverage));
    assert!(s.frontier.unmeasured(&s.coverage).is_empty());
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
    assert!(!s.frontier.is_settled(&s.coverage));
    assert_eq!(s.frontier.unobserved_departures, 1);
    assert_eq!(s.frontier.nodes[0].node_key, key("b"));
    assert_eq!(
        s.frontier.unmeasured(&s.coverage),
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
    assert!(!r.frontier.is_settled(&r.coverage));
    assert_eq!(
        r.frontier.unmeasured(&r.coverage),
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
        e("5", "b", "d").outcome(EdgeOutcome::Error),          // (errored click still lands)
        e("6", "a", "f").kind(NavigationTriggerKind::Replace), // f: URL + back
        e("7", "b", "f").kind(NavigationTriggerKind::Pop),
    ]);
    let g = build_graph(&ledger, &[], &writing(), &no_filter());
    let p = requires_prior_knowledge(&g);
    assert_eq!(
        p.nodes,
        vec![JourneyPriorKnowledgeNode {
            node_key: key("c"),
            arrivals: BTreeMap::from([("initial".to_string(), 1), ("push".to_string(), 1)]),
        }]
    );
    // The errored b -> d click is an unmeasured departure from b.
    assert_eq!(
        p.frontier.unmeasured(&p.coverage),
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
            s.frontier.is_settled(&s.coverage),
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
        !s.frontier.is_settled(&s.coverage),
        "{dimension}: must not settle"
    );
    let dims = s.frontier.unmeasured(&s.coverage);
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
        !r.frontier.is_settled(&r.coverage),
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
    assert_eq!(g.node(&key("b")).unwrap().unmeasured_departures.rejected, 1);
    assert_c_unknown_via(&g, "rejected_departures", "producer_failed");

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
        s.frontier.unmeasured(&s.coverage),
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

    // A settle_timeout that stayed on the node is not a departure at all.
    let ledger = a_b_c(e("2", "b", "b").outcome(EdgeOutcome::SettleTimeout));
    let g = build_graph(&ledger, &[], &writing(), &no_filter());
    assert!(g.node(&key("b")).unwrap().unmeasured_departures.is_zero());
    let d = dead_ends(&g);
    assert!(d.dead_ends.iter().any(|x| x.node_key == key("b")));
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
    assert_eq!(
        p.nodes
            .iter()
            .map(|n| n.node_key.as_str())
            .collect::<Vec<_>>(),
        [key("n")]
    );
    // The filtered back-arrival could disqualify n: the listing is not settled.
    assert_eq!(g.off_graph_departures.excluded_back_arrivals, 1);
    assert_eq!(
        p.frontier.unmeasured(&p.coverage),
        [("excluded_back_arrivals", 1, "input_missing")]
    );
    assert_eq!(
        p.frontier.unsettled_because(&p.coverage, true),
        Some("input_missing")
    );
    // A NAVIGATION question ignores it: a pop is not a navigation step.
    let r = reachable_from(&g, &key("z"));
    assert!(r.frontier.is_settled(&r.coverage), "{:?}", r.frontier);
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
    assert!(d.frontier.unmeasured(&d.coverage).contains(&(
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
    assert!(!s.frontier.is_settled(&s.coverage));
    assert_eq!(
        s.frontier.unmeasured(&s.coverage),
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
        assert!(!s.frontier.is_settled(&s.coverage), "{code}");
        assert_eq!(s.frontier.unmeasured(&s.coverage), [("ledger", 1, code)]);
        assert_eq!(s.frontier.unsettled_because(&s.coverage, true), Some(code));
        let g = build_graph(&[], &[], &ledger_in(state), &no_filter());
        let d = dead_ends(&g);
        assert_eq!(
            d.frontier.unmeasured(&d.coverage),
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
            f.unmeasured(&cov),
            [("filtered_departures", 1, "input_missing")],
            "{name}"
        );
    }
    // A component question about a and b cannot reach x.
    let r = reachable_from(&g, &key("a"));
    assert!(r.frontier.is_settled(&r.coverage));
    assert!(r.frontier.off_graph_departures.is_zero());
}
