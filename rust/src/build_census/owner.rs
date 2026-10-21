//! [`classify_owner`]: the one rule for who a build is for (D1).

use super::types::*;

/// Default orphan bound: an init-parented build older than six hours has
/// outlived any verification a launcher was waiting on.
pub const DEFAULT_ORPHAN_BOUND_S: u64 = 6 * 60 * 60;

/// Classify one build's owner from the facts a census can read.
///
/// Rules, in order; the first that applies decides:
/// 1. An owner record naming a session, or a build in the agent slice:
///    [`OwnerClass::AgentSession`].
/// 2. A build in a CI slice: [`OwnerClass::CiJob`] (`Unknown` if the runner
///    is unnamed).
/// 3. The census's `nested_under_agent` flag: `AgentSession`.
/// 4. An unreadable ancestry: `Unknown`.
/// 5. An undated build or undated nearest non-build ancestor is `Unknown` (a
///    reused pid cannot be ruled out). Then the WHOLE chain is scanned: an
///    agent anywhere above is `AgentSession`, the runner is `Unknown`, and an
///    undated one of either is `Unknown`.
/// 6. The NEAREST non-build ancestor being an IDE checker is `Unknown`
///    ([`UnknownReason::IdeCheck`], never a human build); an unreadable
///    `nested_under_agent` is `Unknown`.
/// 7. An init parent is an `Orphan` only with no tty, no human evidence, a
///    readable placement and once older than the bound; otherwise `Unknown`.
/// 8. Unreadable tty probe: `Unknown`. Otherwise a controlling tty plus a shim-scope placement or a
///    shell / sshd / tmux ancestor is `InteractiveHuman`; anything less is
///    `Unknown`.
pub fn classify_owner(inputs: &OwnerInputs) -> OwnerClass {
    let unknown = |reason| OwnerClass::Unknown { reason };

    if let Some(id) = &inputs.owner_record_session {
        return OwnerClass::AgentSession {
            session_id: Some(id.clone()),
        };
    }
    match inputs.placement {
        Placement::Slice(SliceKind::AgentBuilds) => {
            return OwnerClass::AgentSession { session_id: None };
        }
        Placement::Slice(SliceKind::Ci) => {
            return match &inputs.ci_runner {
                Some(runner) => OwnerClass::CiJob {
                    runner: runner.clone(),
                },
                // Deliberate: unnamed runner stays Unknown (never a guessed CiJob).
                None => unknown(UnknownReason::CiRunnerUnnamed),
            };
        }
        _ => {}
    }
    if inputs.nested_under_agent == Some(true) {
        return OwnerClass::AgentSession { session_id: None };
    }

    let chain = match &inputs.ancestry {
        Ancestry::Unreadable => return unknown(UnknownReason::AncestryUnreadable),
        Ancestry::Readable(chain) => chain,
    };

    let decider = chain
        .iter()
        .find(|a| !matches!(a.kind, AncestorKind::Build | AncestorKind::BuildHop));
    if !inputs.self_dated || decider.is_some_and(|d| !d.dated) {
        return unknown(UnknownReason::UndatedAncestor);
    }
    // The WHOLE chain is scanned for a launcher before any human evidence is
    // weighed: an agent or the runner anywhere above owns the build, however
    // many shells sit between.
    // Deliberate: an agent above a runner or above an undated ancestor is
    // `Unknown`, not `AgentSession` — never Human/Orphan on partial evidence.
    if let Some(launcher) = chain
        .iter()
        .find(|a| matches!(a.kind, AncestorKind::Agent | AncestorKind::Runner))
    {
        return match (launcher.dated, launcher.kind) {
            (false, _) => unknown(UnknownReason::UndatedAncestor),
            (true, AncestorKind::Agent) => OwnerClass::AgentSession { session_id: None },
            (true, _) => unknown(UnknownReason::RunnerSpawned),
        };
    }
    if decider.is_some_and(|d| d.kind == AncestorKind::Ide) {
        return unknown(UnknownReason::IdeCheck);
    }
    // "Could not say" whether an agent is above is not "no agent".
    if inputs.nested_under_agent.is_none() {
        return unknown(UnknownReason::NestingUnreadable);
    }

    // A failed tty probe is not "no tty": neither Orphan nor human without it.
    let tty = match &inputs.tty {
        Reading::Unknown => return unknown(UnknownReason::TtyUnreadable),
        Reading::Measured(tty) => tty,
    };

    let human_evidence = inputs.placement == Placement::Slice(SliceKind::UserBuilds)
        || chain.iter().any(|a| {
            a.dated
                && matches!(
                    a.kind,
                    AncestorKind::Shell | AncestorKind::Sshd | AncestorKind::Tmux
                )
        });
    if decider.is_some_and(|d| d.kind == AncestorKind::Init) {
        // A detached (nohup / setsid) build a person is still attached to is
        // not an orphan: Orphan needs no tty and no human evidence.
        if tty.is_some() || human_evidence {
            return unknown(UnknownReason::InitParentedWithTty);
        }
        // The unread cgroup could be the agent or user slice: no confident Orphan.
        if inputs.placement == Placement::Unreadable {
            return unknown(UnknownReason::PlacementUnreadable);
        }
        return match inputs.age_s {
            None => unknown(UnknownReason::AgeUnreadable),
            Some(age) if age > inputs.orphan_bound_s => OwnerClass::Orphan {
                owner_dead_since: age,
            },
            Some(_) => unknown(UnknownReason::InitParentedWithinBound),
        };
    }

    match (tty, human_evidence) {
        (Some(_), true) if inputs.placement == Placement::Unreadable => {
            unknown(UnknownReason::PlacementUnreadable)
        }
        (Some(tty), true) => OwnerClass::InteractiveHuman { tty: tty.clone() },
        (None, true) => unknown(UnknownReason::NoTty),
        _ => unknown(UnknownReason::NoOwnerEvidence),
    }
}
