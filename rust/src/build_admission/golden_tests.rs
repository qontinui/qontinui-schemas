//! Golden tests for the admission core (plan Phase 1). Every fixture states
//! its inputs; the incident fixture's numbers are the measurements recorded in
//! the plan's "The incident" table (merytshost, 2026-10-06 ~03:06Z).

use super::admit::{admit, Admission, AdmitBasis, Blocking, JobsSource, UnknownInput};
use super::degraded::{degraded_slots, DegradedInputs};
use super::order::{effective_class, order, schedule, HoldReason as SchedHold, Schedule};
use super::overload::{overload_step, GovernorAction, GovernorState, HoldReason, UnleasedTree};
use super::types::*;

const G: u64 = GIB;

fn gib(v: f64) -> u64 {
    (v * G as f64) as u64
}

/// The incident host: 368 GiB / 48 cpus, MemAvailable 50 GiB, memory PSI full
/// avg10 53 %, CI reservation 148.5 GiB (`ci-runners.slice` MemoryHigh),
/// non-build p95 60 GiB, and 218.5 GiB of unleased build RSS (248.5 GiB of
/// rustc/clippy RSS measured, minus the 30 GiB the two leases below account for).
fn incident_facts() -> HostFacts {
    HostFacts {
        mem_total_bytes: 368 * G,
        cpus: 48,
        mem_available_bytes: Fact::Measured(50 * G),
        psi_mem_full_avg10: Fact::Measured(53.0),
        non_build_p95_bytes: Fact::Measured(60 * G),
        ci_reservation_bytes: Fact::Measured(gib(148.5)),
        unleased_build_rss_bytes: Fact::Measured(gib(218.5)),
        admissions_paused: false,
    }
}

/// The same host, calm: no pressure, no unleased trees, 300 GiB available.
fn calm_facts() -> HostFacts {
    HostFacts {
        mem_available_bytes: Fact::Measured(300 * G),
        psi_mem_full_avg10: Fact::Measured(0.0),
        unleased_build_rss_bytes: Fact::Measured(0),
        ..incident_facts()
    }
}

fn est(gib_v: u64) -> Estimate {
    Estimate {
        bytes: Some(gib_v * G),
        duration_p50_s: None,
        duration_p90_s: None,
        source: EstimateSource::Seed,
    }
}

fn ticket(id: &str, dir: &str, class: Class, queued_at_s: u64, e: Estimate) -> Ticket {
    Ticket {
        id: id.to_owned(),
        class,
        output_dir: dir.to_owned(),
        requested_jobs: None,
        queued_at_s,
        est: e,
    }
}

fn lease(id: &str, dir: &str, gib_v: u64, started_at_s: u64) -> Lease {
    Lease {
        id: id.to_owned(),
        class: Class::Agent,
        output_dir: dir.to_owned(),
        state: LeaseState::Running,
        est_bytes: Some(gib_v * G),
        expected_duration_s: None,
        started_at_s,
        paused_at_s: None,
        current_anon_bytes: None,
    }
}

/// Two coord builds running into DIFFERENT output directories (the shared
/// target and one private target), 15 GiB estimated each.
fn incident_leases() -> Vec<Lease> {
    vec![
        lease("coord-shared", "coord/target/debug", 15, 0),
        lease("coord-private-1", "scratch-1/target/debug", 15, 60),
    ]
}

fn waits(a: &Admission) -> &[Blocking] {
    match a {
        Admission::Wait { blocking } => blocking,
        Admission::Admit { .. } => panic!("expected Wait, got {a:?}"),
    }
}

// ---- the incident -------------------------------------------------------

#[test]
fn incident_third_coord_build_in_a_third_directory_is_refused() {
    let t = ticket(
        "coord-3",
        "scratch-2/target/debug",
        Class::Agent,
        100,
        est(15),
    );
    let a = admit(
        &t,
        &incident_leases(),
        &incident_facts(),
        &Policy::default(),
    );
    let b = waits(&a);
    assert!(
        b.iter().any(|x| matches!(
            x,
            Blocking::Reservation {
                budget_bytes: 0,
                ..
            }
        )),
        "the unleased herd leaves no budget: {b:?}"
    );
    assert!(
        b.iter().any(|x| matches!(x, Blocking::Pressure { .. })),
        "53 % PSI full refuses: {b:?}"
    );
    assert!(
        !b.iter().any(|x| matches!(x, Blocking::Target { .. })),
        "a third directory is not a target conflict: {b:?}"
    );
}

