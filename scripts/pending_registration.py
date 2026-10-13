#!/usr/bin/env python3
"""pending_registration.py — declared, bounded waits for bindings ahead of runner main.

Plan: 2026-10-13-schemas-check-drift-red-while-bindings-await-runner-registration
Dossier: schemas-bindings-ahead-of-runner-generator

WHY. qontinui-schemas checks in generated bindings for types that
qontinui-runner's `src-tauri/src/schema_export.rs` registers with `add!`. A
schemas PR that adds a type lands BEFORE its runner registration (the runner's
`add!` cannot compile until the type exists on its pinned sibling). The push
run on `main` then regenerates against runner MAIN, which deletes the new
bindings, and `check-drift` reads red until the runner PR lands. While schemas
main is red, coord holds every PR in the repo.

`codegen/pending-registration.toml` declares those waits. This module decides,
per entry, whether the wait is legitimate RIGHT NOW. Only a legitimate
(ACTIVE) entry may absorb anything, and it absorbs exactly:

  files (2)  ts/src/generated/<T>.d.ts
             src/qontinui_schemas/generated/per_type/<module>.py
  lines (3)  `export type { T } from './T';`    in ts/src/generated/index.ts
             `from .per_type.<module> import T` in src/qontinui_schemas/generated/__init__.py
             `    "T",`                          in that file's __all__

and only where the regeneration DELETED them (present in HEAD, absent from the
regenerated working tree). A file regenerated with different content, or any
other line, is never absorbed.

An entry is ACTIVE only when ALL hold; otherwise it is INERT and absorbs nothing:
  1. `added` is a date, not in the future, and at most MAX_AGE_DAYS old;
  2. `runner_pr` names a PR in SIBLING_REPO;
  3. that PR is OPEN (read on this run);
  4. that PR's head registers T: `add!( "T",` across newlines, comments stripped;
  5. every read above succeeded. GitHub is read with GITHUB_TOKEN only; with no
     token the reader returns unknown(no token) and makes NO unauthenticated call.

`check-drift` is a GATING consumer: an unknown input withholds the tolerance,
it never grants it. Any failure of this module makes the caller absorb nothing.

Modes (all print to stdout; the caller treats a non-zero exit as failure):
  decide   JSON verdict for check-generated-drift.sh arm 2
  require  a leading PR with a TRAILING sibling must carry entries for the bindings it adds
  audit    the scheduled check: red on any INERT, STALE or near-expiry entry
"""

from __future__ import annotations

import argparse
import base64
import datetime as dt
import json
import os
import re
import subprocess
import sys
import tomllib
import urllib.error
import urllib.request
from dataclasses import dataclass
from pathlib import Path

# ── constants ────────────────────────────────────────────────────────────────

# A ceiling on FORGETTING, not an expected wait. Measured windows were about
# 2 days; runner#1990 was open 9 days when this was written.
MAX_AGE_DAYS = 14
# The audit warns this many days before an entry expires.
EXPIRY_WARNING_DAYS = 3

SIBLING_REPO = "qontinui/qontinui-runner"
SCHEMA_EXPORT_PATH = "src-tauri/src/schema_export.rs"

TOML_PATH = "codegen/pending-registration.toml"
TS_DIR = "ts/src/generated"
TS_BARREL = "ts/src/generated/index.ts"
PY_DIR = "src/qontinui_schemas/generated"
PY_BARREL = "src/qontinui_schemas/generated/__init__.py"
BARRELS = (TS_BARREL, PY_BARREL)

# Same filter check-generated-drift.sh arm 2 passes to `git diff -I`.
TIMESTAMP_RE = re.compile(r"^#   timestamp:")

# The trailing declaration forms of qontinui-runner's checkout-sibling action.
TRAILING_FORMS = ("self-upstream-of", "sibling-downstream-of")
# Every declaration form the action can emit.
DECLARED_FORMS = ("downstream-of", "upstream-of") + TRAILING_FORMS

# The only test seam: a JSON file standing in for GitHub. Never honoured inside
# the two workflows that make real decisions.
FIXTURE_ENV = "PENDING_REGISTRATION_FIXTURE"
REFUSE_FIXTURE_IN_WORKFLOWS = ("schema-drift", "pending-registration-audit")

