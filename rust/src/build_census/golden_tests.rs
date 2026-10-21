//! Golden tables for the build census core (plan Phase 1). Every row states
//! its inputs; the incident fixture's numbers are the plan's: seven
//! interactive builds holding about 140 GiB, CI slice PSI 0, the builds'
//! slice stalled.

use super::*;

fn a(kind: AncestorKind) -> Ancestor {
    Ancestor { kind, dated: true }
}

fn undated(kind: AncestorKind) -> Ancestor {
    Ancestor { kind, dated: false }
}

/// A person's build in a shell under sshd, with a tty, unplaced, dated.
fn human_base() -> OwnerInputs {
    OwnerInputs {
        ancestry: Ancestry::Readable(vec![
            a(AncestorKind::Build),
            a(AncestorKind::Shell),
            a(AncestorKind::Sshd),
        ]),
        self_dated: true,
        nested_under_agent: Some(false),
        owner_record_session: None,
        tty: Reading::Measured(Some("pts/3".to_owned())),
        ci_runner: None,
        placement: Placement::Slice(SliceKind::Other),
        age_s: Some(120),
        orphan_bound_s: DEFAULT_ORPHAN_BOUND_S,
    }
}

fn unknown(reason: UnknownReason) -> OwnerClass {
    OwnerClass::Unknown { reason }
}

fn human() -> OwnerClass {
    OwnerClass::InteractiveHuman {
        tty: "pts/3".to_owned(),
    }
}

// ---- owner classification ------------------------------------------------