#[test]
fn incident_build_into_a_held_directory_is_refused_by_target_regardless_of_memory() {
    let t = ticket("coord-3", "coord/target/debug", Class::Agent, 100, est(15));
    let a = admit(&t, &incident_leases(), &calm_facts(), &Policy::default());
    assert_eq!(
        waits(&a),
        &[Blocking::Target {
            lease_id: "coord-shared".to_owned()
        }]
    );
}

#[test]
fn incident_host_when_calm_admits_the_third_build() {
    // What binds in the incident is the herd and the pressure, not the budget:
    // with neither, budget = 368 − 11.04 − 60 − 148.5 ≈ 148.5 GiB and 45 GiB
    // reserved fits.
    let t = ticket(
        "coord-3",
        "scratch-2/target/debug",
        Class::Agent,
        100,
        est(15),
    );
    let a = admit(&t, &incident_leases(), &calm_facts(), &Policy::default());
    assert!(a.is_admit(), "{a:?}");
}

// ---- lone build, unknown, not_supported ---------------------------------

#[test]
fn a_lone_build_gets_the_whole_machine() {
    let t = ticket("only", "coord/target/debug", Class::Agent, 0, est(15));
    let a = admit(&t, &[], &calm_facts(), &Policy::default());
    assert_eq!(
        a,
        Admission::Admit {
            jobs: 48,
            jobs_source: JobsSource::Share,
            basis: AdmitBasis::Fits
        }
    );
}

#[test]
fn contention_shrinks_new_grants() {
    let leases = vec![
        lease("a", "a", 15, 0),
        lease("b", "b", 15, 0),
        lease("c", "c", 15, 0),
    ];
    let t = ticket("d", "d", Class::Agent, 0, est(15));
    match admit(&t, &leases, &calm_facts(), &Policy::default()) {
        // 4 concurrent on 48 cpus → at most 12 jobs.
        Admission::Admit { jobs, .. } => assert_eq!(jobs, 12),
        other => panic!("{other:?}"),
    }
}

#[test]
fn an_explicit_caller_jobs_is_kept() {
    let mut t = ticket("only", "x", Class::Agent, 0, est(15));
    t.requested_jobs = Some(3);
    assert_eq!(
        admit(&t, &[], &calm_facts(), &Policy::default()),
        Admission::Admit {
            jobs: 3,
            jobs_source: JobsSource::Caller,
            basis: AdmitBasis::Fits
        }
    );
}

#[test]
fn unknown_psi_with_a_running_lease_withholds() {
    let mut f = calm_facts();
    f.psi_mem_full_avg10 = Fact::Unknown;
    let t = ticket("t", "y", Class::Agent, 0, est(15));
    let a = admit(&t, &[lease("a", "x", 15, 0)], &f, &Policy::default());
    assert_eq!(
        waits(&a),
        &[Blocking::Unknown {
            inputs: vec![UnknownInput::PsiMemFull]
        }]
    );
}

#[test]
fn unknown_with_nothing_running_admits_one_on_the_progress_floor() {
    let mut f = calm_facts();
    f.psi_mem_full_avg10 = Fact::Unknown;
    f.mem_available_bytes = Fact::Measured(50 * G);
    let t = ticket("t", "y", Class::Agent, 0, Estimate::UNKNOWN);
    // Live memory 50 − reserve 11.04 ≈ 38.96 GiB → 12 jobs.
    assert_eq!(
        admit(&t, &[], &f, &Policy::default()),
        Admission::Admit {
            jobs: 12,
            jobs_source: JobsSource::LiveMemory,
            basis: AdmitBasis::ProgressFloor
        }
    );
}

