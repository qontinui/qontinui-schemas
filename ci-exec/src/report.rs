//! Where a run's log lines and verdict go.
//!
//! The executor does not know whether it is talking to coord (the runner's and
//! the host agent's progress/result POSTs) or to a terminal (`qontinui-ci
//! run`). It streams lines into a [`LogSink`] and files exactly one
//! [`Verdict`] through a [`Reporter`].
//!
//! ## The receipt
//!
//! A dispatch's JUnit report lives INSIDE the worktree that cleanup deletes,
//! and it is coord's Tier-7 credibility-gate input. "Clean up, then report"
//! therefore destroys the artifact before anything can send it — which is what
//! happened before this seam existed, invisibly, because the dispatch itself
//! still went green. So [`ResultReported`] is minted ONLY by [`report`], which
//! requires the verdict (and with it the capture slot), and the executor's
//! cleanup consumes one. No host implementation can mint it: its constructor
//! is private to this crate.

use std::sync::Arc;

use serde::Serialize;

use crate::canonical::Outcome as CanonicalOutcome;
use crate::host::BoxFuture;
use crate::junit::TestArtifact;

/// A run's final conclusion, in the wire vocabulary coord's result route and
/// a GitHub check run both speak.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Conclusion {
    Success,
    Failure,
    /// A non-verdict: the run says nothing about the code under test (it was
    /// cancelled, or it never obtained a tree to test).
    Cancelled,
}

impl Conclusion {
    pub fn as_str(self) -> &'static str {
        match self {
            Conclusion::Success => "success",
            Conclusion::Failure => "failure",
            Conclusion::Cancelled => "cancelled",
        }
    }
}

/// One step's row in the result summary.
#[derive(Debug, Clone, Serialize)]
pub struct StepSummary {
    pub name: String,
    /// `success` | `failure` | `cancelled`.
    pub conclusion: String,
    pub duration_secs: u64,
}

/// Receives the run's log lines. Never blocks and never fails: a line that
/// cannot be delivered is the sink's problem, not the build's.
pub trait LogSink: Send + Sync {
    fn push(&self, line: &str);
}

/// Everything a run concludes with.
pub struct Verdict<'a> {
    pub conclusion: Conclusion,
    pub steps: &'a [StepSummary],
    /// A machine-readable reason next to the steps (`head_sha_unavailable`, a
    /// canonical refusal, …). `None` for an ordinary verdict.
    pub reason: Option<&'a str>,
    /// This run's captured test report. `None` is spelled explicitly at every
    /// exit that genuinely has none (no step ran), because an optional
    /// parameter with a default is exactly how the artifact went missing
    /// before.
    pub test_results: Option<&'a TestArtifact>,
    /// The canonical-configuration verdict. `None` means the gate had not run
    /// yet at this exit (the manifest was not even parsed), which a reporter
    /// renders as `not_evaluated` — a different statement from
    /// `not_requested`.
    pub canonical: Option<&'a CanonicalOutcome>,
}

/// Where one run reports. Created by the host before the run starts.
///
/// A reporter addressed by the dispatch id (coord's progress/result routes)
/// must only be built for an id that passed
/// [`crate::dispatch::dispatch_id_is_safe`]: gate the id first, then build the
/// reporter. `run_dispatch` drops an unsafe id without touching the reporter,
/// but it cannot un-bind an id the reporter already put in a URL.
pub trait Reporter: Send {
    /// The sink this run's log lines go to. The executor holds the returned
    /// handle (and its clones) only until it files the verdict.
    fn sink(&self) -> Arc<dyn LogSink>;
    /// Close the log stream and file the verdict. Called exactly once per run,
    /// after every sink handle has been dropped.
    fn report<'a>(self: Box<Self>, verdict: Verdict<'a>) -> BoxFuture<'a, ()>
    where
        Self: 'a;
}

/// Proof that a run's verdict has been filed. See the module docs: the only
/// way to obtain one is [`report`], and the executor's cleanup consumes it.
///
/// Do not derive `Default`, `Clone` or `Copy` on this, and do not add a public
/// constructor — every one of those reopens the ordering hole.
#[must_use = "a dispatch's worktree may only be cleaned up once its result has been reported"]
pub struct ResultReported(());

impl ResultReported {
    /// TEST-ONLY receipt, so cleanup can be exercised without a reporter.
    #[cfg(test)]
    pub(crate) fn for_test() -> Self {
        ResultReported(())
    }
}

/// File `verdict` through `reporter` and mint the receipt.
///
/// Takes the run's sink handle by value and drops it FIRST: a reporter that
/// closes its stream on `report` (the coord reporter's flusher drains until
/// every sender is gone) would otherwise wait on a handle the executor still
/// holds.
pub(crate) async fn report(
    reporter: Box<dyn Reporter>,
    sink: Arc<dyn LogSink>,
    verdict: Verdict<'_>,
) -> ResultReported {
    drop(sink);
    reporter.report(verdict).await;
    ResultReported(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// `(conclusion, step count, reason)` of the one verdict filed.
    type Filed = Arc<Mutex<Option<(String, usize, Option<String>)>>>;

    struct Recorder {
        lines: Arc<Mutex<Vec<String>>>,
        filed: Filed,
    }

    struct RecSink(Arc<Mutex<Vec<String>>>);
    impl LogSink for RecSink {
        fn push(&self, line: &str) {
            self.0.lock().unwrap().push(line.to_string());
        }
    }

    impl Reporter for Recorder {
        fn sink(&self) -> Arc<dyn LogSink> {
            Arc::new(RecSink(self.lines.clone()))
        }
        fn report<'a>(self: Box<Self>, verdict: Verdict<'a>) -> BoxFuture<'a, ()>
        where
            Self: 'a,
        {
            Box::pin(async move {
                *self.filed.lock().unwrap() = Some((
                    verdict.conclusion.as_str().to_string(),
                    verdict.steps.len(),
                    verdict.reason.map(str::to_string),
                ));
            })
        }
    }

    /// `report` files exactly the verdict it was handed, and the sink handle it
    /// consumed is released before the reporter runs.
    #[tokio::test]
    async fn report_files_the_verdict_and_releases_the_sink_first() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let filed = Arc::new(Mutex::new(None));
        let reporter = Box::new(Recorder {
            lines: lines.clone(),
            filed: filed.clone(),
        });
        let sink = reporter.sink();
        sink.push("hello");
        let weak = Arc::downgrade(&sink);
        let steps = vec![StepSummary {
            name: "s".to_string(),
            conclusion: "failure".to_string(),
            duration_secs: 1,
        }];
        let _receipt = report(
            reporter,
            sink,
            Verdict {
                conclusion: Conclusion::Failure,
                steps: &steps,
                reason: Some("why"),
                test_results: None,
                canonical: None,
            },
        )
        .await;
        assert!(
            weak.upgrade().is_none(),
            "the sink must be dropped before filing"
        );
        assert_eq!(lines.lock().unwrap().as_slice(), ["hello"]);
        assert_eq!(
            filed.lock().unwrap().clone(),
            Some(("failure".to_string(), 1, Some("why".to_string())))
        );
    }

    #[test]
    fn conclusions_speak_the_wire_vocabulary() {
        assert_eq!(Conclusion::Success.as_str(), "success");
        assert_eq!(Conclusion::Failure.as_str(), "failure");
        assert_eq!(Conclusion::Cancelled.as_str(), "cancelled");
    }
}