TYPE_NAME_RE = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*$")
RUNNER_PR_RE = re.compile(r"^([A-Za-z0-9._-]+/[A-Za-z0-9._-]+)#([0-9]+)$")


class ModuleError(Exception):
    """A failure that must make the caller absorb nothing."""


# ── the entry file ───────────────────────────────────────────────────────────


@dataclass
class Entry:
    type: str
    runner_pr: str
    added: object  # a datetime.date when valid; anything else is invalid
    schemas_pr: str = ""


def load_entries(repo_root: Path) -> list[Entry]:
    """Parse the toml. An absent file is no entries; a malformed one is a ModuleError."""
    path = repo_root / TOML_PATH
    if not path.exists():
        return []
    try:
        data = tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as exc:
        raise ModuleError(f"{TOML_PATH} unreadable: {exc}") from exc
    rows = data.get("pending", [])
    if not isinstance(rows, list):
        raise ModuleError(f"{TOML_PATH}: `pending` must be an array of tables")
    entries = []
    for i, row in enumerate(rows):
        if not isinstance(row, dict):
            raise ModuleError(f"{TOML_PATH}: pending[{i}] is not a table")
        t = row.get("type")
        pr = row.get("runner_pr")
        if not isinstance(t, str) or not TYPE_NAME_RE.match(t):
            raise ModuleError(f"{TOML_PATH}: pending[{i}].type must be a Rust type name")
        if not isinstance(pr, str):
            raise ModuleError(f"{TOML_PATH}: pending[{i}].runner_pr must be a string")
        entries.append(Entry(t, pr, row.get("added"), str(row.get("schemas_pr", ""))))
    return entries


# ── reading GitHub ───────────────────────────────────────────────────────────


@dataclass
class PrState:
    state: str  # "open" | "closed"
    head_sha: str


class Unknown(Exception):
    """A read that could not be made. Renders as INERT(unknown(<reason>))."""


class GitHubReader:
    """The only network code. Token-only, timeout-bounded, typed failures."""

    def __init__(self, token: str | None, fetch=None, timeout: float = 15.0):
        self.token = token or None
        self.timeout = timeout
        self._fetch = fetch or self._urlopen
        self.calls = 0

    def _urlopen(self, url: str) -> bytes:
        req = urllib.request.Request(
            url,
            headers={
                "Authorization": f"Bearer {self.token}",
                "Accept": "application/vnd.github+json",
                "X-GitHub-Api-Version": "2022-11-28",
                "User-Agent": "qontinui-schemas-pending-registration",
            },
        )
        with urllib.request.urlopen(req, timeout=self.timeout) as resp:
            return resp.read()

    def _get_json(self, path: str):
        if not self.token:
            raise Unknown("no token")
        self.calls += 1
        try:
            raw = self._fetch(f"https://api.github.com{path}")
        except urllib.error.HTTPError as exc:
            raise Unknown(f"GET {path} -> HTTP {exc.code}") from exc
        except Exception as exc:  # network, timeout, anything: never a grant
            raise Unknown(f"GET {path} -> {type(exc).__name__}") from exc
        try:
            return json.loads(raw)
        except ValueError as exc:
            raise Unknown(f"GET {path} -> unparsable JSON") from exc

    def pr(self, repo: str, number: int) -> PrState:
        data = self._get_json(f"/repos/{repo}/pulls/{number}")
        try:
            return PrState(str(data["state"]), str(data["head"]["sha"]))
        except (KeyError, TypeError) as exc:
            raise Unknown(f"PR {repo}#{number}: unexpected shape") from exc

    def file_at(self, repo: str, path: str, ref: str) -> str:
        data = self._get_json(f"/repos/{repo}/contents/{path}?ref={ref}")
        try:
            return base64.b64decode(data["content"]).decode("utf-8")
        except (KeyError, TypeError, ValueError) as exc:
            raise Unknown(f"{repo}:{path}@{ref}: unexpected shape") from exc