#[test]
fn the_progress_floor_never_overrides_the_controls() {
    let mut f = calm_facts();
    f.psi_mem_full_avg10 = Fact::Unknown;
    f.admissions_paused = true;
    let t = ticket("t", "y", Class::Agent, 0, est(15));
    let a = admit(&t, &[], &f, &Policy::default());
    assert!(waits(&a).contains(&Blocking::Control));
}

#[test]
fn an_idle_host_floors_only_an_estimate_that_can_never_fit() {
    // 200 GiB exceeds the 148.46 GiB budget, but the budget can grow (CI,
    // non-build use and unleased trees all move): it waits, so an idle host
    // never builds into CI's reservation.
    let t = ticket("big", "y", Class::Agent, 0, est(200));
    let a = admit(&t, &[], &calm_facts(), &Policy::default());
    assert!(matches!(waits(&a), [Blocking::Reservation { .. }]), "{a:?}");
    // 400 GiB exceeds MemTotal − reserve (356.96 GiB): it could never fit, so
    // refusing it would stall the queue forever. It runs alone.
    let t = ticket("huge", "y", Class::Agent, 0, est(400));
    match admit(&t, &[], &calm_facts(), &Policy::default()) {
        Admission::Admit { basis, .. } => assert_eq!(basis, AdmitBasis::ProgressFloor),
        other => panic!("{other:?}"),
    }
    // ...nor into a host already below the reserve floor...
    let mut low = calm_facts();
    low.mem_available_bytes = Fact::Measured(5 * G);
    assert!(!admit(&t, &[], &low, &Policy::default()).is_admit());
    // ...and never under pressure.
    let mut f = calm_facts();
    f.psi_mem_full_avg10 = Fact::Measured(50.0);
    assert!(!admit(&t, &[], &f, &Policy::default()).is_admit());
}

#[test]
fn a_non_finite_psi_reading_is_unknown_not_calm() {
    let mut f = calm_facts();
    f.psi_mem_full_avg10 = Fact::Measured(f64::NAN);
    let t = ticket("t", "y", Class::Agent, 0, est(15));
    let a = admit(&t, &[lease("a", "x", 15, 0)], &f, &Policy::default());
    assert_eq!(
        waits(&a),
        &[Blocking::Unknown {
            inputs: vec![UnknownInput::PsiMemFull]
        }]
    );
}

#[test]
fn not_supported_psi_drops_the_conjunct_and_admits_by_memory() {
    let mut f = calm_facts();
    f.psi_mem_full_avg10 = Fact::NotSupported;
    let t = ticket("t", "y", Class::Agent, 0, est(15));
    let leases = [lease("a", "x", 15, 0)];
    assert!(admit(&t, &leases, &f, &Policy::default()).is_admit());
    // ...and memory still refuses: 20 GiB available, 15 + 11.04 needed.
    f.mem_available_bytes = Fact::Measured(20 * G);
    let a = admit(&t, &leases, &f, &Policy::default());
    assert!(matches!(waits(&a), [Blocking::Live { .. }]), "{a:?}");
}

// ---- starvation: promotion, backfill, paused holders --------------------

#[test]
fn promotion_fires_at_promote_after() {
    let p = Policy::default();
    let queue = vec![
        ticket("agent-old", "a", Class::Agent, 0, est(1)),
        ticket("merge-new", "b", Class::Merge, 100, est(1)),
    ];
    let first = |now| queue[order(&queue, now, &p)[0]].id.clone();
    assert_eq!(first(p.promote_after_s - 1), "merge-new");
    assert_eq!(first(p.promote_after_s), "agent-old");
    // One step per period: background reaches merge after two periods.
    let bg = ticket("bg", "c", Class::Background, 0, est(1));
    assert_eq!(effective_class(&bg, p.promote_after_s, &p), Class::Agent);
    assert_eq!(
        effective_class(&bg, 2 * p.promote_after_s, &p),
        Class::Merge
    );
    assert_eq!(
        effective_class(&bg, 50 * p.promote_after_s, &p),
        Class::Merge
    );
    // Promotion never reaches operator.
    assert_eq!(Class::Merge.promoted(), Class::Merge);
    assert_eq!(Class::Background.promoted(), Class::Agent);
}