#[test]
fn owner_golden_table() {
    type Edit = fn(&mut OwnerInputs);
    let rows: Vec<(&str, Edit, OwnerClass)> = vec![
        (
            "interactive human: shell+sshd ancestry with a tty",
            |_| {},
            human(),
        ),
        (
            "interactive human: shim-scope build with a tty",
            |i| {
                i.ancestry = Ancestry::Readable(vec![a(AncestorKind::Other)]);
                i.placement = Placement::Slice(SliceKind::UserBuilds);
            },
            human(),
        ),
        (
            "tmux ancestor with a tty is a human",
            |i| i.ancestry = Ancestry::Readable(vec![a(AncestorKind::Tmux)]),
            human(),
        ),
        (
            "agent session: owner record names the session",
            |i| i.owner_record_session = Some("s-1".to_owned()),
            OwnerClass::AgentSession {
                session_id: Some("s-1".to_owned()),
            },
        ),
        (
            "agent session: agent-builds slice, no record",
            |i| i.placement = Placement::Slice(SliceKind::AgentBuilds),
            OwnerClass::AgentSession { session_id: None },
        ),
        (
            "agent session: census says nested under an agent",
            |i| i.nested_under_agent = Some(true),
            OwnerClass::AgentSession { session_id: None },
        ),
        (
            "agent session: nearest non-build ancestor is an agent",
            |i| {
                i.ancestry = Ancestry::Readable(vec![
                    a(AncestorKind::BuildHop),
                    a(AncestorKind::Agent),
                    a(AncestorKind::Shell),
                ])
            },
            OwnerClass::AgentSession { session_id: None },
        ),
        (
            "ci job: CI-slice build",
            |i| {
                i.placement = Placement::Slice(SliceKind::Ci);
                i.ci_runner = Some("runner-7".to_owned());
                i.tty = Reading::Measured(None);
            },
            OwnerClass::CiJob {
                runner: "runner-7".to_owned(),
            },
        ),
        (
            "ci slice without a runner name is unknown",
            |i| i.placement = Placement::Slice(SliceKind::Ci),
            unknown(UnknownReason::CiRunnerUnnamed),
        ),
        (
            "orphan: init-parented and older than the bound",
            |i| {
                i.ancestry =
                    Ancestry::Readable(vec![a(AncestorKind::Build), a(AncestorKind::Init)]);
                i.age_s = Some(DEFAULT_ORPHAN_BOUND_S + 1);
                i.tty = Reading::Measured(None);
            },
            OwnerClass::Orphan {
                owner_dead_since: DEFAULT_ORPHAN_BOUND_S + 1,
            },
        ),
        (
            "init-parented exactly at the bound is not yet an orphan",
            |i| {
                i.ancestry = Ancestry::Readable(vec![a(AncestorKind::Init)]);
                i.age_s = Some(DEFAULT_ORPHAN_BOUND_S);
                i.tty = Reading::Measured(None);
            },
            unknown(UnknownReason::InitParentedWithinBound),
        ),
        (
            "init-parented with no readable age is unknown",
            |i| {
                i.ancestry = Ancestry::Readable(vec![a(AncestorKind::Init)]);
                i.age_s = None;
                i.tty = Reading::Measured(None);
            },
            unknown(UnknownReason::AgeUnreadable),
        ),
        (
            "ide check: nearest non-build ancestor is rust-analyzer, not a human build",
            |i| {
                i.ancestry = Ancestry::Readable(vec![
                    a(AncestorKind::Build),
                    a(AncestorKind::BuildHop),
                    a(AncestorKind::Ide),
                    a(AncestorKind::Shell),
                ])
            },
            unknown(UnknownReason::IdeCheck),
        ),
        (
            "undated deciding ancestor is unknown, never human",
            |i| {
                i.ancestry =
                    Ancestry::Readable(vec![undated(AncestorKind::Shell), a(AncestorKind::Sshd)])
            },
            unknown(UnknownReason::UndatedAncestor),
        ),
        (
            "undated ide ancestor is unknown (not proof of an IDE either)",
            |i| i.ancestry = Ancestry::Readable(vec![undated(AncestorKind::Ide)]),
            unknown(UnknownReason::UndatedAncestor),
        ),
        (
            "undated build is unknown",
            |i| i.self_dated = false,
            unknown(UnknownReason::UndatedAncestor),
        ),
        (
            "undated shim-scope build is still unknown",
            |i| {
                i.self_dated = false;
                i.placement = Placement::Slice(SliceKind::UserBuilds);
            },
            unknown(UnknownReason::UndatedAncestor),
        ),
        (
            "unreadable ancestry is unknown, never human",
            |i| i.ancestry = Ancestry::Unreadable,
            unknown(UnknownReason::AncestryUnreadable),
        ),
        (
            "unreadable cgroup and no other evidence is unknown",
            |i| {
                i.ancestry = Ancestry::Readable(vec![a(AncestorKind::Other)]);
                i.placement = Placement::Unreadable;
            },
            unknown(UnknownReason::NoOwnerEvidence),
        ),
        (
            "shell ancestry without a tty is a script, not a person",
            |i| i.tty = Reading::Measured(None),
            unknown(UnknownReason::NoTty),
        ),
        (
            "tty without any human ancestry or shim scope is unknown",
            |i| i.ancestry = Ancestry::Readable(vec![a(AncestorKind::Other)]),
            unknown(UnknownReason::NoOwnerEvidence),
        ),
        (
            "agent above a shell with a tty is the agent build, not a human one",
            |i| {
                i.ancestry = Ancestry::Readable(vec![
                    a(AncestorKind::Other),
                    a(AncestorKind::Agent),
                    a(AncestorKind::Shell),
                ])
            },
            OwnerClass::AgentSession { session_id: None },
        ),
        (
            "undated agent above a shell is unknown, never human",
            |i| {
                i.ancestry = Ancestry::Readable(vec![
                    a(AncestorKind::Other),
                    undated(AncestorKind::Agent),
                    a(AncestorKind::Shell),
                ])
            },
            unknown(UnknownReason::UndatedAncestor),
        ),
        (
            "runner-spawned build with a shell above it is unknown, never human",
            |i| {
                i.ancestry = Ancestry::Readable(vec![
                    a(AncestorKind::Other),
                    a(AncestorKind::Runner),
                    a(AncestorKind::Shell),
                ])
            },
            unknown(UnknownReason::RunnerSpawned),
        ),
        (
            "undated runner above a shell is unknown",
            |i| {
                i.ancestry = Ancestry::Readable(vec![
                    a(AncestorKind::Other),
                    undated(AncestorKind::Runner),
                    a(AncestorKind::Shell),
                ])
            },
            unknown(UnknownReason::UndatedAncestor),
        ),
        (
            "nested_under_agent could not say: unknown, never human",
            |i| i.nested_under_agent = None,
            unknown(UnknownReason::NestingUnreadable),
        ),
        (
            "detached (init-parented) build with a tty past the bound is not an orphan",
            |i| {
                i.ancestry = Ancestry::Readable(vec![a(AncestorKind::Init)]);
                i.age_s = Some(DEFAULT_ORPHAN_BOUND_S + 1);
            },
            unknown(UnknownReason::InitParentedWithTty),
        ),
        (
            "detached shim-scope build without a tty past the bound is not an orphan",
            |i| {
                i.ancestry = Ancestry::Readable(vec![a(AncestorKind::Init)]);
                i.age_s = Some(DEFAULT_ORPHAN_BOUND_S + 1);
                i.tty = Reading::Measured(None);
                i.placement = Placement::Slice(SliceKind::UserBuilds);
            },
            unknown(UnknownReason::InitParentedWithTty),
        ),
        (
            "init-parented, no tty, no shell, unreadable cgroup past the bound is not an orphan",
            |i| {
                i.ancestry = Ancestry::Readable(vec![a(AncestorKind::Init)]);
                i.age_s = Some(DEFAULT_ORPHAN_BOUND_S + 1);
                i.tty = Reading::Measured(None);
                i.placement = Placement::Unreadable;
            },
            unknown(UnknownReason::PlacementUnreadable),
        ),
        (
            "unreadable tty probe on an otherwise orphan-shaped build is unknown",
            |i| {
                i.ancestry = Ancestry::Readable(vec![a(AncestorKind::Init)]);
                i.age_s = Some(DEFAULT_ORPHAN_BOUND_S + 1);
                i.tty = Reading::Unknown;
            },
            unknown(UnknownReason::TtyUnreadable),
        ),
        (
            "unreadable tty probe under a shell is not a human",
            |i| i.tty = Reading::Unknown,
            unknown(UnknownReason::TtyUnreadable),
        ),
        (
            "shell with a tty but an unreadable cgroup is not a human",
            |i| i.placement = Placement::Unreadable,
            unknown(UnknownReason::PlacementUnreadable),
        ),
        (
            "empty ancestry is unknown",
            |i| i.ancestry = Ancestry::Readable(vec![]),
            unknown(UnknownReason::NoOwnerEvidence),
        ),
    ];
    for (name, edit, want) in rows {
        let mut i = human_base();
        edit(&mut i);
        assert_eq!(classify_owner(&i), want, "row: {name}");
    }
}