class FixtureReader:
    """Test-only stand-in for GitHubReader, fed from a JSON file.

    Shape: {"prs": {"<owner/repo>#<n>": {"state": "open", "head_sha": "...",
             "schema_export": "<file text at head>"}},
            "main_schema_export": "<file text at the sibling's default branch>"}
    A PR or file missing from the fixture reads as Unknown, like a 404.
    """

    def __init__(self, data: dict):
        self.data = data
        self.calls = 0

    def pr(self, repo: str, number: int) -> PrState:
        self.calls += 1
        row = self.data.get("prs", {}).get(f"{repo}#{number}")
        if row is None:
            raise Unknown(f"PR {repo}#{number} not found")
        return PrState(row["state"], row["head_sha"])

    def file_at(self, repo: str, path: str, ref: str) -> str:
        self.calls += 1
        if ref == "main":
            text = self.data.get("main_schema_export")
        else:
            text = next(
                (r.get("schema_export") for r in self.data.get("prs", {}).values()
                 if r.get("head_sha") == ref),
                None,
            )
        if text is None:
            raise Unknown(f"{repo}:{path}@{ref} not found")
        return text


def make_reader():
    """The fixture seam when allowed, else the real reader."""
    fixture = os.environ.get(FIXTURE_ENV)
    if fixture:
        workflow = os.environ.get("GITHUB_WORKFLOW", "")
        if workflow in REFUSE_FIXTURE_IN_WORKFLOWS:
            raise ModuleError(f"{FIXTURE_ENV} is refused inside the {workflow} workflow")
        path = Path(fixture)
        if not path.is_file():
            raise ModuleError(f"{FIXTURE_ENV} points at a missing file: {fixture}")
        try:
            return FixtureReader(json.loads(path.read_text(encoding="utf-8")))
        except ValueError as exc:
            raise ModuleError(f"{FIXTURE_ENV} is not valid JSON: {exc}") from exc
    return GitHubReader(os.environ.get("GITHUB_TOKEN"))


# ── the `add!` registration test ─────────────────────────────────────────────


def strip_rust_comments(text: str) -> str:
    """Remove `//` line comments and `/* */` block comments, outside string literals."""
    out = []
    i, n = 0, len(text)
    in_str = False
    while i < n:
        c = text[i]
        if in_str:
            out.append(c)
            if c == "\\" and i + 1 < n:
                out.append(text[i + 1])
                i += 2
                continue
            if c == '"':
                in_str = False
            i += 1
        elif c == '"':
            in_str = True
            out.append(c)
            i += 1
        elif text.startswith("//", i):
            j = text.find("\n", i)
            i = n if j == -1 else j
        elif text.startswith("/*", i):
            j = text.find("*/", i + 2)
            i = n if j == -1 else j + 2
            out.append(" ")
        else:
            out.append(c)
            i += 1
    return "".join(out)


def registers(schema_export_text: str, type_name: str) -> bool:
    """True when `add!( "T",` appears in code (rustfmt may split it across lines)."""
    code = strip_rust_comments(schema_export_text)
    return re.search(r"add!\s*\(\s*\"" + re.escape(type_name) + r"\"\s*,", code) is not None


# ── per-entry metadata verdict ───────────────────────────────────────────────


def metadata_verdict(entry: Entry, reader, today: dt.date) -> tuple[bool, str]:
    """(eligible, reason). Conditions 1-5 of the module docstring, in order."""
    added = entry.added
    if not isinstance(added, dt.date) or isinstance(added, dt.datetime):
        return False, "invalid date"
    if added > today:
        return False, "invalid date"
    if (today - added).days > MAX_AGE_DAYS:
        return False, "expired"
    m = RUNNER_PR_RE.match(entry.runner_pr)
    if not m or m.group(1) != SIBLING_REPO:
        return False, "runner_pr not in the sibling repo"
    number = int(m.group(2))
    try:
        pr = reader.pr(SIBLING_REPO, number)
    except Unknown as exc:
        return False, f"unknown({exc})"
    if pr.state != "open":
        return False, "runner PR closed"
    try:
        text = reader.file_at(SIBLING_REPO, SCHEMA_EXPORT_PATH, pr.head_sha)
    except Unknown as exc:
        return False, f"unknown({exc})"
    if not registers(text, entry.type):
        return False, f"not registered at {pr.head_sha[:12]}"
    return True, ""


