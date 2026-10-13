//! Sibling path-dep provisioning for one CI dispatch.
//!
//! # The bug this closes
//!
//! `qontinui-coord/Cargo.toml:95` and `:101` carry Cargo **path** deps on a
//! sibling repo (`../qontinui-schemas/rust`, `../qontinui-schemas/code-graph`),
//! and `qontinui-web/backend/pyproject.toml:57` expresses the same dependency
//! at a different depth (`../../qontinui-schemas`). Both resolve to the same
//! place: **a sibling of the repo checkout**. The CI-node executor checked out
//! only the dispatched repo, one level deeper than the primary clone, so
//! `../qontinui-schemas` resolved into the `.ci-worktrees` directory and a
//! pre-existing clone at the device root did NOT satisfy the dep. On a dev box
//! that clone usually exists, so the failure looked like a cargo bug rather
//! than a layout mismatch.
//!
//! # The layout rule
//!
//! ```text
//! <QONTINUI_ROOT>/.ci-worktrees/<dispatch_id>/          <- the cleanup unit
//!                                 ├── <repo>/           <- dispatched worktree
//!                                 └── <sibling>/        <- one per [[siblings]]
//! ```
//!
//! ONE rule, deliberately not one per toolchain: the dispatched worktree sits
//! where a workspace-root sibling resolves, which satisfies Cargo's
//! `../qontinui-schemas/rust` from the repo root and Poetry's
//! `../../qontinui-schemas` from `backend/` alike. One invariant to hold
//! beats one per language.
//!
//! It also keeps `checkout::cleanup_dispatch`'s guarantee intact: everything
//! this module writes lands inside the dispatch-scoped parent, so cleanup is
//! still a single directory removal and nothing survives a dispatch — success
//! or failure. That is the precise reason an in-manifest `git clone` is
//! rejected: such a step WOULD pass argv validation (no banned metacharacter,
//! and the executor does not sandbox the filesystem), but it would be
//! unpinned and it would leak.
//!
//! # No shared clone cache
//!
//! Each dispatch clones its siblings fresh (`--depth 1`, by exact SHA). A
//! shared cache would have to live OUTSIDE the dispatch-scoped directory —
//! i.e. outside the cleanup guarantee — and would then need its own pruning,
//! its own concurrent-dispatch locking, and its own answer for a half-fetched
//! tree. The cost avoided is one shallow fetch of one repo per dispatch; the
//! property preserved is that a dispatch leaves nothing behind. Tools are
//! cached ([`crate::tools`]) because their key is an exact version; a sibling's
//! key is a commit that moves, so the two are not the same trade.
//!
//! # Which commit — the Actions lane's rule, not a third one
//!
//! Resolution mirrors `.github/actions/checkout-sibling/action.yml` (Phase A
//! landed as `a543cc7e1`/#984; Phase B is PR #1008), because the two lanes
//! must agree on which sibling tree a change compiles against or "parity"
//! means nothing:
//!
//! * The hatch is a **declared adaptation PR** — the coord dep edge the pair
//!   already declares for merge ordering — never a branch NAME. A branch name
//!   is not an identity, and the same name-matching mechanism was exploited in
//!   production on 2026-07-28 when a consumer gate went green against a branch
//!   with no pull request at all.
//! * ALL FOUR label forms are read, ALWAYS, and every match is collected
//!   rather than taking the first or the last — that is the only way two
//!   declarations naming DIFFERENT sibling PRs can be detected. Exactly one
//!   tree can be checked out, so an ambiguity is the author's to resolve, not
//!   ours to guess.
//! * A dep-edge label names the PAIR; its DIRECTION names the merge order,
//!   and merge order is orthogonal to which tree to compile against. Both
//!   directions are therefore read. Reading only the sibling-leads pair
//!   ("`downstream-of` here" / "`upstream-of` there") silently assumed the
//!   sibling always leads — false whenever the type is authored HERE and the
//!   sibling carries only generated bindings, which is a whole standing class
//!   of change, not an edge case. See [`DeclarationForm`].
//! * A pair declared in BOTH directions is a dependency CYCLE and hard-fails
//!   rather than resolving: coord would hold both PRs forever, and a green
//!   check here would hide that.
//! * Comparison is **owner-stripped**, because the owner is optional in
//!   coord's dep-edge grammar.
//! * "Absent" and "unanswered" are different answers and only the first is a
//!   NO. A rate-limited or unreachable probe is UNKNOWN and hard-fails; a
//!   404 on a PR that WAS declared is a semantic red about the declaration.
//! * Resolution is keyed on a pull request, never on the pushed ref. The
//!   CI-node lane dispatches `refs/heads/merge-candidate/<proposal_id>`, and
//!   coord pushes the SAME candidate ref into every repo of a multi-repo
//!   proposal — so keying off the ref name would compile against a tree that
//!   is force-pushed and deleted as the proposal resolves. That event class
//!   resolves to the sibling's branch with no API call at all, exactly as the
//!   composite action does.
//! * The action's LOWEST-priority answer is a **recorded pin**:
//!   `.github/sibling-pins.conf` ([`SIBLING_PIN_FILE`]), consulted only when
//!   no declaration resolved, on every one of the three no-declaration exits
//!   (no pull request; no declaration; trailing declaration declined). A
//!   `[[siblings]]` entry opts into it with `pin = "pin-file"`
//!   ([`SiblingPin::PinFile`]), and [`lookup_pin`] parses the file
//!   field-for-field as the action's `apply_pin` does — `#` strips a
//!   comment, CRs are tolerated, exactly two fields, a 40-hex lowercase SHA,
//!   a duplicate entry is ambiguous — so the two lanes cannot disagree about
//!   what the file says. #1158 pinned the Actions lane after a floating
//!   ui-bridge checkout held this repo's merge train for 5h+ on 2026-08-20;
//!   this is the CI-node half of the same close.
//!
//! **Stronger than #1008 in one place, and only one.** #1008 fetches the
//! declared PR's head SHA but the no-declaration path by BRANCH. Here both
//! paths end at the same primitive: resolve to a SHA first (`ls-remote` for
//! the branch case), then fetch that SHA and assert the checkout landed on
//! exactly it. Same resolution rule, one materialisation invariant instead of
//! two — and the resolved SHA is recorded in the dispatch log either way, so
//! "which tree did this build compile against" is answerable after the fact.

use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::{debug, info, warn};

use crate::host::{GithubAccess, ProcessSpawn, SiblingReadAccess, SiblingReadAnswer};
use crate::manifest::{CiSibling, SiblingPin};
use crate::sibling_git::{require_git_floor, run_sibling_git, ReadToken, SiblingGitEnv};

/// Budget for the sibling fetch (a `--depth 1` fetch of one commit).
const SIBLING_FETCH_TIMEOUT: Duration = Duration::from_secs(600);
/// Budget for local git plumbing.
const GIT_LOCAL_TIMEOUT: Duration = Duration::from_secs(120);
/// Per-request GitHub API budget.
const API_TIMEOUT: Duration = Duration::from_secs(60);
/// Pagination bound on the reverse declaration search. An unbounded scan of
/// open PRs is a denial-of-service surface; 20 pages of 100 is far past any
/// real repo.
const MAX_PR_PAGES: u32 = 20;

const DOWNSTREAM_PREFIX: &str = "coord:downstream-of=";
const UPSTREAM_PREFIX: &str = "coord:upstream-of=";

// ─────────────────────────── pure decision core ───────────────────────────

/// Labels on one open sibling PR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SiblingPrLabels {
    pub number: u64,
    pub labels: Vec<String>,
}

/// Everything the declaration rule reads. Collected by IO, decided purely.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct DeclarationInputs {
    /// Label names on the dispatched repo's pull request.
    pub this_pr_labels: Vec<String>,
    /// Every OPEN pull request on the sibling, with its labels.
    pub sibling_open_prs: Vec<SiblingPrLabels>,
}

/// Which declaration form named the target.
///
/// All FOUR forms name the SAME thing — the paired sibling pull request. They
/// differ only in WHO CARRIES the label and in which side coord's dep graph
/// orders first, and **merge order is orthogonal to which tree this build must
/// compile against**. Reading only the two sibling-leads forms was an unstated
/// assumption that the sibling always leads, which is false for a whole class
/// of change: when the Rust type is authored HERE and the sibling carries only
/// the generated bindings, THIS side must land first, so the pair's only
/// correct dep-edge direction is one the resolver could not see. That made
/// such a pair mutually unlandable — each side's drift gate demanding the
/// other land first (runner#1019 / schemas#136, open 2026-08-08).
// The shared `Of` suffix is the label grammar's own (`coord:downstream-of=`,
// `coord:upstream-of=`); dropping it would make the variants stop reading as
// the labels they are parsed from.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeclarationForm {
    /// `coord:downstream-of=<sibling>#n` on THIS pull request. Sibling leads.
    DownstreamOf,
    /// `coord:upstream-of=<this>#<this_pr>` on a sibling PR. Sibling leads.
    UpstreamOf,
    /// `coord:upstream-of=<sibling>#n` on THIS pull request. This side leads.
    SelfUpstreamOf,
    /// `coord:downstream-of=<this>#<this_pr>` on a sibling PR. This side leads.
    SiblingDownstreamOf,
}

impl DeclarationForm {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::DownstreamOf => "downstream-of",
            Self::UpstreamOf => "upstream-of",
            Self::SelfUpstreamOf => "self-upstream-of",
            Self::SiblingDownstreamOf => "sibling-downstream-of",
        }
    }

    /// True when this declaration orders the SIBLING before this side.
    fn sibling_leads(self) -> bool {
        matches!(self, Self::DownstreamOf | Self::UpstreamOf)
    }
}

/// Join the forms that named a target, for the dispatch log's provenance line.
pub(crate) fn forms_label(forms: &[DeclarationForm]) -> String {
    forms
        .iter()
        .map(|f| f.as_str())
        .collect::<Vec<_>>()
        .join("+")
}

/// Outcome of the declaration rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DeclaredTarget {
    /// No declaration — resolve to the sibling's branch.
    None,
    /// A declaration exists and is unambiguous, but it orders the sibling
    /// AFTER this side and this step did not opt in
    /// ([`CiSibling::accept_trailing_sibling`]). Resolve to the branch — but
    /// as a DECIDED outcome that names itself in the log, never as a silent
    /// fallback that is indistinguishable from "nobody declared anything".
    TrailingSiblingDeclined {
        number: u64,
        forms: Vec<DeclarationForm>,
    },
    Pr {
        number: u64,
        /// Every form that named this target, in a stable order. More than one
        /// is the common case: a single dep-edge declaration satisfies BOTH
        /// repos' gates, so it is visible from both ends.
        forms: Vec<DeclarationForm>,
    },
}

/// Basename of an `[owner/]name` slug — the owner is optional in coord's
/// dep-edge grammar, so every comparison here is owner-stripped. Do NOT
/// "simplify" this to an exact string match: the schemas-side twin does
/// exactly that and silently misses every bare-form label.
fn short_name(slug: &str) -> &str {
    slug.rsplit('/').next().unwrap_or(slug)
}

