// @generated from glossary/terms.toml by rust/tests/glossary_source.rs — do not edit.
// Regenerate: QONTINUI_GLOSSARY_REGENERATE=1 cargo test -p qontinui-types --test glossary_source

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::GlossaryEntry;

/// The glossary's `version` (glossary/terms.toml).
pub const GLOSSARY_VERSION: u32 = 1;
/// SHA-256 of the LF-normalised glossary/terms.toml this table was generated from.
pub const GLOSSARY_CONTENT_SHA256: &str = "818f5c7d95356e67bed52f0643d772b41d755a0056d37398dd0c7bf501f79b97";

/// A term the product glossary defines, by its stable snake_case id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
pub enum GlossaryTerm {
    #[doc = "A watched record of work that stopped because it is waiting on something observable; it resumes by itself when the condition clears."]
    #[serde(rename = "gate")]
    Gate,
    #[doc = "The action a gate performs when it clears: start an agent session, queue a pull request to land, or just notify."]
    #[serde(rename = "continuation")]
    Continuation,
    #[doc = "A declaration by an agent or a person that a gate's condition is met, which clears an approval gate."]
    #[serde(rename = "attestation")]
    Attestation,
    #[doc = "The durable record of one piece of work: a slug with a status, an owner, the pull requests that implement it, and its dependencies."]
    #[serde(rename = "work_unit")]
    WorkUnit,
    #[doc = "A written work document whose status is mirrored into a work unit and whose body is kept, versioned, in the plan library."]
    #[serde(rename = "plan")]
    Plan,
    #[doc = "A short-lived reservation over something an agent is about to change, so other agents see it and do not collide."]
    #[serde(rename = "claim")]
    Claim,
    #[doc = "A level in one of three ladders: coordination (claims, work units, plans), the autonomy dial, or a policy clause's permission level."]
    #[serde(rename = "tier")]
    Tier,
    #[doc = "The service that lands pull requests: it rebases each onto the latest main, runs CI, keeps overlapping changes apart and lands them in order."]
    #[serde(rename = "merge_train")]
    MergeTrain,
    #[doc = "The merge train's read-only answer for one pull request: its state, what blocks it, and the next action."]
    #[serde(rename = "merge_verdict")]
    MergeVerdict,
    #[doc = "A pull request's commits are on main. The code host may show it as Closed or as Merged; both can mean landed."]
    #[serde(rename = "landed")]
    Landed,
    #[doc = "A short investigation result one session posts for others; it expires after about two weeks."]
    #[serde(rename = "finding")]
    Finding,
    #[doc = "A durable, topic-keyed file for a problem that keeps recurring, collecting its occurrences across months."]
    #[serde(rename = "dossier")]
    Dossier,
    #[doc = "A versioned document of rules for how agents act in this tenant, made of individually cited clauses."]
    #[serde(rename = "policy")]
    Policy,
    #[doc = "One rule inside a policy, with a stable id, a status and a tier, that a session cites when it applies it."]
    #[serde(rename = "clause")]
    Clause,
    #[doc = "A document saying what this tenant is building, for whom, and what better means; it steers which work is chosen next."]
    #[serde(rename = "intent_document")]
    IntentDocument,
    #[doc = "Either stopping new work being sent to a device (reversible), or preparing one runner for a planned restart (final)."]
    #[serde(rename = "drain")]
    Drain,
    #[doc = "The runner's answer to whether restarting it now would lose work, with each kind of live session counted separately."]
    #[serde(rename = "restart_readiness")]
    RestartReadiness,
    #[doc = "One running AI agent conversation, tracked with a liveness state and a separate work status."]
    #[serde(rename = "agent_session")]
    AgentSession,
    #[doc = "Two separate axes: liveness (active, stale, closed) and work (working, blocked, finished). Finished is not closed."]
    #[serde(rename = "session_status")]
    SessionStatus,
    #[doc = "Where the runner actually found a capability on this machine: built in, installed, fetched, cached, a developer checkout, or not found."]
    #[serde(rename = "rung")]
    Rung,
    #[doc = "The isolation boundary that owns repositories, devices, gates, work units, policies and memory. Shown as a Project in the app."]
    #[serde(rename = "tenant")]
    Tenant,
    #[doc = "A registered machine running the runner. Pairing links it to your account and issues the credential it uses."]
    #[serde(rename = "device")]
    Device,
    #[doc = "The desktop application on a paired device that hosts agent sessions and automations and talks to the service for you."]
    #[serde(rename = "runner")]
    Runner,
    #[doc = "The coordination service: claims, work units, gates, sessions, devices, tenants, policies and the merge train."]
    #[serde(rename = "coord")]
    Coord,
    #[doc = "Asking the service for an agent's own isolated working copy on a reserved branch, so parallel agents never share one."]
    #[serde(rename = "worktree_allocation")]
    WorktreeAllocation,
    #[doc = "The person who owns the tenant: approves operator gates, drains devices and writes the intent documents."]
    #[serde(rename = "operator")]
    Operator,
    #[doc = "A first-class answer meaning the product could not observe the value. It is never shown as zero, none, idle or a default."]
    #[serde(rename = "unknown")]
    Unknown,
}

