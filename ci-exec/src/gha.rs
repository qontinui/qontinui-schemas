//! A read-only model of a GitHub Actions workflow file — exactly as much of
//! one as `qontinui-ci import` translates or must report.
//!
//! This is NOT an Actions emulator (plan
//! `2026-10-04-coord-managed-ci-for-every-tenant-with-an-actions-free-mode`,
//! D2 rejects that). It reads the YAML into plain values so the importer can
//! decide, construct by construct, what maps onto `.qontinui/ci.toml` and what
//! it must list as untranslated. Every key the importer does not interpret is
//! kept in an `other_keys` list so the importer can name it instead of
//! dropping it.

use serde_yaml_ng::Value;

/// One parsed workflow file.
#[derive(Debug, Clone)]
pub struct Workflow {
    /// The label the file was given (its path as the user passed it).
    pub file: String,
    pub name: Option<String>,
    pub triggers: Triggers,
    /// Workflow-level `env`.
    pub env: Vec<(String, String)>,
    /// `defaults.run.working-directory` at workflow level.
    pub default_working_dir: Option<String>,
    /// `defaults.run.shell` at workflow level.
    pub default_shell: Option<String>,
    /// Top-level keys other than `name`, `on`, `env`, `defaults`, `jobs`
    /// (`permissions`, `concurrency`, `run-name`, …).
    pub other_keys: Vec<String>,
    pub jobs: Vec<Job>,
}

/// The `on:` block, reduced to what decides whether a job is a gate.
#[derive(Debug, Clone, Default)]
pub struct Triggers {
    /// Every event name, in file order.
    pub events: Vec<String>,
    /// `pull_request` / `pull_request_target` filters, when present.
    pub pull_request: Option<EventFilter>,
    pub pull_request_target: Option<EventFilter>,
    pub push: Option<EventFilter>,
    /// `schedule` cron expressions.
    pub schedules: Vec<String>,
}

/// The branch/path filters of one event.
#[derive(Debug, Clone, Default)]
pub struct EventFilter {
    pub branches: Option<Vec<String>>,
    pub branches_ignore: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub tags_ignore: Option<Vec<String>>,
    /// `types` (activity types of a pull request).
    pub types: Option<Vec<String>>,
    /// `paths` or `paths-ignore` was given — the event fires only for some
    /// changes.
    pub path_filtered: bool,
}

impl EventFilter {
    /// Whether this event fires for a ref on `branch`. A `push` filtered only
    /// by `tags` fires for no branch at all.
    pub fn fires_for_branch(&self, branch: &str) -> bool {
        if self.branches.is_none()
            && self.branches_ignore.is_none()
            && (self.tags.is_some() || self.tags_ignore.is_some())
        {
            return false;
        }
        if let Some(ignore) = &self.branches_ignore {
            if ignore.iter().any(|p| glob_match(p, branch)) {
                return false;
            }
        }
        match &self.branches {
            None => true,
            Some(patterns) => {
                // GitHub evaluates the list in order; a `!pattern` re-excludes.
                let mut included = false;
                for p in patterns {
                    if let Some(neg) = p.strip_prefix('!') {
                        if glob_match(neg, branch) {
                            included = false;
                        }
                    } else if glob_match(p, branch) {
                        included = true;
                    }
                }
                included
            }
        }
    }

    /// Whether this event fires for at least one branch (a branch push or
    /// pull request, as opposed to a tag-only push).
    pub fn fires_for_some_branch(&self) -> bool {
        !(self.branches.is_none()
            && self.branches_ignore.is_none()
            && (self.tags.is_some() || self.tags_ignore.is_some()))
    }
}