/// Backfill fixture on the calm host (budget 368 − 11.04 − 60 − 148.5 =
/// 148.46 GiB). Two running leases of 60 GiB end at t=1000 and t=2000, so
/// 28.46 GiB is free now. The head needs 70 GiB: it fits once `a` ends, so its
/// shadow time is t=1000 and the extra at that moment is 88.46 − 70 = 18.46 GiB.
fn backfill_fixture() -> (Vec<Lease>, Ticket) {
    let mut a = lease("a", "a", 60, 0);
    a.expected_duration_s = Some(1000);
    let mut b = lease("b", "b", 60, 0);
    b.expected_duration_s = Some(2000);
    let head = ticket("head", "h", Class::Agent, 0, est(70));
    (vec![a, b], head)
}

fn cand(id: &str, gib_v: u64, duration_s: Option<u64>) -> Ticket {
    let mut t = ticket(id, id, Class::Agent, 10, est(gib_v));
    t.est.duration_p90_s = duration_s;
    t
}

fn started(s: &Schedule) -> Option<&str> {
    match s {
        Schedule::Start {
            ticket_id,
            backfilled: true,
            ..
        } => Some(ticket_id),
        _ => None,
    }
}

#[test]
fn backfill_never_delays_the_head() {
    let p = Policy::default();
    let (leases, head) = backfill_fixture();
    let f = calm_facts();
    let now = 50;
    let run =
        |c: Ticket, now: u64, leases: &[Lease]| schedule(&[head.clone(), c], leases, &f, now, &p);

    // Does not fit now (120 + 29 > 148.46): nothing starts.
    assert_eq!(
        started(&run(cand("big", 29, Some(100)), now, &leases)),
        None
    );
    // Fits now (25 ≤ 28.46) but would still hold 25 GiB at t=1000, when only
    // 18.46 GiB is spare: it would delay the head, so it waits.
    assert_eq!(
        started(&run(cand("long", 25, Some(5000)), now, &leases)),
        None
    );
    // ...and an unknown duration is never assumed short.
    assert_eq!(started(&run(cand("unknown", 25, None), now, &leases)), None);
    // Ends at 50 + 900 = 950, before the shadow: it may start.
    assert_eq!(
        started(&run(cand("early", 25, Some(900)), now, &leases)),
        Some("early")
    );
    // A candidate is judged by its SLOW tail: a median of 900 s does not
    // qualify when its p90 runs past the shadow.
    let mut median_short = cand("median-short", 25, Some(5000));
    median_short.est.duration_p50_s = Some(900);
    assert_eq!(started(&run(median_short, now, &leases)), None);
    // Fits in the 18.46 GiB extra: it may start however long it runs.
    assert_eq!(
        started(&run(cand("small", 18, None), now, &leases)),
        Some("small")
    );

    // Once the head has waited promote_after, backfill stops entirely.
    let late = p.promote_after_s;
    assert_eq!(
        started(&run(cand("early", 25, Some(900)), late, &leases)),
        None
    );

    // A running lease with no expected end makes the shadow unknowable.
    let mut blind = leases.clone();
    blind[0].expected_duration_s = None;
    blind[1].expected_duration_s = None;
    assert_eq!(started(&run(cand("small", 18, None), now, &blind)), None);
}

#[test]
fn a_ticket_whose_directory_a_paused_lease_holds_is_skipped() {
    let p = Policy::default();
    let mut paused = lease("p", "held", 10, 0);
    paused.state = LeaseState::PausedByCommand;
    paused.paused_at_s = Some(5);
    let queue = vec![
        ticket("blocked", "held", Class::Merge, 0, est(5)),
        ticket("next", "free", Class::Agent, 10, est(5)),
    ];
    let s = schedule(&queue, &[paused.clone()], &calm_facts(), 20, &p);
    assert!(
        matches!(s, Schedule::Start { ref ticket_id, backfilled: false, .. } if ticket_id == "next"),
        "{s:?}"
    );
    // A governor-paused lease holds the whole queue: it resumes first.
    paused.state = LeaseState::PausedByGovernor;
    let s = schedule(&queue, &[paused], &calm_facts(), 20, &p);
    assert_eq!(
        s,
        Schedule::Hold {
            head: None,
            reason: SchedHold::PausedLeaseFirst
        }
    );
}

