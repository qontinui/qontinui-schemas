//! The `[canonical]` gate: let a dispatch require the box to be at the
//! canonical configuration for the toolchains it builds with, and let
//! the host's convergence machinery satisfy it.
//!
//! Plan `2026-08-08-ci-tool-registry-and-canonical-configuration-parity`,
//! Phase 2.
//!
//! # The measuring and the acting belong to the host
//!
//! This module decides **whether** the requirement is met and **whether**
//! convergence is permitted. It does not measure a box or drive a version
//! manager itself: [`CanonicalConvergence::measure`] reports how each declared
//! toolchain stands, and [`CanonicalConvergence::converge`] closes drift. On the
//! runner both are its `env_agent` machinery (the pull-and-plan, and the
//! rustup/volta/pyenv apply, run on the blocking pool because a toolchain
//! download is hundreds of megabytes); a host with no canonical source answers
//! `measure` with an error, which this gate refuses on.
//!
//! # Two decisions the plan said must not be left implicit
//!
//! **1. A drifted box with no convergence authority is REFUSED, not run
//! anyway.** Both are defensible and the plan says so; leaving it implicit is
//! the only wrong answer. Refusing wins because the alternative makes the
//! declaration unenforceable: a required check could go green on a box that is
//! not at canonical, and "this build requires canonical" would then mean
//! nothing a repo could rely on. It also matches the sibling decision Phase 4
//! already took for services — a database-backed gate without a container
//! runtime fails loudly rather than turning green — and this lane's standing
//! rule that silence is never success.
//!
//! **2. The requirement comes from the repo; the authority comes from the
//! box.** A manifest declaring `[canonical]` never authorises mutating the
//! owner's global toolchain. [`crate::host::CiSettings::canonical_converge`]
//! does, it defaults to false on every host, and on the runner only the
//! owner-authored coord directive can set it.
//! With it false the dispatch still does not silently build: it refuses, naming
//! the drift.
//!
//! # Agreement is read, never inferred
//!
//! "Is this box at canonical for `rustc`?" is answered from
//! [`Standing::Agreed`] — the positive record of a key both captures carry
//! with the same value — and never from `rustc` failing to appear in a change
//! list. An empty change list is silence, and silence also covers the case
//! where NEITHER capture contains `rustc`: a box with no Rust at all, against a
//! canonical machine with no Rust either, produces no diff row. Gating on the
//! absence would pass that box, which is the reports-success-while-measuring-
//! nothing failure this whole plan exists to remove.

use serde_json::{json, Value};

use crate::host::{CanonicalConvergence, Standing, VersionsReading};
use crate::manifest::CiCanonical;

/// The `versions` section is the only one this gate reads.
pub const VERSIONS_SECTION: &str = "versions";

/// What the gate concluded, and the provenance that goes into the verdict.
///
/// Every dispatch produces one of these — including a dispatch that declared
/// nothing, which produces [`Outcome::not_requested`]. A consumer reading the
/// result should never have to infer a toolchain claim from a missing field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    status: Status,
    /// Per-toolchain provenance: what this box actually ran under.
    toolchains: Vec<ToolchainRecord>,
    canonical_machine: Option<String>,
    /// Present only on [`Status::Refused`].
    reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Status {
    /// The manifest declared no `[canonical]`.
    NotRequested,
    /// This box IS the canonical machine.
    IsCanonical,
    /// Every declared toolchain already matched canonical.
    Satisfied,
    /// Declared toolchains drifted; convergence ran and closed the drift.
    Converged,
    /// The requirement is not met and the dispatch must not run.
    Refused,
}

impl Status {
    fn wire(self) -> &'static str {
        match self {
            Self::NotRequested => "not_requested",
            Self::IsCanonical => "is_canonical",
            Self::Satisfied => "satisfied",
            Self::Converged => "converged",
            Self::Refused => "refused",
        }
    }
}

/// One declared toolchain's provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ToolchainRecord {
    key: String,
    /// The value this box reports, once the gate is done with it. `None` when
    /// the box could not be measured — which is itself a refusal, never a pass.
    local: Option<String>,
    canonical: Option<String>,
    /// `agreed` | `converged` | `drifted` | `unmeasured` | `no_canonical_value`
    /// | `absent_both`.
    state: &'static str,
}

