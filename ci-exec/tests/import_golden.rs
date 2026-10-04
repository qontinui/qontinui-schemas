//! Golden tests for `qontinui-ci import`, `import --report` and
//! `gen-workflow`, over the real workflow files of every PUBLIC repo that
//! carries a `.qontinui/ci.toml` (plan
//! `2026-10-04-coord-managed-ci-for-every-tenant-with-an-actions-free-mode`,
//! Phase 8).
//!
//! # The fixture set
//!
//! `tests/fixtures/<repo>/` holds `ci.toml` and `workflows/*.yml`, each read
//! with `git show origin/main:<path>` — never from a working tree — at the
//! commits recorded in `tests/fixture-shas.txt`. The repos were enumerated by
//! `git ls-tree -r --name-only origin/main` across the workspace: ten carry a
//! `.qontinui/ci.toml`. Nine are here. Left out, deliberately:
//!
//! * **qontinui-coord** — a PRIVATE repo; its workflows are not copied into
//!   this public one.
//! * From qontinui-web: `deploy-web.yml`, `migrate.yml`,
//!   `db-credential-drift.yml`, `oneoff-seed-claude-accounts.yml` and
//!   `verify-frontend-run.yml` — each names a production AWS account role ARN.
//!   None is a pull-request gate (deploys and operator jobs), so the coverage
//!   report loses nothing a Phase 9 flip depends on, but the file set is not
//!   the repo's complete one.
//!
//! # Blessing
//!
//! `QONTINUI_CI_BLESS=1 cargo test -p qontinui-ci-exec --test import_golden`
//! rewrites `tests/golden/`. Review the diff: it is the importer's behaviour.

use std::path::{Path, PathBuf};
use std::process::Command;

use qontinui_ci_exec::gen_workflow::{self, CheckVerdict, GenOptions};
use qontinui_ci_exec::import::{self, Coverage, ItemKind, WorkflowSource};
use qontinui_ci_exec::manifest;

/// Repos whose `ci.toml` declares only `[[siblings]]` (read by coord's
/// allocate lane) and no steps: not a runnable manifest, so the executor —
/// and therefore `import --report` and `gen-workflow` — refuses it. The
/// golden test pins that refusal instead of skipping the repo.
const SIBLINGS_ONLY: &[&str] = &["qontinui-inspect", "qontinui-supervisor"];

/// Every public repo carrying a `.qontinui/ci.toml` on origin/main.
const REPOS: &[&str] = &[
    "multistate",
    "qontinui",
    "qontinui-cloud-control",
    "qontinui-inspect",
    "qontinui-runner",
    "qontinui-schemas",
    "qontinui-supervisor",
    "qontinui-web",
    "ui-bridge",
];

/// A fixed executor pin for generation (any full sha; the goldens embed it).
const EXECUTOR_REV: &str = "84b13b0de3dd322f48e9f472a84c9ba490e88a3d";

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture(repo: &str) -> PathBuf {
    crate_dir().join("tests/fixtures").join(repo)
}

fn workflow(repo: &str, file: &str) -> WorkflowSource {
    let path = fixture(repo).join("workflows").join(file);
    WorkflowSource {
        label: format!(".github/workflows/{file}"),
        text: std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display())),
    }
}

fn all_workflows(repo: &str) -> Vec<WorkflowSource> {
    let mut names: Vec<String> = std::fs::read_dir(fixture(repo).join("workflows"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .filter(|n| n.ends_with(".yml") || n.ends_with(".yaml"))
        .collect();
    names.sort();
    names.iter().map(|n| workflow(repo, n)).collect()
}

fn repo_manifest(repo: &str) -> manifest::CiManifest {
    let text = std::fs::read_to_string(fixture(repo).join("ci.toml")).unwrap();
    manifest::parse_and_validate(&text).unwrap_or_else(|e| panic!("{repo} ci.toml: {e}"))
}

/// Compare against (or, under `QONTINUI_CI_BLESS=1`, write) a golden file.
fn golden(name: &str, actual: &str) {
    let path = crate_dir().join("tests/golden").join(name);
    if std::env::var("QONTINUI_CI_BLESS").as_deref() == Ok("1") {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, actual).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "{}: {e} — run with QONTINUI_CI_BLESS=1 to create it",
            path.display()
        )
    });
    if expected != actual {
        let first = expected
            .lines()
            .zip(actual.lines())
            .position(|(a, b)| a != b)
            .unwrap_or_else(|| expected.lines().count().min(actual.lines().count()));
        panic!(
            "{} differs from the importer's output at line {} — expected {:?}, got {:?}. \
             Re-bless with QONTINUI_CI_BLESS=1 if the change is intended.",
            path.display(),
            first + 1,
            expected.lines().nth(first),
            actual.lines().nth(first)
        );
    }
}

