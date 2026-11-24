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
make agreement UNKNOWN: a missing, unparsable, short or one-sided corpus, a
missing schema file, a `date-time` checker that is not loaded, or an engine
that printed nothing.

Plan 2026-11-24-ledger-envelope-schema-agreement-under-ajv-and-python-jsonschema.
"""

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
JS = Path(__file__).with_name("ledger-envelope-engines.cjs")

# The corpus size this check was last measured at. A smaller corpus means the
# writer dropped cases (or wrote a stub), which is UNKNOWN, not agreement. A
# corpus addition must raise this in the same change.
CORPUS_FLOOR = 301

# Measured per-engine disagreements a writer following the module docs cannot
# be harmed by: case name -> {"engine": the engine's formats-on verdict,
# "reason": why that is writer-safe}. Every entry must be exercised exactly as
# declared — a stale entry fails like a new disagreement. Measured empty for
# both engines on 2026-11-24 (ajv 8.20.0 + ajv-formats 3.0.1, jsonschema
# 4.26.0 + rfc3339-validator 0.1.4). An engine ADMITTING a case validate()
# refuses is never declarable: that is a hole writers can drive data through.
ENGINE_DIFFERENCES = {"python-jsonschema": {}, "ajv": {}}


def error(msg):
    print(f"::error::{msg}")


def load_corpus(path):
    """The corpus and the schema it was written for, or a list of refusals."""
    try:
        doc = json.loads(Path(path).read_text(encoding="utf-8"))
    except (OSError, ValueError) as e:
        return None, None, [f"the corpus {path} is missing or unparsable ({e}) — UNKNOWN, not agreement"]
    problems = []
    cases = doc.get("cases") if isinstance(doc, dict) else None
    if not isinstance(cases, list) or not cases:
        return None, None, [f"the corpus {path} has no cases — UNKNOWN, not agreement"]
    if doc.get("count") != len(cases):
        problems.append(f"the corpus says count={doc.get('count')} but carries {len(cases)} cases — truncated or hand-edited")
    if len(cases) < CORPUS_FLOOR:
        problems.append(
            f"the corpus has {len(cases)} cases, below the floor of {CORPUS_FLOOR} — cases were dropped "
            "(a deliberate corpus change must move CORPUS_FLOOR in the same change)"
        )
    keys = {"name", "instance", "rust", "declared_exception", "format_only"}
    if any(not isinstance(c, dict) or not keys <= c.keys() for c in cases):
        problems.append(f"a corpus case lacks one of {sorted(keys)}")
        return None, None, problems
    # A name is a case's identity in every comparison below. The corpus may
    # repeat a case (`/source = "x\n"` is both a trailing-newline case and a
    # label case), but one name must never stand for two different instances.
    first = {}
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
        return None, None, problems
    return cases, schema, problems


def python_arm(schema, cases):
    """{name: (formats_on, formats_off)} under python-jsonschema."""
    import jsonschema
    from jsonschema.validators import validator_for

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


def ajv_arm(schema, cases):
    """{name: (formats_on, formats_off)} under ajv, via the .cjs arm."""
    payload = json.dumps({"schema": schema, "cases": [{"name": c["name"], "instance": c["instance"]} for c in cases]})
    proc = subprocess.run(["node", str(JS)], input=payload, capture_output=True, text=True, encoding="utf-8")
    if proc.returncode != 0:
        raise RuntimeError(f"node exited {proc.returncode}: {proc.stderr.strip()}")
    if not proc.stdout.strip():
        raise RuntimeError("the ajv arm printed nothing")
    rows = json.loads(proc.stdout)
    if [r[0] for r in rows] != [c["name"] for c in cases]:
        raise RuntimeError(f"the ajv arm evaluated a different case list ({len(rows)} rows vs {len(cases)} cases)")
    return {name: (on, off) for name, on, off in rows}


def compare(engine, cases, verdicts):
    """The problems one engine's verdicts raise, plus its summary line."""
    problems = []
    declared = ENGINE_DIFFERENCES[engine]
    names = {c["name"] for c in cases}
    for name in sorted(set(declared) - names):
        problems.append(f"{engine}: ENGINE_DIFFERENCES names `{name}`, which is not in the corpus — stale entry")
    disagreements = 0
    exceptions = 0
    for c in cases:
        name, rust = c["name"], c["rust"]
        on, _ = verdicts[name]
        if c["declared_exception"]:
            exceptions += 1
            if not (on and not rust):
                problems.append(
                    f"{engine}: declared exception `{name}` no longer disagrees as documented "
                    f"(schema admits, validate refuses): {engine}={on} validate={rust}"
                )
                disagreements += 1
        elif name in declared:
            if on != declared[name]["engine"] or on == rust:
                problems.append(
                    f"{engine}: ENGINE_DIFFERENCES entry `{name}` is stale — declared {engine}="
                    f"{declared[name]['engine']} vs validate={rust}, measured {engine}={on}"
                )
                disagreements += 1
        elif on != rust:
            problems.append(f"{engine}: `{name}`: {engine}={on} validate={rust}")
            disagreements += 1
    format_only = sorted(c["name"] for c in cases if c["format_only"])
    missed = sorted(n for n, (on, off) in verdicts.items() if on != off)
    if missed != format_only:
        problems.append(
            f"{engine}: with formats off the schema must miss exactly the format-only cases {format_only}; "
            f"it missed {missed}"
        )
        disagreements += 1
    summary = (
        f"{engine}: {len(cases)} cases, {exceptions + len(declared)} declared, "
        f"{disagreements} disagreement(s); formats off misses {len(missed)} case(s)"
    )
    return problems, summary


def main():
    if len(sys.argv) != 2:
        sys.exit("usage: ledger-envelope-engines.py <corpus.json>")
    cases, schema, problems = load_corpus(sys.argv[1])
    if cases is None:
        for p in problems:
            error(p)
        return 1
    summaries = []
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