# ── the trees ────────────────────────────────────────────────────────────────


class Trees:
    """HEAD (via git) and the regenerated working tree."""

    def __init__(self, repo_root: Path):
        self.root = repo_root

    def head(self, path: str) -> str | None:
        r = subprocess.run(
            ["git", "-C", str(self.root), "show", f"HEAD:{path}"],
            capture_output=True,
        )
        if r.returncode != 0:
            # Distinguish "not in HEAD" from "git broke".
            probe = subprocess.run(
                ["git", "-C", str(self.root), "rev-parse", "--verify", "HEAD"],
                capture_output=True,
            )
            if probe.returncode != 0:
                raise ModuleError("git cannot read HEAD")
            return None
        return r.stdout.decode("utf-8")

    def work(self, path: str) -> str | None:
        p = self.root / path
        if not p.exists():
            return None
        return p.read_text(encoding="utf-8")


def ts_export_line(t: str) -> str:
    return f"export type {{ {t} }} from './{t}';"


def py_all_line(t: str) -> str:
    return f'    "{t}",'


def py_module_for(t: str, head_py_barrel: str | None) -> str | None:
    if head_py_barrel is None:
        return None
    m = re.search(
        r"^from \.per_type\.([A-Za-z0-9_]+) import " + re.escape(t) + r"$",
        head_py_barrel,
        re.M,
    )
    return m.group(1) if m else None


def lines_of(text: str | None) -> list[str]:
    return [] if text is None else text.splitlines()


def absorb_set(t: str, trees: Trees, head_barrels: dict, work_barrels: dict):
    """(files, {barrel: [lines]}) when T has exactly the pending shape, else (None, reason)."""
    module = py_module_for(t, head_barrels[PY_BARREL])
    if module is None:
        return None, f"shape: no `from .per_type.<m> import {t}` line in HEAD {PY_BARREL}"
    files = [f"{TS_DIR}/{t}.d.ts", f"{PY_DIR}/per_type/{module}.py"]
    for f in files:
        if trees.head(f) is None:
            return None, f"shape: {f} is not in HEAD"
        if trees.work(f) is not None:
            return None, f"shape: {f} was regenerated, not deleted"
    want = {
        TS_BARREL: [ts_export_line(t)],
        PY_BARREL: [f"from .per_type.{module} import {t}", py_all_line(t)],
    }
    for barrel, wanted in want.items():
        head_lines = lines_of(head_barrels[barrel])
        work_lines = lines_of(work_barrels[barrel])
        for line in wanted:
            if head_lines.count(line) != 1:
                return None, f"shape: `{line.strip()}` is not exactly once in HEAD {barrel}"
            if line in work_lines:
                return None, f"shape: `{line.strip()}` is still in the regenerated {barrel}"
    return (files, want), ""


def produced(t: str, trees: Trees) -> bool:
    return trees.work(f"{TS_DIR}/{t}.d.ts") is not None


def produced_label(source: str | None) -> str:
    """How to name a type the regeneration produced, by which tree regenerated it."""
    if source is None or source == "":
        return "PRODUCED (source not declared)"
    if source == "none":
        return "STALE (runner default branch produces it; delete this row)"
    if source.endswith("+pinned"):
        return "PRODUCED (pinned sibling)"
    forms = source.split("+")
    if any(f in DECLARED_FORMS for f in forms):
        return "PRODUCED-BY-DECLARED-SIBLING"
    return f"PRODUCED (source {source})"


def strip_ts(lines: list[str]) -> list[str]:
    return [l for l in lines if not TIMESTAMP_RE.match(l)]