#[test]
fn every_owner_class_has_a_row_and_a_kind() {
    let kinds: Vec<OwnerKind> = [
        human(),
        OwnerClass::AgentSession { session_id: None },
        OwnerClass::CiJob {
            runner: "r".to_owned(),
        },
        OwnerClass::Orphan {
            owner_dead_since: 1,
        },
        unknown(UnknownReason::NoOwnerEvidence),
    ]
    .iter()
    .map(OwnerClass::kind)
    .collect();
    for k in OwnerKind::ALL {
        assert!(kinds.contains(&k), "{k:?} has no kind mapping");
    }
}

// ---- rollup ---------------------------------------------------------------

fn row(owner: OwnerClass, gib: u64, placed: bool) -> BuildRow {
    BuildRow {
        owner,
        anon_rss_bytes: Reading::Measured(gib * GIB),
        placed,
    }
}

#[test]
fn rollup_counts_every_class_and_splits_placed() {
    let builds = vec![
        row(human(), 20, false),
        row(human(), 10, true),
        row(OwnerClass::AgentSession { session_id: None }, 5, true),
        row(unknown(UnknownReason::NoTty), 7, false),
    ];
    let r = class_rollup(&builds);
    assert_eq!(r.by_class.len(), OwnerKind::ALL.len());
    let h = r.class(OwnerKind::InteractiveHuman);
    assert_eq!((h.count, h.placed_count, h.unplaced_count), (2, 1, 1));
    assert_eq!(h.anon.measured, 30 * GIB);
    assert_eq!(h.unplaced_anon.measured, 20 * GIB);
    assert_eq!(h.placed_anon.measured, 10 * GIB);
    assert_eq!(r.class(OwnerKind::CiJob), ClassTotals::default());
    // The unknown-owner build is in its class AND in the total, at its RSS.
    assert_eq!(r.class(OwnerKind::Unknown).anon.measured, 7 * GIB);
    assert_eq!(r.total.count, 4);
    assert_eq!(r.total.anon.measured, 42 * GIB);
    assert!(!r.total.anon.is_lower_bound());
}

#[test]
fn unreadable_rss_is_an_explicit_unknown_never_zero() {
    let builds = vec![
        row(human(), 10, true),
        BuildRow {
            owner: unknown(UnknownReason::AncestryUnreadable),
            anon_rss_bytes: Reading::Unknown,
            placed: false,
        },
    ];
    let r = class_rollup(&builds);
    let u = r.class(OwnerKind::Unknown);
    assert_eq!(u.count, 1);
    assert_eq!(u.anon.measured, 0);
    assert_eq!(u.anon.unknown_members, 1);
    assert!(u.anon.is_lower_bound());
    assert!(r.total.anon.is_lower_bound());
    assert_eq!(r.total.anon.measured, 10 * GIB);
}