/// Split a `[<owner>/]<repo>#<n>` dep-edge spec.
fn parse_dep_spec(spec: &str) -> Option<(&str, &str)> {
    let (repo, num) = spec.rsplit_once('#')?;
    (!repo.is_empty() && !num.is_empty()).then_some((repo, num))
}

/// Deduplicate while preserving order.
fn dedup(mut v: Vec<u64>) -> Vec<u64> {
    let mut seen = Vec::with_capacity(v.len());
    v.retain(|n| {
        if seen.contains(n) {
            false
        } else {
            seen.push(*n);
            true
        }
    });
    v
}

fn ambiguous(what: &str, sibling_repo: &str, numbers: &[u64]) -> String {
    let list = numbers
        .iter()
        .map(|n| format!("{sibling_repo}#{n}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "contradictory declarations — {what}: {list}. Exactly one {} tree can be checked out, \
         so this is yours to resolve: keep the declaration naming the PR this change actually \
         adapts to and remove the other(s).",
        short_name(sibling_repo)
    )
}

/// The declaration rule, in full and pure.
///
/// All FOUR forms are evaluated unconditionally even when an earlier one hits,
/// and every match in each form is collected. Stopping early would mask a
/// contradiction, and exactly one tree can be checked out — so an ambiguity is
/// the author's to resolve, never ours to guess. Ways to be ambiguous, all
/// hard-failing alike:
///
/// * two same-direction labels on the dispatched PR naming different siblings,
/// * two sibling PRs declaring the same form at this one,
/// * any two forms naming DIFFERENT sibling PRs,
/// * both merge ORDERS declared for the same pair — a cycle in coord's dep
///   graph, which holds BOTH pull requests forever. Compiling against the
///   right tree would hide that behind a green check, so it reds here.
pub(crate) fn resolve_declaration(
    sibling_repo: &str,
    this_repo: &str,
    this_pr: u64,
    accept_trailing_sibling: bool,
    inputs: &DeclarationInputs,
) -> Result<DeclaredTarget, String> {
    let sibling_short = short_name(sibling_repo);
    let this_short = short_name(this_repo);

    // SELF-READ: both dep-edge directions on the dispatched pull request.
    //   coord:downstream-of=<sibling>#n -> sibling leads
    //   coord:upstream-of=<sibling>#n   -> this side leads
    let mut downs: Vec<u64> = Vec::new();
    let mut self_ups: Vec<u64> = Vec::new();
    for label in &inputs.this_pr_labels {
        let (spec, prefix, bucket) = if let Some(s) = label.strip_prefix(DOWNSTREAM_PREFIX) {
            (s, DOWNSTREAM_PREFIX, &mut downs)
        } else if let Some(s) = label.strip_prefix(UPSTREAM_PREFIX) {
            (s, UPSTREAM_PREFIX, &mut self_ups)
        } else {
            continue;
        };
        let Some((repo_part, num_part)) = parse_dep_spec(spec) else {
            return Err(format!(
                "malformed label {label:?} — expected {prefix}[<owner>/]<repo>#<n>"
            ));
        };
        if short_name(repo_part) != sibling_short {
            continue;
        }
        let n: u64 = num_part
            .parse()
            .map_err(|_| format!("malformed label {label:?} — {num_part:?} is not a PR number"))?;
        bucket.push(n);
    }

    // REVERSE SEARCH: both dep-edge directions on the sibling's open PRs.
    //   coord:upstream-of=<this>#<this_pr>   -> sibling leads
    //   coord:downstream-of=<this>#<this_pr> -> this side leads
    let mut ups: Vec<u64> = Vec::new();
    let mut sib_downs: Vec<u64> = Vec::new();
    for pr in &inputs.sibling_open_prs {
        for label in &pr.labels {
            let (spec, bucket) = if let Some(s) = label.strip_prefix(UPSTREAM_PREFIX) {
                (s, &mut ups)
            } else if let Some(s) = label.strip_prefix(DOWNSTREAM_PREFIX) {
                (s, &mut sib_downs)
            } else {
                continue;
            };
            let Some((repo_part, num_part)) = parse_dep_spec(spec) else {
                continue; // a malformed label on someone ELSE's PR is not ours to red.
            };
            if short_name(repo_part) == this_short && num_part.parse::<u64>() == Ok(this_pr) {
                bucket.push(pr.number);
            }
        }
    }

    let downs = dedup(downs);
    if downs.len() > 1 {
        return Err(ambiguous(
            &format!(
                "this pull request carries {DOWNSTREAM_PREFIX} labels naming DIFFERENT \
                 {sibling_short} pull requests"
            ),
            sibling_repo,
            &downs,
        ));
    }
    let self_ups = dedup(self_ups);
    if self_ups.len() > 1 {
        return Err(ambiguous(
            &format!(
                "this pull request carries {UPSTREAM_PREFIX} labels naming DIFFERENT \
                 {sibling_short} pull requests"
            ),
            sibling_repo,
            &self_ups,
        ));
    }
    let ups = dedup(ups);
    if ups.len() > 1 {
        return Err(ambiguous(
            &format!(
                "several open {sibling_short} pull requests declare \
                 {UPSTREAM_PREFIX}{this_short}#{this_pr}"
            ),
            sibling_repo,
            &ups,
        ));
    }
    let sib_downs = dedup(sib_downs);
    if sib_downs.len() > 1 {
        return Err(ambiguous(
            &format!(
                "several open {sibling_short} pull requests declare \
                 {DOWNSTREAM_PREFIX}{this_short}#{this_pr}"
            ),
            sibling_repo,
            &sib_downs,
        ));
    }

    // Reconcile the four forms. Order is stable so the provenance string and
    // the tests do not depend on hash iteration order.
    let mut forms: Vec<DeclarationForm> = Vec::new();
    let mut targets: Vec<u64> = Vec::new();
    for (hit, form) in [
        (downs.first().copied(), DeclarationForm::DownstreamOf),
        (ups.first().copied(), DeclarationForm::UpstreamOf),
        (self_ups.first().copied(), DeclarationForm::SelfUpstreamOf),
        (
            sib_downs.first().copied(),
            DeclarationForm::SiblingDownstreamOf,
        ),
    ] {
        if let Some(n) = hit {
            targets.push(n);
            forms.push(form);
        }
    }
    if forms.is_empty() {
        return Ok(DeclaredTarget::None);
    }

    let distinct = dedup(targets);
    if distinct.len() > 1 {
        return Err(ambiguous(
            &format!(
                "the coord dep labels on this pull request and on {sibling_repo}'s open pull \
                 requests name DIFFERENT {sibling_short} pull requests"
            ),
            sibling_repo,
            &distinct,
        ));
    }
    let number = distinct[0];

    // A pair declared in BOTH directions is a cycle: each side waits for the
    // other and coord proposes neither, forever. It is not our business which
    // direction is right, but it IS our business not to paper over it.
    let leads: Vec<DeclarationForm> = forms
        .iter()
        .copied()
        .filter(|f| f.sibling_leads())
        .collect();
    let trails: Vec<DeclarationForm> = forms
        .iter()
        .copied()
        .filter(|f| !f.sibling_leads())
        .collect();
    if !leads.is_empty() && !trails.is_empty() {
        return Err(format!(
            "contradictory merge ORDER declared for the pair {this_short}#{this_pr} / \
             {sibling_repo}#{number}: {} order {sibling_short} first, while {} order \
             {this_short} first. That is a cycle in coord's dependency graph — it would hold \
             BOTH pull requests forever. Keep exactly one direction.",
            forms_label(&leads),
            forms_label(&trails),
        ));
    }

    // The sibling TRAILS. Following it would compile against a tree that is
    // not this repo's main yet and will not be until after this change lands,
    // so only a step that opted in gets it. See
    // [`CiSibling::accept_trailing_sibling`] for why that is the safe default.
    if leads.is_empty() && !accept_trailing_sibling {
        return Ok(DeclaredTarget::TrailingSiblingDeclined { number, forms });
    }

    Ok(DeclaredTarget::Pr { number, forms })
}

/// The fields of a sibling PR the declaration validation reads. Flattened
/// from the API's nested shape at the IO boundary rather than derived, so the
/// validation below is a function of five strings and nothing else.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct SiblingPrDetail {
    pub state: String,
    pub base_ref: String,
    pub head_ref: String,
    pub head_sha: String,
    pub head_repo_full_name: String,
}

/// PRESENT is not enough; a declaration must be VERIFIABLE. Pure, so every
/// rejection is testable without the network. Returns the pinned head SHA.
pub(crate) fn validate_declared_pr(
    sibling_repo: &str,
    number: u64,
    d: &SiblingPrDetail,
) -> Result<String, String> {
    let invalid = |why: String| {
        Err(format!(
            "declared adaptation {sibling_repo}#{number} is invalid: {why} \
             This is a red about the DECLARATION, not an infra fault, and it deliberately does \
             NOT fall back to the default branch — an author who declared something wrong must \
             see it rather than get a silent main build."
        ))
    };
    if d.state != "open" {
        return invalid(format!(
            "it is '{}', not 'open'. If it has ALREADY LANDED, remove the label — it no longer \
             applies.",
            d.state
        ));
    }
    // A PR based on another feature branch AUTO-CLOSES UNLANDED when its
    // parent lands, so accepting one would let this resolve to a declaration
    // that can silently evaporate.
    if d.base_ref != "main" {
        return invalid(format!(
            "it is based on '{}', not 'main'. A PR based on a feature branch auto-closes \
             UNLANDED when its parent lands, so the declaration can silently evaporate.",
            d.base_ref
        ));
    }
    // A fork's head commit is not fetchable from the upstream, so accepting
    // one would produce a fetch failure wearing a declaration's clothes.
    if d.head_repo_full_name != sibling_repo {
        return invalid(format!(
            "its head is on '{}', not '{sibling_repo}'. A fork's head commit is not fetchable \
             from the upstream.",
            d.head_repo_full_name
        ));
    }
    if !is_full_sha(&d.head_sha) {
        return invalid(format!(
            "its head sha {:?} is not a 40-character hex commit id.",
            d.head_sha
        ));
    }
    Ok(d.head_sha.clone())
}

/// A full 40-hex commit id — the only shape accepted as a pin, so a short or
/// abbreviated id can never be handed to `git fetch` where it would resolve
/// ambiguously.
pub(crate) fn is_full_sha(s: &str) -> bool {
    s.len() == 40 && s.chars().all(|c| c.is_ascii_hexdigit())
}

/// Destination directory for a sibling under the dispatch-scoped parent.
/// Pure — the layout rule itself.
pub(crate) fn sibling_dest(dispatch_root: &Path, sibling: &CiSibling) -> PathBuf {
    dispatch_root.join(sibling.dir_name())
}

