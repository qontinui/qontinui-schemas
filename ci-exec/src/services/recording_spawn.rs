//! `RecordingSpawn`: a test [`ProcessSpawn`] that records a container command
//! only when the production path actually EXECUTES it.
//!
//! # Why a real child and not a call log
//!
//! The detector this replaces (`ServiceStack.removals`) pushed each removal
//! argv into a vec just BEFORE the command ran. If the execution was deleted,
//! every CI-run teardown test stayed green (plan
//! `2026-11-06-ci-node-provisioning-children-inherit-only-an-allowlisted-environment`
//! Phase 5, measured in Phase 1.2). A double that records at the
//! `ProcessSpawn::command` call would repeat that hole, because `command` only
//! BUILDS the command. So this double hands back a command for a real recorder
//! child, and the recorder appends its own argv to a log file when it runs:
//!
//! * Unix: `sh -c <script> recorder <program> <argv…>`.
//! * Windows: `cmd.exe /D /C <recorder.cmd> <program> <argv…>`.
//!
//! The production code appends the argv with `.args(…)`, so the recorder sees
//! exactly what the runtime would have seen. A line reaches the log only when
//! a child was spawned and ran.
//!
//! # Scripted exits
//!
//! Each [`Rule`] maps a runtime SUBCOMMAND (`info`, `rm`, `run`, …) to an exit
//! status and a line of output. That lets a test drive `provision_with` down
//! paths no daemon-free test could reach before: the stale pre-run removal,
//! and the removal after a failed start. An unscripted subcommand exits 0
//! with no output.
//!
//! # Limits
//!
//! The log is one line per command with the argv joined on spaces, and
//! [`RecordingSpawn::issued`] splits it on whitespace again. That round-trip
//! is exact only for argv with no spaces, which holds for every container
//! command this module's tests drive. Rule outputs are checked for the
//! characters that would break either recorder script.

use std::path::PathBuf;
use std::sync::Arc;

use crate::host::{ProcessSpawn, StepContainment, TreeGuard};
use crate::standalone::PlainSpawn;

/// The variable that carries the log path to the recorder child. Container
/// runtime children inherit the host environment (`ChildEnv::ContainerRuntime`),
/// so a variable set on the command reaches the recorder. If container-runtime
/// children are ever allowlisted, this name is dropped and every recorder test
/// reds with an empty log. That is the safe direction, and this is its cause.
const LOG_VAR: &str = "QONTINUI_RECORDING_SPAWN_LOG";

/// One scripted answer: when the runtime's first argument is `subcommand`,
/// print `output` and exit with `exit`.
pub(crate) struct Rule {
    pub(crate) subcommand: &'static str,
    pub(crate) exit: i32,
    pub(crate) output: &'static str,
}

/// See the module docs.
pub(crate) struct RecordingSpawn {
    /// Owns the log file and, on Windows, the recorder script. Removed on drop.
    _dir: tempfile::TempDir,
    log: PathBuf,
    #[cfg(unix)]
    script: String,
    #[cfg(windows)]
    script: PathBuf,
}

impl RecordingSpawn {
    /// A recorder under which every container command succeeds silently.
    pub(crate) fn new() -> Arc<Self> {
        Self::scripted(&[])
    }

    /// A recorder that answers per `rules` (first match wins).
    pub(crate) fn scripted(rules: &[Rule]) -> Arc<Self> {
        for rule in rules {
            assert!(
                rule.subcommand
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-'),
                "a RecordingSpawn subcommand must be a plain word: {:?}",
                rule.subcommand
            );
            assert!(
                rule.output
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || " .:-_".contains(c)),
                "a RecordingSpawn output must avoid shell and cmd metacharacters: {:?}",
                rule.output
            );
            assert!(
                (0..=255).contains(&rule.exit),
                "a RecordingSpawn exit status must be a portable 0..=255: {}",
                rule.exit
            );
        }
        let dir = tempfile::tempdir().expect("RecordingSpawn tempdir");
        let log = dir.path().join("issued.log");
        std::fs::write(&log, "").expect("RecordingSpawn log");

        #[cfg(unix)]
        let script = {
            // $1 is the program and $2 the subcommand. `$*` joins the program
            // and its argv on spaces.
            let mut s = format!("printf '%s\\n' \"$*\" >> \"${LOG_VAR}\"\ncase \"$2\" in\n");
            for rule in rules {
                s.push_str(&format!(
                    "  {sub}) printf '%s\\n' '{out}'; exit {code};;\n",
                    sub = rule.subcommand,
                    out = rule.output,
                    code = rule.exit,
                ));
            }
            s.push_str("esac\nexit 0\n");
            s
        };

