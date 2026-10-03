/* eslint-disable */
// @generated from glossary/terms.toml by rust/tests/glossary_source.rs — do not edit.
// Regenerate: QONTINUI_GLOSSARY_REGENERATE=1 cargo test -p qontinui-types --test glossary_source

import type { GlossaryTerm } from "../generated/GlossaryTerm";

/** One glossary definition (the Rust `GlossaryEntry`, as it serialises). */
export interface GlossaryEntry<Id extends GlossaryTerm = GlossaryTerm> {
  readonly id: Id;
  /** Display name. */
  readonly term: string;
  /** Plain text, at most 160 characters (tooltip). */
  readonly short: string;
  /** Markdown, at most 1200 characters. */
  readonly long: string;
  /** Related terms. */
  readonly see_also: readonly GlossaryTerm[];
  /** The glossary version that introduced this term. */
  readonly since: number;
}

/** The glossary's `version` (glossary/terms.toml). */
export const GLOSSARY_VERSION: number = 1;
/** SHA-256 of the canonical glossary content (the parsed terms as JSON; see glossary/versions.lock). */
export const GLOSSARY_CONTENT_SHA256 = "6e753bcf582accfec2223bdde074407dadf79ea4b417c1e32c269033b3e4388c";

/** Every term id, in glossary order. */
export const GLOSSARY_TERMS: readonly GlossaryTerm[] = [
  "gate",
  "continuation",
  "attestation",
  "work_unit",
  "plan",
  "claim",
  "tier",
  "merge_train",
  "merge_verdict",
  "landed",
  "finding",
  "dossier",
  "policy",
  "clause",
  "intent_document",
  "drain",
  "restart_readiness",
  "agent_session",
  "session_status",
  "rung",
  "tenant",
  "device",
  "runner",
  "coord",
  "worktree_allocation",
  "operator",
  "unknown",
];