/// GitHub's filter-pattern subset: `*` matches within one path segment, `**`
/// across segments, `?` one character. Character classes are matched
/// literally — no branch name in a filter we read uses one.
pub fn glob_match(pattern: &str, text: &str) -> bool {
    fn go(p: &[char], t: &[char]) -> bool {
        match p.first() {
            None => t.is_empty(),
            Some('*') => {
                if p.get(1) == Some(&'*') {
                    let rest = &p[2..];
                    (0..=t.len()).any(|i| go(rest, &t[i..]))
                } else {
                    let rest = &p[1..];
                    let mut i = 0;
                    loop {
                        if go(rest, &t[i..]) {
                            return true;
                        }
                        if i == t.len() || t[i] == '/' {
                            return false;
                        }
                        i += 1;
                    }
                }
            }
            Some('?') => !t.is_empty() && t[0] != '/' && go(&p[1..], &t[1..]),
            Some(c) => t.first() == Some(c) && go(&p[1..], &t[1..]),
        }
    }
    let p: Vec<char> = pattern.chars().collect();
    let t: Vec<char> = text.chars().collect();
    go(&p, &t)
}

/// One job.
#[derive(Debug, Clone)]
pub struct Job {
    /// The key under `jobs:`.
    pub id: String,
    pub name: Option<String>,
    pub runs_on: RunsOn,
    pub needs: Vec<String>,
    pub if_expr: Option<String>,
    /// `strategy.matrix`, when present.
    pub matrix: Option<Matrix>,
    pub services: Vec<Service>,
    pub env: Vec<(String, String)>,
    pub default_working_dir: Option<String>,
    pub default_shell: Option<String>,
    pub timeout_minutes: Option<Scalar>,
    /// A reusable-workflow call (`jobs.<id>.uses`).
    pub uses: Option<String>,
    /// `container:` — the job runs inside an image.
    pub container: Option<String>,
    pub environment: Option<String>,
    pub continue_on_error: Option<Scalar>,
    /// Keys other than the ones above (`permissions`, `concurrency`,
    /// `outputs`, …).
    pub other_keys: Vec<String>,
    pub steps: Vec<Step>,
}

/// `runs-on`, as written.
#[derive(Debug, Clone)]
pub enum RunsOn {
    /// Absent (a reusable-workflow call has none).
    Missing,
    /// One or more labels; a label may be a `${{ … }}` expression.
    Labels(Vec<String>),
    /// Something else (a `group:` map, a non-string).
    Other(String),
}

/// `strategy.matrix`.
#[derive(Debug, Clone)]
pub enum Matrix {
    /// The whole matrix is an expression (`${{ fromJSON(…) }}`).
    Expression(String),
    /// Dimension name → its literal values (scalars rendered as strings;
    /// an `include`/`exclude` entry is kept under that key, rendered).
    Dimensions(Vec<(String, Vec<String>)>),
}

/// A scalar YAML value rendered as a string, or an expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scalar(pub String);

impl Scalar {
    pub fn is_expression(&self) -> bool {
        self.0.contains("${{")
    }
}

/// One `services:` entry.
#[derive(Debug, Clone)]
pub struct Service {
    pub key: String,
    /// `image:` (or the bare string form).
    pub image: Option<String>,
    /// Keys other than `image` (`env`, `ports`, `options`, `credentials`, …).
    pub other_keys: Vec<String>,
}

/// One step.
#[derive(Debug, Clone)]
pub struct Step {
    /// 0-based position in the job.
    pub index: usize,
    pub name: Option<String>,
    pub id: Option<String>,
    pub uses: Option<String>,
    pub with: Vec<(String, String)>,
    pub run: Option<String>,
    pub shell: Option<String>,
    pub working_dir: Option<String>,
    pub env: Vec<(String, String)>,
    pub if_expr: Option<String>,
    pub continue_on_error: Option<Scalar>,
    pub timeout_minutes: Option<Scalar>,
    /// Keys the model does not interpret.
    pub other_keys: Vec<String>,
}

impl Step {
    /// A human label: the step's `name`, else its `uses`, else the first line
    /// of its `run`, else its position.
    pub fn label(&self) -> String {
        if let Some(n) = &self.name {
            return n.clone();
        }
        if let Some(u) = &self.uses {
            return u.clone();
        }
        if let Some(r) = &self.run {
            let first = r.lines().find(|l| !l.trim().is_empty()).unwrap_or("").trim();
            let mut s: String = first.chars().take(60).collect();
            if first.chars().count() > 60 {
                s.push('…');
            }
            return s;
        }
        format!("step {}", self.index + 1)
    }
}