/// The Phase 8 arming check: an import over qontinui-web's `backend-ci.yml`
/// and `frontend-ci.yml` yields jobs whose step commands cover what
/// qontinui-web's `.qontinui/ci.toml` declares from those two workflows. It
/// enters through `import::import_workflows`, the function the CLI's `import`
/// subcommand calls.
#[test]
fn import_over_web_backend_and_frontend_ci_covers_web_ci_toml() {
    let outcome = import::import_workflows(&[
        workflow("qontinui-web", "backend-ci.yml"),
        workflow("qontinui-web", "frontend-ci.yml"),
    ])
    .expect("import");
    let generated = outcome.manifest.as_ref().expect("a valid generated manifest");
    // The generated manifest is itself a valid v2 manifest with these jobs.
    let names: Vec<&str> = generated.jobs.iter().map(|j| j.name.as_str()).collect();
    for want in ["lint", "test", "lint-and-typecheck", "composed-cloud-build"] {
        assert!(names.contains(&want), "missing job {want}: {names:?}");
    }

    let web = repo_manifest("qontinui-web");
    let coverage = import::manifest_steps_covered_by(&web, &outcome);
    let uncovered: Vec<&str> = coverage
        .iter()
        .filter(|(_, hit)| hit.is_none())
        .map(|(step, _)| step.as_str())
        .collect();
    // Every step web's manifest mirrors from backend-ci.yml / frontend-ci.yml
    // is covered. What remains are exactly the steps it mirrors from OTHER
    // workflows (its header says so step by step) — the next test covers
    // those by importing their source workflows too.
    assert_eq!(
        uncovered,
        vec![
            "ci/web-boundary-lint",
            "ci/forbid-public-schema",
            "ci/alembic-single-head",
            "ci/coord-column-drop-guard",
            "ci/gitignore-tracked-files",
            "ci/ruff-version-parity",
            "ci/path-case-collisions",
            "ci/forbid-fixed-sleeps-in-e2e",
            "ci/global-state-assertions",
        ],
        "full coverage: {coverage:#?}"
    );
    for step in [
        "ci/frontend-install",
        "ci/frontend-lint",
        "ci/frontend-typecheck",
        "ci/frontend-test",
        "ci/composed-cloud-install",
        "ci/composed-build",
        "ci/backend-install",
        "ci/backend-install-cloud-control",
        "ci/backend-ruff-check",
        "ci/backend-ruff-format",
        "ci/backend-mypy",
        "ci/openapi-export-extended",
        "ci/openapi-export-base",
        "ci/openapi-snapshot-drift",
    ] {
        let hit = coverage.iter().find(|(s, _)| s == step).and_then(|(_, h)| h.clone());
        assert!(hit.is_some(), "{step} is not covered: {coverage:#?}");
    }
    // Nothing was dropped silently: what the import could not carry is listed
    // in the generated header.
    assert!(outcome.count(ItemKind::Untranslated) > 0);
    for item in outcome.items.iter().filter(|i| i.kind == ItemKind::Untranslated) {
        assert!(
            outcome.manifest_toml.contains(&item.location),
            "item at {} missing from the header",
            item.location
        );
    }
}

/// The rest of web's manifest comes from its single-check workflows; importing
/// them too covers every step but one, which is named.
#[test]
fn import_over_every_web_source_workflow_covers_all_but_named_steps() {
    let mut sources = vec![
        workflow("qontinui-web", "backend-ci.yml"),
        workflow("qontinui-web", "frontend-ci.yml"),
    ];
    for f in [
        "web-boundary-lint.yml",
        "forbid-public-schema.yml",
        "alembic-graph-pr.yml",
        "coord-column-drop-guard.yml",
        "gitignore-tracked-files.yml",
        "ruff-version-parity.yml",
        "path-case-collisions.yml",
        "forbid-fixed-sleeps-in-e2e.yml",
        "global-state-assertions.yml",
    ] {
        sources.push(workflow("qontinui-web", f));
    }
    let outcome = import::import_workflows(&sources).expect("import");
    let web = repo_manifest("qontinui-web");
    let uncovered: Vec<String> = import::manifest_steps_covered_by(&web, &outcome)
        .into_iter()
        .filter(|(_, hit)| hit.is_none())
        .map(|(s, _)| s)
        .collect();
    // Two steps stay uncovered, each because the workflow runs the script
    // with arguments the manifest step does not pass, and coverage compares
    // the whole argv:
    // * alembic-graph-pr.yml runs `count_alembic_heads.py --baseline-ref
    //   origin/main`; the manifest runs it bare.
    // * coord-column-drop-guard.yml passes `--base-ref ${BASE_REF}` from the
    //   event payload; a run-time variable has no argv form, so the import
    //   lists that step as untranslated instead of guessing a ref.
    assert_eq!(
        uncovered,
        vec!["ci/alembic-single-head".to_string(), "ci/coord-column-drop-guard".to_string()]
    );
    assert!(outcome.items.iter().any(|i| i.kind == ItemKind::Untranslated
        && i.location.starts_with("coord-column-drop-guard.yml")
        && i.detail.contains("BASE_REF")));
}