/** Every definition, keyed by id. */
export const GLOSSARY: { readonly [Id in GlossaryTerm]: GlossaryEntry<Id> } = {
  "gate": {
    id: "gate",
    term: "Gate",
    short: "A watched record of work that stopped because it is waiting on something observable; it resumes by itself when the condition clears.",
    long: "A **gate** turns \"I am blocked\" into something the product watches. It has a typed predicate (a pull request landed, CI went green, a deploy is healthy, a claim ended, an operator approved, a time elapsed, and more) that is re-checked on a schedule.\n\nVerdicts: **open**, **cleared**, **failed** (the event happened the wrong way), **misconfigured** (can never be evaluated) and **withdrawn**. When the evaluator cannot answer, the gate stays open with a reason rather than guessing.\n\nVerbs: *register* creates a gate, *attest* clears an approval gate, *withdraw* retires it with a reason. A gate can carry a **continuation** that runs when it clears.",
    see_also: ["continuation", "attestation", "work_unit", "unknown"],
    since: 1,
  },
  "continuation": {
    id: "continuation",
    term: "Continuation",
    short: "The action a gate performs when it clears: start an agent session, queue a pull request to land, deploy, migrate, or just notify.",
    long: "A **continuation** is attached to a gate and runs when the gate clears, so blocked work picks itself back up without anyone remembering to restart it.\n\nActions: *run a skill* (start an agent session with arguments), *merge a pull request* (queue it on the merge train), *deploy*, *migrate*, or *notify only*. Deploy and migrate pass a safety check first; they notify by default and run only where the service has been set up to run them.\n\nDelivery: by default a new agent session is started; it can instead be delivered into a live session that still holds the context, falling back to a new session when none is live. The product records when a continuation was dispatched and when it was picked up, so one that never started is detected as stalled.",
    see_also: ["gate", "agent_session", "merge_train"],
    since: 1,
  },
  "attestation": {
    id: "attestation",
    term: "Attestation",
    short: "A declaration by an agent or a person that a gate's condition is met, which clears an approval gate.",
    long: "An **attestation** clears a gate whose predicate is an approval rather than something the product can observe by itself.\n\nWho may attest is decided per gate class by its clearance authority: *operator only*, *an agent other than the author* (separation of duties), or *any agent* in the tenant. Gates addressed to agents can by default be attested by any agent; gates addressed to the operator need a human approval.\n\nForce-clearing a gate that can no longer clear on its own is a separate action, checked against the same authority.",
    see_also: ["gate", "operator", "tenant"],
    since: 1,
  },
  "work_unit": {
    id: "work_unit",
    term: "Work unit",
    short: "The durable record of one piece of work: a slug with a status, an owner, the pull requests that implement it, and its dependencies.",
    long: "A **work unit** is how the product tracks a piece of work over its whole life, independent of any file. It carries a status, an owner, **citations** (the pull requests that implement it), dependency edges and free-form metadata.\n\nEvery status change is kept in its history, and gates can wait on a work unit reaching a status. The status is free text chosen by whoever writes it, so read it as that writer's label rather than a fixed state machine.\n\nWhen plans are enabled, each plan's status is mirrored into a work unit.",
    see_also: ["plan", "gate", "tier", "claim"],
    since: 1,
  },
  "plan": {
    id: "plan",
    term: "Plan",
    short: "A written work document whose status is mirrored into a work unit and whose body is kept, versioned, in the plan library.",
    long: "A **plan** is a markdown document describing a piece of work: why, the phases, and how to tell it is done. Its operational state (status, citations, dependencies) lives in a work unit; its body lives in the **plan library**, which keeps every version.\n\nPlans are optional. A tenant may author entirely through the web app and keep no plan files on disk.\n\nThe library can lag the files it mirrors, so a plan missing from a library read is UNKNOWN until the library's own health signal says the read is current.",
    see_also: ["work_unit", "unknown", "tier"],
    since: 1,
  },
  "claim": {
    id: "claim",
    term: "Claim",
    short: "A short-lived reservation over something an agent is about to change, so other agents see it and do not collide.",
    long: "A **claim** says \"I am working on this right now\". Kinds include a worktree, a file glob, a single symbol (one function or type), a branch name, a plan phase and a database migration revision.\n\nClaims are kept alive by heartbeats and expire on their own, so a session that dies does not hold its claims forever. A conflicting request is answered with who holds the claim, never by silently taking it over.\n\nClaims are the lightest coordination tier; work units and plans build on them.",
    see_also: ["tier", "worktree_allocation", "agent_session"],
    since: 1,
  },
  "tier": {
    id: "tier",
    term: "Tier",
    short: "A level in one of three ladders: coordination (claims, work units, plans), the autonomy dial, or a policy clause's permission level.",
    long: "**Tier** names a level in three separate ladders; read which one from context.\n\n1. **Coordination tiers** stack: *claims* (who is touching what, always on), then *work units* (the durable record of work), then *plan files* (optional; enabled by configuring a plans folder). A tenant with no plan files loses very little.\n2. **The autonomy dial** says how far agents may go on security-sensitive work: *proceed*, *draft required* (open a draft behind an approval gate) or *ask first*.\n3. **Clause tiers** mark what one policy clause permits: *proceed*, *proceed and log*, *proceed and notify*, *ask first* or *never*.",
    see_also: ["claim", "work_unit", "plan", "policy", "clause"],
    since: 1,
  },
  "merge_train": {
    id: "merge_train",
    term: "Merge train",
    short: "The service that lands pull requests: it rebases each onto the latest main, runs CI, keeps overlapping changes apart and lands them in order.",
    long: "By policy, the **merge train** is the one route by which pull requests land on a repository it manages. A green, non-draft pull request is queued with nothing further to do.\n\nFor each one it dry-rebases onto current main, detects file overlap with other queued changes (two that touch the same files are kept apart), runs CI, and lands by fast-forward push. The queue is first in, first out; a *land next* priority lane exists but is off unless enabled.\n\nWhat holds a pull request back: draft status, a gate carrying a merge continuation, or a dependency label naming another pull request that has not landed yet. Its answer for one pull request is the **merge verdict**.",
    see_also: ["merge_verdict", "landed", "continuation"],
    since: 1,
  },
  "merge_verdict": {
    id: "merge_verdict",
    term: "Merge verdict",
    short: "The merge train's read-only answer for one pull request: its state, what blocks it, and the next action.",
    long: "The **merge verdict** answers \"what is the merge train doing with this pull request, and what should I do?\". It is computed when asked, from the train's own records.\n\nIt reports the state (queued, rebasing, awaiting CI, blocked by overlap, landing, merged, conflict, cancelled), the block reason and, for an escalation, its category and the gate it waits on, plus a table-driven next action.\n\nAn in-flight state that also carries an error describes the queue slot, not the outcome. Evidence of an earlier landing reads *landed*, *reverted upstream* or *unknown*; no evidence is unknown, never \"not landed\".",
    see_also: ["merge_train", "landed", "unknown", "gate"],
    since: 1,
  },
  "landed": {
    id: "landed",
    term: "Landed",
    short: "A pull request's commits are on main. The code host may show it as Closed or as Merged; both can mean landed.",
    long: "**Landed** means the change's commits are on the main branch.\n\nThe merge train lands by rebasing and fast-forwarding main. When the rebase rewrote the commits, the code host shows the pull request as **Closed**, not Merged; when it did not, it shows **Merged**. Neither label, nor a pull request still showing open, proves or disproves a landing on its own.\n\nTo know, read the merge verdict or check that the content is on main.",
    see_also: ["merge_train", "merge_verdict"],
    since: 1,
  },
  "finding": {
    id: "finding",
    term: "Finding",
    short: "A short investigation result one session posts for others; it expires after about two weeks.",
    long: "A **finding** sits between a private transcript and permanent memory: what one session learned that the next one working on the same thing should know first.\n\nFindings are scoped to a resource or topic and expire after about 14 days by default. Sessions read the recent findings before starting on something and post one after an investigation. Findings can be marked triaged once someone has routed them.\n\nA problem that keeps coming back outlives that window as a **dossier**.",
    see_also: ["dossier", "agent_session"],
    since: 1,
  },
  "dossier": {
    id: "dossier",
    term: "Dossier",
    short: "A durable, topic-keyed file for a problem that keeps recurring, collecting its occurrences across months.",
    long: "A **dossier** is a long-lived finding: one per topic, for a defect or question that recurs across many sessions. It collects each occurrence, the current understanding and what would close it.\n\nIt does not expire. Updates are written forward as a new head that supersedes the previous one; there is never a second head for the same topic.",
    see_also: ["finding"],
    since: 1,
  },
  "policy": {
    id: "policy",
    term: "Policy",
    short: "A versioned document of rules for how agents act in this tenant, made of individually cited clauses.",
    long: "A **policy** is a prompt document in the *behavior* family: it says how sessions act, not what to build. It is made of **clauses**, and its text is rebuilt from them.\n\nEvery edit creates a new, unchangeable version, so older versions can be restored. Sessions read the policies fresh rather than from memory, cite the clause and version they applied, and record a *policy gap* when no clause covers a decision.\n\nWhat the tenant is building is described separately, by **intent documents**.",
    see_also: ["clause", "intent_document", "tier"],
    since: 1,
  },
  "clause": {
    id: "clause",
    term: "Clause",
    short: "One rule inside a policy, with a stable id, a status and a tier, that a session cites when it applies it.",
    long: "A **clause** is the unit of a policy. It has a stable kebab-case id, a status (*gap*, *proposed*, *confirmed*, *active* or *retired*), a tier saying what it permits, and trigger, action, bounds and escalate-if text.\n\nCiting a clause by id and policy version makes a decision auditable: a reader can open the exact rule that was applied.",
    see_also: ["policy", "tier"],
    since: 1,
  },
  "intent_document": {
    id: "intent_document",
    term: "Intent document",
    short: "A document saying what this tenant is building, for whom, and what better means; it steers which work is chosen next.",
    long: "**Intent documents** are the *intent* family of prompt documents. Their subject is the tenant's own product. Six kinds:\n\n- **product intent**: vision, non-goals, open questions\n- **initiative**: a time-boxed push that sets the bar for new work\n- **success metric**: one measure with baseline, target and direction\n- **domain spec**: what a subsystem is supposed to do\n- **audience profile**: who the users are\n- **decision record**: settled choices and what would reopen them\n\nSeeded documents start as skeletons, and an unedited skeleton is not the tenant's position. With no live initiative there is no bar, so no new work is proposed.",
    see_also: ["policy", "tenant"],
    since: 1,
  },
  "drain": {
    id: "drain",
    term: "Drain",
    short: "Either stopping new work being sent to a device (reversible), or preparing one runner for a planned restart (final).",
    long: "**Drain** names two different operations.\n\n**Device drain** (in the coordination service): the operator stops new work being sent to a device or machine until a set time: CI, builds, merges, agent-session starts and gate continuations. It is reversible and expires. Nothing already running is interrupted; sessions that declared themselves finished and sat idle are closed.\n\n**Runner drain**: a graceful stop before a restart. It refuses new AI turns, saves in-flight turns and parks uncommitted worktree changes on a saved ref. It covers only the runner's own AI and task sessions, not agents in its terminals, and it is final: a drained runner is not un-drained.",
    see_also: ["restart_readiness", "device", "runner", "session_status"],
    since: 1,
  },
  "restart_readiness": {
    id: "restart_readiness",
    term: "Restart readiness",
    short: "The runner's answer to whether restarting it now would lose work, with each kind of live session counted separately.",
    long: "**Restart readiness** is the runner's one answer to \"is it safe to restart this runner?\". It is safe only when no live agent process would be cut off and no AI session is running.\n\nIt never reports a single session count: agents in terminals, headless agents and AI sessions are listed separately, because a runner drain covers only some of them. A session marked *finished* does not block, but its process is still running and a restart ends it. A status it cannot read counts as blocking, never as idle.",
    see_also: ["drain", "session_status", "runner", "unknown"],
    since: 1,
  },
  "agent_session": {
    id: "agent_session",
    term: "Agent session",
    short: "One running AI agent conversation, tracked with a liveness state and a separate work status.",
    long: "An **agent session** is one AI agent at work. It starts when a person launches it or when a gate continuation starts it; a continuation-started session is *expected* until it actually begins.\n\nSessions declare what they intend to work on, take claims, receive messages from other sessions, and can be resumed after the runner is rebuilt. Each has two separate states: whether it is alive, and whether its work is done; see **session status**.",
    see_also: ["session_status", "continuation", "claim", "runner"],
    since: 1,
  },
  "session_status": {
    id: "session_status",
    term: "Session status",
    short: "Two separate axes: liveness (active, stale, closed) and work (working, blocked, finished). Finished is not closed.",
    long: "A session has two independent states.\n\n**Liveness**: *expected* (started by a continuation but not yet running), *active*, *pending resolution*, *stale* (no heartbeat) and *closed*.\n\n**Work**: *working*, *blocked*, *waiting on a person*, *stalled* (heartbeating but making no progress) and *finished* (the agent declared its work complete).\n\n**Finished is not closed, and closed is not finished.** A finished session may still be running; a closed one may have stopped mid-task. Finished and waiting sessions are never nudged, only unfinished sessions are offered for resume, and finished can be undone.",
    see_also: ["agent_session", "restart_readiness", "drain"],
    since: 1,
  },
  "rung": {
    id: "rung",
    term: "Rung",
    short: "Where the runner actually found a capability on this machine: built in, installed, fetched, cached, a developer checkout, or not found.",
    long: "A **rung** records where a capability was found, so a difference between an installed product and a developer build is a visible value rather than a surprise.\n\n- **embedded**: compiled into the application\n- **bundle resource**: shipped by the installer\n- **served**: fetched from the service (unavailable offline or unpaired)\n- **disk cache**: a local copy of an earlier fetch\n- **developer checkout** rungs: found only on a machine that builds the product\n- **unresolved**: every rung was tried and none answered\n- **unknown**: nothing looked\n\nA capability that resolves from a developer checkout but is unresolved on an installed product is a defect.",
    see_also: ["runner", "unknown"],
    since: 1,
  },
  "tenant": {
    id: "tenant",
    term: "Tenant",
    short: "The isolation boundary that owns repositories, devices, gates, work units, policies and memory. Shown as a Project in the app.",
    long: "A **tenant** is the unit of ownership and isolation. Repositories, devices, gates, work units, findings, policies and intent documents all belong to exactly one tenant, and every read and write is scoped to the tenant resolved from the caller's identity.\n\nThe web app calls a tenant a **Project**, and users can create their own. The merge train lands changes automatically only where the tenant has automatic merging enabled, the repository has merging enabled, and the tenant has not paused merging; a tenant-wide pause overrides every repository.",
    see_also: ["device", "operator", "policy", "intent_document"],
    since: 1,
  },
  "device": {
    id: "device",
    term: "Device",
    short: "A registered machine running the runner. Pairing links it to your account and issues the credential it uses.",
    long: "A **device** is a machine (or runner instance) the product knows by its device id. The id is an identifier, not a secret.\n\n**Pairing** links a device to a person who belongs to the tenant: a short-lived pairing token is completed by that person, and the device is issued its own credential. On authenticated requests the service works out the device's tenant from that credential rather than from anything the device asserts.\n\nWork is sent only to devices that are paired, heartbeating, capable of the work and not drained.",
    see_also: ["tenant", "runner", "drain"],
    since: 1,
  },
  "runner": {
    id: "runner",
    term: "Runner",
    short: "The desktop application on a paired device that hosts agent sessions and automations and talks to the service for you.",
    long: "The **runner** is the Qontinui desktop application. It hosts agent sessions and automation runs, carries out gate continuations sent to its device, and forwards requests to the coordination service with its own device credential.\n\nIt serves a local API on the machine it runs on, including its health, restart readiness and drain. It can run with a window or headless.",
    see_also: ["device", "agent_session", "coord", "restart_readiness", "rung"],
    since: 1,
  },
  "coord": {
    id: "coord",
    term: "Coord",
    short: "The coordination service: claims, work units, gates, sessions, devices, tenants, policies and the merge train.",
    long: "**Coord** is the central coordination service. It holds claims, work units, gates and their continuations, findings and dossiers, policy and intent documents, agent sessions, devices and tenants, and it runs the merge train, which policy makes the one route by which pull requests land.\n\nAgents reach it through tools and over HTTP. It does not read plan files; plans reach it as work units.",
    see_also: ["merge_train", "gate", "work_unit", "tenant", "runner"],
    since: 1,
  },
  "worktree_allocation": {
    id: "worktree_allocation",
    term: "Worktree allocation",
    short: "Asking the service for an agent's own isolated working copy on a reserved branch, so parallel agents never share one.",
    long: "A **worktree allocation** gives an agent its own git worktree for each repository it needs, on a branch reserved for that agent, and records what the agent intends to do there so overlapping work is visible.\n\nIt can answer *wait* when the machine's isolation budget is used up; a refused allocation records nothing. Repositories a checkout builds against are allocated beside it automatically.\n\nEvery successful allocation creates a record, so never make one just to test whether the service is reachable.",
    see_also: ["claim", "agent_session", "device"],
    since: 1,
  },
  "operator": {
    id: "operator",
    term: "Operator",
    short: "The person who owns the tenant: approves operator gates, drains devices and writes the intent documents.",
    long: "The **operator** is the human who owns a tenant. Only the operator approves gates addressed to them, drains devices, authors intent documents and restores policy defaults.\n\nPolicies decide what agents may do without asking; an agent escalates to the operator only for the cases the policy lists, and brings a recommendation when it does.",
    see_also: ["tenant", "attestation", "intent_document", "policy"],
    since: 1,
  },
  "unknown": {
    id: "unknown",
    term: "UNKNOWN",
    short: "A first-class answer meaning the product could not observe the value. It is never shown as zero, none, idle or a default.",
    long: "**UNKNOWN** is what the product shows when a read cannot support an answer, instead of guessing.\n\n- An empty result from a probe whose errors were hidden is UNKNOWN, not \"none\".\n- A well-formed value whose source cannot back it (a stale latest row, a constant where a measurement was promised) is UNKNOWN too.\n\nYou will meet it as a gate that stays open with a reason, a rung reading *unknown* rather than *unresolved*, a missing plan in a lagging library, or restart readiness counting an unreadable status as blocking. UNKNOWN is a prompt to look, not a verdict.",
    see_also: ["gate", "rung", "restart_readiness", "merge_verdict"],
    since: 1,
  },
};