/// The recorded-pin manifest, repo-relative — the SAME file
/// `.github/actions/checkout-sibling` reads through its `pin-file` input
/// (whose default this is) and `.github/workflows/sibling-pin-bump.yml`
/// keeps current. Deliberately a constant and not a `[[siblings]]` knob: a
/// per-lane path would let the two lanes read two files, which is the
/// divergence the pin exists to end.
pub const SIBLING_PIN_FILE: &str = ".github/sibling-pins.conf";

/// Look `repo` up in the text of [`SIBLING_PIN_FILE`].
///
/// `Ok(Some(sha))` — listed once, with a usable pin. `Ok(None)` — not listed
/// at all. `Err` — listed, but the entry is not a pin: a duplicate, a missing
/// SHA, trailing fields, or a SHA that is not 40 lowercase hex characters.
/// "Not listed" and "listed but unusable" are kept apart because conflating
/// them is a fail-open: a half-finished bump (old SHA deleted, new one never
/// pasted) would otherwise read as "never pinned" and float, while LOOKING
/// deliberate.
///
/// Field-for-field the action's `apply_pin` awk — `sub(/#.*/, "")`, `$1 ==
/// repo`, `NF` carried out beside `$2`, the 40-hex lowercase glob, every
/// match counted rather than the first taken — and the bump workflow's own
/// parser, which the #1221 pre-PR review cross-checked against the action.
/// Two programs already read this file and agree; this is the third, and it
/// must not be the one that disagrees. The agreement is held by the tests
/// below against a FIXTURE of that grammar, so a Rust-side drift fails
/// `cargo test`; a change to either awk has to be mirrored here by hand.
pub fn lookup_pin(manifest_text: &str, repo: &str) -> Result<Option<String>, String> {
    let mut matches: Vec<(usize, Vec<&str>)> = Vec::new();
    for (idx, raw) in manifest_text.lines().enumerate() {
        // `lines()` strips `\r\n` as a unit, so a CRLF-committed file is
        // already clean here; the trim covers a stray `\r` that `lines()`
        // cannot pair with a `\n`, which the bump workflow's `tr -d '\r'`
        // also removes.
        let line = raw.trim_end_matches('\r');
        let code = line.split_once('#').map_or(line, |(before, _)| before);
        // Space and tab ONLY — awk's default `FS`. `split_whitespace` would
        // also split on NBSP, form-feed and the Unicode separators, and then
        // `repo<NBSP>sha` reads as pinned here while the action reads it as
        // one unlisted token and floats, or `sha<NBSP>` reads as a clean SHA
        // here while both awks red it. NBSP arrives by copy-paste from a
        // rendered page, so this is not exotic.
        let fields: Vec<&str> = code.split([' ', '\t']).filter(|f| !f.is_empty()).collect();
        if fields.first().copied() == Some(repo) {
            matches.push((idx + 1, fields));
        }
    }
    let (lineno, fields) = match matches.as_slice() {
        [] => return Ok(None),
        [one] => one,
        many => {
            let lines: Vec<String> = many.iter().map(|(n, _)| n.to_string()).collect();
            return Err(format!(
                "{SIBLING_PIN_FILE} lists {repo} {} times (lines {}). Exactly one tree can be \
                 checked out, so the file is ambiguous about which commit CI builds against. \
                 Keep one entry and delete the rest.",
                many.len(),
                lines.join(", ")
            ));
        }
    };
    match fields.len() {
        1 => Err(format!(
            "{SIBLING_PIN_FILE} line {lineno} lists {repo} with no commit SHA after it. Refusing \
             to treat a half-finished entry as 'unpinned' and fall back to the default branch — \
             that silently reinstates the floating checkout this file exists to remove. Record a \
             full SHA, or delete the line and set pin = \"default-branch\" to float on purpose."
        )),
        2 => {
            let pin = fields[1];
            // Lowercase only — `is_full_sha` accepts either case because git
            // does, but the action's glob and the bump workflow's both spell
            // `[0-9a-f]`, and a pin the Actions lane would red must red here.
            if is_full_sha(pin) && !pin.bytes().any(|b| b.is_ascii_uppercase()) {
                Ok(Some(pin.to_string()))
            } else {
                Err(format!(
                    "{SIBLING_PIN_FILE} line {lineno} pins {repo} to {pin:?}, which is not a \
                     40-character lowercase hex commit SHA. Refusing to fall back to the default \
                     branch — that is the floating clone this pin exists to remove."
                ))
            }
        }
        nf => Err(format!(
            "{SIBLING_PIN_FILE} line {lineno}: {repo} has {nf} fields; expected exactly 2 \
             (<owner>/<repo> <40-hex-sha>). Trailing text must be a '#' comment."
        )),
    }
}

/// Read `repo`'s recorded pin out of the dispatched repo's checkout.
///
/// Both absences are hard errors here, where the action floats: the action's
/// opt-in IS the file (an unlisted sibling has simply not opted in), whereas a
/// `[[siblings]]` entry that says `pin = "pin-file"` has — so a missing entry,
/// or a missing file, contradicts the manifest that asked for it rather than
/// declining an offer. The error names the two honest ways out.
fn read_pin(worktree: &Path, repo: &str) -> Result<String, String> {
    let path = worktree.join(SIBLING_PIN_FILE);
    // Bytes, not `read_to_string`: both awks are byte-oriented and accept a
    // Latin-1 `é` in a `#` comment, so an invalid UTF-8 byte must not red
    // this lane alone. Lossy decoding cannot manufacture a 40-hex SHA.
    let bytes = std::fs::read(&path).map_err(|e| {
        format!(
            "sibling '{repo}' is pinned with pin = \"pin-file\", but {SIBLING_PIN_FILE} could \
             not be read from the dispatched checkout ({}): {e}. Refusing to resolve the sibling \
             by default branch instead — that is the floating checkout #1158 removed. Restore the \
             manifest, or set pin = \"default-branch\" to float on purpose.",
            path.display()
        )
    })?;
    let text = String::from_utf8_lossy(&bytes);
    match lookup_pin(&text, repo)? {
        Some(sha) => Ok(sha),
        None => Err(format!(
            "sibling '{repo}' is pinned with pin = \"pin-file\", but {SIBLING_PIN_FILE} does not \
             list it. Refusing to resolve it by default branch instead — that is the floating \
             checkout that held this repo's merge train for 5h+ on 2026-08-20. Add the entry \
             (gh api repos/{repo}/commits/<default-branch> --jq .sha), or set \
             pin = \"default-branch\" in .qontinui/ci.toml so the float is visible."
        )),
    }
}

/// Clone URL for a sibling slug against `base` (GitHub in production). It
/// never carries a credential: a granted sibling authenticates through a
/// URL-scoped `http.<this url>.extraHeader` in its isolated git environment
/// ([`crate::sibling_git`]), whose key is spelled from THIS function — the
/// same call the fetch's URL comes from — so the header matches exactly this
/// URL. Nothing from the node owner's git config (helper, `insteadOf`) reaches
/// the fetch.
fn clone_url_at(base: &str, repo: &str) -> String {
    format!("{base}/{repo}.git")
}

/// The production clone URL ([`clone_url_at`] on GitHub).
#[cfg(test)]
fn clone_url(repo: &str) -> String {
    clone_url_at(GITHUB_WEB, repo)
}

const GITHUB_WEB: &str = "https://github.com";
const GITHUB_API: &str = "https://api.github.com";

/// Where this module's network reads go: GitHub in production; local doubles
/// in tests. Never configurable from a manifest.
pub(crate) struct Endpoints {
    /// The REST API base (labels, PRs, token revocation).
    pub api: String,
    /// The git base a sibling is cloned from.
    pub clone: String,
}

impl Endpoints {
    pub(crate) fn github() -> Self {
        Self {
            api: GITHUB_API.to_string(),
            clone: GITHUB_WEB.to_string(),
        }
    }
}

// ──────────────────────────────── IO ──────────────────────────────────────

/// A GitHub API answer, split the way the Actions action splits it: an
/// exchange that COMPLETED (any status) is an answer; anything else is
/// UNKNOWN and every caller hard-fails on it.
struct ApiAnswer {
    status: u16,
    body: String,
}

/// One provision's GitHub door: the host's access (token source, ETag cache,
/// budget meter), one HTTP client, and the token to read with.
///
/// Two of these exist per granted sibling: the HOST one (the host's token, the
/// host's books) for reads of the dispatched repo itself, and the SIBLING one,
/// which carries the door's minted token and keeps OUT of the host's books
/// (`host_books: false`). A private PR-list body must never land in an ETag
/// cache that unauthenticated reads replay from, and the minted token spends
/// the installation's budget, not the host token's.
#[derive(Clone, Copy)]
struct Api<'a> {
    github: &'a dyn GithubAccess,
    client: &'a reqwest::Client,
    base: &'a str,
    token: Option<&'a str>,
    host_books: bool,
}

/// The ONE door every GitHub read in this module goes through, so that metering
/// and conditional requests are properties of the module rather than something
/// each call site has to remember.
///
/// Conditional: each read echoes the `ETag` the host holds, and a `304` — which
/// GitHub does NOT charge against the bucket — is answered out of the host's
/// cache and handed back to the caller as the `200` it stands in for. Every one
/// of these reads is a declaration probe against a sibling repo whose labels
/// change rarely and whose PR list changes not at all between the retries of
/// one provision, so this is the endpoint family conditional requests were
/// meant for.
///
/// A request that never produced a response is recorded as a transport error,
/// never as a charged call: it cost a socket, and dropping it would make a
/// storm of DNS failures read as "nothing happened".
async fn api_get(api: &Api<'_>, url: &str) -> Option<ApiAnswer> {
    if !api.host_books {
        return api_get_unbooked(api, url).await;
    }
    let cached_etag = api.github.cached_etag(url);

    let mut req = api
        .client
        .get(url)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "qontinui-ci");
    if let Some(t) = api.token {
        req = req.header("Authorization", format!("Bearer {t}"));
    }
    if let Some(tag) = &cached_etag {
        req = req.header(reqwest::header::IF_NONE_MATCH, tag.clone());
    }

    let resp = match req.send().await {
        Ok(r) => r,
        Err(_) => {
            api.github.record_transport_error(url);
            return None;
        }
    };

    let status = resp.status().as_u16();
    let headers = resp.headers().clone();
    api.github.record_response(url, status, &headers);

    if status == 304 {
        // Free. Replay, and report the 200 the cached body stands for — no
        // caller here has a 304 branch, and inventing one would push the cache
        // into three call sites instead of this one.
        if let Some(body) = api.github.replay(url) {
            debug!("ci: 304 on {url} — replayed from the ETag cache, no budget spent");
            return Some(ApiAnswer {
                status: 200,
                body: String::from_utf8_lossy(&body).into_owned(),
            });
        }
        // Validator held, body evicted. UNKNOWN rather than a guess: `expect_ok`
        // turns a `None` into the hard-fail these probes already take, and the
        // next provision re-reads unconditionally now that the entry is gone.
        api.github.invalidate(url);
        return None;
    }

    let etag = headers
        .get(reqwest::header::ETAG)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let body = resp.text().await.ok()?;

    match cache_decision(status, etag.as_deref()) {
        CacheDecision::Store(tag) => api.github.store(url, tag, body.as_bytes()),
        CacheDecision::Drop => api.github.invalidate(url),
    }

    Some(ApiAnswer { status, body })
}