def decide(repo_root: Path, entries: list[Entry], reader, today: dt.date,
           source: str | None, trees: Trees | None = None) -> dict:
    """The arm-2 verdict. Pure apart from reading the two trees and the reader."""
    trees = trees or Trees(repo_root)
    head_barrels = {b: trees.head(b) for b in BARRELS}
    work_barrels = {b: trees.work(b) for b in BARRELS}
    rows, exclude, absorbed_lines = [], [], {b: [] for b in BARRELS}
    for e in entries:
        if produced(e.type, trees):
            rows.append((e.type, produced_label(source)))
            continue
        ok, reason = metadata_verdict(e, reader, today)
        if not ok:
            rows.append((e.type, f"INERT ({reason})"))
            continue
        result, why = absorb_set(e.type, trees, head_barrels, work_barrels)
        if result is None:
            rows.append((e.type, f"INERT ({why})"))
            continue
        files, lines = result
        exclude.extend(files)
        for b, ls in lines.items():
            absorbed_lines[b].extend(ls)
        rows.append((e.type, "ACTIVE (absorbed 2 files, 3 lines)"))

    barrel_errors = []
    if exclude:
        # The barrels leave the git diff; compare them here, line by line.
        for b in BARRELS:
            head = lines_of(head_barrels[b])
            for line in absorbed_lines[b]:
                head.remove(line)
            if strip_ts(head) != strip_ts(lines_of(work_barrels[b])):
                barrel_errors.append(b)
    return {
        "version": 1,
        "exclude": exclude,
        "barrels_excluded": bool(exclude),
        "barrel_errors": barrel_errors,
        "rows": [{"type": t, "verdict": v} for t, v in rows],
    }


# ── require mode ─────────────────────────────────────────────────────────────


def is_trailing(source: str | None) -> bool:
    return bool(source) and any(f in TRAILING_FORMS for f in source.split("+"))


def ensure_commit(repo_root: Path, sha: str) -> None:
    """CI checks out the PR merge ref at depth 1, so the base commit is usually
    absent. A tree diff needs only the commit, so fetch exactly that, shallow."""
    have = subprocess.run(["git", "-C", str(repo_root), "cat-file", "-e", f"{sha}^{{commit}}"],
                          capture_output=True)
    if have.returncode == 0:
        return
    r = subprocess.run(["git", "-C", str(repo_root), "fetch", "--no-tags", "--depth=1", "origin", sha],
                       capture_output=True, text=True)
    if r.returncode != 0:
        raise ModuleError(f"cannot fetch base {sha}: {r.stderr.strip()}")


def added_types(repo_root: Path, base: str) -> list[str]:
    ensure_commit(repo_root, base)
    r = subprocess.run(
        ["git", "-C", str(repo_root), "diff", "--name-only", "--diff-filter=A",
         base, "HEAD", "--", f"{TS_DIR}/*.d.ts"],
        capture_output=True, text=True,
    )
    if r.returncode != 0:
        raise ModuleError(f"git diff against base {base} failed: {r.stderr.strip()}")
    return sorted(Path(p).name[: -len(".d.ts")] for p in r.stdout.split())


def require(entries: list[Entry], new_types: list[str], pr_created: dt.date,
            today: dt.date, runner_pr_hint: str) -> list[str]:
    """Problems with a trailing-sibling PR's entries; empty means satisfied."""
    by_type = {e.type: e for e in entries}
    problems = []
    for t in new_types:
        e = by_type.get(t)
        if e is None:
            problems.append(f"missing entry for {t}")
            continue
        if not RUNNER_PR_RE.match(e.runner_pr) or RUNNER_PR_RE.match(e.runner_pr).group(1) != SIBLING_REPO:
            problems.append(f"{t}: runner_pr must be {SIBLING_REPO}#<n>")
        a = e.added
        if not isinstance(a, dt.date) or isinstance(a, dt.datetime):
            problems.append(f"{t}: added must be a date")
        elif not (pr_created <= a <= today):
            problems.append(f"{t}: added {a} is outside [{pr_created}, {today}] "
                            "(the PR's creation date to the run date)")
    return problems


def paste_rows(types: list[str], runner_pr_hint: str, today: dt.date) -> str:
    out = []
    for t in types:
        out.append("[[pending]]")
        out.append(f'type = "{t}"')
        out.append(f'runner_pr = "{runner_pr_hint}"')
        out.append(f"added = {today.isoformat()}")
        out.append('schemas_pr = "qontinui/qontinui-schemas#<this PR>"')
        out.append("")
    return "\n".join(out)


# ── audit mode ───────────────────────────────────────────────────────────────