/// Parse one workflow file.
pub fn parse_workflow(file: &str, text: &str) -> Result<Workflow, String> {
    let root: Value =
        serde_yaml_ng::from_str(text).map_err(|e| format!("{file}: YAML parse error: {e}"))?;
    let Value::Mapping(map) = &root else {
        return Err(format!("{file}: a workflow must be a YAML mapping"));
    };
    let mut wf = Workflow {
        file: file.to_string(),
        name: None,
        triggers: Triggers::default(),
        env: Vec::new(),
        default_working_dir: None,
        default_shell: None,
        other_keys: Vec::new(),
        jobs: Vec::new(),
    };
    let mut saw_jobs = false;
    for (k, v) in map {
        // YAML 1.1 readers turn a bare `on` into `true`; accept either.
        let key = match k {
            Value::Bool(true) => "on".to_string(),
            other => scalar_string(other).unwrap_or_default(),
        };
        match key.as_str() {
            "name" => wf.name = scalar_string(v),
            "on" => wf.triggers = parse_triggers(v),
            "env" => wf.env = string_map(v),
            "defaults" => {
                let (wd, shell) = run_defaults(v);
                wf.default_working_dir = wd;
                wf.default_shell = shell;
            }
            "jobs" => {
                saw_jobs = true;
                let Value::Mapping(jobs) = v else {
                    return Err(format!("{file}: `jobs` must be a mapping"));
                };
                for (jk, jv) in jobs {
                    let id = scalar_string(jk).unwrap_or_default();
                    wf.jobs.push(parse_job(&id, jv));
                }
            }
            other => wf.other_keys.push(other.to_string()),
        }
    }
    if !saw_jobs {
        return Err(format!("{file}: no `jobs:` — not a workflow file"));
    }
    Ok(wf)
}

fn parse_triggers(v: &Value) -> Triggers {
    let mut t = Triggers::default();
    match v {
        Value::String(s) => {
            t.events.push(s.clone());
            apply_event(&mut t, s, &Value::Null);
        }
        Value::Sequence(seq) => {
            for e in seq {
                if let Some(s) = scalar_string(e) {
                    t.events.push(s.clone());
                    apply_event(&mut t, &s, &Value::Null);
                }
            }
        }
        Value::Mapping(m) => {
            for (k, ev) in m {
                if let Some(s) = scalar_string(k) {
                    t.events.push(s.clone());
                    apply_event(&mut t, &s, ev);
                }
            }
        }
        _ => {}
    }
    t
}