/// A read with a minted credential: no `If-None-Match`, no cache write, no
/// replay, no host metering — none of the host's books see it.
async fn api_get_unbooked(api: &Api<'_>, url: &str) -> Option<ApiAnswer> {
    let mut req = api
        .client
        .get(url)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "qontinui-ci");
    if let Some(t) = api.token {
        req = req.header("Authorization", format!("Bearer {t}"));
    }
    let resp = req.send().await.ok()?;
    let status = resp.status().as_u16();
    let body = resp.text().await.ok()?;
    Some(ApiAnswer { status, body })
}

/// What a conditional read does to the cache entry for its URL.
#[derive(Debug, PartialEq, Eq)]
enum CacheDecision<'a> {
    /// A success carrying a validator — cache the body against it.
    Store(&'a str),
    /// A success with NO validator, or any non-success. Whatever was held is no
    /// longer known to be replayable, so stop claiming it is — the arm that
    /// keeps a deleted PR's body from being served forever.
    Drop,
}

fn cache_decision(status: u16, etag: Option<&str>) -> CacheDecision<'_> {
    match etag {
        Some(tag) if (200..300).contains(&status) => CacheDecision::Store(tag),
        _ => CacheDecision::Drop,
    }
}

/// Turn an answer into either its body or an UNKNOWN hard-fail. `404` is left
/// to the caller, because whether a 404 is semantic depends on whether the
/// thing being read was DECLARED.
fn expect_ok(what: &str, answer: Option<ApiAnswer>) -> Result<ApiAnswer, String> {
    let Some(a) = answer else {
        return Err(format!(
            "{what}: no answer at all (network/DNS/TLS). An unanswered probe is UNKNOWN, not \
             'no declaration' — refusing to guess a sibling tree."
        ));
    };
    match a.status {
        200 => Ok(a),
        403 | 429 => Err(format!(
            "{what}: rate-limited (HTTP {}). UNKNOWN, not 'no declaration'. Set GITHUB_TOKEN/\
             GH_TOKEN or authenticate `gh` on this device to raise the limit.",
            a.status
        )),
        404 => Ok(a),
        s => Err(format!(
            "{what}: unexpected HTTP {s}. UNKNOWN — refusing to guess."
        )),
    }
}

/// Read the label names on a pull request.
async fn fetch_pr_labels(api: &Api<'_>, repo: &str, number: u64) -> Result<Vec<String>, String> {
    let what = format!("reading labels on {repo}#{number}");
    let url = format!(
        "{}/repos/{repo}/issues/{number}/labels?per_page=100",
        api.base
    );
    let a = expect_ok(&what, api_get(api, &url).await)?;
    if a.status == 404 {
        return Err(format!("{what}: HTTP 404 — no such pull request."));
    }
    let arr: Vec<serde_json::Value> = serde_json::from_str(&a.body)
        .map_err(|e| format!("{what}: response was not a JSON array ({e}). UNKNOWN."))?;
    Ok(arr
        .iter()
        .filter_map(|v| v.get("name")?.as_str().map(|s| s.to_string()))
        .collect())
}

/// Read every OPEN pull request on the sibling, with labels. Paginated
/// EXPLICITLY: a silently-truncated page reads as "no declaration", which is
/// the wrong answer in the safe-looking direction.
async fn fetch_sibling_open_prs(
    api: &Api<'_>,
    sibling_repo: &str,
) -> Result<Vec<SiblingPrLabels>, String> {
    let mut out = Vec::new();
    for page in 1..=MAX_PR_PAGES {
        let what = format!("reading open pull requests on {sibling_repo} (page {page})");
        let url = format!(
            "{}/repos/{sibling_repo}/pulls?state=open&per_page=100&page={page}",
            api.base
        );
        let a = expect_ok(&what, api_get(api, &url).await)?;
        if a.status == 404 {
            return Err(format!("{what}: HTTP 404 — no such repository."));
        }
        let arr: Vec<serde_json::Value> = serde_json::from_str(&a.body)
            .map_err(|e| format!("{what}: response was not a JSON array ({e}). UNKNOWN."))?;
        let n = arr.len();
        for pr in arr {
            let Some(number) = pr.get("number").and_then(|v| v.as_u64()) else {
                continue;
            };
            let labels = pr
                .get("labels")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|l| l.get("name")?.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            out.push(SiblingPrLabels { number, labels });
        }
        if n < 100 {
            return Ok(out);
        }
    }
    Err(format!(
        "refusing to paginate {sibling_repo} open pull requests past {} entries — declaration \
         lookup is unbounded.",
        MAX_PR_PAGES * 100
    ))
}

/// Read one sibling PR's detail. A 404 here is a SEMANTIC red (the
/// declaration names a PR that does not exist), never an infra fault —
/// reporting it as infra would be the "absent vs unanswered" confusion
/// inverted.
async fn fetch_sibling_pr_detail(
    api: &Api<'_>,
    sibling_repo: &str,
    number: u64,
) -> Result<SiblingPrDetail, String> {
    let what = format!("reading {sibling_repo}#{number}");
    let url = format!("{}/repos/{sibling_repo}/pulls/{number}", api.base);
    let a = expect_ok(&what, api_get(api, &url).await)?;
    if a.status == 404 {
        return Err(format!(
            "declared adaptation {sibling_repo}#{number} is invalid: no such pull request \
             (HTTP 404). Check the number."
        ));
    }
    let v: serde_json::Value = serde_json::from_str(&a.body)
        .map_err(|e| format!("{what}: response was not a JSON object ({e}). UNKNOWN."))?;
    let s = |p: &[&str]| -> String {
        let mut cur = &v;
        for k in p {
            match cur.get(*k) {
                Some(next) => cur = next,
                None => return String::new(),
            }
        }
        cur.as_str().unwrap_or_default().to_string()
    };
    Ok(SiblingPrDetail {
        state: s(&["state"]),
        base_ref: s(&["base", "ref"]),
        head_ref: s(&["head", "ref"]),
        head_sha: s(&["head", "sha"]),
        head_repo_full_name: s(&["head", "repo", "full_name"]),
    })
}

/// Why provisioning one sibling failed, and whether the failure was a READ OF
/// THE SIBLING ITSELF — its own API read, or a git ls-remote/fetch of it. Only
/// those can be caused by the sibling being private, so only those are dressed
/// with the read door's answer and the `[[ci_readers]]` fix
/// ([`DoorNote::public_path_failure`]); anything else (the dispatched repo's
/// own label read, a malformed pin file, a local git step) is reported as it
/// came.
#[derive(Debug)]
struct SiblingFailure {
    message: String,
    sibling_read: bool,
}

impl SiblingFailure {
    /// A failed read of the sibling itself.
    fn sibling_read(message: String) -> Self {
        Self {
            message,
            sibling_read: true,
        }
    }
}

impl From<String> for SiblingFailure {
    fn from(message: String) -> Self {
        Self {
            message,
            sibling_read: false,
        }
    }
}

/// One sibling's git: the host's process posture, the sibling's isolated
/// environment, and the base its clone URL is spelled against.
struct SiblingIo<'a> {
    process: &'a dyn ProcessSpawn,
    git_env: &'a SiblingGitEnv,
    clone_base: &'a str,
}

impl SiblingIo<'_> {
    fn clone_url(&self, repo: &str) -> String {
        clone_url_at(self.clone_base, repo)
    }

    async fn git(&self, cwd: &Path, args: &[&str], timeout: Duration) -> Result<String, String> {
        run_sibling_git(self.process, cwd, args, timeout, self.git_env).await
    }
}

/// Resolve a branch tip to a SHA so both resolution paths converge on the
/// same "fetch this exact commit" primitive.
async fn ls_remote_branch_sha(
    io: &SiblingIo<'_>,
    cwd: &Path,
    repo: &str,
    branch: &str,
) -> Result<String, SiblingFailure> {
    let url = io.clone_url(repo);
    let refspec = format!("refs/heads/{branch}");
    let out = io
        .git(
            cwd,
            &["ls-remote", "--exit-code", &url, &refspec],
            SIBLING_FETCH_TIMEOUT,
        )
        .await
        .map_err(|e| SiblingFailure::sibling_read(format!("resolving {repo}@{branch}: {e}")))?;
    let sha = out
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_string();
    if !is_full_sha(&sha) {
        return Err(format!(
            "resolving {repo}@{branch}: ls-remote returned {sha:?}, not a commit id"
        )
        .into());
    }
    Ok(sha)
}

/// What a sibling resolved to, for the dispatch log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedSibling {
    pub repo: String,
    pub sha: String,
    /// Human-readable account of WHY this commit — printed into the dispatch
    /// log so "which tree did this build compile against" stays answerable.
    pub provenance: String,
}

/// The no-declaration answer — the action's `emit()` → `apply_pin` funnel.
///
/// Every exit of [`resolve_one`] that found no declaration lands here with
/// `why` saying which one, and the answer is then the recorded pin under
/// [`SiblingPin::PinFile`] or the branch tip otherwise. ONE funnel rather
/// than a pin check at each exit, for the reason the action gives for
/// putting `apply_pin` inside `emit()`: three exits is three places to forget
/// the pin, and the push-to-`main` build — the one that reds the repo — is
/// the first of them.
///
/// The pin is read LAZILY, here, so a malformed manifest cannot red a
/// dispatch whose declared adaptation would have outranked it — exactly the
/// action's `if [ -n "${SHA}" ]; then return 0; fi`.
async fn no_declaration(
    io: &SiblingIo<'_>,
    cwd: &Path,
    worktree: &Path,
    sibling: &CiSibling,
    why: &str,
) -> Result<ResolvedSibling, SiblingFailure> {
    if sibling.pin == SiblingPin::PinFile {
        let sha = read_pin(worktree, &sibling.repo)?;
        return Ok(ResolvedSibling {
            repo: sibling.repo.clone(),
            sha,
            provenance: format!("pinned in {SIBLING_PIN_FILE} ({why}; no declaration applied)"),
        });
    }
    let sha = ls_remote_branch_sha(io, cwd, &sibling.repo, &sibling.branch).await?;
    Ok(ResolvedSibling {
        repo: sibling.repo.clone(),
        sha,
        provenance: format!("{}@{} ({why})", sibling.repo, sibling.branch),
    })
}