def audit(entries: list[Entry], reader, today: dt.date) -> list[tuple[str, str, bool]]:
    """(type, verdict, is_problem) per entry."""
    out = []
    main_text = None
    for e in entries:
        ok, reason = metadata_verdict(e, reader, today)
        if not ok:
            out.append((e.type, f"INERT ({reason})", True))
            continue
        if main_text is None:
            try:
                main_text = reader.file_at(SIBLING_REPO, SCHEMA_EXPORT_PATH, "main")
            except Unknown as exc:
                out.append((e.type, f"INERT (unknown({exc}))", True))
                continue
        if registers(main_text, e.type):
            out.append((e.type, "STALE (runner main registers it; delete this row)", True))
            continue
        left = MAX_AGE_DAYS - (today - e.added).days
        if left < EXPIRY_WARNING_DAYS:
            out.append((e.type, f"ACTIVE, expires in {left} day(s)", True))
            continue
        out.append((e.type, f"ACTIVE ({left} day(s) left)", False))
    return out


# ── CLI ──────────────────────────────────────────────────────────────────────


def _today() -> dt.date:
    return dt.datetime.now(dt.timezone.utc).date()


def _summary(lines: list[str]) -> None:
    path = os.environ.get("GITHUB_STEP_SUMMARY")
    if not path:
        return
    try:
        with open(path, "a", encoding="utf-8") as fh:
            fh.write("\n".join(lines) + "\n")
    except OSError:
        pass  # diagnostic only; never the reason the gate fails


def _rows_table(title: str, rows) -> list[str]:
    out = [f"### {title}", "", "| type | verdict |", "| --- | --- |"]
    out += [f"| `{t}` | {v} |" for t, v in rows]
    return out + [""]


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("mode", choices=["decide", "require", "audit"])
    ap.add_argument("--repo-root", default=".")
    ap.add_argument("--source", default=os.environ.get("SIBLING_SOURCE"))
    ap.add_argument("--base", default=os.environ.get("PR_BASE_SHA"))
    ap.add_argument("--pr-created-at", default=os.environ.get("PR_CREATED_AT"))
    args = ap.parse_args(argv)
    root = Path(args.repo_root)
    today = _today()

    try:
        entries = load_entries(root)
        if args.mode == "decide":
            reader = make_reader() if entries else None
            verdict = decide(root, entries, reader, today, args.source)
            print(json.dumps(verdict))
            rows = [(r["type"], r["verdict"]) for r in verdict["rows"]]
            if rows:
                for t, v in rows:
                    print(f"pending-registration: {t}: {v}", file=sys.stderr)
                _summary(_rows_table("Pending registrations (codegen/pending-registration.toml)", rows))
            return 0

        if args.mode == "require":
            if not is_trailing(args.source):
                print("pending-registration require: source is not a trailing declaration; nothing to require")
                return 0
            if not args.base or not args.pr_created_at:
                raise ModuleError("require needs --base and --pr-created-at")
            try:
                created = dt.datetime.fromisoformat(args.pr_created_at.replace("Z", "+00:00")).date()
            except ValueError as exc:
                raise ModuleError(f"--pr-created-at is not ISO-8601: {args.pr_created_at}") from exc
            new_types = added_types(root, args.base)
            hint = f"{SIBLING_REPO}#<runner PR number>"
            problems = require(entries, new_types, created, today, hint)
            if not problems:
                print(f"pending-registration require: {len(new_types)} added binding(s), all declared")
                return 0
            print("::error::This PR adds bindings that only its trailing qontinui-runner PR "
                  "produces, without matching codegen/pending-registration.toml entries:")
            for p in problems:
                print(f"  - {p}")
            missing = [t for t in new_types if t not in {e.type for e in entries}]
            if missing:
                print("Add these rows to codegen/pending-registration.toml:\n")
                print(paste_rows(missing, hint, today))
            return 1

        # audit
        if not entries:
            print("pending-registration audit: no entries")
            return 0
        results = audit(entries, make_reader(), today)
        for t, v, bad in results:
            print(f"{'::error::' if bad else ''}pending-registration: {t}: {v}")
        _summary(_rows_table("Pending-registration audit", [(t, v) for t, v, _ in results]))
        return 1 if any(bad for _, _, bad in results) else 0

    except ModuleError as exc:
        print(f"::error::pending-registration: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