#[test]
fn rollup_of_nothing_is_zero_builds_in_every_class() {
    let r = class_rollup(&[]);
    assert_eq!(r.total, ClassTotals::default());
    assert_eq!(r.by_class.len(), OwnerKind::ALL.len());
}

// ---- capacity cause -------------------------------------------------------

fn m(v: f64) -> Reading<f64> {
    Reading::Measured(v)
}

/// A host whose non-build memory is small and whose slices are calm.
fn calm_pressure() -> SlicePressure {
    SlicePressure {
        builds_psi: m(0.0),
        ci_psi: m(0.0),
        sessions_psi: m(0.0),
        other_psi: m(0.0),
        sessions_anon_bytes: Reading::Measured(20 * GIB),
        other_anon_bytes: Reading::Measured(10 * GIB),
    }
}

/// Seven interactive builds totalling 140 GiB.
fn incident_builds() -> Vec<BuildRow> {
    (0..7)
        .map(|i| {
            row(
                OwnerClass::InteractiveHuman {
                    tty: format!("{i}"),
                },
                20,
                i % 2 == 0,
            )
        })
        .collect()
}

#[test]
fn cause_golden_table() {
    type Edit = fn(&mut SlicePressure);
    let rows: Vec<(&str, Vec<BuildRow>, Edit, Cause)> = vec![
        (
            "INCIDENT: 7 interactive builds ~140 GiB, CI PSI 0, builds slice stalled",
            incident_builds(),
            |p| p.builds_psi = m(42.0),
            Cause::Attributed(CauseClass::Builds),
        ),
        (
            "builds dominate but their slice is calm: no positive evidence, mixed",
            incident_builds(),
            |_| {},
            Cause::Mixed,
        ),
        (
            "builds dominate, builds slice stalled exactly at the threshold: not stalled",
            incident_builds(),
            |p| p.builds_psi = m(STALL_PSI_SOME_AVG60),
            Cause::Mixed,
        ),
        (
            "a class at EXACTLY the dominance share and stalled is not dominant: mixed",
            vec![row(human(), 60, true)],
            |p| {
                p.sessions_anon_bytes = Reading::Measured(30 * GIB);
                p.other_anon_bytes = Reading::Measured(10 * GIB);
                p.builds_psi = m(42.0);
            },
            Cause::Mixed,
        ),
        (
            "NaN PSI of the dominant class is unreadable: unknown, not mixed",
            incident_builds(),
            |p| p.builds_psi = m(f64::NAN),
            Cause::Unknown,
        ),
        (
            "infinite PSI of a class holding memory is unreadable: unknown",
            incident_builds(),
            |p| p.sessions_psi = m(f64::INFINITY),
            Cause::Unknown,
        ),
        (
            "PSI above 100 percent is unreadable: unknown",
            incident_builds(),
            |p| p.builds_psi = m(150.0),
            Cause::Unknown,
        ),
        (
            "negative PSI is unreadable: unknown",
            incident_builds(),
            |p| p.builds_psi = m(-1.0),
            Cause::Unknown,
        ),
        (
            "builds stalled but not dominant (builds 20 of 50 GiB): mixed",
            vec![row(human(), 20, true)],
            |p| p.builds_psi = m(80.0),
            Cause::Mixed,
        ),
        (
            "ci dominates and its slice stalls",
            vec![row(
                OwnerClass::CiJob {
                    runner: "r".to_owned(),
                },
                120,
                true,
            )],
            |p| p.ci_psi = m(35.0),
            Cause::Attributed(CauseClass::Ci),
        ),
        (
            "sessions dominate and stall",
            vec![row(human(), 5, true)],
            |p| {
                p.sessions_anon_bytes = Reading::Measured(200 * GIB);
                p.sessions_psi = m(30.0);
            },
            Cause::Attributed(CauseClass::Sessions),
        ),
        (
            "other dominates and stalls",
            vec![row(human(), 5, true)],
            |p| {
                p.other_anon_bytes = Reading::Measured(300 * GIB);
                p.other_psi = m(30.0);
            },
            Cause::Attributed(CauseClass::Other),
        ),
        (
            "unknown-owner builds count at full RSS toward builds",
            vec![row(unknown(UnknownReason::NoTty), 140, false)],
            |p| p.builds_psi = m(42.0),
            Cause::Attributed(CauseClass::Builds),
        ),
        (
            "unreadable cgroup pressure for the dominant class: unknown, not other",
            incident_builds(),
            |p| p.builds_psi = Reading::Unknown,
            Cause::Unknown,
        ),
        (
            "unreadable CI pressure while CI holds memory: unknown",
            {
                let mut b = incident_builds();
                b.push(row(
                    OwnerClass::CiJob {
                        runner: "r".to_owned(),
                    },
                    1,
                    true,
                ));
                b
            },
            |p| {
                p.builds_psi = m(42.0);
                p.ci_psi = Reading::Unknown;
            },
            Cause::Unknown,
        ),
        (
            "unreadable pressure of a class holding no memory does not withhold",
            incident_builds(),
            |p| {
                p.builds_psi = m(42.0);
                p.ci_psi = Reading::Unknown;
            },
            Cause::Attributed(CauseClass::Builds),
        ),
        (
            "unreadable non-build anon total: unknown",
            incident_builds(),
            |p| {
                p.builds_psi = m(42.0);
                p.other_anon_bytes = Reading::Unknown;
            },
            Cause::Unknown,
        ),
        (
            "nothing to attribute: unknown",
            vec![],
            |p| {
                p.sessions_anon_bytes = Reading::Measured(0);
                p.other_anon_bytes = Reading::Measured(0);
            },
            Cause::Unknown,
        ),
    ];
    for (name, builds, edit, want) in rows {
        let mut p = calm_pressure();
        edit(&mut p);
        assert_eq!(
            capacity_cause(&class_rollup(&builds), &p),
            want,
            "row: {name}"
        );
    }
}

