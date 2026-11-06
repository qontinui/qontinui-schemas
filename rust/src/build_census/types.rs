//! The vocabulary of the build census: readings, the owner class and the
//! inputs the classifier is allowed to look at. Wire-shaped (`serde`).

use serde::{Deserialize, Serialize};

pub use crate::build_admission::GIB;

/// A measurement that may not have been readable. `Unknown` is carried
/// explicitly so it can never be mistaken for a measured zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "provenance", content = "value", rename_all = "snake_case")]
pub enum Reading<T> {
    /// A value the census read.
    Measured(T),
    /// Should be readable and was not.
    Unknown,
}

impl<T: Copy> Reading<T> {
    /// The value when there is one, `None` for `Unknown`.
    pub fn value(&self) -> Option<T> {
        match *self {
            Reading::Measured(v) => Some(v),
            Reading::Unknown => None,
        }
    }
}

impl Reading<f64> {
    /// The value when it is a finite number: a NaN or infinite "measurement"
    /// is a broken probe, so it reads as `None` rather than comparing false
    /// against every threshold (which would look calm). Mirrors
    /// `build_admission::Fact::<f64>::resolve_finite`.
    pub fn value_finite(&self) -> Option<f64> {
        self.value().filter(|v| v.is_finite())
    }
}

/// Why a build could not be attributed. Closed: every `Unknown` owner says why.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnknownReason {
    /// The ancestry could not be read at all.
    AncestryUnreadable,
    /// The build or the ancestor that decides its owner has no creation time,
    /// so a reused pid cannot be ruled out: not proof of any owner.
    UndatedAncestor,
    /// The nearest non-build ancestor is an IDE's checker (rust-analyzer):
    /// nobody's verification build, so not charged to a human.
    IdeCheck,
    /// The build is in a CI slice but no runner name could be read.
    CiRunnerUnnamed,
    /// Init-parented but its age could not be read, so the orphan bound
    /// cannot be applied.
    AgeUnreadable,
    /// Init-parented and not older than the orphan bound: its owner may be
    /// a daemonised launcher still alive, so it is not yet an orphan.
    InitParentedWithinBound,
    /// Shell or sshd ancestry but no controlling tty: a script, not proof of
    /// a person.
    NoTty,
    /// The runner process launched it: neither a person's nor an agent's
    /// verification build.
    RunnerSpawned,
    /// The census could not say whether an agent is above the build.
    NestingUnreadable,
    /// Init-parented but still holding a tty or human evidence (a detached
    /// build a person may be attached to): not an orphan.
    InitParentedWithTty,
    /// The build's cgroup could not be read, and it might be the agent or the
    /// user slice: neither Orphan nor InteractiveHuman on that evidence.
    PlacementUnreadable,
    /// The tty probe failed: "no controlling tty" was not established.
    TtyUnreadable,
    /// No positive evidence of any owner.
    NoOwnerEvidence,
}

/// Who a build is for (D1). Closed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "class", rename_all = "snake_case")]
pub enum OwnerClass {
    /// A live agent session's build. `session_id` is known when the owner
    /// record the build scope writes named it.
    AgentSession { session_id: Option<String> },
    /// A CI job's build, on the named runner.
    CiJob { runner: String },
    /// A person's own build at the named controlling tty.
    InteractiveHuman { tty: String },
    /// Init-parented and older than the orphan bound; `owner_dead_since` is
    /// the build's age in seconds when it was found so (a lower bound on how
    /// long its owner has been gone).
    Orphan { owner_dead_since: u64 },
    /// Not attributable, for the stated reason. Counted at full cost.
    Unknown { reason: UnknownReason },
}

/// An [`OwnerClass`] without its payload, for rollup keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnerKind {
    /// See [`OwnerClass::AgentSession`].
    AgentSession,
    /// See [`OwnerClass::CiJob`].
    CiJob,
    /// See [`OwnerClass::InteractiveHuman`].
    InteractiveHuman,
    /// See [`OwnerClass::Orphan`].
    Orphan,
    /// See [`OwnerClass::Unknown`].
    Unknown,
}

impl OwnerKind {
    /// Every kind, in rollup order.
    pub const ALL: [OwnerKind; 5] = [
        OwnerKind::AgentSession,
        OwnerKind::CiJob,
        OwnerKind::InteractiveHuman,
        OwnerKind::Orphan,
        OwnerKind::Unknown,
    ];
}

impl OwnerClass {
    /// The payload-free kind.
    pub fn kind(&self) -> OwnerKind {
        match self {
            OwnerClass::AgentSession { .. } => OwnerKind::AgentSession,
            OwnerClass::CiJob { .. } => OwnerKind::CiJob,
            OwnerClass::InteractiveHuman { .. } => OwnerKind::InteractiveHuman,
            OwnerClass::Orphan { .. } => OwnerKind::Orphan,
            OwnerClass::Unknown { .. } => OwnerKind::Unknown,
        }
    }
}

/// What an ancestor process is, as the census already classifies it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AncestorKind {
    /// Another build process (cargo, rustc, a build script).
    Build,
    /// A build driver hop (a wrapper, `make`, a script runner).
    BuildHop,
    /// An agent CLI process.
    Agent,
    /// The product's runner process.
    Runner,
    /// An IDE's language server or checker.
    Ide,
    /// An interactive shell.
    Shell,
    /// An sshd session process.
    Sshd,
    /// A terminal multiplexer.
    Tmux,
    /// Pid 1 or a subreaper: the build was re-parented.
    Init,
    /// Anything else.
    Other,
}

/// One ancestor, nearest first in [`Ancestry::Readable`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ancestor {
    /// What the ancestor is.
    pub kind: AncestorKind,
    /// Whether it has a creation time. Without one a reused pid cannot be
    /// ruled out, so it proves nothing.
    pub dated: bool,
}

/// The build's process ancestry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "readable", content = "ancestors", rename_all = "snake_case")]
pub enum Ancestry {
    /// The process table could not be walked.
    Unreadable,
    /// Nearest ancestor first.
    Readable(Vec<Ancestor>),
}

/// The kind of slice a build's cgroup sits in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SliceKind {
    /// The slice the agent verification wrapper places builds in.
    AgentBuilds,
    /// The slice the PATH shim places human and unattributed builds in.
    UserBuilds,
    /// A CI runner's slice.
    Ci,
    /// Any other slice (a session's, the user's, the root).
    Other,
}

/// Where the build's cgroup is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "placement", content = "slice", rename_all = "snake_case")]
pub enum Placement {
    /// In a classified slice.
    Slice(SliceKind),
    /// The cgroup path could not be read.
    Unreadable,
}

/// Every fact [`crate::build_census::classify_owner`] may look at. Nothing
/// else about a build is an input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnerInputs {
    /// The process ancestry.
    pub ancestry: Ancestry,
    /// Whether the build itself has a creation time.
    pub self_dated: bool,
    /// The census's `nested_under_agent` flag; `None` when it could not say.
    pub nested_under_agent: Option<bool>,
    /// The session id the build scope's owner record names, if one was read.
    pub owner_record_session: Option<String>,
    /// The controlling tty: `Measured(None)` is "read, and there is none";
    /// `Unknown` is "the probe failed" and is never read as no tty.
    pub tty: Reading<Option<String>>,
    /// The CI runner's name, when the build is in a CI slice.
    pub ci_runner: Option<String>,
    /// The build's cgroup placement.
    pub placement: Placement,
    /// The build's age in seconds; `None` when it has no creation time.
    pub age_s: Option<u64>,
    /// An init-parented build older than this is an orphan.
    pub orphan_bound_s: u64,
}