// ---- the overload ladder ------------------------------------------------

fn pressured(psi: f64) -> HostFacts {
    HostFacts {
        psi_mem_full_avg10: Fact::Measured(psi),
        ..calm_facts()
    }
}

#[test]
fn pause_waits_for_sustained_pressure_then_freezes_the_newest_lowest_class() {
    let p = Policy::default();
    let mut op = lease("op", "o", 10, 300);
    op.class = Class::Operator;
    let mut merge = lease("merge", "m", 10, 200);
    merge.class = Class::Merge;
    let leases = vec![
        lease("old-agent", "a", 10, 0),
        lease("new-agent", "b", 10, 100),
        merge,
        op,
    ];
    let (s1, a1) = overload_step(
        GovernorState::default(),
        &leases,
        &[],
        &pressured(30.0),
        1000,
        &p,
    );
    assert_eq!(
        a1,
        GovernorAction::Hold {
            reason: HoldReason::Calm
        }
    );
    assert_eq!(s1.pressure_since_s, Some(1000));
    let (s2, a2) = overload_step(s1, &leases, &[], &pressured(30.0), 1010, &p);
    assert_eq!(
        a2,
        GovernorAction::PauseLease {
            lease_id: "new-agent".to_owned()
        }
    );
    // Settle: nothing for settle_s.
    let (_, a3) = overload_step(s2, &leases, &[], &pressured(30.0), 1020, &p);
    assert_eq!(
        a3,
        GovernorAction::Hold {
            reason: HoldReason::Settling
        }
    );
}

#[test]
fn operator_is_never_paused_and_unleased_trees_go_last() {
    let p = Policy::default();
    let mut op = lease("op", "o", 10, 300);
    op.class = Class::Operator;
    let trees = vec![
        UnleasedTree {
            id: "t-old".into(),
            started_at_s: 0,
            paused_at_s: None,
        },
        UnleasedTree {
            id: "t-new".into(),
            started_at_s: 50,
            paused_at_s: None,
        },
    ];
    let mut f = calm_facts();
    f.mem_available_bytes = Fact::Measured(5 * G); // below the 11.04 GiB floor
    let (_, a) = overload_step(GovernorState::default(), &[op], &trees, &f, 10, &p);
    assert_eq!(
        a,
        GovernorAction::PauseUnleased {
            tree_id: "t-new".into()
        }
    );
}

#[test]
fn resume_uses_est_minus_current_anon_oldest_paused_first() {
    let p = Policy::default();
    // Reserve floor on 368 GiB = 11.04 GiB.
    let mut older = lease("older", "a", 20, 0);
    older.state = LeaseState::PausedByGovernor;
    older.paused_at_s = Some(100);
    older.current_anon_bytes = Some(15 * G); // needs max(0, 20 − 15) = 5 GiB more
    let mut younger = lease("younger", "b", 1, 0);
    younger.state = LeaseState::PausedByGovernor;
    younger.paused_at_s = Some(200);
    let leases = vec![younger, older];
    let at = |avail: f64| {
        let mut f = calm_facts();
        f.mem_available_bytes = Fact::Measured(gib(avail));
        overload_step(GovernorState::default(), &leases, &[], &f, 1000, &p).1
    };
    assert_eq!(
        at(16.1),
        GovernorAction::ResumeLease {
            lease_id: "older".into()
        }
    );
    // 16.0 < 11.04 + 5: not yet — and the younger one does not jump the queue.
    assert_eq!(
        at(16.0),
        GovernorAction::Hold {
            reason: HoldReason::NotYetResumable
        }
    );

    // Pages already resident beyond the estimate need nothing more.
    let mut over = lease("over", "c", 20, 0);
    over.state = LeaseState::PausedByGovernor;
    over.paused_at_s = Some(1);
    over.current_anon_bytes = Some(25 * G);
    let mut f = calm_facts();
    f.mem_available_bytes = Fact::Measured(gib(11.1));
    let (_, a) = overload_step(GovernorState::default(), &[over], &[], &f, 1000, &p);
    assert_eq!(
        a,
        GovernorAction::ResumeLease {
            lease_id: "over".into()
        }
    );
}