#[test]
fn unreadable_build_rss_withholds_the_cause() {
    let mut builds = incident_builds();
    builds.push(BuildRow {
        owner: unknown(UnknownReason::AncestryUnreadable),
        anon_rss_bytes: Reading::Unknown,
        placed: false,
    });
    let mut p = calm_pressure();
    p.builds_psi = m(42.0);
    assert_eq!(capacity_cause(&class_rollup(&builds), &p), Cause::Unknown);
}

#[test]
fn incident_fixture_totals_about_140_gib() {
    let r = class_rollup(&incident_builds());
    assert_eq!(r.total.count, 7);
    assert_eq!(r.total.anon.measured, 140 * GIB);
}

#[test]
fn verdicts_round_trip_through_serde() {
    for c in [
        Cause::Attributed(CauseClass::Builds),
        Cause::Mixed,
        Cause::Unknown,
    ] {
        let back: Cause = serde_json::from_value(serde_json::to_value(c).unwrap()).unwrap();
        assert_eq!(back, c);
    }
    assert_eq!(
        serde_json::to_value(Cause::Attributed(CauseClass::Builds)).unwrap()["verdict"],
        "attributed"
    );
    let o = OwnerClass::Orphan {
        owner_dead_since: 9,
    };
    let v = serde_json::to_value(&o).unwrap();
    assert_eq!(v["class"], "orphan");
    assert_eq!(serde_json::from_value::<OwnerClass>(v).unwrap(), o);
}

// ---- the literal lint -------------------------------------------------------