/// Resolve one sibling to an exact commit. `worktree` is the dispatched
/// repo's checkout — where [`SIBLING_PIN_FILE`] is read from, at the
/// dispatched commit, exactly as `.qontinui/ci.toml` itself was.
///
/// `host_api` reads the DISPATCHED repo (its PR's labels) with the host's
/// token; `sibling_api` reads the SIBLING (its open PRs, one PR's detail) —
/// with the door's minted token when the sibling was granted.
#[allow(clippy::too_many_arguments)]
async fn resolve_one(
    io: &SiblingIo<'_>,
    host_api: &Api<'_>,
    sibling_api: &Api<'_>,
    cwd: &Path,
    worktree: &Path,
    sibling: &CiSibling,
    this_repo: &str,
    pr_number: Option<u64>,
) -> Result<ResolvedSibling, SiblingFailure> {
    if sibling.pin == SiblingPin::DefaultBranch {
        let sha = ls_remote_branch_sha(io, cwd, &sibling.repo, &sibling.branch).await?;
        return Ok(ResolvedSibling {
            repo: sibling.repo.clone(),
            sha,
            provenance: format!("{}@{} (pin = default-branch)", sibling.repo, sibling.branch),
        });
    }

    // Property 3: no pull request ⇒ no declaration probe, with NO api call
    // at all — then the pin or the branch, as the action's emit() does.
    let Some(pr) = pr_number else {
        return no_declaration(
            io,
            cwd,
            worktree,
            sibling,
            "no pull request on this dispatch — no declaration probe",
        )
        .await;
    };

    let this_labels = fetch_pr_labels(host_api, this_repo, pr).await?;
    let sibling_prs = fetch_sibling_open_prs(sibling_api, &sibling.repo)
        .await
        .map_err(SiblingFailure::sibling_read)?;
    let inputs = DeclarationInputs {
        this_pr_labels: this_labels,
        sibling_open_prs: sibling_prs,
    };
    match resolve_declaration(
        &sibling.repo,
        this_repo,
        pr,
        sibling.accept_trailing_sibling,
        &inputs,
    )? {
        DeclaredTarget::None => no_declaration(io, cwd, worktree, sibling, "no declaration").await,
        DeclaredTarget::TrailingSiblingDeclined { number, forms } => {
            no_declaration(
                io,
                cwd,
                worktree,
                sibling,
                &format!(
                    "declaration {}#{number} (via {}) orders {} AFTER this side; \
                     this step does not accept a trailing sibling",
                    sibling.repo,
                    forms_label(&forms),
                    short_name(&sibling.repo),
                ),
            )
            .await
        }
        DeclaredTarget::Pr { number, forms } => {
            let detail = fetch_sibling_pr_detail(sibling_api, &sibling.repo, number)
                .await
                .map_err(SiblingFailure::sibling_read)?;
            let sha = validate_declared_pr(&sibling.repo, number, &detail)?;
            Ok(ResolvedSibling {
                repo: sibling.repo.clone(),
                sha,
                provenance: format!(
                    "declared adaptation {}#{number} (via {}, head {})",
                    sibling.repo,
                    forms_label(&forms),
                    detail.head_ref
                ),
            })
        }
    }
}

/// Materialise a resolved sibling at `dest`, pinned. Mirrors #1008's
/// `git init` → `fetch --depth 1 <sha>` → `checkout FETCH_HEAD` → **assert we
/// got exactly that commit**. The assertion is the point: without it a fetch
/// that resolved something else is a silent substitution.
async fn materialise(
    io: &SiblingIo<'_>,
    dest: &Path,
    resolved: &ResolvedSibling,
) -> Result<(), SiblingFailure> {
    if dest.exists() {
        return Err(format!(
            "{} already exists — refusing to overwrite a checkout that is already there",
            dest.display()
        )
        .into());
    }
    let parent = dest
        .parent()
        .ok_or_else(|| format!("{} has no parent", dest.display()))?;
    std::fs::create_dir_all(dest).map_err(|e| format!("create {}: {e}", dest.display()))?;
    let dest_str = dest.to_string_lossy().to_string();
    let url = io.clone_url(&resolved.repo);

    io.git(parent, &["init", "-q", &dest_str], GIT_LOCAL_TIMEOUT)
        .await?;
    io.git(dest, &["remote", "add", "origin", &url], GIT_LOCAL_TIMEOUT)
        .await?;
    io.git(
        dest,
        &[
            "fetch",
            "--depth",
            "1",
            "--no-tags",
            "origin",
            &resolved.sha,
        ],
        SIBLING_FETCH_TIMEOUT,
    )
    .await
    .map_err(|e| {
        SiblingFailure::sibling_read(format!(
            "fetching {}@{}: {e}. (The commit must be reachable from {url}; a fork's head is not.)",
            resolved.repo, resolved.sha
        ))
    })?;
    io.git(dest, &["checkout", "-q", "FETCH_HEAD"], GIT_LOCAL_TIMEOUT)
        .await?;

    let got = io
        .git(dest, &["rev-parse", "HEAD"], GIT_LOCAL_TIMEOUT)
        .await?;
    if got.trim() != resolved.sha {
        return Err(format!(
            "asked for {} but checked out {} — refusing a substituted tree",
            resolved.sha,
            got.trim()
        )
        .into());
    }
    Ok(())
}

// ─────────────────────────── the read door (D3, D4) ───────────────────────────

/// The door's answer with the token taken out: what log lines and error
/// messages may name.
#[derive(Debug, Clone, PartialEq, Eq)]
enum DoorNote {
    Granted { expires_at: String },
    NoGrant { code: String },
    NotOffered { reason: String },
    Unknown { reason: String },
}

impl DoorNote {
    /// Split an answer into its loggable half and the token, if any.
    fn from_door_answer(answer: SiblingReadAnswer) -> (Self, Option<ReadToken>) {
        match answer {
            SiblingReadAnswer::Granted(token, expires_at) => {
                (Self::Granted { expires_at }, Some(token))
            }
            SiblingReadAnswer::NoGrant { code } => (Self::NoGrant { code }, None),
            SiblingReadAnswer::NotOffered { reason } => (Self::NotOffered { reason }, None),
            SiblingReadAnswer::Unknown { reason } => (Self::Unknown { reason }, None),
        }
    }

    /// The one log line per answer — sibling, answer, code, expiry; never the
    /// token.
    fn log_line(&self, repo: &str) -> String {
        match self {
            Self::Granted { expires_at } => format!(
                "[ci-node] sibling {repo}: read door answered granted (expires {expires_at})"
            ),
            Self::NoGrant { code } => {
                format!("[ci-node] sibling {repo}: read door answered no grant ({code})")
            }
            Self::NotOffered { reason } => {
                format!("[ci-node] sibling {repo}: read door not offered ({reason})")
            }
            Self::Unknown { reason } => {
                format!("[ci-node] sibling {repo}: read door answer UNKNOWN ({reason})")
            }
        }
    }

    /// A public-path failure, with D3's fix text. An UNKNOWN door is named as
    /// UNKNOWN and is never rendered as a refusal.
    fn public_path_failure(&self, err: &str, repo: &str, consumer: &str) -> String {
        let fix = format!(
            "if {repo} is private, its default branch must declare [[ci_readers]] \
             repo = \"{consumer}\""
        );
        match self {
            Self::NoGrant { code } => format!("{err}. {fix}; the door answered {code}"),
            Self::NotOffered { reason } => format!(
                "{err}. This host offers no sibling read door ({reason}), so a private \
                 sibling cannot be read here; on a coord-dispatched node, {fix}"
            ),
            Self::Unknown { reason } => format!(
                "{err}. The read door's answer was UNKNOWN ({reason}) — not a refusal, so \
                 this sibling was read on the public path without a grant. {fix}, and the \
                 door must answer"
            ),
            // Not a public-path failure; reported as it came.
            Self::Granted { .. } => err.to_string(),
        }
    }
}

/// Revoke a minted installation token with itself (`DELETE /installation/token`).
async fn revoke_token(
    client: &reqwest::Client,
    url: &str,
    token: &ReadToken,
) -> Result<(), String> {
    let resp = client
        .delete(url)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "qontinui-ci")
        .header("Authorization", format!("Bearer {}", token.secret()))
        .send()
        .await
        .map_err(|_| "no answer (network/DNS/TLS)".to_string())?;
    match resp.status().as_u16() {
        204 => Ok(()),
        s => Err(format!("HTTP {s}")),
    }
}

/// Holds a granted sibling's token until it is revoked. [`Self::revoke`] is the
/// normal path, taken after `materialise` on success AND failure; if the
/// provision is dropped mid-flight (a cancelled dispatch), `Drop` hands the
/// revocation to the runtime so a cancellation does not leave the token live
/// for its full hour.
struct RevokeGuard {
    token: Option<ReadToken>,
    client: reqwest::Client,
    url: String,
    repo: String,
    expires_at: String,
}

impl RevokeGuard {
    fn token(&self) -> &ReadToken {
        self.token
            .as_ref()
            .expect("a RevokeGuard holds its token until revoke or drop")
    }

    /// Revoke now; returns the one log line for it. Best-effort: a failure is
    /// logged, never fatal — the token still expires at `expires_at`.
    ///
    /// The token stays IN the guard until the `DELETE` has completed: a
    /// cancellation that drops this future mid-request leaves it there, and
    /// `Drop` then revokes it — taking it out first would skip revocation.
    async fn revoke(mut self) -> String {
        let Some(token) = self.token.as_ref() else {
            return String::new();
        };
        let result = revoke_token(&self.client, &self.url, token).await;
        // Completed (either way): nothing is left for Drop to revoke.
        self.token = None;
        let line = match result {
            Ok(()) => format!("[ci-node] sibling {}: read token revoked", self.repo),
            Err(e) => format!(
                "[ci-node] sibling {}: read token revocation FAILED ({e}); it expires at {}",
                self.repo, self.expires_at
            ),
        };
        info!("ci_node: {line}");
        line
    }
}

impl Drop for RevokeGuard {
    fn drop(&mut self) {
        let Some(token) = self.token.take() else {
            return;
        };
        let (client, url, repo, expires_at) = (
            self.client.clone(),
            std::mem::take(&mut self.url),
            std::mem::take(&mut self.repo),
            std::mem::take(&mut self.expires_at),
        );
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                handle.spawn(async move {
                    match revoke_token(&client, &url, &token).await {
                        Ok(()) => info!(
                            "ci_node: sibling {repo}: read token revoked (provision cancelled)"
                        ),
                        Err(e) => warn!(
                            "ci_node: sibling {repo}: read token revocation FAILED after a \
                             cancelled provision ({e}); it expires at {expires_at}"
                        ),
                    }
                });
            }
            Err(_) => warn!(
                "ci_node: sibling {repo}: no runtime to revoke its read token on; it expires \
                 at {expires_at}"
            ),
        }
    }
}

/// Provision every declared sibling into the dispatch-scoped parent.
///
/// `worktree` is the dispatched repo's checkout: its directory name is what a
/// sibling must not land on (rejected before any IO, because materialising
/// over the worktree is the one collision the "destination already exists"
/// refusal would report far too late to be legible), and it is where a
/// `pin = "pin-file"` sibling's [`SIBLING_PIN_FILE`] is read from.
///
/// Per sibling, `reads` is asked ONCE, before any read of it (D3). A granted
/// sibling is read — API and git — with the minted token, and the token is
/// revoked once that sibling is provisioned, on every exit path (D4). Every
/// other answer reads on the public path, under the same isolated git env.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn provision(
    github: &dyn GithubAccess,
    process: &dyn ProcessSpawn,
    reads: &dyn SiblingReadAccess,
    dispatch_id: &str,
    dispatch_root: &Path,
    worktree: &Path,
    siblings: &[CiSibling],
    this_repo: &str,
    pr_number: Option<u64>,
    // `+ Send` because a dispatch runs on a `tokio::spawn`ed task and this
    // callback is held across awaits.
    log: &mut (dyn FnMut(String) + Send),
) -> Result<Vec<ResolvedSibling>, String> {
    provision_at(
        &Endpoints::github(),
        github,
        process,
        reads,
        dispatch_id,
        dispatch_root,
        worktree,
        siblings,
        this_repo,
        pr_number,
        log,
    )
    .await
}