#[test]
fn a_lone_paused_lease_with_no_estimate_still_resumes() {
    let p = Policy::default();
    let mut floor = lease("floor", "a", 0, 0);
    floor.est_bytes = None; // admitted on the progress floor
    floor.state = LeaseState::PausedByGovernor;
    floor.paused_at_s = Some(1);
    let mut f = calm_facts();
    f.mem_available_bytes = Fact::Measured(gib(11.1)); // just above the 11.04 floor
    let (_, a) = overload_step(GovernorState::default(), &[floor.clone()], &[], &f, 100, &p);
    assert_eq!(
        a,
        GovernorAction::ResumeLease {
            lease_id: "floor".into()
        }
    );
    // With another build running its need is unknowable: it waits.
    let others = [floor, lease("r", "b", 5, 0)];
    let (_, a) = overload_step(GovernorState::default(), &others, &[], &f, 100, &p);
    assert_eq!(
        a,
        GovernorAction::Hold {
            reason: HoldReason::UnknownInput
        }
    );
}

#[test]
fn hysteresis_and_unknown_inputs_never_move_anything() {
    let p = Policy::default();
    let mut paused = lease("p", "a", 5, 0);
    paused.state = LeaseState::PausedByGovernor;
    paused.paused_at_s = Some(1);
    let leases = [paused, lease("r", "b", 5, 0)];
    // Between psi_resume (5) and psi_pause (20): hold.
    let (_, a) = overload_step(
        GovernorState::default(),
        &leases,
        &[],
        &pressured(10.0),
        100,
        &p,
    );
    assert_eq!(
        a,
        GovernorAction::Hold {
            reason: HoldReason::NotYetResumable
        }
    );
    // Unknown PSI and unknown memory: never overloaded, never resumed.
    let mut f = calm_facts();
    f.psi_mem_full_avg10 = Fact::Unknown;
    f.mem_available_bytes = Fact::Unknown;
    let (s, a) = overload_step(GovernorState::default(), &leases, &[], &f, 100, &p);
    assert_eq!(
        a,
        GovernorAction::Hold {
            reason: HoldReason::UnknownInput
        }
    );
    assert_eq!(s.pressure_since_s, None);
    // A command-paused lease is never resumed by the governor.
    let mut cmd = lease("cmd", "c", 5, 0);
    cmd.state = LeaseState::PausedByCommand;
    cmd.paused_at_s = Some(1);
    let (_, a) = overload_step(
        GovernorState::default(),
        &[cmd],
        &[],
        &calm_facts(),
        100,
        &p,
    );
    assert_eq!(
        a,
        GovernorAction::Hold {
            reason: HoldReason::Calm
        }
    );
}

// ---- degraded arm -------------------------------------------------------

#[test]
fn degraded_n_on_incident_inputs_is_nine() {
    let inputs = DegradedInputs {
        mem_total_bytes: Some(368 * G),
        non_build_p95_bytes: Some(60 * G),
        ci_reservation_bytes: Some(gib(148.5)),
        max_est_bytes: Some(15 * G),
    };
    // (368 − 11.04 − 60 − 148.5) / 15 ≈ 9.9 → 9.
    assert_eq!(degraded_slots(&inputs, &Policy::default()), 9);
}

#[test]
fn degraded_n_is_one_with_any_unreadable_term() {
    let full = DegradedInputs {
        mem_total_bytes: Some(368 * G),
        non_build_p95_bytes: Some(60 * G),
        ci_reservation_bytes: Some(gib(148.5)),
        max_est_bytes: Some(15 * G),
    };
    for missing in 0..4 {
        let mut i = full;
        match missing {
            0 => i.mem_total_bytes = None,
            1 => i.non_build_p95_bytes = None,
            2 => i.ci_reservation_bytes = None,
            _ => i.max_est_bytes = None,
        }
        assert_eq!(degraded_slots(&i, &Policy::default()), 1, "term {missing}");
    }
    // An over-subscribed host still gets one slot, never zero.
    let mut i = full;
    i.ci_reservation_bytes = Some(400 * G);
    assert_eq!(degraded_slots(&i, &Policy::default()), 1);
}

