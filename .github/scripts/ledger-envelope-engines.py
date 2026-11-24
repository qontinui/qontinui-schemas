#!/usr/bin/env python3
"""Cross-engine check: the frozen ledger envelope schema agrees with validate().

rust/src/ledger.rs says non-Rust writers validate against the COMMITTED schema
(rust/tests/fixtures/ledger_envelope.v<N>.schema.json) while Rust writers call
`validate`. rust/tests/ledger_envelope_golden.rs proves that agreement under
the Rust `jsonschema` crate only. This script proves it under the two
validators non-Rust writers actually run — python-jsonschema and ajv — over the
same corpus, using the Rust verdicts that test writes:

    LEDGER_ENVELOPE_CORPUS_OUT=/tmp/corpus.json cargo test -p qontinui-types \\
        --test ledger_envelope_golden agreement_corpus_for_the_cross_engine_check
    NODE_PATH=<dir>/node_modules <venv>/bin/python \\
        .github/scripts/ledger-envelope-engines.py /tmp/corpus.json

Dependencies are pinned in .github/scripts/ledger-envelope-engines/
(package-lock.json for ajv + ajv-formats, a hashed requirements.txt for
jsonschema + rfc3339-validator); rust-ci's `ledger-envelope-engines` job
installs them with `npm ci --ignore-scripts` and `pip install --require-hashes`.

Per engine: the formats-on verdict must equal Rust's, except on the corpus's
declared exceptions (which must be schema-admits / Rust-refuses) and the
engine's own ENGINE_DIFFERENCES (each must disagree exactly as declared); and
the formats-off verdict must differ from formats-on on exactly the corpus's
format-only cases. Exit 1 on any disagreement, and on everything that would
make agreement UNKNOWN: a missing or unparsable corpus, one of other than
CORPUS_SIZE cases or exercising one outcome only, a malformed ENGINE_DIFFERENCES
entry, a missing schema file, a `date-time` checker that is not loaded, an ajv
or ajv-formats whose version is not the lockfile's, or an engine that printed
nothing. Once the corpus loads, every problem is counted on a summary line, so
a failing run never prints only lines that read as agreement; a corpus, case
shape or schema that stops the run before the engines prints `corpus: not
checked`.

Plan 2026-11-24-ledger-envelope-schema-agreement-under-ajv-and-python-jsonschema.
"""

import json
import subprocess
import sys
from pathlib import Path
from typing import Any

# One corpus case, the committed schema, and one engine's verdicts by case name.
Case = dict[str, Any]
Schema = dict[str, Any]
Verdicts = dict[str, tuple[bool, bool]]

ROOT = Path(__file__).resolve().parents[2]
JS = Path(__file__).with_name("ledger-envelope-engines.cjs")

# The corpus size this check was last measured at, matched EXACTLY, as both
# entries and distinct case names (the corpus repeats one case under one name):
# fewer means the writer dropped cases or wrote a stub, more means cases were
# added without moving these (so a later drop of as many would go unseen), and
# the right entry count over fewer names means a case was swapped for a repeat
# of another and goes unchecked. Any corpus change moves both in the same change.
CORPUS_SIZE = 301
CORPUS_DISTINCT = 300

# Measured per-engine disagreements a writer following the module docs cannot
# be harmed by: case name -> {"engine": the engine's formats-on verdict,
# "reason": why that is writer-safe}. Every entry must be exercised exactly as
# declared — a stale entry fails like a new disagreement. Measured empty for
# both engines on 2026-11-24 (ajv 8.20.0 + ajv-formats 3.0.1, jsonschema
# 4.26.0 + rfc3339-validator 0.1.4). An engine ADMITTING a case validate()
# refuses is never declarable: that is a hole writers can drive data through,
# and compare() refuses such an entry. A declared exception (DECLARED_EXCEPTIONS
# in the Rust test) is never an entry here either.
ENGINE_DIFFERENCES: dict[str, dict[str, dict[str, Any]]] = {"python-jsonschema": {}, "ajv": {}}


def error(msg: str) -> None:
    print(f"::error::{msg}")