impl GlossaryTerm {
    /// Every term, in glossary order.
    pub const ALL: &'static [GlossaryTerm] = &[
        GlossaryTerm::Gate,
        GlossaryTerm::Continuation,
        GlossaryTerm::Attestation,
        GlossaryTerm::WorkUnit,
        GlossaryTerm::Plan,
        GlossaryTerm::Claim,
        GlossaryTerm::Tier,
        GlossaryTerm::MergeTrain,
        GlossaryTerm::MergeVerdict,
        GlossaryTerm::Landed,
        GlossaryTerm::Finding,
        GlossaryTerm::Dossier,
        GlossaryTerm::Policy,
        GlossaryTerm::Clause,
        GlossaryTerm::IntentDocument,
        GlossaryTerm::Drain,
        GlossaryTerm::RestartReadiness,
        GlossaryTerm::AgentSession,
        GlossaryTerm::SessionStatus,
        GlossaryTerm::Rung,
        GlossaryTerm::Tenant,
        GlossaryTerm::Device,
        GlossaryTerm::Runner,
        GlossaryTerm::Coord,
        GlossaryTerm::WorktreeAllocation,
        GlossaryTerm::Operator,
        GlossaryTerm::Unknown,
    ];

    /// The stable id (the wire value).
    pub const fn as_str(self) -> &'static str {
        match self {
            GlossaryTerm::Gate => "gate",
            GlossaryTerm::Continuation => "continuation",
            GlossaryTerm::Attestation => "attestation",
            GlossaryTerm::WorkUnit => "work_unit",
            GlossaryTerm::Plan => "plan",
            GlossaryTerm::Claim => "claim",
            GlossaryTerm::Tier => "tier",
            GlossaryTerm::MergeTrain => "merge_train",
            GlossaryTerm::MergeVerdict => "merge_verdict",
            GlossaryTerm::Landed => "landed",
            GlossaryTerm::Finding => "finding",
            GlossaryTerm::Dossier => "dossier",
            GlossaryTerm::Policy => "policy",
            GlossaryTerm::Clause => "clause",
            GlossaryTerm::IntentDocument => "intent_document",
            GlossaryTerm::Drain => "drain",
            GlossaryTerm::RestartReadiness => "restart_readiness",
            GlossaryTerm::AgentSession => "agent_session",
            GlossaryTerm::SessionStatus => "session_status",
            GlossaryTerm::Rung => "rung",
            GlossaryTerm::Tenant => "tenant",
            GlossaryTerm::Device => "device",
            GlossaryTerm::Runner => "runner",
            GlossaryTerm::Coord => "coord",
            GlossaryTerm::WorktreeAllocation => "worktree_allocation",
            GlossaryTerm::Operator => "operator",
            GlossaryTerm::Unknown => "unknown",
        }
    }
}