// ---- policy and the literal lint ----------------------------------------

#[test]
fn default_policy_is_valid_and_resume_must_be_below_pause() {
    assert_eq!(Policy::default().validate(), Ok(()));
    let bad = Policy {
        psi_resume: 30.0,
        ..Policy::default()
    };
    assert_eq!(bad.validate(), Err(PolicyError::ResumeNotBelowPause));
    let bad = Policy {
        psi_admit_max: 25.0,
        ..Policy::default()
    };
    assert_eq!(bad.validate(), Err(PolicyError::AdmitAbovePause));
    let bad = Policy {
        min_measurements: 21,
        ..Policy::default()
    };
    assert_eq!(bad.validate(), Err(PolicyError::MinMeasurementsAboveWindow));
}

#[test]
fn facts_round_trip_with_provenance() {
    let f = incident_facts();
    let json = serde_json::to_value(f).unwrap();
    assert_eq!(json["psi_mem_full_avg10"]["provenance"], "measured");
    assert_eq!(
        serde_json::to_value(Fact::<u64>::NotSupported).unwrap()["provenance"],
        "not_supported"
    );
    let back: HostFacts = serde_json::from_value(json).unwrap();
    assert_eq!(back, f);
}

/// No hostname or path literal in a rule or a default: every string literal in
/// the module's non-test code must be a serde attribute value. (Numbers are not
/// scanned: every numeric default is a named `Policy` field, and a host-specific
/// number such as a uid has no field to live in.)
#[test]
fn no_host_literals_in_rules() {
    const SOURCES: &[(&str, &str)] = &[
        ("mod.rs", include_str!("mod.rs")),
        ("types.rs", include_str!("types.rs")),
        ("jobs.rs", include_str!("jobs.rs")),
        ("estimate.rs", include_str!("estimate.rs")),
        ("budget.rs", include_str!("budget.rs")),
        ("admit.rs", include_str!("admit.rs")),
        ("order.rs", include_str!("order.rs")),
        ("overload.rs", include_str!("overload.rs")),
        ("degraded.rs", include_str!("degraded.rs")),
    ];
    const ALLOWED: &[&str] = &[
        "snake_case",
        "provenance",
        "value",
        "source",
        "state",
        "check",
        "decision",
        "action",
        "reason",
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
            "{m} is a module of build_admission but is not scanned"
        );
    }
    for (name, src) in SOURCES {
        for lit in string_literals(&non_test_code(src)) {
            assert!(
                ALLOWED.contains(&lit.as_str()),
                "{name}: string literal {lit:?} in rule code — hostnames and paths \
                 belong in measurements or Policy, never in a rule"
            );
        }
    }
    // The scanner is not vacuous: it sees a literal, including one that spans
    // lines, and ignores comments.
    assert_eq!(
        string_literals("let h = \"merytshost\"; // \"in a comment\"\n/// \"doc\"\nlet x = 1;"),
        vec!["merytshost".to_owned()]
    );
    assert_eq!(
        string_literals("let h = \"a\nmerytshost\";"),
        vec!["a\nmerytshost".to_owned()]
    );
    // ...and the test-module cut is the LAST one, so a mention earlier hides nothing.
    assert_eq!(
        string_literals(&non_test_code(
            "//! #[cfg(test)]\r\nlet h = \"x\";\r\n#[cfg(test)]\r\nmod tests { \"y\" }"
        )),
        vec!["x".to_owned()]
    );
}

/// `src` up to its trailing `#[cfg(test)] mod tests` block, if any.
fn non_test_code(src: &str) -> String {
    let src = src.replace("\r\n", "\n");
    match src.rfind("#[cfg(test)]\nmod tests") {
        Some(i) => src[..i].to_owned(),
        None => src,
    }
}

/// String literals in `code`, skipping `//` comments (incl. doc comments).
/// String state carries across lines, so a multi-line literal is seen whole.
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
                cur.push(match n {
                    'n' => '\n',
                    o => o,
                });
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
