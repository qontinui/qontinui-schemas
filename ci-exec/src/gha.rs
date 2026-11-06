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
    /// Sections given in a shape the model cannot read (an `env:` that is an
    /// expression, `on` given twice, …). Never silently empty: each entry
    /// says what could not be read.
    pub unreadable: Vec<String>,
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
    /// Whether this event fires for a ref on `branch`: `None` when a filter
    /// pattern uses syntax the matcher does not model — the caller must then
    /// treat the workflow as firing (CONSIDERED), never as excluded. A `push`
    /// filtered only by `tags` fires for no branch at all.
    pub fn fires_for_branch(&self, branch: &str) -> Option<bool> {
        if self.branches.is_none()
            && self.branches_ignore.is_none()
            && (self.tags.is_some() || self.tags_ignore.is_some())
        {
            return Some(false);
        }
        if let Some(ignore) = &self.branches_ignore {
            for p in ignore {
                if glob_match(p, branch)? {
                    return Some(false);
                }
            }
        }
        match &self.branches {
            None => Some(true),
            Some(patterns) => {
                // GitHub evaluates the list in order; a `!pattern` re-excludes.
                let mut included = false;
                for p in patterns {
                    if let Some(neg) = p.strip_prefix('!') {
                        if glob_match(neg, branch)? {
                            included = false;
                        }
                    } else if glob_match(p, branch)? {
                        included = true;
                    }
                }
                Some(included)
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

/// One element of a filter pattern.
#[derive(Debug, Clone)]
enum PTok {
    Lit(char),
    /// `*`: any run of characters except `/`.
    Star,
    /// `**`: any run of characters.
    DStar,
    /// `[…]`: a character class of inclusive ranges.
    Class(Vec<(char, char)>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Quant {
    One,
    /// `?` after a character: zero or one of it.
    ZeroOrOne,
    /// `+` after a character: one or more of it.
    OneOrMore,
}

/// Parse GitHub's filter-pattern syntax: `*`, `**`, `?` (zero or one of the
/// preceding character), `+` (one or more of it), `[…]` classes with ranges,
/// and backslash escapes. `None` for anything this does not model (a
/// quantifier with nothing to apply to, an unclosed or negated class).
fn parse_pattern(p: &str) -> Option<Vec<(PTok, Quant)>> {
    let chars: Vec<char> = p.chars().collect();
    let mut out: Vec<(PTok, Quant)> = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '*' if chars.get(i + 1) == Some(&'*') => {
                out.push((PTok::DStar, Quant::One));
                i += 2;
            }
            '*' => {
                out.push((PTok::Star, Quant::One));
                i += 1;
            }
            q @ ('?' | '+') => {
                let last = out.last_mut()?;
                if !matches!(last.0, PTok::Lit(_) | PTok::Class(_)) || last.1 != Quant::One {
                    return None;
                }
                last.1 = if q == '?' {
                    Quant::ZeroOrOne
                } else {
                    Quant::OneOrMore
                };
                i += 1;
            }
            '[' => {
                let close = chars[i + 1..].iter().position(|c| *c == ']')? + i + 1;
                let body = &chars[i + 1..close];
                if body.is_empty() || body[0] == '!' || body[0] == '^' {
                    return None;
                }
                let mut ranges = Vec::new();
                let mut k = 0;
                while k < body.len() {
                    if k + 2 < body.len() && body[k + 1] == '-' {
                        ranges.push((body[k], body[k + 2]));
                        k += 3;
                    } else {
                        ranges.push((body[k], body[k]));
                        k += 1;
                    }
                }
                out.push((PTok::Class(ranges), Quant::One));
                i = close + 1;
            }
            '\\' => {
                out.push((PTok::Lit(*chars.get(i + 1)?), Quant::One));
                i += 2;
            }
            ']' => return None,
            c => {
                out.push((PTok::Lit(c), Quant::One));
                i += 1;
            }
        }
    }
    Some(out)
}

fn tok_matches(t: &PTok, c: char) -> bool {
    match t {
        PTok::Lit(l) => *l == c,
        PTok::Class(r) => r.iter().any(|(a, b)| *a <= c && c <= *b),
        PTok::Star | PTok::DStar => false,
    }
}

fn match_pieces(p: &[(PTok, Quant)], t: &[char]) -> bool {
    let Some(((tok, q), rest)) = p.split_first() else {
        return t.is_empty();
    };
    match tok {
        PTok::DStar => (0..=t.len()).any(|i| match_pieces(rest, &t[i..])),
        PTok::Star => {
            let mut i = 0;
            loop {
                if match_pieces(rest, &t[i..]) {
                    return true;
                }
                if i == t.len() || t[i] == '/' {
                    return false;
                }
                i += 1;
            }
        }
        _ => match q {
            Quant::One => !t.is_empty() && tok_matches(tok, t[0]) && match_pieces(rest, &t[1..]),
            Quant::ZeroOrOne => {
                match_pieces(rest, t)
                    || (!t.is_empty() && tok_matches(tok, t[0]) && match_pieces(rest, &t[1..]))
            }
            Quant::OneOrMore => {
                let mut i = 0;
                while i < t.len() && tok_matches(tok, t[i]) {
                    i += 1;
                    if match_pieces(rest, &t[i..]) {
                        return true;
                    }
                }
                false
            }
        },
    }
}

/// Match a GitHub branch filter pattern. `None` when the pattern uses syntax
/// [`parse_pattern`] does not model.
pub fn glob_match(pattern: &str, text: &str) -> Option<bool> {
    let pieces = parse_pattern(pattern)?;
    let t: Vec<char> = text.chars().collect();
    Some(match_pieces(&pieces, &t))
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
    /// Sections the model cannot read (see [`Workflow::unreadable`]).
    pub unreadable: Vec<String>,
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
    /// Sections the model cannot read (a `run:` that is not a string, a
    /// `with:` that is an expression, …).
    pub unreadable: Vec<String>,
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
            let first = r
                .lines()
                .find(|l| !l.trim().is_empty())
                .unwrap_or("")
                .trim();
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
        unreadable: Vec::new(),
        jobs: Vec::new(),
    };
    let mut saw_jobs = false;
    let mut saw_on = false;
    for (k, v) in map {
        // YAML 1.1 readers turn a bare `on` into `true`; accept either.
        let key = match k {
            Value::Bool(true) => "on".to_string(),
            other => scalar_string(other).unwrap_or_default(),
        };
        match key.as_str() {
            "name" => wf.name = scalar_string(v),
            "on" => {
                if saw_on {
                    wf.unreadable.push(
                        "`on` is given twice (as `on` and as a YAML 1.1 `true` key)".to_string(),
                    );
                }
                saw_on = true;
                if !matches!(v, Value::String(_) | Value::Sequence(_) | Value::Mapping(_)) {
                    wf.unreadable.push(format!("`on` is {}", render(v)));
                }
                wf.triggers = parse_triggers(v);
            }
            "env" => wf.env = string_map(v, "workflow `env`", &mut wf.unreadable),
            "defaults" => {
                let (wd, shell) = run_defaults(v, "workflow", &mut wf.unreadable);
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
        unreadable: Vec::new(),
        steps: Vec::new(),
    };
    let Value::Mapping(m) = v else {
        job.unreadable
            .push(format!("the job is {}, not a mapping", render(v)));
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
            "if" => {
                job.if_expr = scalar_string(val);
                if job.if_expr.is_none() {
                    job.unreadable.push(format!("job `if` is {}", render(val)));
                }
            }
            "strategy" => {
                match val {
                    Value::Mapping(sm) => {
                        for sk in sm.keys().filter_map(scalar_string) {
                            if !matches!(sk.as_str(), "matrix" | "fail-fast" | "max-parallel") {
                                job.unreadable.push(format!("`strategy` key `{sk}`"));
                            }
                        }
                    }
                    other => job
                        .unreadable
                        .push(format!("`strategy` is {}", render(other))),
                }
                if let Value::Mapping(sm) = val {
                    if let Some(mx) = sm.get("matrix") {
                        job.matrix = Some(match mx {
                            Value::Mapping(dims) => Matrix::Dimensions(
                                dims.iter()
                                    .map(|(dk, dv)| {
                                        let name = scalar_string(dk).unwrap_or_default();
                                        let values = match dv {
                                            Value::Sequence(seq) => {
                                                seq.iter().map(render).collect()
                                            }
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
                if !matches!(val, Value::Mapping(_) | Value::Null) {
                    job.unreadable
                        .push(format!("`services` is {}", render(val)));
                }
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
                            Value::String(s) => (Some(s.clone()), Vec::new()),
                            other => {
                                job.unreadable
                                    .push(format!("service `{key}` is {}", render(other)));
                                (None, Vec::new())
                            }
                        };
                        job.services.push(Service {
                            key,
                            image,
                            other_keys,
                        });
                    }
                }
            }
            "env" => job.env = string_map(val, "job `env`", &mut job.unreadable),
            "defaults" => {
                let (wd, shell) = run_defaults(val, "job", &mut job.unreadable);
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
            "steps" => match val {
                Value::Sequence(seq) => {
                    for (i, sv) in seq.iter().enumerate() {
                        job.steps.push(parse_step(i, sv));
                    }
                }
                other => job.unreadable.push(format!("`steps` is {}", render(other))),
            },
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
        unreadable: Vec::new(),
    };
    let Value::Mapping(m) = v else {
        step.unreadable.push(format!(
            "step {} is {}, not a mapping",
            index + 1,
            render(v)
        ));
        return step;
    };
    for (k, val) in m {
        let key = scalar_string(k).unwrap_or_default();
        match key.as_str() {
            "name" => step.name = scalar_string(val),
            "id" => step.id = scalar_string(val),
            "uses" => match val {
                Value::String(u) => step.uses = Some(u.clone()),
                other => step.unreadable.push(format!("`uses` is {}", render(other))),
            },
            "with" => step.with = string_map(val, "`with`", &mut step.unreadable),
            "run" => match val {
                Value::String(r) => step.run = Some(r.clone()),
                other => step
                    .unreadable
                    .push(format!("`run` is {}, not a string", render(other))),
            },
            "shell" => step.shell = scalar_string(val),
            "working-directory" => step.working_dir = scalar_string(val),
            "env" => step.env = string_map(val, "step `env`", &mut step.unreadable),
            "if" => {
                step.if_expr = scalar_string(val);
                if step.if_expr.is_none() {
                    step.unreadable
                        .push(format!("step `if` is {}", render(val)));
                }
            }
            "continue-on-error" => step.continue_on_error = scalar_string(val).map(Scalar),
            "timeout-minutes" => step.timeout_minutes = scalar_string(val).map(Scalar),
            other => step.other_keys.push(other.to_string()),
        }
    }
    step
}

fn run_defaults(v: &Value, scope: &str, bad: &mut Vec<String>) -> (Option<String>, Option<String>) {
    let Value::Mapping(m) = v else {
        bad.push(format!("{scope} `defaults` is {}", render(v)));
        return (None, None);
    };
    for k in m.keys().filter_map(scalar_string).filter(|k| k != "run") {
        bad.push(format!("{scope} `defaults` key `{k}`"));
    }
    let run = match m.get("run") {
        None => return (None, None),
        Some(Value::Mapping(run)) => run,
        Some(other) => {
            bad.push(format!("{scope} `defaults.run` is {}", render(other)));
            return (None, None);
        }
    };
    for k in run.keys().filter_map(scalar_string) {
        if !matches!(k.as_str(), "working-directory" | "shell") {
            bad.push(format!("{scope} `defaults.run` key `{k}`"));
        }
    }
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

/// A `KEY: value` mapping. Anything else — an expression, a list, a
/// non-scalar value, a tagged value — is recorded in `bad` (never silently
/// read as empty). `null` is an empty mapping.
fn string_map(v: &Value, what: &str, bad: &mut Vec<String>) -> Vec<(String, String)> {
    let m = match v {
        Value::Mapping(m) => m,
        Value::Null => return Vec::new(),
        other => {
            bad.push(format!("{what} is {}, not a mapping", render(other)));
            return Vec::new();
        }
    };
    let mut out = Vec::new();
    for (k, val) in m {
        match (k, val) {
            (Value::String(k), Value::String(_) | Value::Number(_) | Value::Bool(_)) => {
                out.push((k.clone(), scalar_string(val).unwrap_or_default()))
            }
            (Value::String(k), Value::Null) => out.push((k.clone(), String::new())),
            _ => bad.push(format!("{what} entry {} = {}", render(k), render(val))),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_matches_github_filter_semantics() {
        let g = |p: &str, t: &str| glob_match(p, t);
        assert_eq!(g("main", "main"), Some(true));
        assert_eq!(g("main", "maint"), Some(false));
        assert_eq!(g("release/*", "release/1"), Some(true));
        assert_eq!(g("release/*", "release/1/2"), Some(false));
        assert_eq!(g("merge-candidate/**", "merge-candidate/a/b"), Some(true));
        assert_eq!(g("*", "main"), Some(true));
        // `?` is zero-or-one of the PRECEDING character, not "any character".
        assert_eq!(g("mainn?", "main"), Some(true));
        assert_eq!(g("ma?n", "main"), Some(false));
        assert_eq!(g("ma+in", "maaain"), Some(true));
        assert_eq!(g("ma+in", "main"), Some(true));
        assert_eq!(g("mai[n]", "main"), Some(true));
        assert_eq!(g("v[0-9]", "v7"), Some(true));
        assert_eq!(g("[!x]", "y"), None);
        assert_eq!(g("a[", "a"), None);
    }

    #[test]
    fn push_filters_decide_default_branch_firing() {
        let f = EventFilter {
            branches: Some(vec!["main".into(), "merge-candidate/**".into()]),
            ..Default::default()
        };
        assert_eq!(f.fires_for_branch("main"), Some(true));
        assert_eq!(f.fires_for_branch("develop"), Some(false));
        let tags_only = EventFilter {
            tags: Some(vec!["v*".into()]),
            ..Default::default()
        };
        assert_eq!(tags_only.fires_for_branch("main"), Some(false));
        assert!(!tags_only.fires_for_some_branch());
        let negated = EventFilter {
            branches: Some(vec!["**".into(), "!main".into()]),
            ..Default::default()
        };
        assert_eq!(negated.fires_for_branch("main"), Some(false));
        let ignored = EventFilter {
            branches_ignore: Some(vec!["main".into()]),
            ..Default::default()
        };
        assert_eq!(ignored.fires_for_branch("main"), Some(false));
        assert_eq!(EventFilter::default().fires_for_branch("main"), Some(true));
        let unknown = EventFilter {
            branches: Some(vec!["[!x]".into()]),
            ..Default::default()
        };
        assert_eq!(unknown.fires_for_branch("main"), None);
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