/// [`provision`] against explicit [`Endpoints`].
#[allow(clippy::too_many_arguments)]
pub(crate) async fn provision_at(
    endpoints: &Endpoints,
    github: &dyn GithubAccess,
    process: &dyn ProcessSpawn,
    reads: &dyn SiblingReadAccess,
    dispatch_id: &str,
    dispatch_root: &Path,
    worktree: &Path,
    siblings: &[CiSibling],
    this_repo: &str,
    pr_number: Option<u64>,
    log: &mut (dyn FnMut(String) + Send),
) -> Result<Vec<ResolvedSibling>, String> {
    if siblings.is_empty() {
        return Ok(Vec::new());
    }
    let worktree_dir_name = worktree
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    for s in siblings {
        if s.dir_name() == worktree_dir_name {
            return Err(format!(
                "sibling '{}' would be materialised over the dispatched repo's own worktree \
                 directory {worktree_dir_name:?}",
                s.repo
            ));
        }
    }

    let token = github.token().await;
    if token.is_none() && pr_number.is_some() {
        // Unauthenticated GitHub reads work for public repos but are limited
        // to 60/hour per IP. Say so BEFORE the rate-limit hard-fail, so the
        // fix is obvious when it happens.
        warn!(
            "ci_node: no GitHub token (GITHUB_TOKEN/GH_TOKEN or `gh auth token`) — sibling \
             declaration probes will run unauthenticated at 60 requests/hour"
        );
    }
    let client = reqwest::Client::builder()
        .timeout(API_TIMEOUT)
        .build()
        .map_err(|e| format!("build http client: {e}"))?;
    let host_api = Api {
        github,
        client: &client,
        base: &endpoints.api,
        token: token.as_deref(),
        host_books: true,
    };
    let revoke_url = format!("{}/installation/token", endpoints.api);

    // Once per provision: a git too old to honour the isolation would ignore
    // it SILENTLY and read the owner's helper again, so it is refused here.
    require_git_floor(process, dispatch_root).await?;

    let mut resolved_all = Vec::with_capacity(siblings.len());
    for sibling in siblings {
        // D3: ask first, once, before any read of this sibling.
        let (door, minted) =
            DoorNote::from_door_answer(reads.read_token(dispatch_id, &sibling.repo).await);
        let line = door.log_line(&sibling.repo);
        info!("ci_node: {line}");
        log(line);
        let guard = minted.map(|token| RevokeGuard {
            token: Some(token),
            client: client.clone(),
            url: revoke_url.clone(),
            repo: sibling.repo.clone(),
            expires_at: match &door {
                DoorNote::Granted { expires_at } => expires_at.clone(),
                _ => String::new(),
            },
        });

        let outcome = async {
            let url = clone_url_at(&endpoints.clone, &sibling.repo);
            let git_env = SiblingGitEnv::isolated(
                dispatch_root,
                guard.as_ref().map(|g| (url.as_str(), g.token())),
            )?;
            let sibling_api = match &guard {
                Some(g) => Api {
                    token: Some(g.token().secret()),
                    host_books: false,
                    ..host_api
                },
                None => host_api,
            };
            let io = SiblingIo {
                process,
                git_env: &git_env,
                clone_base: &endpoints.clone,
            };
            let resolved = resolve_one(
                &io,
                &host_api,
                &sibling_api,
                dispatch_root,
                worktree,
                sibling,
                this_repo,
                pr_number,
            )
            .await?;
            let dest = sibling_dest(dispatch_root, sibling);
            log(format!(
                "[ci-node] sibling {} -> {} @ {} [{}]",
                sibling.repo,
                dest.display(),
                resolved.sha,
                resolved.provenance
            ));
            materialise(&io, &dest, &resolved).await?;
            info!(
                "ci_node: sibling {} materialised at {} ({})",
                sibling.repo,
                dest.display(),
                resolved.sha
            );
            Ok::<_, SiblingFailure>(resolved)
        }
        .await;

        // D4: revoke on every exit path — success and failure here; a
        // cancellation (this future dropped above) through the guard's Drop.
        if let Some(guard) = guard {
            log(guard.revoke().await);
        }
        // Only a read OF THE SIBLING is dressed with the door's answer and the
        // `[[ci_readers]]` fix; every other failure is reported as it came.
        let resolved = outcome.map_err(|f| {
            if f.sibling_read {
                door.public_path_failure(&f.message, &sibling.repo, this_repo)
            } else {
                f.message
            }
        })?;
        resolved_all.push(resolved);
    }
    Ok(resolved_all)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::standalone::PlainSpawn;

    /// Shim shadowing [`super::resolve_declaration`] with the SAFE default —
    /// trailing siblings declined. Every case that predates the
    /// trailing-sibling opt-in exercises that default and reads unchanged;
    /// the cases that are ABOUT the opt-in call `super::resolve_declaration`
    /// directly with an explicit flag, so the flag is never implicit in a test
    /// whose subject it is.
    fn resolve_declaration(
        sibling_repo: &str,
        this_repo: &str,
        this_pr: u64,
        inputs: &DeclarationInputs,
    ) -> Result<DeclaredTarget, String> {
        super::resolve_declaration(sibling_repo, this_repo, this_pr, false, inputs)
    }

    fn sib(repo: &str) -> CiSibling {
        let text = format!(
            "version = 1\n[[siblings]]\nrepo = {repo:?}\n\
             [[steps]]\nname = \"x\"\ncommand = [\"true\"]\n"
        );
        crate::manifest::parse_and_validate(&text)
            .expect("test sibling must parse")
            .siblings
            .remove(0)
    }

    const SCHEMAS: &str = "qontinui/qontinui-schemas";
    const RUNNER: &str = "qontinui/qontinui-runner";

    fn inputs(this: &[&str], sibling: &[(u64, &[&str])]) -> DeclarationInputs {
        DeclarationInputs {
            this_pr_labels: this.iter().map(|s| s.to_string()).collect(),
            sibling_open_prs: sibling
                .iter()
                .map(|(n, l)| SiblingPrLabels {
                    number: *n,
                    labels: l.iter().map(|s| s.to_string()).collect(),
                })
                .collect(),
        }
    }

    // ── the layout rule ───────────────────────────────────────────────────

    /// The whole point of Phase 1: a sibling lands NEXT TO the dispatched
    /// worktree, inside the dispatch-scoped parent, so `../<sibling>` from
    /// the repo root resolves — and so cleanup is still one directory.
    #[test]
    fn sibling_is_a_worktree_sibling_inside_the_dispatch_scope() {
        let dispatch_root = Path::new("/root/.ci-worktrees/d-123");
        let dest = sibling_dest(dispatch_root, &sib(SCHEMAS));
        assert_eq!(
            dest,
            PathBuf::from("/root/.ci-worktrees/d-123/qontinui-schemas")
        );
        // Cargo's `../qontinui-schemas/rust` from <dispatch>/qontinui-coord
        // and Poetry's `../../qontinui-schemas` from
        // <dispatch>/qontinui-web/backend both name this same directory.
        let from_cargo = dispatch_root
            .join("qontinui-coord")
            .join("../qontinui-schemas");
        let from_poetry = dispatch_root
            .join("qontinui-web")
            .join("backend")
            .join("../../qontinui-schemas");
        for p in [from_cargo, from_poetry] {
            let normalised: PathBuf = p.components().fold(PathBuf::new(), |mut acc, c| match c {
                std::path::Component::ParentDir => {
                    acc.pop();
                    acc
                }
                other => {
                    acc.push(other);
                    acc
                }
            });
            assert_eq!(normalised, dest);
        }
        // Nothing escapes the cleanup unit.
        assert!(dest.starts_with(dispatch_root));
    }

    // ── declaration resolution (pure) ─────────────────────────────────────

    #[test]
    fn no_declaration_resolves_to_none() {
        let got = resolve_declaration(SCHEMAS, RUNNER, 7, &inputs(&["bug", "coord:blocked"], &[]))
            .unwrap();
        assert_eq!(got, DeclaredTarget::None);
    }

    #[test]
    fn downstream_of_on_this_pr_resolves() {
        let got = resolve_declaration(
            SCHEMAS,
            RUNNER,
            7,
            &inputs(&["coord:downstream-of=qontinui/qontinui-schemas#104"], &[]),
        )
        .unwrap();
        assert_eq!(
            got,
            DeclaredTarget::Pr {
                number: 104,
                forms: vec![DeclarationForm::DownstreamOf]
            }
        );
    }

    /// The owner is OPTIONAL in coord's grammar, so comparison is
    /// owner-stripped. An exact match here is the schemas twin's live bug.
    #[test]
    fn bare_and_owned_label_forms_both_match() {
        for spec in [
            "qontinui-schemas#104",
            "qontinui/qontinui-schemas#104",
            "someoneelse/qontinui-schemas#104",
        ] {
            let got = resolve_declaration(
                SCHEMAS,
                RUNNER,
                7,
                &inputs(&[&format!("coord:downstream-of={spec}")], &[]),
            )
            .unwrap();
            assert!(
                matches!(got, DeclaredTarget::Pr { number: 104, .. }),
                "{spec}: got {got:?}"
            );
        }
    }

    #[test]
    fn upstream_of_on_the_sibling_pr_resolves() {
        let got = resolve_declaration(
            SCHEMAS,
            RUNNER,
            7,
            &inputs(&[], &[(104, &["coord:upstream-of=qontinui-runner#7"])]),
        )
        .unwrap();
        assert_eq!(
            got,
            DeclaredTarget::Pr {
                number: 104,
                forms: vec![DeclarationForm::UpstreamOf]
            }
        );
    }

    /// THE RUNNER-LEADS DIRECTION — the case the two-form resolver could not
    /// see, and the reason runner#1019 / schemas#136 deadlocked from
    /// 2026-08-08. The Rust type is authored HERE and the sibling carries only
    /// the regenerated bindings, so THIS side must land first; the pair's only
    /// correct dep-edge direction is `coord:downstream-of=<this>#<n>` on the
    /// SIBLING PR. Both runner-leads forms must resolve the same tree the
    /// sibling-leads forms would.
    #[test]
    fn runner_leads_declarations_resolve_when_accepted() {
        // Form 4 (reverse search): coord:downstream-of on the sibling PR.
        // This is the label qontinui-schemas#136 actually carried.
        let got = super::resolve_declaration(
            SCHEMAS,
            RUNNER,
            7,
            true,
            &inputs(
                &[],
                &[(104, &["coord:downstream-of=qontinui/qontinui-runner#7"])],
            ),
        )
        .unwrap();
        assert_eq!(
            got,
            DeclaredTarget::Pr {
                number: 104,
                forms: vec![DeclarationForm::SiblingDownstreamOf]
            }
        );

        // Form 3 (self-read): coord:upstream-of on THIS PR.
        let got = super::resolve_declaration(
            SCHEMAS,
            RUNNER,
            7,
            true,
            &inputs(&["coord:upstream-of=qontinui-schemas#104"], &[]),
        )
        .unwrap();
        assert_eq!(
            got,
            DeclaredTarget::Pr {
                number: 104,
                forms: vec![DeclarationForm::SelfUpstreamOf]
            }
        );

        // And the two runner-leads forms agreeing is not a contradiction.
        let got = super::resolve_declaration(
            SCHEMAS,
            RUNNER,
            7,
            true,
            &inputs(
                &["coord:upstream-of=qontinui-schemas#104"],
                &[(104, &["coord:downstream-of=qontinui-runner#7"])],
            ),
        )
        .unwrap();
        assert_eq!(
            got,
            DeclaredTarget::Pr {
                number: 104,
                forms: vec![
                    DeclarationForm::SelfUpstreamOf,
                    DeclarationForm::SiblingDownstreamOf
                ]
            }
        );
    }

    /// The safe default. A compile step must NOT build against a tree that
    /// lands after it — but the outcome must still name itself, so a declined
    /// trailing sibling is distinguishable in the log from "nobody declared
    /// anything". Same inputs as the test above, flag off.
    #[test]
    fn a_trailing_sibling_is_declined_by_default_and_says_so() {
        let got = super::resolve_declaration(
            SCHEMAS,
            RUNNER,
            7,
            false,
            &inputs(
                &[],
                &[(104, &["coord:downstream-of=qontinui/qontinui-runner#7"])],
            ),
        )
        .unwrap();
        assert_eq!(
            got,
            DeclaredTarget::TrailingSiblingDeclined {
                number: 104,
                forms: vec![DeclarationForm::SiblingDownstreamOf]
            }
        );

        let got = super::resolve_declaration(
            SCHEMAS,
            RUNNER,
            7,
            false,
            &inputs(&["coord:upstream-of=qontinui-schemas#104"], &[]),
        )
        .unwrap();
        assert_eq!(
            got,
            DeclaredTarget::TrailingSiblingDeclined {
                number: 104,
                forms: vec![DeclarationForm::SelfUpstreamOf]
            }
        );

        // A LEADING sibling is unaffected by the flag — it is what main will
        // contain when this change lands, in either setting.
        let got = super::resolve_declaration(
            SCHEMAS,
            RUNNER,
            7,
            false,
            &inputs(&["coord:downstream-of=qontinui-schemas#104"], &[]),
        )
        .unwrap();
        assert!(matches!(got, DeclaredTarget::Pr { number: 104, .. }));
    }

    /// Declaring BOTH merge orders for one pair is a cycle in coord's dep
    /// graph — both PRs are held forever. Resolving the (unambiguous) tree and
    /// going green would hide it, so it reds instead.
    #[test]
    fn both_merge_orders_declared_is_a_cycle_and_hard_fails() {
        let err = resolve_declaration(
            SCHEMAS,
            RUNNER,
            7,
            &inputs(
                &["coord:downstream-of=qontinui-schemas#104"],
                &[(104, &["coord:downstream-of=qontinui-runner#7"])],
            ),
        )
        .unwrap_err();
        assert!(err.contains("merge ORDER"), "got: {err}");
        assert!(err.contains("cycle"), "got: {err}");
        assert!(err.contains("#104"), "got: {err}");
    }

    /// One declaration satisfies both repos' gates, so both forms agreeing is
    /// the common case — and must not read as a contradiction.
    #[test]
    fn both_forms_agreeing_is_not_a_contradiction() {
        let got = resolve_declaration(
            SCHEMAS,
            RUNNER,
            7,
            &inputs(
                &["coord:downstream-of=qontinui-schemas#104"],
                &[(104, &["coord:upstream-of=qontinui-runner#7"])],
            ),
        )
        .unwrap();
        assert_eq!(
            got,
            DeclaredTarget::Pr {
                number: 104,
                forms: vec![DeclarationForm::DownstreamOf, DeclarationForm::UpstreamOf]
            }
        );
    }

    /// Ways to be ambiguous; all hard-fail alike, because exactly one tree can
    /// be checked out and picking one would resolve an ambiguity that is the
    /// author's to resolve. (Declaring both merge ORDERS is a fourth failure,
    /// covered separately by
    /// `both_merge_orders_declared_is_a_cycle_and_hard_fails` — it is a cycle
    /// rather than an ambiguity about WHICH tree.)
    #[test]
    fn ambiguities_hard_fail() {
        // (1) two downstream-of labels on this PR.
        let err = resolve_declaration(
            SCHEMAS,
            RUNNER,
            7,
            &inputs(
                &[
                    "coord:downstream-of=qontinui-schemas#104",
                    "coord:downstream-of=qontinui-schemas#105",
                ],
                &[],
            ),
        )
        .unwrap_err();
        assert!(err.contains("contradictory"), "got: {err}");
        assert!(err.contains("#104") && err.contains("#105"), "got: {err}");

        // (2) two sibling PRs both claiming this one.
        let err = resolve_declaration(
            SCHEMAS,
            RUNNER,
            7,
            &inputs(
                &[],
                &[
                    (104, &["coord:upstream-of=qontinui-runner#7"]),
                    (105, &["coord:upstream-of=qontinui-runner#7"]),
                ],
            ),
        )
        .unwrap_err();
        assert!(err.contains("contradictory"), "got: {err}");

        // (3) the two forms naming DIFFERENT PRs.
        let err = resolve_declaration(
            SCHEMAS,
            RUNNER,
            7,
            &inputs(
                &["coord:downstream-of=qontinui-schemas#104"],
                &[(105, &["coord:upstream-of=qontinui-runner#7"])],
            ),
        )
        .unwrap_err();
        assert!(err.contains("contradictory"), "got: {err}");
    }

    /// Duplicates of the SAME declaration are not a contradiction.
    #[test]
    fn duplicate_identical_declarations_are_not_ambiguous() {
        let got = resolve_declaration(
            SCHEMAS,
            RUNNER,
            7,
            &inputs(
                &[
                    "coord:downstream-of=qontinui-schemas#104",
                    "coord:downstream-of=qontinui/qontinui-schemas#104",
                ],
                &[],
            ),
        )
        .unwrap();
        assert!(matches!(got, DeclaredTarget::Pr { number: 104, .. }));
    }

    /// Labels naming a DIFFERENT sibling, or a different PR of this repo, are
    /// simply not ours.
    #[test]
    fn unrelated_declarations_are_ignored() {
        let got = resolve_declaration(
            SCHEMAS,
            RUNNER,
            7,
            &inputs(
                &["coord:downstream-of=some-other-repo#1"],
                &[
                    (104, &["coord:upstream-of=qontinui-runner#8"]),
                    (105, &["coord:upstream-of=qontinui-web#7"]),
                ],
            ),
        )
        .unwrap();
        assert_eq!(got, DeclaredTarget::None);
        // `#12` must not match `#123`.
        let got = resolve_declaration(
            SCHEMAS,
            RUNNER,
            12,
            &inputs(&[], &[(104, &["coord:upstream-of=qontinui-runner#123"])]),
        )
        .unwrap();
        assert_eq!(got, DeclaredTarget::None);
    }

    /// A malformed declaration ON THIS PR is the author's error and reds; a
    /// malformed label on somebody else's PR is not ours to red.
    #[test]
    fn malformed_labels_red_only_when_they_are_ours() {
        for bad in [
            "coord:downstream-of=qontinui-schemas#not-a-number",
            "coord:downstream-of=qontinui-schemas#",
        ] {
            let err = resolve_declaration(SCHEMAS, RUNNER, 7, &inputs(&[bad], &[])).unwrap_err();
            assert!(err.contains("malformed label"), "{bad}: got {err}");
        }
        let got = resolve_declaration(
            SCHEMAS,
            RUNNER,
            7,
            &inputs(&[], &[(104, &["coord:upstream-of=garbage"])]),
        )
        .unwrap();
        assert_eq!(got, DeclaredTarget::None);
    }

    // ── declaration validation (pure) ─────────────────────────────────────

    fn good_detail() -> SiblingPrDetail {
        SiblingPrDetail {
            state: "open".into(),
            base_ref: "main".into(),
            head_ref: "feat/x".into(),
            head_sha: "a".repeat(40),
            head_repo_full_name: SCHEMAS.into(),
        }
    }

    #[test]
    fn a_valid_declaration_pins_to_its_head_sha() {
        let sha = validate_declared_pr(SCHEMAS, 104, &good_detail()).unwrap();
        assert_eq!(sha, "a".repeat(40));
    }

    #[test]
    fn declaration_validation_rejections() {
        let cases: Vec<(SiblingPrDetail, &str)> = vec![
            (
                SiblingPrDetail {
                    state: "closed".into(),
                    ..good_detail()
                },
                "not 'open'",
            ),
            (
                SiblingPrDetail {
                    base_ref: "feat/parent".into(),
                    ..good_detail()
                },
                "not 'main'",
            ),
            (
                SiblingPrDetail {
                    head_repo_full_name: "fork/qontinui-schemas".into(),
                    ..good_detail()
                },
                "not fetchable",
            ),
            (
                SiblingPrDetail {
                    head_sha: "abc123".into(),
                    ..good_detail()
                },
                "40-character hex",
            ),
        ];
        for (detail, needle) in cases {
            let err = validate_declared_pr(SCHEMAS, 104, &detail).unwrap_err();
            assert!(err.contains(needle), "expected {needle:?}, got: {err}");
            // Every rejection says it is about the DECLARATION, so nobody
            // reads it as an infra flake and retries.
            assert!(err.contains("declared adaptation"), "got: {err}");
        }
    }

    /// Only a full 40-hex id may be handed to `git fetch` as a pin.
    #[test]
    fn sha_shape_gate() {
        assert!(is_full_sha(&"0".repeat(40)));
        assert!(is_full_sha("a543cc7e1fab11c83299b3f8648a4686f223a093"));
        assert!(!is_full_sha("a543cc7e1"));
        assert!(!is_full_sha(&"z".repeat(40)));
        assert!(!is_full_sha(""));
    }

    /// The clone URL never carries a credential — not for a public sibling,
    /// and not for a granted one either: the grant travels in the isolated
    /// env's URL-scoped header, so the URL (and the remote git records from
    /// it) holds no userinfo.
    #[test]
    fn clone_urls_carry_no_credential() {
        assert_eq!(
            clone_url(SCHEMAS),
            "https://github.com/qontinui/qontinui-schemas.git"
        );
        let token = crate::sibling_git::ReadToken::new("ghs_GRANTEDtoken");
        let dir = tempfile::tempdir().unwrap();
        let url = clone_url(SCHEMAS);
        let env = SiblingGitEnv::isolated(dir.path(), Some((&url, &token))).unwrap();
        let authority = url
            .strip_prefix("https://")
            .and_then(|rest| rest.split('/').next())
            .unwrap();
        assert_eq!(authority, "github.com", "userinfo in {url}");
        assert!(!url.contains("ghs_GRANTEDtoken"));
        assert!(!format!("{env:?}").contains("ghs_GRANTEDtoken"));
    }

    /// git matches `http.<url>.*` on whole path segments, so the header key is
    /// asserted the way git itself reads it — `git config --get-urlmatch`
    /// under the BUILT env — not by comparing strings: it must match the
    /// granted sibling's clone URL, and must not match a second repo's.
    #[tokio::test]
    async fn the_auth_header_matches_the_granted_clone_url_and_no_other() {
        let token = crate::sibling_git::ReadToken::new("ghs_URLMATCHtoken");
        let dir = tempfile::tempdir().unwrap();
        let granted = clone_url("qontinui/qontinui-docs-reference");
        let env = SiblingGitEnv::isolated(dir.path(), Some((&granted, &token))).unwrap();
        let urlmatch = |url: String| {
            let mut cmd = crate::sibling_git::sibling_git_command(&PlainSpawn, &env);
            cmd.current_dir(dir.path())
                .args(["config", "--get-urlmatch", "http.extraHeader"])
                .arg(url);
            async move { cmd.output().await.unwrap() }
        };
        let hit = urlmatch(granted.clone()).await;
        assert!(hit.status.success(), "no header for the granted URL");
        let value = String::from_utf8_lossy(&hit.stdout).trim().to_string();
        assert!(
            value.starts_with("Authorization: Basic "),
            "got a non-auth value"
        );
        for other in [
            clone_url("qontinui/qontinui-docs-evals"),
            clone_url("qontinui/qontinui-docs-reference-fork"),
            "https://github.com/qontinui/qontinui-docs-reference".to_string(),
            "https://example.com/qontinui/qontinui-docs-reference.git".to_string(),
        ] {
            let miss = urlmatch(other.clone()).await;
            assert!(
                String::from_utf8_lossy(&miss.stdout).trim().is_empty(),
                "the header leaked to {other}"
            );
        }
    }

    // ── the recorded pin (`.github/sibling-pins.conf`) ──────────────────

    const UI_BRIDGE: &str = "qontinui/ui-bridge";
    const PIN_A: &str = "0aedd171fc4ef777ab4da6c08b75fb806b0bd414";
    const PIN_B: &str = "ec64cbb1e2d7174107bd753c704b730b34872833";

    /// The shape the real manifest has: a comment header, blank lines, a
    /// trailing `# …` after an entry, and CRLF line endings the action's
    /// `tr -d '\r'` tolerates. Every one of those must parse to the SAME
    /// answer the action reaches, or the two lanes disagree about one file.
    #[test]
    fn pin_lookup_reads_the_file_as_the_action_does() {
        let text = format!(
            "# header comment\r\n\r\n{UI_BRIDGE}      {PIN_A}   # trailing note\r\n\
             qontinui/qontinui-web   {PIN_B}\r\n# {UI_BRIDGE} deadbeef (commented out)\r\n"
        );
        assert_eq!(
            lookup_pin(&text, UI_BRIDGE).unwrap().as_deref(),
            Some(PIN_A)
        );
        assert_eq!(
            lookup_pin(&text, "qontinui/qontinui-web")
                .unwrap()
                .as_deref(),
            Some(PIN_B)
        );
        // Not listed is a distinct answer from listed-but-unusable, and it is
        // the ONLY absence this parser reports as `None`.
        assert_eq!(lookup_pin(&text, SCHEMAS).unwrap(), None);
        assert_eq!(lookup_pin("", UI_BRIDGE).unwrap(), None);
        // `$1 == repo` is exact: a prefix, a different owner, or the short
        // name alone is not the entry.
        assert_eq!(lookup_pin(&text, "ui-bridge").unwrap(), None);
        assert_eq!(lookup_pin(&text, "other/ui-bridge").unwrap(), None);
        // awk's default FS is space and tab. An NBSP (the copy-paste
        // separator) is NOT a separator to either awk, so `repo<NBSP>sha`
        // is one token that equals no repo — unlisted, exactly as the action
        // reads it — and never a pin.
        assert_eq!(
            lookup_pin(&format!("{UI_BRIDGE}\u{a0}{PIN_A}\n"), UI_BRIDGE).unwrap(),
            None
        );
        // A tab IS a separator, as in the real file.
        assert_eq!(
            lookup_pin(&format!("{UI_BRIDGE}\t{PIN_A}\n"), UI_BRIDGE)
                .unwrap()
                .as_deref(),
            Some(PIN_A)
        );
    }

    /// The fail-open the action refuses, refused here too: a listed entry
    /// whose value is not a pin is an ERROR, never `None` — otherwise the
    /// ordinary "delete the old SHA, paste the new one" bump, with the paste
    /// missed, floats the sibling while looking deliberate.
    #[test]
    fn pin_lookup_rejects_unusable_entries_instead_of_floating() {
        let cases: [(&str, &str); 7] = [
            (
                "listed twice",
                &format!("{UI_BRIDGE} {PIN_A}\n{UI_BRIDGE} {PIN_B}\n"),
            ),
            ("no sha", &format!("{UI_BRIDGE}\n")),
            (
                "sha commented out mid-edit",
                &format!("{UI_BRIDGE}   # {PIN_A}\n"),
            ),
            ("three fields", &format!("{UI_BRIDGE} {PIN_A} extra\n")),
            ("short sha", &format!("{UI_BRIDGE} 0aedd171f\n")),
            (
                "uppercase sha",
                &format!("{UI_BRIDGE} {}\n", PIN_A.to_ascii_uppercase()),
            ),
            // Both awks read `sha<NBSP>` as a 41-byte second field that fails
            // the 40-hex glob; `split_whitespace` would have eaten the NBSP
            // and passed a pin the Actions lane reds.
            (
                "trailing NBSP glued to the sha",
                &format!("{UI_BRIDGE} {PIN_A}\u{a0}\n"),
            ),
        ];
        for (what, text) in cases {
            let err = lookup_pin(text, UI_BRIDGE)
                .expect_err(&format!("{what}: must be an error, not a float"));
            assert!(
                err.contains(SIBLING_PIN_FILE) && err.contains(UI_BRIDGE),
                "{what}: the error must name the file and the sibling, got: {err}"
            );
        }
        // A duplicate names every line it found, so the author can see the
        // ambiguity rather than hunt for it.
        let err = lookup_pin(
            &format!("# x\n{UI_BRIDGE} {PIN_A}\n\n{UI_BRIDGE} {PIN_B}\n"),
            UI_BRIDGE,
        )
        .unwrap_err();
        assert!(
            err.contains("2 times") && err.contains("lines 2, 4"),
            "got: {err}"
        );
    }

    /// `read_pin` is the opt-in's other half: under `pin = "pin-file"` the
    /// file being absent, or the sibling being absent from it, contradicts
    /// the manifest rather than declining an offer — so both are hard errors
    /// that name the way out, where the action would float.
    #[test]
    fn pin_file_absence_is_an_error_under_pin_file_not_a_float() {
        let dir = tempfile::tempdir().unwrap();
        let err = read_pin(dir.path(), UI_BRIDGE).unwrap_err();
        assert!(
            err.contains("could not be read") && err.contains("pin = \"default-branch\""),
            "got: {err}"
        );

        let manifest = dir.path().join(SIBLING_PIN_FILE);
        std::fs::create_dir_all(manifest.parent().unwrap()).unwrap();
        std::fs::write(&manifest, format!("qontinui/qontinui-web {PIN_B}\n")).unwrap();
        let err = read_pin(dir.path(), UI_BRIDGE).unwrap_err();
        assert!(
            err.contains("does not list it") && err.contains("pin = \"default-branch\""),
            "got: {err}"
        );

        std::fs::write(
            &manifest,
            format!("qontinui/qontinui-web {PIN_B}\n{UI_BRIDGE} {PIN_A}\n"),
        )
        .unwrap();
        assert_eq!(read_pin(dir.path(), UI_BRIDGE).unwrap(), PIN_A);
    }

    /// The no-declaration funnel: under `pin-file` every exit that found no
    /// declaration answers with the recorded commit and says so in the
    /// provenance, with NO network call — the branch is never consulted.
    #[tokio::test]
    async fn no_declaration_answers_with_the_recorded_pin() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = dir.path().join(SIBLING_PIN_FILE);
        std::fs::create_dir_all(manifest.parent().unwrap()).unwrap();
        std::fs::write(&manifest, format!("{UI_BRIDGE} {PIN_A}\n")).unwrap();

        let mut sibling = sib(UI_BRIDGE);
        sibling.pin = SiblingPin::PinFile;
        // The SHA assertion below is what catches a regression that consulted
        // the branch: `ls-remote` needs no repository cwd, so on a networked
        // box it would answer with ui-bridge's real tip, which is not PIN_A.
        let git_env = SiblingGitEnv::isolated(dir.path(), None).unwrap();
        let io = SiblingIo {
            process: &PlainSpawn,
            git_env: &git_env,
            clone_base: GITHUB_WEB,
        };
        let resolved = no_declaration(&io, dir.path(), dir.path(), &sibling, "no declaration")
            .await
            .unwrap();
        assert_eq!(resolved.sha, PIN_A);
        assert_eq!(resolved.repo, UI_BRIDGE);
        assert!(
            resolved.provenance.contains(SIBLING_PIN_FILE)
                && resolved.provenance.contains("no declaration"),
            "got: {}",
            resolved.provenance
        );

        // A half-finished bump reaches the dispatch as the manifest's own
        // error, not as a float: the branch is still never consulted.
        std::fs::write(&manifest, format!("{UI_BRIDGE}\n")).unwrap();
        let err = no_declaration(&io, dir.path(), dir.path(), &sibling, "no declaration")
            .await
            .unwrap_err();
        assert!(!err.sibling_read, "a pin-file error is not a sibling read");
        let err = err.message;
        assert!(err.contains("no commit SHA after it"), "got: {err}");
    }

    /// The conditional-read cache keeps only a validated success, and drops
    /// whatever it held on anything else — otherwise a deleted PR's body is
    /// replayed forever off an ETag GitHub will happily keep matching.
    #[test]
    fn cache_keeps_only_a_validated_success() {
        assert_eq!(
            cache_decision(200, Some("W/\"x\"")),
            CacheDecision::Store("W/\"x\"")
        );
        assert_eq!(cache_decision(201, Some("e")), CacheDecision::Store("e"));
        assert_eq!(cache_decision(200, None), CacheDecision::Drop);
        for status in [301, 401, 403, 404, 422, 500, 502] {
            assert_eq!(
                cache_decision(status, Some("e")),
                CacheDecision::Drop,
                "{status}"
            );
        }
    }
}

#[cfg(test)]
mod door_tests;