def load_corpus(path: str) -> tuple[tuple[list[Case], Schema] | None, list[str]]:
    """(the corpus, the schema it was written for) or None, plus the refusals.

    None means nothing can be checked; the refusals then say why. Otherwise the
    refusals are corpus problems the run still reports beside the engines.
    """
    try:
        doc = json.loads(Path(path).read_text(encoding="utf-8"))
    except (OSError, ValueError) as e:
        return None, [f"the corpus {path} is missing or unparsable ({e}) — UNKNOWN, not agreement"]
    problems = []
    cases = doc.get("cases") if isinstance(doc, dict) else None
    if not isinstance(cases, list) or not cases:
        return None, [f"the corpus {path} has no cases — UNKNOWN, not agreement"]
    if doc.get("count") != len(cases):
        problems.append(f"the corpus says count={doc.get('count')} but carries {len(cases)} cases — truncated or hand-edited")
    if len(cases) != CORPUS_SIZE:
        problems.append(
            f"the corpus has {len(cases)} cases, not the {CORPUS_SIZE} this check was measured at — cases were "
            "dropped or added (a deliberate corpus change must move CORPUS_SIZE in the same change)"
        )
    keys = {"name", "instance", "rust", "declared_exception", "format_only"}
    flags = ("rust", "declared_exception", "format_only")
    if any(
        not isinstance(c, dict)
        or not keys <= c.keys()
        or not isinstance(c["name"], str)
        or not all(isinstance(c[f], bool) for f in flags)
        for c in cases
    ):
        problems.append(
            f"a corpus case is not an object carrying {sorted(keys)} with a string `name` and boolean {list(flags)}"
        )
        return None, problems
    distinct = len({c["name"] for c in cases})
    if distinct != CORPUS_DISTINCT:
        problems.append(
            f"the corpus has {distinct} distinct cases, not the {CORPUS_DISTINCT} this check was measured at — a case "
            "was dropped, added or replaced by a repeat (a deliberate corpus change must move CORPUS_DISTINCT too)"
        )
    # A name is a case's identity in every comparison below. The corpus may
    # repeat a case (`/source = "x\n"` is both a trailing-newline case and a
    # label case), but one name must never stand for two different instances.
    first: dict[str, Case] = {}
    for c in cases:
        seen = first.setdefault(c["name"], c)
        if seen["instance"] != c["instance"] or seen["rust"] != c["rust"]:
            problems.append(f"the corpus name `{c['name']}` stands for two different cases")
    verdicts = {c["rust"] for c in cases}
    if verdicts != {True, False}:
        problems.append(f"the corpus's Rust verdicts are all {verdicts} — it exercises only one outcome")
    if not any(c["declared_exception"] for c in cases):
        problems.append("no corpus case is flagged declared_exception — the writer lost DECLARED_EXCEPTIONS")
    if not any(c["format_only"] for c in cases):
        problems.append("no corpus case is flagged format_only — the writer lost FORMAT_ONLY_CASES")

    schema_file = ROOT / "rust/tests/fixtures" / f"ledger_envelope.v{doc.get('schema_version')}.schema.json"
    try:
        schema = json.loads(schema_file.read_text(encoding="utf-8"))
    except (OSError, ValueError) as e:
        problems.append(f"the committed schema {schema_file} is missing or unparsable ({e})")
        return None, problems
    return (cases, schema), problems


def python_arm(schema: Schema, cases: list[Case]) -> Verdicts:
    """{name: (formats_on, formats_off)} under python-jsonschema."""
    # Untyped for mypy unless types-jsonschema is installed, which this script
    # does not depend on: mypy --strict is clean with or without the stubs.
    import jsonschema  # type: ignore[import-untyped,unused-ignore]
    from jsonschema.validators import validator_for  # type: ignore[import-untyped,unused-ignore]

    cls = validator_for(schema)
    if cls is not jsonschema.Draft202012Validator:
        raise RuntimeError(f"the schema resolved to {cls.__name__}, not Draft202012Validator")
    cls.check_schema(schema)
    checker = jsonschema.FormatChecker()
    if "date-time" not in checker.checkers:
        raise RuntimeError(
            "FormatChecker has no `date-time` checker — rfc3339-validator is not installed, so "
            "python-jsonschema would silently skip date-time (2026-02-30 would pass)"
        )
    on = cls(schema, format_checker=checker)
    off = cls(schema)
    return {c["name"]: (on.is_valid(c["instance"]), off.is_valid(c["instance"])) for c in cases}


def ajv_arm(schema: Schema, cases: list[Case]) -> Verdicts:
    """{name: (formats_on, formats_off)} under ajv, via the .cjs arm."""
    payload = json.dumps({"schema": schema, "cases": [{"name": c["name"], "instance": c["instance"]} for c in cases]})
    # Bytes in and out, decoded once after the call: a decode failure is then
    # a RuntimeError the caller reports as "did not run", never a lost stream.
    proc = subprocess.run(["node", str(JS)], input=payload.encode("utf-8"), capture_output=True)
    stderr = proc.stderr.decode("utf-8", errors="replace").strip()
    if proc.returncode != 0:
        raise RuntimeError(f"node exited {proc.returncode}: {stderr}")
    try:
        stdout = proc.stdout.decode("utf-8")
    except UnicodeDecodeError as e:
        raise RuntimeError(f"the ajv arm printed output that is not UTF-8 ({e})") from e
    if not stdout.strip():
        raise RuntimeError("the ajv arm printed nothing")
    rows = json.loads(stdout)
    if [r[0] for r in rows] != [c["name"] for c in cases]:
        raise RuntimeError(f"the ajv arm evaluated a different case list ({len(rows)} rows vs {len(cases)} cases)")
    return {name: (on, off) for name, on, off in rows}


