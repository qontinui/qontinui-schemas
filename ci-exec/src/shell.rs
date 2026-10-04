//! Reading a workflow `run:` script as argv commands.
//!
//! A manifest step is an argv vector, never a shell string (see
//! [`crate::manifest::CiStep`]). So `qontinui-ci import` can carry a `run:`
//! script across only when the script is a sequence of plain commands. This
//! module is the deliberately SMALL shell subset that decides that:
//!
//! | Accepted | Becomes |
//! |---|---|
//! | `cmd arg 'quoted arg' "double"` (no expansion) | one argv command |
//! | `a && b`, one command per line, `a; b` | one command each, in order (each is fail-fast — GitHub's default `bash -e` stops at the first failing line, and so does a manifest job) |
//! | `KEY=value cmd`, `export KEY=value` | the command's `env` |
//! | `cd dir` | the following commands' working directory |
//! | `$GITHUB_WORKSPACE`, `${{ github.workspace }}` | the checkout root, as a relative path |
//! | `git -C <dir-inside-the-repo> …` | `git …` in that working directory |
//! | `if ! cmd; then echo …; exit N; fi`, `cmd \|\| exit N`, `cmd \|\| { echo …; exit N; }` | `cmd` (the message lines are dropped, and said so) |
//! | `echo …` / `printf …` to the terminal, `set -euo pipefail` | nothing (said so) |
//!
//! Everything else — a pipe, a redirect to a file, a variable or command
//! substitution, a glob, a loop, a conditional of any other shape, `|| true` —
//! makes the WHOLE step untranslatable, with the reason. A step is never
//! half-translated: dropping one line of a script silently is exactly what the
//! importer must not do.
//!
//! [`extract_loose`] is the other half: a tolerant reading that pulls every
//! command name out of ANY script, for `import --report`'s coverage check. It
//! never fails and never feeds a manifest.

/// Marks where `$GITHUB_WORKSPACE` stood in a word, before it is resolved
/// against the working directory.
const ROOT: char = '\u{1}';

/// One command a translated script runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    pub argv: Vec<String>,
    /// `KEY=value` prefixes and earlier `export`s, in order.
    pub env: Vec<(String, String)>,
    /// Repo-relative working directory; `""` is the checkout root.
    pub working_dir: String,
}

/// A translated script: its commands, plus what was dropped on the way (each
/// note is for the importer's report — nothing is dropped silently).
#[derive(Debug, Clone, Default)]
pub struct Translation {
    pub commands: Vec<Command>,
    pub notes: Vec<String>,
}

/// A command read tolerantly out of any script (see [`extract_loose`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LooseCommand {
    /// Words as written; an unexpanded `$VAR` stays as text.
    pub argv: Vec<String>,
    pub working_dir: String,
    /// Literal `KEY=value` words written before the command.
    pub assignments: Vec<(String, String)>,
    /// Followed by exactly `|| true` or `|| :` — the only fallbacks that
    /// make a failure harmless. Any other fallback (`|| rc=$?`, `|| ( … )`,
    /// `|| { … }`) leaves the command a gate.
    pub guarded: bool,
    /// The working directory could not be resolved (a `cd` to a variable or
    /// an unresolvable `git -C`): `working_dir` is then meaningless.
    pub wd_known: bool,
    /// The command changes the environment of what follows (`export`,
    /// `declare -x`, `set -a`, `source`/`.`): a description, else `None`.
    pub env_effect: Option<String>,
}

/// `$GITHUB_ENV` / `$GITHUB_PATH` / `$GITHUB_OUTPUT` named anywhere in a
/// script: a write there changes what later steps see.
pub fn github_file_writes(script: &str) -> Vec<&'static str> {
    ["GITHUB_ENV", "GITHUB_PATH", "GITHUB_OUTPUT", "GITHUB_STATE"]
        .into_iter()
        .filter(|f| script.contains(f))
        .collect()
}

#[derive(Debug, Clone, Default)]
struct Word {
    text: String,
    quoted: bool,
    vars: Vec<String>,
    subst: bool,
    glob: bool,
    tilde: bool,
    /// An unquoted `{` was seen (a `}` after it makes a brace expansion).
    brace_open: bool,
    /// `text.len()` when the first quote or escape was met — an assignment's
    /// `KEY=` must come before it, or bash reads the word as a command name.
    quote_at: Option<usize>,
}

impl Word {
    fn is(&self, s: &str) -> bool {
        !self.quoted && self.text == s
    }
}