fn apply_event(t: &mut Triggers, name: &str, body: &Value) {
    match name {
        "pull_request" => t.pull_request = Some(event_filter(body)),
        "pull_request_target" => t.pull_request_target = Some(event_filter(body)),
        "push" => t.push = Some(event_filter(body)),
        "schedule" => {
            if let Value::Sequence(seq) = body {
                for entry in seq {
                    if let Value::Mapping(m) = entry {
                        if let Some(cron) = m.get("cron").and_then(scalar_string) {
                            t.schedules.push(cron);
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

fn event_filter(body: &Value) -> EventFilter {
    let Value::Mapping(m) = body else {
        return EventFilter::default();
    };
    EventFilter {
        branches: m.get("branches").map(string_list),
        branches_ignore: m.get("branches-ignore").map(string_list),
        tags: m.get("tags").map(string_list),
        tags_ignore: m.get("tags-ignore").map(string_list),
        types: m.get("types").map(string_list),
        path_filtered: m.contains_key("paths") || m.contains_key("paths-ignore"),
    }
}

fn parse_job(id: &str, v: &Value) -> Job {
    let mut job = Job {
        id: id.to_string(),
        name: None,
        runs_on: RunsOn::Missing,
        needs: Vec::new(),
        if_expr: None,
        matrix: None,
        services: Vec::new(),
        env: Vec::new(),
        default_working_dir: None,
        default_shell: None,
        timeout_minutes: None,
        uses: None,
        container: None,
        environment: None,
        continue_on_error: None,
        other_keys: Vec::new(),
        steps: Vec::new(),
    };
    let Value::Mapping(m) = v else {
        return job;
    };
    for (k, val) in m {
        let key = scalar_string(k).unwrap_or_default();
        match key.as_str() {
            "name" => job.name = scalar_string(val),
            "runs-on" => {
                job.runs_on = match val {
                    Value::Sequence(_) => RunsOn::Labels(string_list(val)),
                    other => match scalar_string(other) {
                        Some(s) => RunsOn::Labels(vec![s]),
                        None => RunsOn::Other(render(other)),
                    },
                }
            }
            "needs" => job.needs = string_list(val),
            "if" => job.if_expr = scalar_string(val),
            "strategy" => {
                if let Value::Mapping(sm) = val {
                    if let Some(mx) = sm.get("matrix") {
                        job.matrix = Some(match mx {
                            Value::Mapping(dims) => Matrix::Dimensions(
                                dims.iter()
                                    .map(|(dk, dv)| {
                                        let name = scalar_string(dk).unwrap_or_default();
                                        let values = match dv {
                                            Value::Sequence(seq) => seq.iter().map(render).collect(),
                                            other => vec![render(other)],
                                        };
                                        (name, values)
                                    })
                                    .collect(),
                            ),
                            other => Matrix::Expression(render(other)),
                        });
                    }
                }
            }
            "services" => {
                if let Value::Mapping(sm) = val {
                    for (sk, sv) in sm {
                        let key = scalar_string(sk).unwrap_or_default();
                        let (image, other_keys) = match sv {
                            Value::Mapping(body) => (
                                body.get("image").and_then(scalar_string),
                                body.keys()
                                    .filter_map(scalar_string)
                                    .filter(|k| k != "image")
                                    .collect(),
                            ),
                            other => (scalar_string(other), Vec::new()),
                        };
                        job.services.push(Service {
                            key,
                            image,
                            other_keys,
                        });
                    }
                }
            }
            "env" => job.env = string_map(val),
            "defaults" => {
                let (wd, shell) = run_defaults(val);
                job.default_working_dir = wd;
                job.default_shell = shell;
            }
            "timeout-minutes" => job.timeout_minutes = scalar_string(val).map(Scalar),
            "uses" => job.uses = scalar_string(val),
            "container" => {
                job.container = Some(match val {
                    Value::Mapping(cm) => cm
                        .get("image")
                        .and_then(scalar_string)
                        .unwrap_or_else(|| render(val)),
                    other => render(other),
                })
            }
            "environment" => {
                job.environment = Some(match val {
                    Value::Mapping(em) => em
                        .get("name")
                        .and_then(scalar_string)
                        .unwrap_or_else(|| render(val)),
                    other => render(other),
                })
            }
            "continue-on-error" => job.continue_on_error = scalar_string(val).map(Scalar),
            "steps" => {
                if let Value::Sequence(seq) = val {
                    for (i, sv) in seq.iter().enumerate() {
                        job.steps.push(parse_step(i, sv));
                    }
                }
            }
            other => job.other_keys.push(other.to_string()),
        }
    }
    job
}

fn parse_step(index: usize, v: &Value) -> Step {
    let mut step = Step {
        index,
        name: None,
        id: None,
        uses: None,
        with: Vec::new(),
        run: None,
        shell: None,
        working_dir: None,
        env: Vec::new(),
        if_expr: None,
        continue_on_error: None,
        timeout_minutes: None,
        other_keys: Vec::new(),
    };
    let Value::Mapping(m) = v else {
        return step;
    };
    for (k, val) in m {
        let key = scalar_string(k).unwrap_or_default();
        match key.as_str() {
            "name" => step.name = scalar_string(val),
            "id" => step.id = scalar_string(val),
            "uses" => step.uses = scalar_string(val),
            "with" => step.with = string_map(val),
            "run" => step.run = scalar_string(val),
            "shell" => step.shell = scalar_string(val),
            "working-directory" => step.working_dir = scalar_string(val),
            "env" => step.env = string_map(val),
            "if" => step.if_expr = scalar_string(val),
            "continue-on-error" => step.continue_on_error = scalar_string(val).map(Scalar),
            "timeout-minutes" => step.timeout_minutes = scalar_string(val).map(Scalar),
            other => step.other_keys.push(other.to_string()),
        }
    }
    step
}

fn run_defaults(v: &Value) -> (Option<String>, Option<String>) {
    let Value::Mapping(m) = v else {
        return (None, None);
    };
    let Some(Value::Mapping(run)) = m.get("run") else {
        return (None, None);
    };
    (
        run.get("working-directory").and_then(scalar_string),
        run.get("shell").and_then(scalar_string),
    )
}

/// A scalar as a string: strings verbatim, numbers and booleans rendered.
pub(crate) fn scalar_string(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// A one-line rendering of any value, for reports.
fn render(v: &Value) -> String {
    match scalar_string(v) {
        Some(s) => s,
        None => serde_yaml_ng::to_string(v)
            .map(|s| s.trim().replace('\n', " "))
            .unwrap_or_default(),
    }
}

fn string_list(v: &Value) -> Vec<String> {
    match v {
        Value::Sequence(seq) => seq.iter().filter_map(scalar_string).collect(),
        other => scalar_string(other).into_iter().collect(),
    }
}

fn string_map(v: &Value) -> Vec<(String, String)> {
    let Value::Mapping(m) = v else {
        return Vec::new();
    };
    m.iter()
        .filter_map(|(k, val)| Some((scalar_string(k)?, scalar_string(val).unwrap_or_else(|| render(val)))))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_matches_github_filter_semantics() {
        assert!(glob_match("main", "main"));
        assert!(!glob_match("main", "maint"));
        assert!(glob_match("release/*", "release/1"));
        assert!(!glob_match("release/*", "release/1/2"));
        assert!(glob_match("merge-candidate/**", "merge-candidate/a/b"));
        assert!(glob_match("*", "main"));
        assert!(glob_match("ma?n", "main"));
    }

    #[test]
    fn push_filters_decide_default_branch_firing() {
        let f = EventFilter {
            branches: Some(vec!["main".into(), "merge-candidate/**".into()]),
            ..Default::default()
        };
        assert!(f.fires_for_branch("main"));
        assert!(!f.fires_for_branch("develop"));
        let tags_only = EventFilter {
            tags: Some(vec!["v*".into()]),
            ..Default::default()
        };
        assert!(!tags_only.fires_for_branch("main"));
        assert!(!tags_only.fires_for_some_branch());
        let negated = EventFilter {
            branches: Some(vec!["**".into(), "!main".into()]),
            ..Default::default()
        };
        assert!(!negated.fires_for_branch("main"));
        let ignored = EventFilter {
            branches_ignore: Some(vec!["main".into()]),
            ..Default::default()
        };
        assert!(!ignored.fires_for_branch("main"));
        assert!(EventFilter::default().fires_for_branch("main"));
    }

    #[test]
    fn parses_on_forms_and_steps() {
        let wf = parse_workflow(
            "x.yml",
            "on: [push, pull_request]\njobs:\n  a:\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo hi\n",
        )
        .unwrap();
        assert_eq!(wf.triggers.events, vec!["push", "pull_request"]);
        assert!(wf.triggers.push.is_some());
        assert_eq!(wf.jobs[0].steps[0].run.as_deref(), Some("echo hi"));
        let wf = parse_workflow(
            "y.yml",
            "on:\n  schedule:\n    - cron: '0 3 * * *'\njobs:\n  a:\n    runs-on: ubuntu-latest\n    steps: []\n",
        )
        .unwrap();
        assert_eq!(wf.triggers.schedules, vec!["0 3 * * *"]);
        assert!(parse_workflow("z.yml", "name: x\n").is_err());
    }
}