/// `import --report` over web's real manifest and every fixture workflow:
/// the backend test suite is NOT covered (the plan's Phase 9 finding), the
/// single-check workflows are.
#[test]
fn web_report_finds_the_uncovered_backend_suite() {
    let report = import::coverage_report(
        ".qontinui/ci.toml",
        &repo_manifest("qontinui-web"),
        &all_workflows("qontinui-web"),
        "main",
    )
    .unwrap();
    let verdict = |wf: &str, job: &str| {
        report
            .jobs
            .iter()
            .find(|j| j.workflow == wf && j.job_id == job)
            .unwrap_or_else(|| panic!("{wf} › {job} not considered"))
            .verdict
    };
    assert_eq!(verdict("web-boundary-lint.yml", "web-boundary-lint"), Coverage::Covered);
    // The manifest runs count_alembic_heads.py without the workflow's
    // `--baseline-ref origin/main`: a different command, so not covered.
    assert_eq!(verdict("alembic-graph-pr.yml", "alembic-heads-pr"), Coverage::Uncovered);
    assert_eq!(verdict("forbid-public-schema.yml", "forbid-public-schema"), Coverage::Covered);
    assert_eq!(verdict("backend-ci.yml", "test"), Coverage::Partial);
    // Its installs match the manifest's; Trivy and `safety check` do not.
    assert_eq!(verdict("backend-ci.yml", "security-scan"), Coverage::Partial);
    assert!(!report.all_covered());
    let test = report
        .jobs
        .iter()
        .find(|j| j.workflow == "backend-ci.yml" && j.job_id == "test")
        .unwrap();
    assert!(
        test.missing.iter().any(|m| m.contains("pytest") && m.contains("--cov")),
        "{:#?}",
        test.missing
    );
}

/// For every repo: the import of all its workflows, the coverage report of
/// its manifest, and the generated workflow, against golden files.
#[test]
fn golden_import_report_and_gen_workflow_for_every_repo() {
    for repo in REPOS {
        let sources = all_workflows(repo);
        assert!(!sources.is_empty(), "{repo} has no workflow fixtures");
        let outcome = import::import_workflows(&sources)
            .unwrap_or_else(|e| panic!("{repo}: import failed: {e}"));
        if outcome.manifest.is_none() {
            assert!(outcome.jobs.is_empty());
            assert!(outcome.manifest_toml.contains("NO JOB COULD BE TRANSLATED"));
        }
        // Every construct is accounted for in the header.
        for item in &outcome.items {
            let first_word = item.detail.split_whitespace().next().unwrap_or("");
            assert!(
                outcome.manifest_toml.contains(&item.location) && outcome.manifest_toml.contains(first_word),
                "{repo}: item {item:?} missing from the header"
            );
        }
        golden(&format!("{repo}.import.toml"), &outcome.manifest_toml);

        if SIBLINGS_ONLY.contains(repo) {
            let text = std::fs::read_to_string(fixture(repo).join("ci.toml")).unwrap();
            let err = manifest::parse_and_validate(&text).unwrap_err();
            assert!(err.contains("no [[steps]]"), "{repo}: {err}");
            continue;
        }
        let m = repo_manifest(repo);
        let report = import::coverage_report(".qontinui/ci.toml", &m, &sources, "main")
            .unwrap_or_else(|e| panic!("{repo}: report failed: {e}"));
        golden(&format!("{repo}.report.txt"), &import::render_report(&report));

        let generated = gen_workflow::generate(
            &m,
            &GenOptions {
                manifest_path: ".qontinui/ci.toml".to_string(),
                executor_rev: EXECUTOR_REV.to_string(),
            },
        );
        assert_eq!(gen_workflow::check(Some(&generated), &generated), CheckVerdict::UpToDate);
        let y: serde_yaml_ng::Value = serde_yaml_ng::from_str(&generated)
            .unwrap_or_else(|e| panic!("{repo}: generated workflow is not YAML: {e}"));
        let jobs = y.get("jobs").and_then(|j| j.as_mapping()).unwrap();
        let gate: Vec<&str> = m.gate_jobs().map(|j| j.name.as_str()).collect();
        if gate.is_empty() {
            assert_eq!(jobs.len(), 1);
        } else {
            assert_eq!(jobs.len(), gate.len(), "{repo}: one generated job per gate job");
            for name in &gate {
                let job = jobs.get(*name).unwrap_or_else(|| panic!("{repo}: no job {name}"));
                let steps = job.get("steps").unwrap().as_sequence().unwrap();
                let runs: Vec<&str> = steps.iter().filter_map(|s| s.get("run")?.as_str()).collect();
                assert!(
                    runs.iter().any(|r| r.starts_with(&format!("qontinui-ci run --job {name} "))),
                    "{repo}: job {name} does not run qontinui-ci run --job {name}"
                );
            }
        }
        golden(&format!("{repo}.qontinui-ci.yml"), &generated);
    }
}