        #[cfg(windows)]
        let script = {
            // `echo %*` sits outside any parenthesised block, so an argv
            // holding `)` cannot end a block early. The redirect comes FIRST:
            // a trailing `… 2>>log` would read a final single-digit argument
            // as a handle number. `%~2` is the subcommand.
            let mut s = format!("@echo off\r\n>>\"%{LOG_VAR}%\" echo %*\r\n");
            for (i, rule) in rules.iter().enumerate() {
                s.push_str(&format!(
                    "if \"%~2\"==\"{}\" goto r{i}\r\n",
                    rule.subcommand
                ));
            }
            s.push_str("exit /b 0\r\n");
            for (i, rule) in rules.iter().enumerate() {
                s.push_str(&format!(":r{i}\r\n"));
                if !rule.output.is_empty() {
                    // `echo(` prints `on`/`off` literally instead of
                    // switching the echo state.
                    s.push_str(&format!("echo({}\r\n", rule.output));
                }
                s.push_str(&format!("exit /b {}\r\n", rule.exit));
            }
            let path = dir.path().join("recorder.cmd");
            std::fs::write(&path, s).expect("RecordingSpawn recorder script");
            path
        };

        Arc::new(Self {
            _dir: dir,
            log,
            script,
        })
    }

    /// Every command the recorder RAN, in order: `[program, argv…]`.
    pub(crate) fn issued(&self) -> Vec<Vec<String>> {
        std::fs::read_to_string(&self.log)
            .expect("RecordingSpawn log is readable")
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.split_whitespace().map(str::to_string).collect())
            .collect()
    }

    /// The argv (program dropped) of every `rm` the recorder RAN, in order.
    /// This is the shape `removal_argv` builds.
    pub(crate) fn removals(&self) -> Vec<Vec<String>> {
        self.issued()
            .into_iter()
            .filter(|line| line.get(1).map(String::as_str) == Some("rm"))
            .map(|line| line[1..].to_vec())
            .collect()
    }

    #[cfg(unix)]
    fn recorder_argv(&self, program: &str) -> (&'static str, Vec<std::ffi::OsString>) {
        (
            "sh",
            vec![
                "-c".into(),
                self.script.clone().into(),
                "recorder".into(),
                program.into(),
            ],
        )
    }

    #[cfg(windows)]
    fn recorder_argv(&self, program: &str) -> (&'static str, Vec<std::ffi::OsString>) {
        (
            "cmd.exe",
            vec![
                "/D".into(),
                "/C".into(),
                self.script.clone().into_os_string(),
                program.into(),
            ],
        )
    }
}

impl ProcessSpawn for RecordingSpawn {
    fn command(&self, program: &str) -> tokio::process::Command {
        let (shell, argv) = self.recorder_argv(program);
        let mut cmd = tokio::process::Command::new(shell);
        cmd.args(argv).env(LOG_VAR, &self.log);
        cmd
    }

    fn std_command(&self, program: &str) -> std::process::Command {
        let (shell, argv) = self.recorder_argv(program);
        let mut cmd = std::process::Command::new(shell);
        cmd.args(argv).env(LOG_VAR, &self.log);
        cmd
    }

    fn arm_tree(&self, cmd: &mut tokio::process::Command) {
        PlainSpawn.arm_tree(cmd)
    }

    fn attach_tree(&self, child: &tokio::process::Child) -> Box<dyn TreeGuard> {
        PlainSpawn.attach_tree(child)
    }

    fn step_containment(&self) -> Box<dyn StepContainment> {
        PlainSpawn.step_containment()
    }
}

mod tests {
    use super::*;
    use crate::child_env::ChildEnv;
    use std::time::Duration;

    /// The recorder logs only what RAN: building a command records nothing.
    #[tokio::test]
    async fn a_built_but_unspawned_command_records_nothing() {
        let spawn = RecordingSpawn::new();
        let _cmd = spawn.command("docker");
        let _std = spawn.std_command("docker");
        assert!(spawn.issued().is_empty());

        crate::tools::run_capture_env(
            spawn.as_ref(),
            ChildEnv::ContainerRuntime,
            "docker",
            &["rm", "-f", "-v", "c-1"],
            Duration::from_secs(30),
            None,
            &[],
        )
        .await
        .expect("an unscripted subcommand exits 0");
        assert_eq!(
            spawn.issued(),
            vec![vec!["docker", "rm", "-f", "-v", "c-1"]]
        );
        assert_eq!(spawn.removals(), vec![vec!["rm", "-f", "-v", "c-1"]]);
    }

    /// A scripted rule sets both the exit status and the output the caller
    /// reads.
    #[tokio::test]
    async fn rules_script_exit_and_output() {
        let spawn = RecordingSpawn::scripted(&[
            Rule {
                subcommand: "info",
                exit: 0,
                output: "24.0.7",
            },
            Rule {
                subcommand: "run",
                exit: 3,
                output: "Error: boom",
            },
        ]);
        let ok = crate::tools::run_capture_env(
            spawn.as_ref(),
            ChildEnv::ContainerRuntime,
            "docker",
            &["info"],
            Duration::from_secs(30),
            None,
            &[],
        )
        .await
        .expect("info is scripted to succeed");
        assert_eq!(ok.trim(), "24.0.7");
        let err = crate::tools::run_capture_env(
            spawn.as_ref(),
            ChildEnv::ContainerRuntime,
            "docker",
            &["run", "-d"],
            Duration::from_secs(30),
            None,
            &[],
        )
        .await
        .expect_err("run is scripted to fail");
        assert!(err.contains("Error: boom"), "got: {err}");
        assert_eq!(spawn.issued().len(), 2);
    }
}