/// Every definition, in [`GlossaryTerm::ALL`] order.
pub static GLOSSARY: &[GlossaryEntry] = &[
    GlossaryEntry {
        id: GlossaryTerm::Gate,
        term: "Gate",
        short: "A watched record of work that stopped because it is waiting on something observable; it resumes by itself when the condition clears.",
        long: "A **gate** turns \"I am blocked\" into something the product watches. It has a typed predicate (a pull request landed, CI went green, a deploy is healthy, a claim ended, an operator approved, a time elapsed, and more) that is re-checked on a schedule.\n\nVerdicts: **open**, **cleared**, **failed** (the event happened the wrong way), **misconfigured** (can never be evaluated) and **withdrawn**. When the evaluator cannot answer, the gate stays open with a reason rather than guessing.\n\nVerbs: *register* creates a gate, *attest* clears an approval gate, *withdraw* retires it with a reason. A gate can carry a **continuation** that runs when it clears.",
        see_also: &[GlossaryTerm::Continuation, GlossaryTerm::Attestation, GlossaryTerm::WorkUnit, GlossaryTerm::Unknown],
        since: 1,
    },
    GlossaryEntry {
        id: GlossaryTerm::Continuation,
        term: "Continuation",
        short: "The action a gate performs when it clears: start an agent session, queue a pull request to land, or just notify.",
        long: "A **continuation** is attached to a gate and runs when the gate clears, so blocked work picks itself back up without anyone remembering to restart it.\n\nActions: *run a skill* (start an agent session with arguments), *merge a pull request* (queue it on the merge train), or *notify only*. Deploys and migrations only ever notify.\n\nDelivery: by default a new agent session is started; it can instead be delivered into a live session that still holds the context, falling back to a new session when none is live. The product records when a continuation was dispatched and when it was picked up, so one that never started is detected as stalled.",
        see_also: &[GlossaryTerm::Gate, GlossaryTerm::AgentSession, GlossaryTerm::MergeTrain],
        since: 1,
    },
    GlossaryEntry {
        id: GlossaryTerm::Attestation,
        term: "Attestation",
        short: "A declaration by an agent or a person that a gate's condition is met, which clears an approval gate.",
        long: "An **attestation** clears a gate whose predicate is an approval rather than something the product can observe by itself.\n\nWho may attest is decided per gate class by its clearance authority: *operator only*, *an agent other than the author* (separation of duties), or *any agent* in the tenant. Gates addressed to agents can by default be attested by any agent; gates addressed to the operator need a human approval.\n\nForce-clearing a gate that can no longer clear on its own is a separate action, checked against the same authority.",
        see_also: &[GlossaryTerm::Gate, GlossaryTerm::Operator, GlossaryTerm::Tenant],
        since: 1,
    },
    GlossaryEntry {
        id: GlossaryTerm::WorkUnit,
        term: "Work unit",
        short: "The durable record of one piece of work: a slug with a status, an owner, the pull requests that implement it, and its dependencies.",
        long: "A **work unit** is how the product tracks a piece of work over its whole life, independent of any file. It carries a status, an owner, **citations** (the pull requests that implement it), dependency edges and free-form metadata.\n\nEvery status change is kept in its history, and gates can wait on a work unit reaching a status. The status is free text chosen by whoever writes it, so read it as that writer's label rather than a fixed state machine.\n\nWhen plans are enabled, each plan's status is mirrored into a work unit.",
        see_also: &[GlossaryTerm::Plan, GlossaryTerm::Gate, GlossaryTerm::Tier, GlossaryTerm::Claim],
        since: 1,
    },
    GlossaryEntry {
        id: GlossaryTerm::Plan,
        term: "Plan",
        short: "A written work document whose status is mirrored into a work unit and whose body is kept, versioned, in the plan library.",
        long: "A **plan** is a markdown document describing a piece of work: why, the phases, and how to tell it is done. Its operational state (status, citations, dependencies) lives in a work unit; its body lives in the **plan library**, which keeps every version.\n\nPlans are optional. A tenant may author entirely through the web app and keep no plan files on disk.\n\nThe library can lag the files it mirrors, so a plan missing from a library read is UNKNOWN until the library's own health signal says the read is current.",
        see_also: &[GlossaryTerm::WorkUnit, GlossaryTerm::Unknown, GlossaryTerm::Tier],
        since: 1,
    },
    GlossaryEntry {
        id: GlossaryTerm::Claim,
        term: "Claim",
        short: "A short-lived reservation over something an agent is about to change, so other agents see it and do not collide.",
        long: "A **claim** says \"I am working on this right now\". Kinds include a worktree, a file glob, a single symbol (one function or type), a branch name, a plan phase and a database migration revision.\n\nClaims are kept alive by heartbeats and expire on their own, so a session that dies does not hold its claims forever. A conflicting request is answered with who holds the claim, never by silently taking it over.\n\nClaims are the lightest coordination tier; work units and plans build on them.",
        see_also: &[GlossaryTerm::Tier, GlossaryTerm::WorktreeAllocation, GlossaryTerm::AgentSession],
        since: 1,
    },
    GlossaryEntry {
        id: GlossaryTerm::Tier,
        term: "Tier",
        short: "A level in one of three ladders: coordination (claims, work units, plans), the autonomy dial, or a policy clause's permission level.",
        long: "**Tier** names a level in three separate ladders; read which one from context.\n\n1. **Coordination tiers** stack: *claims* (who is touching what, always on), then *work units* (the durable record of work), then *plan files* (optional; enabled by configuring a plans folder). A tenant with no plan files loses very little.\n2. **The autonomy dial** says how far agents may go on security-sensitive work: *proceed*, *draft required* (open a draft behind an approval gate) or *ask first*.\n3. **Clause tiers** mark what one policy clause permits: *proceed*, *proceed and log*, *proceed and notify*, *ask first* or *never*.",
        see_also: &[GlossaryTerm::Claim, GlossaryTerm::WorkUnit, GlossaryTerm::Plan, GlossaryTerm::Policy, GlossaryTerm::Clause],
        since: 1,
    },
    GlossaryEntry {
        id: GlossaryTerm::MergeTrain,
        term: "Merge train",
        short: "The service that lands pull requests: it rebases each onto the latest main, runs CI, keeps overlapping changes apart and lands them in order.",
        long: "The **merge train** is the only thing that lands pull requests on a managed repository. A green, non-draft pull request is queued with nothing further to do.\n\nFor each one it dry-rebases onto current main, detects file overlap with other queued changes (two that touch the same files are kept apart), runs CI, and lands by fast-forward push. The queue is first in, first out, with a label that moves a change into a *land next* band.\n\nOnly draft status, or a gate carrying a merge continuation, holds a pull request back. Its answer for one pull request is the **merge verdict**.",
        see_also: &[GlossaryTerm::MergeVerdict, GlossaryTerm::Landed, GlossaryTerm::Continuation],
        since: 1,
    },
    GlossaryEntry {
        id: GlossaryTerm::MergeVerdict,
        term: "Merge verdict",
        short: "The merge train's read-only answer for one pull request: its state, what blocks it, and the next action.",
        long: "The **merge verdict** answers \"what is the merge train doing with this pull request, and what should I do?\". It is computed when asked, from the train's own records.\n\nIt reports the state (queued, rebasing, awaiting CI, blocked by overlap, landing, merged, conflict, cancelled), the block reason and, for an escalation, its category and the gate it waits on, plus a table-driven next action.\n\nAn in-flight state that also carries an error describes the queue slot, not the outcome. Evidence of an earlier landing reads *landed*, *reverted upstream* or *unknown*; no evidence is unknown, never \"not landed\".",
        see_also: &[GlossaryTerm::MergeTrain, GlossaryTerm::Landed, GlossaryTerm::Unknown, GlossaryTerm::Gate],
        since: 1,
    },
    GlossaryEntry {
        id: GlossaryTerm::Landed,
        term: "Landed",
        short: "A pull request's commits are on main. The code host may show it as Closed or as Merged; both can mean landed.",
        long: "**Landed** means the change's commits are on the main branch.\n\nThe merge train lands by rebasing and fast-forwarding main. When the rebase rewrote the commits, the code host shows the pull request as **Closed**, not Merged; when it did not, it shows **Merged**. Neither label, nor a pull request still showing open, proves or disproves a landing on its own.\n\nTo know, read the merge verdict or check that the content is on main.",
        see_also: &[GlossaryTerm::MergeTrain, GlossaryTerm::MergeVerdict],
        since: 1,
    },
    GlossaryEntry {
        id: GlossaryTerm::Finding,
        term: "Finding",
        short: "A short investigation result one session posts for others; it expires after about two weeks.",
        long: "A **finding** sits between a private transcript and permanent memory: what one session learned that the next one working on the same thing should know first.\n\nFindings are scoped to a resource or topic and expire after about 14 days by default. Sessions read the recent findings before starting on something and post one after an investigation. Findings can be marked triaged once someone has routed them.\n\nA problem that keeps coming back outlives that window as a **dossier**.",
        see_also: &[GlossaryTerm::Dossier, GlossaryTerm::AgentSession],
        since: 1,
    },
    GlossaryEntry {
        id: GlossaryTerm::Dossier,
        term: "Dossier",
        short: "A durable, topic-keyed file for a problem that keeps recurring, collecting its occurrences across months.",
        long: "A **dossier** is a long-lived finding: one per topic, for a defect or question that recurs across many sessions. It collects each occurrence, the current understanding and what would close it.\n\nIt does not expire. Updates are written forward as a new head that supersedes the previous one; there is never a second head for the same topic.",
        see_also: &[GlossaryTerm::Finding],
        since: 1,
    },
    GlossaryEntry {
        id: GlossaryTerm::Policy,
        term: "Policy",
        short: "A versioned document of rules for how agents act in this tenant, made of individually cited clauses.",
        long: "A **policy** is a prompt document in the *behavior* family: it says how sessions act, not what to build. It is made of **clauses**, and its text is rebuilt from them.\n\nEvery edit creates a new, unchangeable version, so older versions can be restored. Sessions read the policies fresh rather than from memory, cite the clause and version they applied, and record a *policy gap* when no clause covers a decision.\n\nWhat the tenant is building is described separately, by **intent documents**.",
        see_also: &[GlossaryTerm::Clause, GlossaryTerm::IntentDocument, GlossaryTerm::Tier],
        since: 1,
    },
    GlossaryEntry {
        id: GlossaryTerm::Clause,
        term: "Clause",
        short: "One rule inside a policy, with a stable id, a status and a tier, that a session cites when it applies it.",
        long: "A **clause** is the unit of a policy. It has a stable kebab-case id, a status (*gap*, *proposed*, *confirmed*, *active* or *retired*), a tier saying what it permits, and trigger, action, bounds and escalate-if text.\n\nCiting a clause by id and policy version makes a decision auditable: a reader can open the exact rule that was applied.",
        see_also: &[GlossaryTerm::Policy, GlossaryTerm::Tier],
        since: 1,
    },
    GlossaryEntry {
        id: GlossaryTerm::IntentDocument,
        term: "Intent document",
        short: "A document saying what this tenant is building, for whom, and what better means; it steers which work is chosen next.",
        long: "**Intent documents** are the *intent* family of prompt documents. Their subject is the tenant's own product. Six kinds:\n\n- **product intent**: vision, non-goals, open questions\n- **initiative**: a time-boxed push that sets the bar for new work\n- **success metric**: one measure with baseline, target and direction\n- **domain spec**: what a subsystem is supposed to do\n- **audience profile**: who the users are\n- **decision record**: settled choices and what would reopen them\n\nSeeded documents start as skeletons, and an unedited skeleton is not the tenant's position. With no live initiative there is no bar, so no new work is proposed.",
        see_also: &[GlossaryTerm::Policy, GlossaryTerm::Tenant],
        since: 1,
    },
    GlossaryEntry {
        id: GlossaryTerm::Drain,
        term: "Drain",
        short: "Either stopping new work being sent to a device (reversible), or preparing one runner for a planned restart (final).",
        long: "**Drain** names two different operations.\n\n**Device drain** (in the coordination service): the operator stops new work being sent to a device or machine until a set time: CI, builds, merges, agent-session starts and gate continuations. It is reversible and expires. Nothing already running is interrupted; sessions that declared themselves finished and sat idle are closed.\n\n**Runner drain**: a graceful stop before a restart. It refuses new AI turns, saves in-flight turns and parks uncommitted worktree changes on a saved ref. It covers only the runner's own AI and task sessions, not agents in its terminals, and it is final: a drained runner is not un-drained.",
        see_also: &[GlossaryTerm::RestartReadiness, GlossaryTerm::Device, GlossaryTerm::Runner, GlossaryTerm::SessionStatus],
        since: 1,
    },
    GlossaryEntry {
        id: GlossaryTerm::RestartReadiness,
        term: "Restart readiness",
        short: "The runner's answer to whether restarting it now would lose work, with each kind of live session counted separately.",
        long: "**Restart readiness** is the runner's one answer to \"is it safe to restart this runner?\". It is safe only when no live agent process would be cut off and no AI session is running.\n\nIt never reports a single session count: agents in terminals, headless agents and AI sessions are listed separately, because a runner drain covers only some of them. A session marked *finished* does not block, but its process is still running and a restart ends it. A status it cannot read counts as blocking, never as idle.",
        see_also: &[GlossaryTerm::Drain, GlossaryTerm::SessionStatus, GlossaryTerm::Runner, GlossaryTerm::Unknown],
        since: 1,
    },
    GlossaryEntry {
        id: GlossaryTerm::AgentSession,
        term: "Agent session",
        short: "One running AI agent conversation, tracked with a liveness state and a separate work status.",
        long: "An **agent session** is one AI agent at work. It starts when a person launches it or when a gate continuation starts it; a continuation-started session is *expected* until it actually begins.\n\nSessions declare what they intend to work on, take claims, receive messages from other sessions, and can be resumed after the runner is rebuilt. Each has two separate states: whether it is alive, and whether its work is done; see **session status**.",
        see_also: &[GlossaryTerm::SessionStatus, GlossaryTerm::Continuation, GlossaryTerm::Claim, GlossaryTerm::Runner],
        since: 1,
    },
    GlossaryEntry {
        id: GlossaryTerm::SessionStatus,
        term: "Session status",
        short: "Two separate axes: liveness (active, stale, closed) and work (working, blocked, finished). Finished is not closed.",
        long: "A session has two independent states.\n\n**Liveness**: *expected* (started by a continuation but not yet running), *active*, *pending resolution*, *stale* (no heartbeat) and *closed*.\n\n**Work**: *working*, *blocked*, *waiting on a person*, *stalled* (heartbeating but making no progress) and *finished* (the agent declared its work complete).\n\n**Finished is not closed, and closed is not finished.** A finished session may still be running; a closed one may have stopped mid-task. Finished and waiting sessions are never nudged, only unfinished sessions are offered for resume, and finished can be undone.",
        see_also: &[GlossaryTerm::AgentSession, GlossaryTerm::RestartReadiness, GlossaryTerm::Drain],
        since: 1,
    },
    GlossaryEntry {
        id: GlossaryTerm::Rung,
        term: "Rung",
        short: "Where the runner actually found a capability on this machine: built in, installed, fetched, cached, a developer checkout, or not found.",
        long: "A **rung** records where a capability was found, so a difference between an installed product and a developer build is a visible value rather than a surprise.\n\n- **embedded**: compiled into the application\n- **bundle resource**: shipped by the installer\n- **served**: fetched from the service (unavailable offline or unpaired)\n- **disk cache**: a local copy of an earlier fetch\n- **developer checkout** rungs: found only on a machine that builds the product\n- **unresolved**: every rung was tried and none answered\n- **unknown**: nothing looked\n\nA capability that resolves from a developer checkout but is unresolved on an installed product is a defect.",
        see_also: &[GlossaryTerm::Runner, GlossaryTerm::Unknown],
        since: 1,
    },
    GlossaryEntry {
        id: GlossaryTerm::Tenant,
        term: "Tenant",
        short: "The isolation boundary that owns repositories, devices, gates, work units, policies and memory. Shown as a Project in the app.",
        long: "A **tenant** is the unit of ownership and isolation. Repositories, devices, gates, work units, findings, policies and intent documents all belong to exactly one tenant, and every read and write is scoped to the tenant resolved from the caller's identity.\n\nThe web app calls a tenant a **Project**, and users can create their own. The merge train lands changes automatically only once the tenant's rollout is live.",
        see_also: &[GlossaryTerm::Device, GlossaryTerm::Operator, GlossaryTerm::Policy, GlossaryTerm::IntentDocument],
        since: 1,
    },
    GlossaryEntry {
        id: GlossaryTerm::Device,
        term: "Device",
        short: "A registered machine running the runner. Pairing links it to your account and issues the credential it uses.",
        long: "A **device** is a machine (or runner instance) the product knows by its device id. The id is an identifier, not a secret.\n\n**Pairing** links a device to a person who belongs to the tenant: a short-lived pairing token is completed by that person, and the device is issued its own credential. The service works out the device's tenant from that credential; the device never claims it.\n\nWork is sent only to devices that are paired, heartbeating, capable of the work and not drained.",
        see_also: &[GlossaryTerm::Tenant, GlossaryTerm::Runner, GlossaryTerm::Drain],
        since: 1,
    },
    GlossaryEntry {
        id: GlossaryTerm::Runner,
        term: "Runner",
        short: "The desktop application on a paired device that hosts agent sessions and automations and talks to the service for you.",
        long: "The **runner** is the Qontinui desktop application. It hosts agent sessions and automation runs, carries out gate continuations sent to its device, and forwards requests to the coordination service with its own device credential.\n\nIt serves a local API on the machine it runs on, including its health, restart readiness and drain. It can run with a window or headless.",
        see_also: &[GlossaryTerm::Device, GlossaryTerm::AgentSession, GlossaryTerm::Coord, GlossaryTerm::RestartReadiness, GlossaryTerm::Rung],
        since: 1,
    },
    GlossaryEntry {
        id: GlossaryTerm::Coord,
        term: "Coord",
        short: "The coordination service: claims, work units, gates, sessions, devices, tenants, policies and the merge train.",
        long: "**Coord** is the central coordination service. It holds claims, work units, gates and their continuations, findings and dossiers, policy and intent documents, agent sessions, devices and tenants, and it runs the merge train, which makes it the only authority that lands pull requests.\n\nAgents reach it through tools and over HTTP. It knows nothing about plan files; plans reach it as work units.",
        see_also: &[GlossaryTerm::MergeTrain, GlossaryTerm::Gate, GlossaryTerm::WorkUnit, GlossaryTerm::Tenant, GlossaryTerm::Runner],
        since: 1,
    },
    GlossaryEntry {
        id: GlossaryTerm::WorktreeAllocation,
        term: "Worktree allocation",
        short: "Asking the service for an agent's own isolated working copy on a reserved branch, so parallel agents never share one.",
        long: "A **worktree allocation** gives an agent its own git worktree for each repository it needs, on a branch reserved for that agent, and records what the agent intends to do there so overlapping work is visible.\n\nIt can answer *wait* when the machine's isolation budget is used up; a refused allocation records nothing. Repositories a checkout builds against are allocated beside it automatically.\n\nEvery successful allocation creates a record, so never make one just to test whether the service is reachable.",
        see_also: &[GlossaryTerm::Claim, GlossaryTerm::AgentSession, GlossaryTerm::Device],
        since: 1,
    },
    GlossaryEntry {
        id: GlossaryTerm::Operator,
        term: "Operator",
        short: "The person who owns the tenant: approves operator gates, drains devices and writes the intent documents.",
        long: "The **operator** is the human who owns a tenant. Only the operator approves gates addressed to them, drains devices, authors intent documents and restores policy defaults.\n\nPolicies decide what agents may do without asking; an agent escalates to the operator only for the cases the policy lists, and brings a recommendation when it does.",
        see_also: &[GlossaryTerm::Tenant, GlossaryTerm::Attestation, GlossaryTerm::IntentDocument, GlossaryTerm::Policy],
        since: 1,
    },
    GlossaryEntry {
        id: GlossaryTerm::Unknown,
        term: "UNKNOWN",
        short: "A first-class answer meaning the product could not observe the value. It is never shown as zero, none, idle or a default.",
        long: "**UNKNOWN** is what the product shows when a read cannot support an answer, instead of guessing.\n\n- An empty result from a probe whose errors were hidden is UNKNOWN, not \"none\".\n- A well-formed value whose source cannot back it (a stale latest row, a constant where a measurement was promised) is UNKNOWN too.\n\nYou will meet it as a gate that stays open with a reason, a rung reading *unknown* rather than *unresolved*, a missing plan in a lagging library, or restart readiness counting an unreadable status as blocking. UNKNOWN is a prompt to look, not a verdict.",
        see_also: &[GlossaryTerm::Gate, GlossaryTerm::Rung, GlossaryTerm::RestartReadiness, GlossaryTerm::MergeVerdict],
        since: 1,
    },
];