/// The repos whose `.github/workflows/qontinui-ci.yml` is hand-written today
/// are reported as such, never as stale — and the CLI refuses to overwrite
/// them.
#[test]
fn hand_written_qontinui_ci_yml_is_not_generated() {
    for repo in ["multistate", "ui-bridge", "qontinui-inspect"] {
        let existing = std::fs::read_to_string(fixture(repo).join("workflows/qontinui-ci.yml")).unwrap();
        // qontinui-inspect's manifest is siblings-only (see SIBLINGS_ONLY), so
        // there is nothing to generate for it; the header check decides first
        // either way.
        let generated = if SIBLINGS_ONLY.contains(&repo) {
            String::new()
        } else {
            gen_workflow::generate(
                &repo_manifest(repo),
                &GenOptions {
                    manifest_path: ".qontinui/ci.toml".to_string(),
                    executor_rev: EXECUTOR_REV.to_string(),
                },
            )
        };
        assert_eq!(gen_workflow::check(Some(&existing), &generated), CheckVerdict::NotGenerated);
    }
}

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_qontinui-ci"))
}

fn write(dir: &Path, rel: &str, text: &str) {
    let p = dir.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
}

/// The CLI end to end: gen-workflow writes, --check passes, a hand-edit fails
/// --check, a hand-written file is not overwritten, and import refuses to
/// clobber an existing manifest.
#[test]
fn cli_gen_workflow_check_and_import_refusals() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(root, ".qontinui/ci.toml", &std::fs::read_to_string(fixture("qontinui-runner").join("ci.toml")).unwrap());

    let check = |root: &Path| {
        cli()
            .current_dir(root)
            .args(["gen-workflow", "--check", "--executor-rev", EXECUTOR_REV])
            .output()
            .unwrap()
    };
    let out = check(root);
    assert_eq!(out.status.code(), Some(1), "missing file must fail --check");

    // No pin, or a movable one, is refused.
    let out = cli().current_dir(root).args(["gen-workflow"]).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("--executor-rev"));
    let out = cli().current_dir(root).args(["gen-workflow", "--executor-rev", "main"]).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    let out = cli().current_dir(root).args(["gen-workflow", "--executor-rev", EXECUTOR_REV]).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(check(root).status.code(), Some(0));

    let wf = root.join(gen_workflow::WORKFLOW_PATH);
    let text = std::fs::read_to_string(&wf).unwrap();
    std::fs::write(&wf, text.replacen("runs-on: ubuntu-latest", "runs-on: ubuntu-22.04", 1)).unwrap();
    let out = check(root);
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("hand-edited"));

    std::fs::write(&wf, "name: hand written\non: push\njobs: {}\n").unwrap();
    let out = cli().current_dir(root).args(["gen-workflow", "--executor-rev", EXECUTOR_REV]).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("not a generated file"));
    let out = cli()
        .current_dir(root)
        .args(["gen-workflow", "--executor-rev", EXECUTOR_REV, "--force"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(check(root).status.code(), Some(0));

    // import: stdout by default; --out refuses an existing file.
    let backend = fixture("qontinui-web").join("workflows/backend-ci.yml");
    let out = cli()
        .args(["import", backend.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("version = 2"));
    assert!(String::from_utf8_lossy(&out.stderr).contains("UNTRANSLATED"));
    let out = cli()
        .args(["import", "--out", root.join(".qontinui/ci.toml").to_str().unwrap(), backend.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("refusing to overwrite"));

    // --report exits 1 when a considered job is not covered.
    let out = cli()
        .args([
            "import",
            "--report",
            fixture("qontinui-web").join("ci.toml").to_str().unwrap(),
            backend.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stdout).contains("NOT MET"));
}