#[derive(Debug, Clone)]
enum Tok {
    Word(Word),
    Op(&'static str),
    Newline,
}

/// Replace the `${{ github.workspace }}` expression (any spacing) with
/// `$GITHUB_WORKSPACE`, which the tokenizer resolves.
pub(crate) fn substitute_workspace_expr(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(i) = rest.find("${{") {
        out.push_str(&rest[..i]);
        let after = &rest[i + 3..];
        match after.find("}}") {
            Some(j) if after[..j].trim() == "github.workspace" => {
                out.push_str("$GITHUB_WORKSPACE");
                rest = &after[j + 2..];
            }
            _ => {
                out.push_str("${{");
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

/// Tokenize a script. Tolerant: unterminated quotes run to the end.
fn tokenize(script: &str) -> Vec<Tok> {
    let chars: Vec<char> = script.chars().collect();
    let mut toks: Vec<Tok> = Vec::new();
    let mut cur: Option<Word> = None;
    let mut heredocs: Vec<(String, bool)> = Vec::new();
    let mut expect_heredoc_delim: Option<bool> = None;
    let mut i = 0;
    let n = chars.len();

    fn flush(
        cur: &mut Option<Word>,
        toks: &mut Vec<Tok>,
        expect: &mut Option<bool>,
        heredocs: &mut Vec<(String, bool)>,
    ) {
        if let Some(w) = cur.take() {
            if let Some(strip) = expect.take() {
                heredocs.push((w.text.clone(), strip));
            }
            toks.push(Tok::Word(w));
        }
    }

    // Reads `$NAME`, `${…}`, `$(…)`, `$((…))` starting at `chars[i] == '$'`.
    // Returns the new index.
    fn dollar(chars: &[char], mut i: usize, w: &mut Word) -> usize {
        let n = chars.len();
        if i + 1 >= n {
            w.text.push('$');
            return i + 1;
        }
        let c = chars[i + 1];
        if c == '\'' || c == '"' {
            // `$'…'` (ANSI-C) and `$"…"` (locale) quoting: no argv form.
            w.subst = true;
            w.text.push('$');
            return i + 1;
        }
        if c == '(' {
            w.subst = true;
            let mut depth = 0i32;
            while i < n {
                match chars[i] {
                    '(' => depth += 1,
                    ')' => {
                        depth -= 1;
                        if depth == 0 {
                            w.text.push(')');
                            return i + 1;
                        }
                    }
                    _ => {}
                }
                w.text.push(chars[i]);
                i += 1;
            }
            return i;
        }
        if c == '{' {
            let start = i + 2;
            let mut j = start;
            while j < n && chars[j] != '}' {
                j += 1;
            }
            let inner: String = chars[start..j.min(n)].iter().collect();
            let name: String = inner
                .chars()
                .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
                .collect();
            if name == "GITHUB_WORKSPACE" && inner == name {
                w.text.push(ROOT);
            } else {
                w.vars
                    .push(if name.is_empty() { inner.clone() } else { name });
                w.text.push_str("${");
                w.text.push_str(&inner);
                w.text.push('}');
            }
            return (j + 1).min(n);
        }
        if c.is_ascii_alphabetic() || c == '_' {
            let mut j = i + 1;
            while j < n && (chars[j].is_ascii_alphanumeric() || chars[j] == '_') {
                j += 1;
            }
            let name: String = chars[i + 1..j].iter().collect();
            if name == "GITHUB_WORKSPACE" {
                w.text.push(ROOT);
            } else {
                w.text.push('$');
                w.text.push_str(&name);
                w.vars.push(name);
            }
            return j;
        }
        if c.is_ascii_digit() || matches!(c, '@' | '*' | '#' | '?' | '$' | '!' | '-') {
            w.vars.push(c.to_string());
            w.text.push('$');
            w.text.push(c);
            return i + 2;
        }
        w.text.push('$');
        i + 1
    }

    while i < n {
        let c = chars[i];
        match c {
            '\\' if i + 1 < n && chars[i + 1] == '\n' => {
                // Line continuation.
                i += 2;
            }
            '\\' => {
                let w = cur.get_or_insert_with(Word::default);
                w.quoted = true;
                w.quote_at.get_or_insert(w.text.len());
                if i + 1 < n {
                    w.text.push(chars[i + 1]);
                }
                i += 2;
            }
            ' ' | '\t' | '\r' => {
                flush(
                    &mut cur,
                    &mut toks,
                    &mut expect_heredoc_delim,
                    &mut heredocs,
                );
                i += 1;
            }
            '\n' => {
                flush(
                    &mut cur,
                    &mut toks,
                    &mut expect_heredoc_delim,
                    &mut heredocs,
                );
                toks.push(Tok::Newline);
                i += 1;
                // Skip any here-document bodies that start on this line.
                for (delim, strip) in std::mem::take(&mut heredocs) {
                    loop {
                        if i >= n {
                            break;
                        }
                        let end = chars[i..]
                            .iter()
                            .position(|&ch| ch == '\n')
                            .map(|p| i + p)
                            .unwrap_or(n);
                        let line: String = chars[i..end].iter().collect();
                        i = end + 1;
                        let cmp = if strip {
                            line.trim_start_matches('\t')
                        } else {
                            line.as_str()
                        };
                        if cmp == delim {
                            break;
                        }
                    }
                }
            }
            '#' if cur.is_none() => {
                while i < n && chars[i] != '\n' {
                    i += 1;
                }
            }
            '\'' => {
                let w = cur.get_or_insert_with(Word::default);
                w.quoted = true;
                w.quote_at.get_or_insert(w.text.len());
                i += 1;
                while i < n && chars[i] != '\'' {
                    w.text.push(chars[i]);
                    i += 1;
                }
                i += 1;
            }
            '"' => {
                let w = cur.get_or_insert_with(Word::default);
                w.quoted = true;
                w.quote_at.get_or_insert(w.text.len());
                i += 1;
                while i < n && chars[i] != '"' {
                    match chars[i] {
                        '\\' if i + 1 < n && matches!(chars[i + 1], '"' | '\\' | '$' | '`') => {
                            w.text.push(chars[i + 1]);
                            i += 2;
                        }
                        '\\' if i + 1 < n && chars[i + 1] == '\n' => i += 2,
                        '$' => i = dollar(&chars, i, w),
                        '`' => {
                            w.subst = true;
                            i += 1;
                            while i < n && chars[i] != '`' {
                                i += 1;
                            }
                            i += 1;
                        }
                        ch => {
                            w.text.push(ch);
                            i += 1;
                        }
                    }
                }
                i += 1;
            }
            '$' => {
                let w = cur.get_or_insert_with(Word::default);
                i = dollar(&chars, i, w);
            }
            '`' => {
                let w = cur.get_or_insert_with(Word::default);
                w.subst = true;
                i += 1;
                while i < n && chars[i] != '`' {
                    i += 1;
                }
                i += 1;
            }
            '|' | '&' | ';' | '<' | '>' | '(' | ')' => {
                flush(
                    &mut cur,
                    &mut toks,
                    &mut expect_heredoc_delim,
                    &mut heredocs,
                );
                let next = chars.get(i + 1).copied();
                let next2 = chars.get(i + 2).copied();
                let (op, len): (&'static str, usize) = match (c, next, next2) {
                    ('|', Some('|'), _) => ("||", 2),
                    ('|', _, _) => ("|", 1),
                    ('&', Some('&'), _) => ("&&", 2),
                    ('&', Some('>'), _) => ("&>", 2),
                    ('&', _, _) => ("&", 1),
                    (';', Some(';'), _) => (";;", 2),
                    (';', _, _) => (";", 1),
                    ('<', Some('<'), Some('<')) => ("<<<", 3),
                    ('<', Some('<'), Some('-')) => ("<<-", 3),
                    ('<', Some('<'), _) => ("<<", 2),
                    ('<', _, _) => ("<", 1),
                    ('>', Some('>'), _) => (">>", 2),
                    ('>', Some('&'), _) => (">&", 2),
                    ('>', _, _) => (">", 1),
                    ('(', _, _) => ("(", 1),
                    _ => (")", 1),
                };
                if op == "<<" {
                    expect_heredoc_delim = Some(false);
                } else if op == "<<-" {
                    expect_heredoc_delim = Some(true);
                }
                toks.push(Tok::Op(op));
                i += len;
            }
            '*' | '?' | '[' => {
                let w = cur.get_or_insert_with(Word::default);
                w.glob = true;
                w.text.push(c);
                i += 1;
            }
            '~' if cur.is_none() => {
                let w = cur.get_or_insert_with(Word::default);
                w.tilde = true;
                w.text.push(c);
                i += 1;
            }
            _ => {
                let w = cur.get_or_insert_with(Word::default);
                if c == '{' {
                    w.brace_open = true;
                } else if c == '}' && w.brace_open {
                    // `a{b,c}` / `{1..3}`: brace expansion, a glob by another name.
                    w.glob = true;
                }
                w.text.push(c);
                i += 1;
            }
        }
    }
    flush(
        &mut cur,
        &mut toks,
        &mut expect_heredoc_delim,
        &mut heredocs,
    );
    toks
}

/// Lexically join a repo-relative base with a relative path. `None` when the
/// result would leave the repository.
pub(crate) fn join_rel(base: &str, rel: &str) -> Option<String> {
    if rel.starts_with('/') || rel.starts_with('\\') || rel.contains(':') {
        return None;
    }
    let mut parts: Vec<&str> = base
        .split('/')
        .filter(|p| !p.is_empty() && *p != ".")
        .collect();
    for p in rel.split('/') {
        match p {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            other => parts.push(other),
        }
    }
    Some(parts.join("/"))
}

/// `../..` from `wd` back to the root (`.` when `wd` is the root).
fn up_to_root(wd: &str) -> String {
    let depth = wd.split('/').filter(|p| !p.is_empty()).count();
    if depth == 0 {
        ".".to_string()
    } else {
        vec![".."; depth].join("/")
    }
}

/// Resolve the root marker inside a word against `wd`.
fn resolve_root(text: &str, wd: &str) -> String {
    text.replace(ROOT, &up_to_root(wd))
}

/// A `cd` target as a new working directory. `None` when it leaves the repo
/// or cannot be known.
fn cd_target(w: &Word, wd: &str) -> Option<String> {
    // `cd -` is the PREVIOUS directory, which nothing here tracks.
    if !w.vars.is_empty() || w.subst || w.glob || w.tilde || w.text == "-" || w.text.is_empty() {
        return None;
    }
    if let Some(rest) = w.text.strip_prefix(ROOT) {
        return join_rel("", rest.trim_start_matches('/'));
    }
    if w.text.contains(ROOT) {
        return None;
    }
    join_rel(wd, &w.text)
}

fn split_statements(toks: &[Tok]) -> Vec<Vec<Tok>> {
    let mut out = Vec::new();
    let mut cur = Vec::new();
    for t in toks {
        match t {
            Tok::Newline | Tok::Op(";") => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            other => cur.push(other.clone()),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn first_word(stmt: &[Tok]) -> Option<&Word> {
    match stmt.first() {
        Some(Tok::Word(w)) => Some(w),
        _ => None,
    }
}

fn describe_op(op: &str) -> &'static str {
    match op {
        "|" => "a pipe (`|`)",
        "||" => "an `||` fallback",
        "&" => "a background job (`&`)",
        ">" | ">>" | ">&" | "&>" => "an output redirect",
        "<" | "<<" | "<<-" | "<<<" => "an input redirect or here-document",
        "(" | ")" => "a subshell",
        ";;" => "a `case` arm",
        _ => "a shell operator",
    }
}

/// Is `stmt` a terminal-only `echo`/`printf` line, safe to drop?
/// `Err` when it writes somewhere that changes later behaviour.
fn droppable_output_line(stmt: &[Tok]) -> Result<bool, String> {
    let Some(w) = first_word(stmt) else {
        return Ok(false);
    };
    if !(w.is("echo") || w.is("printf")) {
        return Ok(false);
    }
    if stmt.iter().any(|t| matches!(t, Tok::Word(x) if x.subst)) {
        return Err(
            "its `echo`/`printf` line runs a command substitution, which is more than output"
                .to_string(),
        );
    }
    let mut i = 1;
    while i < stmt.len() {
        if let Tok::Op(op) = &stmt[i] {
            match *op {
                ">&" => {
                    let fd = match stmt.get(i + 1) {
                        Some(Tok::Word(t)) => t.text.clone(),
                        _ => String::new(),
                    };
                    if !matches!(fd.as_str(), "1" | "2") {
                        return Err(format!(
                            "its `echo`/`printf` line redirects to {fd:?}, which may be a file"
                        ));
                    }
                }
                ">" | ">>" | "&>" => {
                    let target = match stmt.get(i + 1) {
                        Some(Tok::Word(t)) => t.text.clone(),
                        _ => String::new(),
                    };
                    if !matches!(target.as_str(), "/dev/stderr" | "/dev/stdout" | "/dev/null") {
                        return Err(format!(
                            "writes to {target} (an `echo`/`printf` into a file — e.g. \
                             $GITHUB_ENV / $GITHUB_OUTPUT / $GITHUB_PATH — changes what later steps \
                             see, and a manifest has no such channel)"
                        ));
                    }
                }
                other => {
                    return Err(format!(
                        "its `echo`/`printf` line uses {}",
                        describe_op(other)
                    ))
                }
            }
        }
        i += 1;
    }
    Ok(true)
}

/// The exit status of an `exit N` statement, if `stmt` is one.
fn exit_status(stmt: &[Tok]) -> Option<Option<i64>> {
    let w = first_word(stmt)?;
    if !w.is("exit") {
        return None;
    }
    Some(match stmt.get(1) {
        Some(Tok::Word(n)) => n.text.parse::<i64>().ok(),
        _ => Some(-1), // bare `exit`: the previous command's status
    })
}

const CONTROL_WORDS: &[&str] = &[
    "if", "then", "else", "elif", "fi", "for", "while", "until", "do", "done", "case", "esac",
    "function", "{", "}", "[[", "]]", "((", "select", "!",
];

/// Builtins that act on the shell process itself; as an argv program they
/// would fail or silently do nothing.
const BUILTINS: &[&str] = &[
    "source",
    ".",
    "pushd",
    "popd",
    "ulimit",
    "umask",
    "trap",
    "unset",
    "shopt",
    "eval",
    "exec",
    "command",
    ":",
    "alias",
    "readonly",
    "declare",
    "typeset",
    "local",
    "wait",
    "shift",
    "return",
    "break",
    "continue",
    "hash",
    "builtin",
    "enable",
    "let",
    "read",
    "mapfile",
    "readarray",
];

/// Translate a `run:` script that starts in working directory `start_wd`.
/// `Err` carries the reason the step cannot become argv commands.
pub fn translate(script: &str, start_wd: &str) -> Result<Translation, String> {
    let script = substitute_workspace_expr(script);
    if let Some(i) = script.find("${{") {
        let tail = &script[i..];
        let expr: String = match tail.find("}}") {
            Some(end) => tail[..end + 2].to_string(),
            None => tail.chars().take_while(|c| *c != '\n').take(80).collect(),
        };
        return Err(format!(
            "uses the expression {expr} — a manifest command is a literal, evaluated by no one"
        ));
    }
    let stmts = split_statements(&tokenize(&script));
    if stmts.is_empty() {
        return Err("the script is empty".to_string());
    }
    let mut out = Translation::default();
    let mut wd = start_wd.to_string();
    let mut exported: Vec<(String, String)> = Vec::new();
    let mut dropped_output = 0usize;
    let mut dropped_messages = 0usize;
    let mut i = 0;
    while i < stmts.len() {
        let stmt = &stmts[i];
        let last = i + 1 == stmts.len();
        let Some(first) = first_word(stmt) else {
            return Err(format!(
                "a line starts with {}",
                match &stmt[0] {
                    Tok::Op(op) => describe_op(op),
                    _ => "something that is not a command",
                }
            ));
        };
        // `set -euo pipefail` and friends.
        if first.is("set")
            && (stmt.iter().any(|t| matches!(t, Tok::Word(w) if w.text.starts_with('+') && w.text.contains('e')))
                || stmt.windows(2).any(|p| matches!(p, [Tok::Word(a), Tok::Word(b)] if a.text == "+o" && b.text == "errexit")))
        {
            return Err(
                "turns fail-fast off (`set +e`) — what follows may deliberately ignore failures, \
                 which a manifest step cannot"
                    .to_string(),
            );
        }
        if first.is("set") {
            if !out.notes.iter().any(|n| n.starts_with("shell options")) {
                out.notes.push(
                    "shell options (`set …`) dropped: every manifest command is already its own \
                     fail-fast step"
                        .to_string(),
                );
            }
            i += 1;
            continue;
        }
        // `if ! cmd; then <messages>; exit N; fi`
        if first.is("if") {
            let negated = matches!(stmt.get(1), Some(Tok::Word(w)) if w.is("!"));
            if !negated {
                return Err(
                    "uses an `if` conditional (only the `if ! cmd; then …; exit N; fi` failure \
                     guard has an argv form)"
                        .to_string(),
                );
            }
            let cond: Vec<Tok> = stmt[2..].to_vec();
            if cond.iter().any(|t| matches!(t, Tok::Op(_))) {
                return Err(
                    "its `if ! …` condition is more than one command (`if ! a && b` means \
                     `(! a) && b` to bash)"
                        .to_string(),
                );
            }
            let mut j = i + 1;
            let mut saw_then = false;
            let mut exits_nonzero = false;
            loop {
                let Some(body) = stmts.get(j) else {
                    return Err("an `if` with no `fi`".to_string());
                };
                let mut body: &[Tok] = body;
                if !saw_then {
                    match first_word(body) {
                        Some(w) if w.is("then") => {
                            saw_then = true;
                            body = &body[1..];
                            if body.is_empty() {
                                j += 1;
                                continue;
                            }
                        }
                        _ => return Err("an `if` with no `then`".to_string()),
                    }
                }
                match first_word(body) {
                    Some(w) if w.is("fi") && body.len() == 1 => break,
                    Some(w) if w.is("else") || w.is("elif") => {
                        return Err("uses an `if … else` conditional".to_string())
                    }
                    _ => {}
                }
                if let Some(status) = exit_status(body) {
                    match status {
                        Some(n) if n != 0 && n != -1 => exits_nonzero = true,
                        _ => {
                            return Err(
                                "its `if ! …` guard does not exit with a literal non-zero status, \
                                 so it may swallow the failure"
                                    .to_string(),
                            )
                        }
                    }
                } else if !droppable_output_line(body)?
                    && !(first_word(body).is_some_and(|w| w.is(":") || w.is("true")))
                {
                    return Err(
                        "its `if ! …` guard runs commands other than messages and `exit`"
                            .to_string(),
                    );
                } else {
                    dropped_messages += 1;
                }
                j += 1;
            }
            if !exits_nonzero {
                return Err(
                    "its `if ! …` guard never exits non-zero, which swallows the failure"
                        .to_string(),
                );
            }
            let cmds = simple_commands(&cond, &mut wd, &exported)?;
            out.commands.extend(cmds);
            i = j + 1;
            continue;
        }
        // `cmd || exit N` and `cmd || { echo …; exit N; }`
        if let Some(pos) = stmt.iter().position(|t| matches!(t, Tok::Op("||"))) {
            let left = &stmt[..pos];
            let right = &stmt[pos + 1..];
            if left.iter().any(|t| matches!(t, Tok::Op(_))) {
                return Err("combines `||` with another shell operator".to_string());
            }
            let mut j = i;
            match first_word(right) {
                Some(w) if w.is("exit") && right.len() != 2 => return Err(
                    "has words or operators after its `|| exit N` — only a bare `cmd || exit N` \
                         has an argv form"
                        .to_string(),
                ),
                Some(w) if w.is("exit") => match exit_status(right) {
                    Some(Some(n)) if n != 0 && n != -1 => {}
                    _ => return Err(
                        "an `|| exit` without a literal non-zero status may swallow the failure"
                            .to_string(),
                    ),
                },
                Some(w) if w.is("true") || w.is(":") => {
                    return Err(
                        "deliberately ignores a failure (`|| true`) — a manifest step has no \
                         non-gating form"
                            .to_string(),
                    )
                }
                Some(w) if w.is("{") => {
                    // The group's statements: what follows `{` on this line,
                    // then whole statements until one that ends in `}`.
                    let mut exits_nonzero = false;
                    let mut body: Vec<Tok> = right[1..].to_vec();
                    loop {
                        let closes = matches!(body.last(), Some(Tok::Word(w)) if w.is("}"));
                        if closes {
                            body.pop();
                        }
                        if !body.is_empty() {
                            check_guard_line(&body, &mut exits_nonzero, &mut dropped_messages)?;
                        }
                        if closes {
                            break;
                        }
                        j += 1;
                        match stmts.get(j) {
                            Some(next) => body = next.clone(),
                            None => return Err("an `|| {` group with no `}`".to_string()),
                        }
                    }
                    if !exits_nonzero {
                        return Err(
                            "its `|| { … }` fallback never exits non-zero, which swallows the \
                             failure"
                                .to_string(),
                        );
                    }
                }
                _ => return Err("uses an `||` fallback".to_string()),
            }
            let cmds = simple_commands(left, &mut wd, &exported)?;
            out.commands.extend(cmds);
            i = j + 1;
            continue;
        }
        if CONTROL_WORDS.iter().any(|k| first.is(k)) {
            return Err(format!("uses the shell construct `{}`", first.text));
        }
        if let Some(status) = exit_status(stmt) {
            match status {
                Some(0) if last => {
                    i += 1;
                    continue;
                }
                _ => return Err("an `exit` changes the script's control flow".to_string()),
            }
        }
        if droppable_output_line(stmt)? {
            dropped_output += 1;
            i += 1;
            continue;
        }
        if first.is("export") {
            for t in &stmt[1..] {
                match t {
                    Tok::Word(w) => {
                        let (k, v) = assignment(w).ok_or_else(|| {
                            format!("`export {}` is not a literal KEY=value", w.text)
                        })?;
                        exported.retain(|(ek, _)| ek != &k);
                        exported.push((k, v));
                    }
                    Tok::Op(op) => return Err(format!("its `export` uses {}", describe_op(op))),
                    Tok::Newline => {}
                }
            }
            i += 1;
            continue;
        }
        if !last && stmt.iter().any(|t| matches!(t, Tok::Op("&&"))) {
            out.notes.push(
                "an `a && b` line that is not the script's last: bash -e carries on when `a` \
                 fails there, while each manifest command fails the job — the import is stricter"
                    .to_string(),
            );
        }
        let cmds = simple_commands(stmt, &mut wd, &exported)?;
        out.commands.extend(cmds);
        i += 1;
    }
    if dropped_output > 0 {
        out.notes.push(format!(
            "{dropped_output} `echo`/`printf` line(s) dropped (terminal output only)"
        ));
    }
    if dropped_messages > 0 {
        out.notes.push(format!(
            "{dropped_messages} failure-message line(s) of an `if ! …` / `|| …` guard dropped; \
             the guarded command still fails the step"
        ));
    }
    if out.commands.is_empty() {
        return Err("the script runs no command (only output or shell settings)".to_string());
    }
    Ok(out)
}

fn check_guard_line(
    body: &[Tok],
    exits_nonzero: &mut bool,
    dropped: &mut usize,
) -> Result<(), String> {
    if let Some(status) = exit_status(body) {
        match status {
            Some(n) if n != 0 && n != -1 => {
                *exits_nonzero = true;
                Ok(())
            }
            _ => Err(
                "its `|| { … }` fallback does not exit with a literal non-zero status, so it may \
                 swallow the failure"
                    .to_string(),
            ),
        }
    } else if droppable_output_line(body)? {
        *dropped += 1;
        Ok(())
    } else {
        Err("its `|| { … }` fallback runs commands other than messages and `exit`".to_string())
    }
}

/// `NAME=anything`, literal or not — for the tolerant reader.
fn looks_like_assignment(w: &Word) -> bool {
    w.text.split_once('=').is_some_and(|(k, _)| {
        !k.is_empty()
            && k.chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
            && k.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    })
}

/// `KEY=value` as a literal assignment.
fn assignment(w: &Word) -> Option<(String, String)> {
    let (k, v) = w.text.split_once('=')?;
    if w.quote_at.is_some_and(|q| q <= k.len()) {
        return None;
    }
    if w.tilde || v.starts_with('~') || v.contains(":~") || w.glob {
        return None;
    }
    let ok_key = !k.is_empty()
        && k.chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && k.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    if !ok_key || !w.vars.is_empty() || w.subst || v.contains(ROOT) {
        return None;
    }
    Some((k.to_string(), v.to_string()))
}

/// One statement of `&&`-joined plain commands (and `cd`s).
fn simple_commands(
    stmt: &[Tok],
    wd: &mut String,
    exported: &[(String, String)],
) -> Result<Vec<Command>, String> {
    let mut out = Vec::new();
    for part in stmt.split(|t| matches!(t, Tok::Op("&&"))) {
        if part.is_empty() {
            return Err("an empty `&&` operand".to_string());
        }
        if let Some(Tok::Op(op)) = part.iter().find(|t| matches!(t, Tok::Op(_))) {
            return Err(format!("uses {}", describe_op(op)));
        }
        let words: Vec<&Word> = part
            .iter()
            .filter_map(|t| match t {
                Tok::Word(w) => Some(w),
                _ => None,
            })
            .collect();
        if words[0].is("cd") {
            if words.len() != 2 {
                return Err("a `cd` without exactly one directory".to_string());
            }
            *wd = cd_target(words[1], wd).ok_or_else(|| {
                format!(
                    "`cd {}` leaves the repository or depends on a variable",
                    words[1].text.replace(ROOT, "$GITHUB_WORKSPACE")
                )
            })?;
            continue;
        }
        if CONTROL_WORDS.iter().any(|k| words[0].is(k)) {
            return Err(format!("uses the shell construct `{}`", words[0].text));
        }
        if let Some(b) = BUILTINS.iter().find(|b| words[0].is(b)) {
            return Err(format!(
                "uses the shell builtin `{b}`, which changes the shell itself — as a program it \
                 would do nothing or fail"
            ));
        }
        let mut env: Vec<(String, String)> = exported.to_vec();
        let mut k = 0;
        while k < words.len() {
            match assignment(words[k]) {
                Some((key, val)) => {
                    env.retain(|(ek, _)| ek != &key);
                    env.push((key, val));
                    k += 1;
                }
                None => break,
            }
        }
        if k == words.len() {
            return Err("sets a shell variable (`KEY=value` with no command)".to_string());
        }
        let rest = &words[k..];
        let eq_unquoted = rest[0]
            .text
            .find('=')
            .is_some_and(|e| rest[0].quote_at.is_none_or(|q| q > e));
        if looks_like_assignment(rest[0]) && eq_unquoted {
            return Err(format!(
                "sets {:?}, an env assignment that is not a literal (bash would expand it)",
                rest[0].text.replace(ROOT, "$GITHUB_WORKSPACE")
            ));
        }
        if rest[0].text.is_empty() {
            return Err("runs a command with an empty name".to_string());
        }
        if let Some(b) = BUILTINS.iter().find(|b| rest[0].is(b)) {
            return Err(format!("uses the shell builtin `{b}` after an env prefix"));
        }
        // `git -C <dir>` moves the command first, so `$GITHUB_WORKSPACE` in its
        // arguments resolves against the directory it actually runs in.
        let mut cmd_wd = wd.clone();
        let mut skip = 0;
        if rest.len() >= 3 && rest[0].is("git") && rest[1].is("-C") {
            cmd_wd = cd_target(rest[2], wd).ok_or_else(|| {
                format!(
                    "`git -C {}` leaves the repository or depends on a variable",
                    rest[2].text.replace(ROOT, "$GITHUB_WORKSPACE")
                )
            })?;
            skip = 2;
        }
        let mut argv = Vec::new();
        for (idx, w) in rest.iter().enumerate() {
            if idx >= 1 && idx <= skip {
                continue;
            }
            if let Some(v) = w.vars.first() {
                return Err(format!(
                    "expands ${v} at run time — a manifest argument is a literal"
                ));
            }
            if w.subst {
                return Err("uses a command substitution (`$(…)` or backticks)".to_string());
            }
            if w.glob {
                return Err(format!(
                    "relies on shell glob or brace expansion of {:?}",
                    w.text
                ));
            }
            if w.tilde {
                return Err(format!("relies on `~` expansion in {:?}", w.text));
            }
            let text = resolve_root(&w.text, &cmd_wd);
            if !crate::manifest::argv_token_ok(&text) {
                return Err(format!(
                    "argument {text:?} contains a character the manifest bans in a command (one \
                     of & | < > ^ \" % or a newline)"
                ));
            }
            argv.push(text);
        }
        out.push(Command {
            argv,
            env,
            working_dir: cmd_wd,
        });
    }
    Ok(out)
}

/// The bodies of the `$( … )` substitutions in a word's text.
fn substitutions(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i + 1 < chars.len() {
        if chars[i] == '$' && chars[i + 1] == '(' {
            let start = i + 2;
            let mut depth = 1;
            let mut j = start;
            while j < chars.len() && depth > 0 {
                match chars[j] {
                    '(' => depth += 1,
                    ')' => depth -= 1,
                    _ => {}
                }
                j += 1;
            }
            let end = if depth == 0 { j - 1 } else { j };
            let body: String = chars[start..end].iter().collect();
            // `$((…))` is arithmetic, not a command.
            if !body.starts_with('(') {
                out.push(body);
            }
            i = j;
        } else {
            i += 1;
        }
    }
    out
}

/// The report's ALLOWLIST grammar: `Ok` only when every segment of the script
/// is one of
///
/// * a simple command `prog args…` whose words are literal (no `$`, no
///   expansion, no glob, no `~`) and whose program is not an assignment, a
///   wrapper (`env`, `timeout`, `nice`, `xargs`, `sudo`, `tee`, `exec`,
///   `command`, …) or a shell-state builtin (`export`, `declare`, `source`,
///   `.`, `eval`, `pushd`, `popd`, …);
/// * `cd <literal relative path>` inside the repository;
/// * `echo` / `printf` / `:` / `true`;
/// * exactly `set -e`, `set -eu`, `set -euo pipefail` or `set -x`;
///
/// joined by newlines, `;` or `&&`, each optionally ending in `|| true` or
/// `|| :`. Any operator other than those (a pipe, any redirect, a
/// here-document, a subshell, a background job) and any control flow fails.
/// `Err` names the first thing outside the grammar. Deliberately not a
/// blocklist: what is not listed here is not understood.
pub fn report_grammar(script: &str) -> Result<(), String> {
    if script.contains("${{") {
        return Err("uses a `${{ … }}` expression".to_string());
    }
    // Comments: a whole-line comment is fine; a `#` anywhere else is not
    // modelled (it may or may not start a comment, depending on position).
    // (`str::lines` strips a trailing `\r`, so look for it in the script.)
    if script.contains('\r') {
        return Err("contains a carriage return".to_string());
    }
    for line in script.lines() {
        let t = line.trim_start();
        if !t.starts_with('#') && t.contains('#') {
            return Err(
                "has a `#` after a command (a trailing comment or a literal `#`)".to_string(),
            );
        }
        if line
            .chars()
            .any(|c| c != ' ' && c != '\t' && c.is_whitespace())
        {
            return Err("contains non-ASCII whitespace".to_string());
        }
    }
    let toks = tokenize(script);
    // Each statement, and whether it follows an `&&`.
    let mut stmts: Vec<(Vec<Tok>, bool)> = vec![(Vec::new(), false)];
    for t in toks {
        match t {
            Tok::Newline | Tok::Op(";") => stmts.push((Vec::new(), false)),
            // `true`: the statement follows an `&&` (it is not the chain's first).
            Tok::Op("&&") => stmts.push((Vec::new(), true)),
            other => {
                if let Some(s) = stmts.last_mut() {
                    s.0.push(other);
                }
            }
        }
    }
    const WRAPPERS_AND_STATE: &[&str] = &[
        "env",
        "timeout",
        "nice",
        "nohup",
        "xargs",
        "sudo",
        "doas",
        "tee",
        "exec",
        "command",
        "builtin",
        "time",
        "stdbuf",
        "export",
        "declare",
        "typeset",
        "local",
        "readonly",
        "source",
        ".",
        "eval",
        "pushd",
        "popd",
        "unset",
        "alias",
        "trap",
        "shopt",
        "ulimit",
        "umask",
        "read",
        "mapfile",
        "readarray",
        "exit",
        "return",
        "shift",
        "let",
        "wait",
        "hash",
        "enable",
        "getopts",
        "coproc",
        "disown",
        "suspend",
        "fc",
        "history",
    ];
    for (stmt, chained) in stmts.into_iter().filter(|s| !s.0.is_empty()) {
        // An optional trailing `|| true` / `|| :`.
        let has_fallback = stmt.iter().any(|t| matches!(t, Tok::Op("||")));
        if has_fallback && matches!(stmt.first(), Some(Tok::Word(w)) if w.is("cd")) {
            return Err(
                "a `cd` with a fallback (if it fails, later commands run elsewhere)".to_string(),
            );
        }
        let body: &[Tok] = match stmt.iter().position(|t| matches!(t, Tok::Op("||"))) {
            Some(p) => {
                let fallback = &stmt[p + 1..];
                let ok = fallback.len() == 1
                    && matches!(&fallback[0], Tok::Word(w) if w.is("true") || w.is(":"));
                if !ok {
                    return Err("an `||` fallback other than `|| true` / `|| :`".to_string());
                }
                &stmt[..p]
            }
            None => &stmt,
        };
        let mut words: Vec<&Word> = Vec::new();
        for t in body {
            match t {
                Tok::Word(w) => words.push(w),
                Tok::Op(op) => return Err(format!("uses {}", describe_op(op))),
                Tok::Newline => {}
            }
        }
        let Some(first) = words.first() else {
            return Err("an empty command".to_string());
        };
        for w in &words {
            // bash removes quotes and backslashes BEFORE it looks a word up,
            // so `"export"`, `\export` and `ex''port` are all `export`. A
            // quoted or escaped word is therefore not understood.
            if w.quote_at.is_some() || w.quoted {
                return Err(format!("has a quoted or escaped word ({:?})", w.text));
            }
            if w.text.contains('~') {
                return Err(format!("has a `~` in {:?}", w.text));
            }
            if !w.vars.is_empty() || w.subst || w.text.contains('$') || w.text.contains(ROOT) {
                return Err(format!(
                    "expands a variable or substitution in {:?}",
                    w.text.replace(ROOT, "$GITHUB_WORKSPACE")
                ));
            }
            if w.glob || w.tilde {
                return Err(format!("relies on shell expansion of {:?}", w.text));
            }
        }
        if looks_like_assignment(first)
            && first
                .quote_at
                .is_none_or(|q| first.text.find('=').is_some_and(|e| q > e))
        {
            return Err(format!("assigns a variable ({:?})", first.text));
        }
        if CONTROL_WORDS.iter().any(|k| first.is(k)) {
            return Err(format!("uses the shell construct `{}`", first.text));
        }
        if let Some(wr) = WRAPPERS_AND_STATE.iter().find(|k| first.is(k)) {
            return Err(format!("uses `{wr}`, which the report does not model"));
        }
        if first.is("echo") {
            if let Some(f) = words[1..]
                .iter()
                .take_while(|w| w.text.starts_with('-'))
                .find(|w| !matches!(w.text.as_str(), "-n" | "-e" | "-ne" | "-en"))
            {
                return Err(format!("`echo {}` uses a flag other than -n/-e", f.text));
            }
            continue;
        }
        if first.is("printf") {
            if words.iter().any(|w| w.text == "-v") {
                return Err("`printf -v` assigns a variable".to_string());
            }
            continue;
        }
        if first.is("cd") {
            // `cd dir && cmd` is understood; a `cd` after an `&&` (`make && cd
            // sub`) is not.
            if chained {
                return Err("a `cd` after an `&&`".to_string());
            }
            if words.len() != 2
                || words[1].text.starts_with('-')
                || words[1].text.split('/').any(|c| c == ".." || c == ".")
                || cd_target(words[1], "").is_none()
            {
                return Err(
                    "a `cd` that is not to one literal path inside the repository".to_string(),
                );
            }
            continue;
        }
        if first.is("set") {
            let args: Vec<&str> = words[1..].iter().map(|w| w.text.as_str()).collect();
            let ok = matches!(
                args.as_slice(),
                ["-e"] | ["-eu"] | ["-euo", "pipefail"] | ["-x"]
            );
            if !ok {
                return Err(format!(
                    "`set {}` is not one of set -e / -eu / -euo pipefail / -x",
                    args.join(" ")
                ));
            }
            continue;
        }
    }
    Ok(())
}

/// Remove the `pattern)` labels of `case … esac` arms, which would otherwise
/// read as commands named after the patterns.
fn drop_case_patterns(toks: Vec<Tok>) -> Vec<Tok> {
    let mut out = Vec::with_capacity(toks.len());
    let mut depth = 0usize;
    let mut at_start = true;
    let mut i = 0;
    while i < toks.len() {
        match &toks[i] {
            Tok::Word(w) => {
                if at_start && w.is("case") {
                    depth += 1;
                } else if at_start && w.is("esac") {
                    depth = depth.saturating_sub(1);
                } else if at_start && depth > 0 {
                    // A pattern runs to the `)` that ends it on this line.
                    let close = toks[i..]
                        .iter()
                        .take_while(|t| !matches!(t, Tok::Newline | Tok::Op(";;")))
                        .position(|t| matches!(t, Tok::Op(")")));
                    if let Some(rel) = close {
                        i += rel + 1;
                        continue;
                    }
                }
                at_start = false;
                out.push(toks[i].clone());
            }
            Tok::Newline | Tok::Op(";" | ";;" | "&&" | "||" | "|" | "(") => {
                at_start = true;
                out.push(toks[i].clone());
            }
            Tok::Op(_) => {
                at_start = false;
                out.push(toks[i].clone());
            }
        }
        i += 1;
    }
    out
}

/// Read every command name out of ANY script, tolerantly — for coverage
/// checks only. Control keywords, assignments and redirects are skipped; `cd`
/// is followed when its target is literal; unexpanded variables stay as text.
pub fn extract_loose(script: &str, start_wd: &str) -> Vec<LooseCommand> {
    extract_loose_in(script, Some(start_wd.to_string()))
}

/// [`extract_loose`] from a working directory that may already be unknown.
fn extract_loose_in(script: &str, start_wd: Option<String>) -> Vec<LooseCommand> {
    let script = substitute_workspace_expr(script).replace("${{", "$EXPR{{");
    let toks = drop_case_patterns(tokenize(&script));
    let mut out = Vec::new();
    // `None`: the directory is no longer known.
    let mut wd: Option<String> = start_wd;
    let mut seg: Vec<Tok> = Vec::new();
    // Each segment with the operator that ended it.
    let mut segments: Vec<(Vec<Tok>, &'static str)> = Vec::new();
    for t in toks {
        match &t {
            Tok::Newline => segments.push((std::mem::take(&mut seg), "\n")),
            Tok::Op(op) if matches!(*op, ";" | "&&" | "||" | "|" | "&" | "(" | ")" | ";;") => {
                segments.push((std::mem::take(&mut seg), op))
            }
            _ => seg.push(t),
        }
    }
    segments.push((seg, "\n"));
    let harmless_fallback = |seg: &[Tok]| -> bool {
        let words: Vec<&Word> = seg
            .iter()
            .filter_map(|t| match t {
                Tok::Word(w) => Some(w),
                _ => None,
            })
            .collect();
        seg.len() == 1 && words.len() == 1 && (words[0].is("true") || words[0].is(":"))
    };
    let unknown = "<unknown directory>".to_string();
    for (si, (seg, term)) in segments.iter().enumerate() {
        let guarded = *term == "||"
            && segments
                .get(si + 1)
                .is_some_and(|(n, _)| harmless_fallback(n));
        let seg = seg.clone();
        // Commands inside `$( … )` run too: `out="$(zizmor …)"` is a gate.
        for t in &seg {
            if let Tok::Word(w) = t {
                if w.subst {
                    for inner in substitutions(&w.text) {
                        out.extend(extract_loose_in(&inner, wd.clone()));
                    }
                }
            }
        }
        // Drop redirects and their targets (and an fd number before them).
        let mut words: Vec<Word> = Vec::new();
        let mut skip_next = false;
        for t in &seg {
            match t {
                Tok::Op(_) => {
                    if let Some(last) = words.last() {
                        if !last.quoted
                            && last.text.chars().all(|c| c.is_ascii_digit())
                            && !last.text.is_empty()
                        {
                            words.pop();
                        }
                    }
                    skip_next = true;
                }
                Tok::Word(w) => {
                    if skip_next {
                        skip_next = false;
                        continue;
                    }
                    words.push(w.clone());
                }
                Tok::Newline => {}
            }
        }
        let mut k = 0;
        let mut assignments = Vec::new();
        while k < words.len() {
            let w = &words[k];
            if [
                "if", "then", "else", "elif", "fi", "do", "done", "while", "until", "!", "{", "}",
                "time", "esac",
            ]
            .iter()
            .any(|kw| w.is(kw))
            {
                k += 1;
                continue;
            }
            if looks_like_assignment(w) {
                if let Some((key, val)) = w.text.split_once('=') {
                    assignments.push((key.to_string(), val.to_string()));
                }
                k += 1;
                continue;
            }
            break;
        }
        let words = &words[k..];
        let Some(first) = words.first() else {
            continue;
        };
        let text = |w: &Word| w.text.replace(ROOT, "$GITHUB_WORKSPACE");
        let env_effect: Option<String> = if first.is("export") {
            Some(format!(
                "`export {}`",
                words[1..].iter().map(text).collect::<Vec<_>>().join(" ")
            ))
        } else if (first.is("declare") || first.is("typeset"))
            && words[1..]
                .iter()
                .any(|w| w.text.starts_with('-') && w.text.contains('x'))
        {
            Some(format!("`{} -x …`", first.text))
        } else if first.is("set")
            && words[1..]
                .iter()
                .any(|w| (w.text.starts_with('-') && w.text.contains('a')) || w.text == "allexport")
        {
            Some("`set -a` (exports every later assignment)".to_string())
        } else if first.is("source") || first.is(".") {
            Some(format!(
                "`{} {}` (reads an environment file)",
                first.text,
                words.get(1).map(text).unwrap_or_default()
            ))
        } else {
            None
        };
        if env_effect.is_some() && !(first.is("source") || first.is(".")) {
            out.push(LooseCommand {
                argv: words.iter().map(text).collect(),
                working_dir: wd.clone().unwrap_or_else(|| unknown.clone()),
                assignments,
                guarded,
                wd_known: wd.is_some(),
                env_effect,
            });
            continue;
        }
        if [
            "for", "case", "function", "local", "readonly", "declare", "typeset",
        ]
        .iter()
        .any(|kw| first.is(kw))
            || first.text.ends_with(')')
        {
            continue;
        }
        if first.is("cd") {
            // A target that is not a literal — a variable, `cd -` (the
            // previous directory, which is not tracked), no argument, or a
            // second argument — makes every later directory unknown.
            wd = match (&wd, words.get(1)) {
                _ if words.len() != 2 => None,
                (_, Some(w)) if w.text.starts_with(ROOT) => cd_target(w, ""),
                (Some(cur), Some(w)) => cd_target(w, cur),
                _ => None,
            };
            continue;
        }
        // Same order as the translator: `git -C` moves the command first, then
        // `$GITHUB_WORKSPACE` resolves against where it runs.
        let git_c = words.len() >= 3 && words[0].is("git") && words[1].is("-C");
        let (cmd_wd, from): (Option<String>, usize) = if git_c {
            let target = match &wd {
                _ if words[2].text.starts_with(ROOT) => cd_target(&words[2], ""),
                Some(cur) => cd_target(&words[2], cur),
                None => None,
            };
            (target, 3)
        } else {
            (wd.clone(), 0)
        };
        let resolve_in = cmd_wd.clone().unwrap_or_default();
        let mut argv: Vec<String> = Vec::with_capacity(words.len());
        if from == 3 {
            argv.push("git".to_string());
        }
        argv.extend(
            words[from..]
                .iter()
                .map(|w| resolve_root(&w.text, &resolve_in)),
        );
        out.push(LooseCommand {
            argv,
            working_dir: cmd_wd.clone().unwrap_or_else(|| unknown.clone()),
            assignments,
            guarded,
            wd_known: cmd_wd.is_some(),
            env_effect,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argvs(t: &Translation) -> Vec<Vec<String>> {
        t.commands.iter().map(|c| c.argv.clone()).collect()
    }

    #[test]
    fn plain_lines_become_commands() {
        let t = translate("npm ci\nnpm run lint\n", "frontend").unwrap();
        assert_eq!(
            argvs(&t),
            vec![vec!["npm", "ci"], vec!["npm", "run", "lint"]]
        );
        assert!(t.commands.iter().all(|c| c.working_dir == "frontend"));
    }

    #[test]
    fn quotes_continuations_and_env_prefixes() {
        let t = translate("RUST_LOG=debug cargo test \\\n  --workspace -- 'a b'\n", "").unwrap();
        assert_eq!(
            t.commands[0].argv,
            vec!["cargo", "test", "--workspace", "--", "a b"]
        );
        assert_eq!(
            t.commands[0].env,
            vec![("RUST_LOG".to_string(), "debug".to_string())]
        );
    }

    #[test]
    fn cd_and_and_chain_set_working_dir() {
        let t = translate("set -euo pipefail\ncd backend && poetry install\n", "").unwrap();
        assert_eq!(t.commands[0].working_dir, "backend");
        assert_eq!(t.commands[0].argv, vec!["poetry", "install"]);
        assert!(t.notes.iter().any(|n| n.contains("shell options")));
        assert!(translate("cd ..\nls\n", "").is_err());
    }

    #[test]
    fn failure_guard_is_its_command() {
        let script = "poetry run python scripts/export_openapi.py\n\
                      if ! git -C \"$GITHUB_WORKSPACE\" diff --exit-code \\\n    a.json \\\n    b.json; then\n  \
                      echo \"::error::stale $X\"\n  exit 1\nfi\n";
        let t = translate(script, "backend").unwrap();
        assert_eq!(t.commands.len(), 2);
        assert_eq!(
            t.commands[1].argv,
            vec!["git", "diff", "--exit-code", "a.json", "b.json"]
        );
        assert_eq!(t.commands[1].working_dir, "");
        assert!(t.notes.iter().any(|n| n.contains("failure-message")));
    }

    #[test]
    fn or_exit_and_brace_fallback() {
        let t = translate("cargo fmt --check || exit 1\n", "").unwrap();
        assert_eq!(t.commands[0].argv, vec!["cargo", "fmt", "--check"]);
        let t = translate("make lint || { echo bad >&2; exit 2; }\n", "").unwrap();
        assert_eq!(t.commands[0].argv, vec!["make", "lint"]);
        assert!(translate("make lint || true\n", "")
            .unwrap_err()
            .contains("|| true"));
    }

    #[test]
    fn refuses_what_argv_cannot_say() {
        assert!(translate("curl x | sh\n", "").unwrap_err().contains("pipe"));
        assert!(translate("echo X=1 >> $GITHUB_ENV\n", "")
            .unwrap_err()
            .contains("GITHUB_ENV"));
        assert!(translate("pytest $ARGS\n", "")
            .unwrap_err()
            .contains("$ARGS"));
        assert!(translate("twine check dist/*\n", "")
            .unwrap_err()
            .contains("glob"));
        assert!(translate("for f in a b; do x $f; done\n", "").is_err());
        assert!(translate("echo ${{ secrets.TOKEN }}\n", "")
            .unwrap_err()
            .contains("secrets.TOKEN"));
        assert!(translate("if [ -f x ]; then y; fi\n", "").is_err());
        assert!(translate("echo only\n", "").is_err());
        assert!(translate("date +%s\n", "").unwrap_err().contains("bans"));
    }

    #[test]
    fn review_findings_are_refused_or_noted() {
        assert!(translate("make lint || exit 1 && make test\n", "").is_err());
        assert!(translate("if ! a && b; then\n exit 1\nfi\n", "").is_err());
        assert!(translate("echo \"$(./gen.sh)\"\nmake\n", "").is_err());
        assert!(translate("echo X=1 >& out.env\nmake\n", "").is_err());
        assert!(translate("twine check \"$GITHUB_WORKSPACE\"/dist/*\n", "")
            .unwrap_err()
            .contains("glob"));
        assert!(translate("mkdir -p out/{debug,release}\n", "")
            .unwrap_err()
            .contains("brace"));
        assert!(translate("source .venv/bin/activate\npytest\n", "")
            .unwrap_err()
            .contains("builtin"));
        assert!(translate("ulimit -n 4096\n", "").is_err());
        assert!(translate("printf $'a\\tb'\nmake\n", "").is_err());
        assert!(translate("tool $'a\\tb'\n", "").is_err());
        assert!(translate("CARGO_HOME=~/.cargo cargo build\n", "").is_err());
        assert!(translate("set +e\nmake\n", "").is_err());
        assert!(translate("set +o errexit\nmake\n", "").is_err());
        let loose = extract_loose(
            "git -C frontend diff --exit-code \"$GITHUB_WORKSPACE/openapi.json\"\n",
            "",
        );
        assert_eq!(
            loose[0].argv,
            vec!["git", "diff", "--exit-code", "../openapi.json"]
        );
        assert_eq!(loose[0].working_dir, "frontend");
        let t = translate("\"FOO=1\" cmd\n", "").unwrap();
        assert_eq!(t.commands[0].argv, vec!["FOO=1", "cmd"]);
        let t = translate("test -f a && cp a b\nmake\n", "").unwrap();
        assert!(t.notes.iter().any(|n| n.contains("stricter")));
        // git -C resolves the workspace root against its own directory.
        let t = translate(
            "git -C frontend diff --exit-code \"$GITHUB_WORKSPACE/openapi.json\"\n",
            "",
        )
        .unwrap();
        assert_eq!(t.commands[0].working_dir, "frontend");
        assert_eq!(
            t.commands[0].argv,
            vec!["git", "diff", "--exit-code", "../openapi.json"]
        );
    }

    #[test]
    fn workspace_expression_resolves_to_root() {
        let t = translate("python ${{ github.workspace }}/scripts/x.py\n", "backend").unwrap();
        assert_eq!(t.commands[0].argv, vec!["python", "../scripts/x.py"]);
    }

    #[test]
    fn heredoc_bodies_are_skipped_loosely() {
        let cmds = extract_loose(
            "python3 - <<'EOF'\nimport os\nprint(1)\nEOF\nnpm test\n",
            "",
        );
        let names: Vec<&str> = cmds.iter().map(|c| c.argv[0].as_str()).collect();
        assert_eq!(names, vec!["python3", "npm"]);
    }

    #[test]
    fn loose_extraction_reads_through_control_flow() {
        let cmds = extract_loose(
            "set -e\nif ! poetry run pytest -q; then\n  echo fail >&2\n  exit 1\nfi\ncd sub && make check 2>&1 | tee log\n",
            "",
        );
        let lines: Vec<String> = cmds
            .iter()
            .map(|c| format!("{}:{}", c.working_dir, c.argv.join(" ")))
            .collect();
        assert!(lines.contains(&":poetry run pytest -q".to_string()));
        assert!(lines.contains(&"sub:make check".to_string()));
        assert!(lines.contains(&"sub:tee log".to_string()));
        let g = extract_loose(
            "make lint || true\nmake check || exit 1\nRUSTFLAGS=-Dwarnings cargo build\n",
            "",
        );
        assert!(g[0].guarded);
        assert!(!g[2].guarded, "make check || exit 1 is a gate");
        for script in [
            "make a || rc=$?\n",
            "make a || ( echo x )\n",
            "make a || { echo x; }\n",
            "make a || echo x\n",
        ] {
            assert!(!extract_loose(script, "")[0].guarded, "{script}");
        }
        let u = extract_loose("cd \"$DIR\"\nmake check\ncd sub\nmake x\n", "");
        assert!(u.iter().all(|c| !c.wd_known), "{u:#?}");
        let e = extract_loose("export A=1\nset -a\nsource .env\ndeclare -x B=2\n", "");
        assert_eq!(
            e.iter().filter(|c| c.env_effect.is_some()).count(),
            4,
            "{e:#?}"
        );
        assert_eq!(
            github_file_writes("echo x >> \"$GITHUB_ENV\""),
            vec!["GITHUB_ENV"]
        );
        let cb = g.iter().find(|c| c.argv[0] == "cargo").unwrap();
        assert_eq!(
            cb.assignments,
            vec![("RUSTFLAGS".to_string(), "-Dwarnings".to_string())]
        );
    }

    #[test]
    fn substitutions_are_read_loosely() {
        let cmds = extract_loose("out=\"$(zizmor --format github .)\"\nn=$((1+2))\n", "");
        let names: Vec<&str> = cmds.iter().map(|c| c.argv[0].as_str()).collect();
        assert_eq!(names, vec!["zizmor"]);
    }

    #[test]
    fn report_grammar_is_an_allowlist() {
        for ok in [
            "python scripts/ci/check.py\n",
            "set -euo pipefail\ncd frontend && npm ci\nnpm run lint || true\necho done\n",
            "make a; make b\n",
        ] {
            assert!(report_grammar(ok).is_ok(), "{ok}: {:?}", report_grammar(ok));
        }
        for bad in [
            "make check && cd sub\n",
            "\"export\" A=1\n",
            "\\export A=1\n",
            "ex''port A=1\n",
            "make DESTDIR=~/x\n",
            "cd --\n",
            "cd -P sub\n",
            "make check # && rm -rf x\n",
            "printf -v PATH %s x\n",
            "echo -E x\n",
            "hash -p x make\n",
            "make check\r\n",
        ] {
            assert!(
                report_grammar(bad).is_err(),
                "{bad:?} should be outside the grammar"
            );
        }
        for ok in ["echo -n done\n", "# a comment\nmake check\n"] {
            assert!(report_grammar(ok).is_ok(), "{ok}: {:?}", report_grammar(ok));
        }
        for bad in [
            "pushd sub\nmake\n",
            "env FOO=1 make\n",
            "FOO=1 make\n",
            "FOO=1\n",
            "make > out.txt\n",
            "make < in.txt\n",
            "cat <<EOF\nx\nEOF\n",
            "make | tee log\n",
            "tee log\n",
            "( make )\n",
            "if true; then make; fi\n",
            "for f in a; do make; done\n",
            "echo $(date)\n",
            "export A=1\n",
            "source .env\n",
            ". .env\n",
            "eval make\n",
            "set -a\n",
            "cd -\n",
            "cd \"$DIR\"\n",
            "cd ..\n",
            "timeout 60 make\n",
            "make || rc=$?\n",
            "make $TARGET\n",
            "echo x >&2\n",
            "python ${{ github.workspace }}/x.py\n",
        ] {
            assert!(
                report_grammar(bad).is_err(),
                "{bad} should be outside the grammar"
            );
        }
    }

    #[test]
    fn case_patterns_are_not_commands() {
        let cmds = extract_loose(
            "case \"$R\" in\n  success) echo ok ;;\n  skipped|*) make fail; exit 1 ;;\nesac\n",
            "",
        );
        let names: Vec<&str> = cmds.iter().map(|c| c.argv[0].as_str()).collect();
        assert_eq!(names, vec!["echo", "make", "exit"]);
    }

    #[test]
    fn join_rel_stays_inside() {
        assert_eq!(join_rel("backend", "..").as_deref(), Some(""));
        assert_eq!(join_rel("", "./frontend/").as_deref(), Some("frontend"));
        assert_eq!(join_rel("", ".."), None);
        assert_eq!(join_rel("", "/abs"), None);
    }
}