impl Outcome {
    /// The dispatch declared nothing. Recorded EXPLICITLY rather than omitted:
    /// an absent field would make "no claim was made" and "a claim was made and
    /// this consumer does not know about it" the same bytes.
    pub fn not_requested() -> Self {
        Self {
            status: Status::NotRequested,
            toolchains: Vec::new(),
            canonical_machine: None,
            reason: None,
        }
    }

    /// True when the dispatch must not proceed.
    pub fn is_refusal(&self) -> bool {
        self.status == Status::Refused
    }

    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }

    /// One line for the dispatch log.
    pub fn summary_line(&self) -> String {
        match self.status {
            Status::NotRequested => {
                "[ci-node] canonical: not required by this manifest".to_string()
            }
            Status::Refused => format!(
                "[ci-node] canonical: REFUSED — {}",
                self.reason.as_deref().unwrap_or("no reason recorded")
            ),
            _ => format!(
                "[ci-node] canonical: {} ({})",
                self.status.wire(),
                self.render_toolchains()
            ),
        }
    }

    fn render_toolchains(&self) -> String {
        self.toolchains
            .iter()
            .map(|t| {
                format!(
                    "{}={} [{}]",
                    t.key,
                    t.local.as_deref().unwrap_or("<unmeasured>"),
                    t.state
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// The verdict block. Lands under `summary.canonical` in the result POST,
    /// which is what coord persists with the dispatch — so a green produced
    /// under a converged toolchain is distinguishable from one produced at
    /// canonical, and both from one that never made the claim.
    pub fn to_json(&self) -> Value {
        json!({
            "status": self.status.wire(),
            "canonical_machine": self.canonical_machine,
            "reason": self.reason,
            "toolchains": self.toolchains.iter().map(|t| json!({
                "key": t.key,
                "local": t.local,
                "canonical": t.canonical,
                "state": t.state,
            })).collect::<Vec<_>>(),
        })
    }

    fn refused(
        reason: String,
        canonical_machine: Option<String>,
        toolchains: Vec<ToolchainRecord>,
    ) -> Self {
        Self {
            status: Status::Refused,
            toolchains,
            canonical_machine,
            reason: Some(reason),
        }
    }
}

/// Run the gate.
///
/// Returns `Ok(outcome)` for every reachable conclusion INCLUDING a refusal —
/// the refusal is a verdict with provenance, not an error string, because it
/// has to reach the dispatch result rather than only a log line. `Err` is
/// reserved for "the gate could not run at all" (the canonical pull failed),
/// which is also a refusal but carries no per-toolchain detail.
pub async fn ensure(
    declared: Option<&CiCanonical>,
    converge_authorized: bool,
    canonical: &dyn CanonicalConvergence,
    log: &mut (dyn FnMut(String) + Send),
) -> Outcome {
    let Some(declared) = declared else {
        return Outcome::not_requested();
    };
    let keys: Vec<String> = declared.toolchains.clone();

    log(format!(
        "[ci-node] canonical: manifest requires {keys:?}; convergence authority on this box: {}",
        if converge_authorized {
            "GRANTED"
        } else {
            "withheld (canonical_converge = false)"
        }
    ));

    // Measurement half.
    let reading = match canonical.measure(&keys).await {
        Ok(p) => p,
        Err(e) => {
            // A gate that cannot measure must not pass. This is the same
            // `silent-empty-is-unknown` rule the rest of the module runs on:
            // an unreachable canonical is UNKNOWN, and unknown is not "at
            // canonical".
            return Outcome::refused(
                format!(
                    "could not read the canonical configuration ({e}). The manifest requires \
                     this box to be at canonical for {keys:?}, and a requirement that cannot \
                     be checked is not a requirement that passed"
                ),
                None,
                Vec::new(),
            );
        }
    };
    let machine = reading.canonical_machine.clone().filter(|s| !s.is_empty());

    // NOTE what does NOT happen here: `is_canonical_self` does not
    // short-circuit the check.
    //
    // "This box defines canonical, therefore it is at canonical" is true, and
    // it is also not the question the manifest asked. A canonical machine with
    // no Rust installed still cannot run a build that requires `rustc`, and
    // returning satisfied on the flag alone would pass it — while every OTHER
    // box declaring `rustc` would be refused for canonical carrying no value.
    // One box exempt from the check it defines is exactly the shape of vacuous
    // green this lane keeps rediscovering.
    //
    // What the flag DOES change is the predicate below: a self box is compared
    // against its own last uploaded capture, so a difference means "this box
    // has moved since it last published", not "this box disagrees with
    // canonical". Presence and measurement are still required; agreement with
    // a stale snapshot of itself is not.
    if reading.is_canonical_self {
        log("[ci-node] canonical: this box IS the canonical machine — requiring the declared toolchains to be present and measured, not to match its own last upload".to_string());
    }

    let standings: Vec<(String, Standing)> = match reading.versions {
        VersionsReading::NoCanonicalSection => {
            return Outcome::refused(
                format!(
                    "the canonical configuration carries no '{VERSIONS_SECTION}' section, so \
                     there is nothing to compare {keys:?} against"
                ),
                machine,
                Vec::new(),
            );
        }
        VersionsReading::LocalSectionAbsent => {
            return Outcome::refused(
                format!(
                    "this box produced no '{VERSIONS_SECTION}' capture at all, so it cannot be \
                     shown to be at canonical for {keys:?}. Nothing measured is not nothing wrong"
                ),
                machine,
                Vec::new(),
            );
        }
        VersionsReading::Standings(standings) => standings,
    };
    // Every declared key must have been answered. A host that measured fewer
    // keys than it was asked about left the rest UNKNOWN, and unknown is not
    // at canonical.
    if let Some(missing) = keys
        .iter()
        .find(|k| !standings.iter().any(|(key, _)| key == *k))
    {
        return Outcome::refused(
            format!("the canonical measurement did not report '{missing}', so it is unmeasured"),
            machine,
            Vec::new(),
        );
    }
    // Declared order, whatever order the host answered in.
    let standings: Vec<(String, Standing)> = keys
        .iter()
        .filter_map(|k| standings.iter().find(|(key, _)| key == k).cloned())
        .collect();

    // Refusals that no amount of convergence could fix are decided BEFORE any
    // apply, so a box that cannot satisfy the requirement never pays for a
    // toolchain download first.
    let mut records: Vec<ToolchainRecord> = Vec::new();
    let mut hard: Vec<String> = Vec::new();
    let mut drifted: Vec<String> = Vec::new();
    let self_canonical = reading.is_canonical_self;
    for (key, standing) in &standings {
        match standing {
            Standing::Agreed { value } => records.push(ToolchainRecord {
                key: key.clone(),
                local: Some(value.clone()),
                canonical: Some(value.clone()),
                state: "agreed",
            }),
            // On a NON-canonical box this is drift to close. On the canonical
            // box itself it is only "moved since the last upload", and the
            // requirement is met as long as the toolchain is actually there —
            // converging the canonical machine toward its own stale snapshot
            // would be backwards.
            Standing::Drifted { local, canonical } => {
                if self_canonical && local.is_some() {
                    records.push(ToolchainRecord {
                        key: key.clone(),
                        local: local.clone(),
                        canonical: Some(canonical.clone()),
                        state: "canonical_self",
                    });
                } else if self_canonical {
                    // The stored capture has the key; this box does not report
                    // it NOW. There is nothing to converge toward but itself.
                    hard.push(format!(
                        "{key}: this box defines canonical but does not currently report it"
                    ));
                    records.push(ToolchainRecord {
                        key: key.clone(),
                        local: None,
                        canonical: Some(canonical.clone()),
                        state: "absent_locally",
                    });
                } else {
                    drifted.push(key.clone());
                    records.push(ToolchainRecord {
                        key: key.clone(),
                        local: local.clone(),
                        canonical: Some(canonical.clone()),
                        state: "drifted",
                    });
                }
            }
            Standing::Unmeasured => {
                hard.push(format!(
                    "{key}: this box could not measure it (the capture probe did not answer), \
                     and an unread value is not an agreeing value"
                ));
                records.push(ToolchainRecord {
                    key: key.clone(),
                    local: None,
                    canonical: None,
                    state: "unmeasured",
                });
            }
            Standing::NoCanonicalValue { local } => {
                // Same asymmetry: on the canonical box this only means the key
                // post-dates its last upload. It is measured and present, which
                // is the whole requirement there.
                if self_canonical {
                    records.push(ToolchainRecord {
                        key: key.clone(),
                        local: Some(local.clone()),
                        canonical: None,
                        state: "canonical_self",
                    });
                } else {
                    hard.push(format!(
                        "{key}: the canonical machine reports no value for it, so there is no canonical state to be at"
                    ));
                    records.push(ToolchainRecord {
                        key: key.clone(),
                        local: Some(local.clone()),
                        canonical: None,
                        state: "no_canonical_value",
                    });
                }
            }
            Standing::AbsentBoth => {
                hard.push(format!(
                    "{key}: neither this box nor the canonical machine reports it. An empty \
                     diff between two absences is not agreement"
                ));
                records.push(ToolchainRecord {
                    key: key.clone(),
                    local: None,
                    canonical: None,
                    state: "absent_both",
                });
            }
        }
    }
    if !hard.is_empty() {
        return Outcome::refused(
            format!(
                "this box cannot be shown to be at canonical ({}) — {}",
                machine.as_deref().unwrap_or("unknown machine"),
                hard.join("; ")
            ),
            machine,
            records,
        );
    }
    if drifted.is_empty() {
        log("[ci-node] canonical: every declared toolchain already matches".to_string());
        return Outcome {
            status: if self_canonical {
                Status::IsCanonical
            } else {
                Status::Satisfied
            },
            toolchains: records,
            canonical_machine: machine,
            reason: None,
        };
    }

    if !converge_authorized {
        return Outcome::refused(
            format!(
                "{} drifted from canonical ({}), and this box has not authorised convergence. \
                 Enable ci-node 'canonical_converge' for this device to let a dispatch drive \
                 the version managers, or bring the box to canonical yourself with \
                 `qontinui env apply --confirm`. The dispatch is refused rather than run under \
                 a toolchain the canonical machine never validated",
                drifted.join(", "),
                machine.as_deref().unwrap_or("unknown machine")
            ),
            machine,
            records,
        );
    }

    // Acting half — only the drifted keys, never more: the blast radius of a
    // convergence is the requirement.
    log(format!(
        "[ci-node] canonical: converging {drifted:?} toward {}",
        machine.as_deref().unwrap_or("canonical")
    ));
    let report = match canonical.converge(&drifted).await {
        Ok(r) => r,
        Err(e) => {
            return Outcome::refused(
                format!("the convergence task did not complete ({e})"),
                machine,
                records,
            );
        }
    };
    for note in &report.notes {
        log(format!("[ci-node] canonical: {note}"));
    }

    let still: Vec<&String> = drifted
        .iter()
        .filter(|k| !report.moved.iter().any(|(moved, _)| moved == *k))
        .collect();
    if !still.is_empty() {
        return Outcome::refused(
            format!(
                "convergence did not bring {} to canonical — {}",
                still
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
                report.failure_reason
            ),
            machine,
            records,
        );
    }
    for (key, to) in &report.moved {
        if let Some(record) = records.iter_mut().find(|r| &r.key == key) {
            record.local = Some(to.clone());
            record.state = "converged";
        }
    }
    Outcome {
        status: Status::Converged,
        toolchains: records,
        canonical_machine: machine,
        reason: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::{BoxFuture, CanonicalReading, ConvergeReport};
    use std::sync::Mutex;

    /// A scripted measurement + convergence, recording what it was asked to
    /// converge.
    struct Scripted {
        reading: Result<CanonicalReading, String>,
        converge: Result<ConvergeReport, String>,
        asked: Mutex<Vec<Vec<String>>>,
    }

    impl CanonicalConvergence for Scripted {
        fn measure<'a>(
            &'a self,
            _keys: &'a [String],
        ) -> BoxFuture<'a, Result<CanonicalReading, String>> {
            Box::pin(async move { self.reading.clone() })
        }
        fn converge<'a>(
            &'a self,
            keys: &'a [String],
        ) -> BoxFuture<'a, Result<ConvergeReport, String>> {
            self.asked.lock().unwrap().push(keys.to_vec());
            Box::pin(async move { self.converge.clone() })
        }
    }

    fn scripted(
        is_self: bool,
        standings: Vec<(&str, Standing)>,
        converge: Result<ConvergeReport, String>,
    ) -> Scripted {
        Scripted {
            reading: Ok(CanonicalReading {
                canonical_machine: Some("spaceship".to_string()),
                is_canonical_self: is_self,
                versions: VersionsReading::Standings(
                    standings
                        .into_iter()
                        .map(|(k, s)| (k.to_string(), s))
                        .collect(),
                ),
            }),
            converge,
            asked: Mutex::new(Vec::new()),
        }
    }

    fn declared(keys: &[&str]) -> CiCanonical {
        CiCanonical {
            toolchains: keys.iter().map(|k| k.to_string()).collect(),
        }
    }

    async fn run(host: &Scripted, keys: &[&str], authorized: bool) -> Outcome {
        let mut log = |_line: String| {};
        ensure(Some(&declared(keys)), authorized, host, &mut log).await
    }

    fn drifted(local: &str, canonical: &str) -> Standing {
        Standing::Drifted {
            local: Some(local.to_string()),
            canonical: canonical.to_string(),
        }
    }

    /// The verdict must distinguish "no claim" from every claim, and say so in
    /// bytes rather than by an absent field.
    #[test]
    fn not_requested_is_stated_not_omitted() {
        let o = Outcome::not_requested();
        assert!(!o.is_refusal());
        assert_eq!(o.to_json()["status"], "not_requested");
        assert!(o.summary_line().contains("not required"));
    }

    /// A refusal carries its reason into the verdict, because the dispatch
    /// result is where an operator will look — not the runner's local log.
    #[test]
    fn a_refusal_carries_its_reason_into_the_verdict() {
        let o = Outcome::refused(
            "node drifted from canonical".to_string(),
            Some("spaceship".to_string()),
            vec![ToolchainRecord {
                key: "node".to_string(),
                local: Some("v20.9.0".to_string()),
                canonical: Some("v22.11.0".to_string()),
                state: "drifted",
            }],
        );
        assert!(o.is_refusal());
        let v = o.to_json();
        assert_eq!(v["status"], "refused");
        assert_eq!(v["canonical_machine"], "spaceship");
        assert_eq!(v["toolchains"][0]["local"], "v20.9.0");
        assert_eq!(v["toolchains"][0]["canonical"], "v22.11.0");
        assert!(o.reason().unwrap().contains("drifted"));
    }

    /// No `[canonical]` declared: the host is never asked anything.
    #[tokio::test]
    async fn an_undeclared_requirement_never_measures() {
        let host = Scripted {
            reading: Err("must not be called".to_string()),
            converge: Err("must not be called".to_string()),
            asked: Mutex::new(Vec::new()),
        };
        let mut log = |_line: String| {};
        let o = ensure(None, true, &host, &mut log).await;
        assert_eq!(o.to_json()["status"], "not_requested");
    }

    /// A host that cannot read canonical (the standalone CLI, an unenrolled
    /// box) is REFUSED, never passed: unknown is not at canonical.
    #[tokio::test]
    async fn an_unreadable_canonical_is_a_refusal() {
        let host = Scripted {
            reading: Err("not enrolled".to_string()),
            converge: Err("unused".to_string()),
            asked: Mutex::new(Vec::new()),
        };
        let o = run(&host, &["rustc"], true).await;
        assert!(o.is_refusal());
        assert!(
            o.reason().unwrap().contains("not enrolled"),
            "{:?}",
            o.reason()
        );
    }

    #[tokio::test]
    async fn every_agreed_key_is_satisfied_without_converging() {
        let host = scripted(
            false,
            vec![(
                "rustc",
                Standing::Agreed {
                    value: "1.95.0".to_string(),
                },
            )],
            Err("unused".to_string()),
        );
        let o = run(&host, &["rustc"], false).await;
        assert_eq!(o.to_json()["status"], "satisfied");
        assert!(host.asked.lock().unwrap().is_empty());
    }

    /// Absence on both sides, an unread key, and a key canonical has no value
    /// for are all refusals that no convergence could fix — decided BEFORE any
    /// apply, so a box never pays for a download first.
    #[tokio::test]
    async fn unsatisfiable_standings_refuse_before_any_convergence() {
        for standing in [
            Standing::AbsentBoth,
            Standing::Unmeasured,
            Standing::NoCanonicalValue {
                local: "1.95.0".to_string(),
            },
        ] {
            let host = scripted(
                false,
                vec![("node", drifted("v20", "v22")), ("rustc", standing.clone())],
                Err("unused".to_string()),
            );
            let o = run(&host, &["node", "rustc"], true).await;
            assert!(o.is_refusal(), "{standing:?} must refuse");
            assert!(
                host.asked.lock().unwrap().is_empty(),
                "{standing:?} converged anyway"
            );
        }
    }

    /// A key the host did not answer for is unmeasured, not agreed.
    #[tokio::test]
    async fn an_unanswered_key_is_a_refusal() {
        let host = scripted(
            false,
            vec![(
                "node",
                Standing::Agreed {
                    value: "v22".to_string(),
                },
            )],
            Err("unused".to_string()),
        );
        let o = run(&host, &["node", "rustc"], true).await;
        assert!(o.is_refusal());
        assert!(o.reason().unwrap().contains("'rustc'"), "{:?}", o.reason());
    }

    /// Drift with the authority withheld is REFUSED, not run anyway.
    #[tokio::test]
    async fn drift_without_authority_is_refused() {
        let host = scripted(
            false,
            vec![("node", drifted("v20", "v22"))],
            Err("unused".to_string()),
        );
        let o = run(&host, &["node"], false).await;
        assert!(o.is_refusal());
        assert!(o
            .reason()
            .unwrap()
            .contains("has not authorised convergence"));
        assert!(host.asked.lock().unwrap().is_empty());
    }

    /// With authority, only the DRIFTED keys are handed to convergence, and the
    /// verdict records what they moved to.
    #[tokio::test]
    async fn authorised_drift_converges_exactly_the_drifted_keys() {
        let host = scripted(
            false,
            vec![
                ("node", drifted("v20", "v22")),
                (
                    "rustc",
                    Standing::Agreed {
                        value: "1.95.0".to_string(),
                    },
                ),
            ],
            Ok(ConvergeReport {
                moved: vec![("node".to_string(), "v22".to_string())],
                notes: vec!["volta install node@22".to_string()],
                failure_reason: String::new(),
            }),
        );
        let o = run(&host, &["node", "rustc"], true).await;
        assert!(!o.is_refusal(), "{:?}", o.reason());
        let v = o.to_json();
        assert_eq!(v["status"], "converged");
        assert_eq!(v["toolchains"][0]["local"], "v22");
        assert_eq!(v["toolchains"][0]["state"], "converged");
        assert_eq!(v["toolchains"][1]["state"], "agreed");
        assert_eq!(
            host.asked.lock().unwrap().clone(),
            vec![vec!["node".to_string()]]
        );
    }

    /// A key convergence did not move is a refusal naming why, in the
    /// convergence machinery's own words.
    #[tokio::test]
    async fn a_partial_convergence_is_a_refusal() {
        let host = scripted(
            false,
            vec![
                ("node", drifted("v20", "v22")),
                ("python", drifted("3.11", "3.12")),
            ],
            Ok(ConvergeReport {
                moved: vec![("node".to_string(), "v22".to_string())],
                notes: Vec::new(),
                failure_reason: "python: no supported version manager detected".to_string(),
            }),
        );
        let o = run(&host, &["node", "python"], true).await;
        assert!(o.is_refusal());
        let reason = o.reason().unwrap();
        assert!(reason.contains("python") && reason.contains("no supported version manager"));
    }

    /// The canonical machine is NOT exempt from its own check: present and
    /// measured satisfies it, absent locally refuses it.
    #[tokio::test]
    async fn the_canonical_box_still_has_to_report_the_toolchain() {
        let moved = scripted(
            true,
            vec![("rustc", drifted("1.95.0", "1.90.0"))],
            Err("unused".into()),
        );
        let o = run(&moved, &["rustc"], false).await;
        assert_eq!(o.to_json()["status"], "is_canonical");

        let gone = scripted(
            true,
            vec![(
                "rustc",
                Standing::Drifted {
                    local: None,
                    canonical: "1.90.0".to_string(),
                },
            )],
            Err("unused".into()),
        );
        assert!(run(&gone, &["rustc"], true).await.is_refusal());

        let never = scripted(
            true,
            vec![("rustc", Standing::AbsentBoth)],
            Err("unused".into()),
        );
        assert!(run(&never, &["rustc"], true).await.is_refusal());
    }

    #[tokio::test]
    async fn a_missing_or_uncaptured_versions_section_is_a_refusal() {
        for versions in [
            VersionsReading::NoCanonicalSection,
            VersionsReading::LocalSectionAbsent,
        ] {
            let host = Scripted {
                reading: Ok(CanonicalReading {
                    canonical_machine: None,
                    is_canonical_self: false,
                    versions: versions.clone(),
                }),
                converge: Err("unused".to_string()),
                asked: Mutex::new(Vec::new()),
            };
            assert!(
                run(&host, &["node"], true).await.is_refusal(),
                "{versions:?}"
            );
        }
    }
}