def compare(engine: str, cases: list[Case], verdicts: Verdicts) -> tuple[list[str], str]:
    """The problems one engine's verdicts raise, plus its summary line.

    Every problem counts as one disagreement, so this engine's summary line
    never reads as agreement beside a failure of its own. Each bad entry
    raises exactly one entry problem; its case is then compared like any
    other, so a case that also disagrees adds its own.
    """
    problems = []
    declared = ENGINE_DIFFERENCES[engine]
    by_name = {c["name"]: c for c in cases}
    usable = set()
    for name, entry in sorted(declared.items()):
        if not (isinstance(entry, dict) and isinstance(entry.get("engine"), bool)
                and isinstance(entry.get("reason"), str) and entry["reason"].strip()):
            problems.append(f"{engine}: ENGINE_DIFFERENCES entry `{name}` needs a bool `engine` and a `reason`")
        elif name not in by_name:
            problems.append(f"{engine}: ENGINE_DIFFERENCES names `{name}`, which is not in the corpus — stale entry")
        elif by_name[name]["declared_exception"]:
            problems.append(
                f"{engine}: ENGINE_DIFFERENCES names `{name}`, a declared exception — it is checked as one, "
                "so the entry is never exercised; remove it"
            )
        elif entry["engine"] and not by_name[name]["rust"]:
            problems.append(
                f"{engine}: ENGINE_DIFFERENCES entry `{name}` declares {engine} ADMITS a case validate() refuses — "
                "never declarable: a writer could drive that data through"
            )
        else:
            usable.add(name)
    # Only well-formed, in-corpus, declarable entries are compared as declared
    # differences; a refused entry's case is compared like any other.
    exceptions = 0
    for name, c in by_name.items():
        rust = c["rust"]
        on, _ = verdicts[name]
        if c["declared_exception"]:
            exceptions += 1
            if not (on and not rust):
                problems.append(
                    f"{engine}: declared exception `{name}` no longer disagrees as documented "
                    f"(schema admits, validate refuses): {engine}={on} validate={rust}"
                )
        elif name in usable:
            if on != declared[name]["engine"] or on == rust:
                problems.append(
                    f"{engine}: ENGINE_DIFFERENCES entry `{name}` is stale — declared {engine}="
                    f"{declared[name]['engine']} vs validate={rust}, measured {engine}={on}"
                )
        elif on != rust:
            problems.append(f"{engine}: `{name}`: {engine}={on} validate={rust}")
    format_only = sorted({c["name"] for c in cases if c["format_only"]})
    missed = sorted(n for n, (on, off) in verdicts.items() if on != off)
    if missed != format_only:
        problems.append(
            f"{engine}: with formats off the schema must miss exactly the format-only cases {format_only}; "
            f"it missed {missed}"
        )
    summary = (
        f"{engine}: {len(by_name)} distinct cases, {exceptions + len(usable)} declared, "
        f"{len(problems)} disagreement(s); formats off misses {len(missed)} case(s)"
    )
    return problems, summary


def main() -> int:
    if len(sys.argv) != 2:
        sys.exit("usage: ledger-envelope-engines.py <corpus.json>")
    loaded, problems = load_corpus(sys.argv[1])
    if loaded is None:
        # Nothing was checked: the corpus, a case's shape or the committed
        # schema stopped the run before either engine saw a case.
        print("corpus: not checked")
        for p in problems:
            error(p)
        return 1
    cases, schema = loaded
    # The corpus's own problems get a summary line too: a failing run whose
    # engines agreed must not print only lines that read as agreement.
    summaries = [f"corpus: {len(cases)} entries, {len({c['name'] for c in cases})} distinct, {len(problems)} problem(s)"]
    for engine, arm in (("python-jsonschema", python_arm), ("ajv", ajv_arm)):
        try:
            verdicts = arm(schema, cases)
        except Exception as e:  # noqa: BLE001 — any arm failure is UNKNOWN, and is reported
            problems.append(f"{engine}: the engine did not run ({type(e).__name__}: {e}) — UNKNOWN, not agreement")
            summaries.append(f"{engine}: did not run")
            continue
        engine_problems, summary = compare(engine, cases, verdicts)
        problems += engine_problems
        summaries.append(summary)
    for s in summaries:
        print(s)
    for p in problems:
        error(p)
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