/// No hostname, uid or path literal in a rule: every string literal in the
/// module's non-test code must be a serde attribute value.
#[test]
fn no_host_literals_in_rules() {
    const SOURCES: &[(&str, &str)] = &[
        ("mod.rs", include_str!("mod.rs")),
        ("types.rs", include_str!("types.rs")),
        ("owner.rs", include_str!("owner.rs")),
        ("rollup.rs", include_str!("rollup.rs")),
        ("cause.rs", include_str!("cause.rs")),
    ];
    const ALLOWED: &[&str] = &[
        "snake_case",
        "provenance",
        "value",
        "class",
        "readable",
        "ancestors",
        "placement",
        "slice",
        "verdict",
    ];
    // Every `pub mod` in mod.rs is scanned: a new module file cannot slip past.
    let declared: Vec<String> = include_str!("mod.rs")
        .lines()
        .filter_map(|l| {
            let l = l.trim();
            l.strip_prefix("pub mod ")
                .or_else(|| l.strip_prefix("mod "))
        })
        .filter(|m| m.ends_with(';') && *m != "golden_tests;")
        .map(|m| format!("{}.rs", m.trim_end_matches(';')))
        .collect();
    for m in &declared {
        assert!(
            SOURCES.iter().any(|(n, _)| n == m),
            "{m} is a module of build_census but is not scanned"
        );
    }
    for (name, src) in SOURCES {
        let code = non_test_code(src).unwrap_or_else(|e| panic!("{name}: {e}"));
        let ints = long_integer_literals(&code);
        assert!(
            ints.is_empty(),
            "{name}: integer literal(s) {ints:?} of 3+ digits in rule code — a uid or \
             port belongs in a measurement, never in a rule"
        );
        for lit in string_literals(&code) {
            assert!(
                ALLOWED.contains(&lit.as_str()),
                "{name}: string literal {lit:?} in rule code — hostnames, uids and \
                 paths belong in measurements, never in a rule"
            );
        }
    }
    // The scanners are not vacuous: they see a literal and ignore comments.
    assert_eq!(
        string_literals("let h = \"/home/x\"; // \"in a comment\"\n/// \"doc\"\nlet x = 1;"),
        vec!["/home/x".to_owned()]
    );
    assert_eq!(
        long_integer_literals(
            "let uid = 1000; let p = 8080_u16; let ok = 1 << 30; // 4096\nlet f = 0.600; let n = x123; \"9999\""
        ),
        vec!["1000".to_owned(), "8080_u16".to_owned()]
    );
    // The test-module cut hides nothing: code after the test block is refused.
    assert!(non_test_code("let a = 1;\n#[cfg(test)]\nmod tests { fn t() {} }\n").is_ok());
    assert!(
        non_test_code("let a = 1;\n#[cfg(test)]\nmod tests { fn t() {} }\nlet uid = 1000;\n")
            .is_err()
    );
}

/// `src` up to its trailing `#[cfg(test)] mod tests` block, if any. Errs when
/// anything but whitespace follows that block, so the cut cannot hide rule code.
fn non_test_code(src: &str) -> Result<String, String> {
    let src = src.replace("\r\n", "\n");
    let Some(i) = src.rfind("#[cfg(test)]\nmod tests") else {
        return Ok(src);
    };
    let tail = &src[i..];
    let mut depth = 0usize;
    let mut end = None;
    for (off, c) in tail.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    end = Some(off + 1);
                    break;
                }
            }
            _ => {}
        }
    }
    match end {
        Some(e) if tail[e..].trim().is_empty() => Ok(src[..i].to_owned()),
        _ => Err("code follows (or the block never closes) after the test module".to_owned()),
    }
}

/// Integer literals of three or more digits in `code` (suffix and underscores
/// included), skipping comments, string literals and float literals.
fn long_integer_literals(code: &str) -> Vec<String> {
    let chars: Vec<char> = code.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let attached =
            i > 0 && (chars[i - 1].is_alphanumeric() || chars[i - 1] == '_' || chars[i - 1] == '.');
        if c == '/' && chars.get(i + 1) == Some(&'/') {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
        } else if c == '"' {
            i += 1;
            while i < chars.len() && chars[i] != '"' {
                i += if chars[i] == '\\' { 2 } else { 1 };
            }
            i += 1;
        } else if c.is_ascii_digit() && !attached {
            let start = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let tok: String = chars[start..i].iter().collect();
            let is_float =
                chars.get(i) == Some(&'.') && chars.get(i + 1).is_some_and(|d| d.is_ascii_digit());
            let digits = tok
                .chars()
                .take_while(|d| d.is_ascii_digit() || *d == '_')
                .filter(char::is_ascii_digit)
                .count();
            if !is_float && digits >= 3 {
                out.push(tok);
            }
        } else {
            i += 1;
        }
    }
    out
}

/// String literals in `code`, skipping `//` comments (incl. doc comments).
fn string_literals(code: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_str = false;
    let mut cur = String::new();
    let mut chars = code.chars().peekable();
    let mut prev = '\0';
    while let Some(c) = chars.next() {
        if !in_str {
            if c == '/' && chars.peek() == Some(&'/') {
                for n in chars.by_ref() {
                    if n == '\n' {
                        break;
                    }
                }
                prev = '\n';
                continue;
            }
            if c == '"' && prev != '\'' {
                in_str = true;
                cur.clear();
            }
        } else if c == '\\' {
            if let Some(n) = chars.next() {
                cur.push(if n == 'n' { '\n' } else { n });
            }
        } else if c == '"' {
            in_str = false;
            out.push(cur.clone());
        } else {
            cur.push(c);
        }
        prev = c;
    }
    out
}
