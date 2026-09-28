#!/usr/bin/env python3
"""Cross-engine check for fleet-nouns.toml: Rust, Python and JS must agree.

fleet-nouns.toml promises that every class pattern means the same thing under
Rust `regex`, Python `re` and JavaScript `new RegExp(p)` (no flags), because its
consumers are written in all three. rust/tests/fleet_nouns_vocabulary.rs can run
only the Rust engine, so this script runs the other two over the SAME texts and
compares their hit sets with the matrix that test writes.

    FLEET_NOUNS_MATRIX_OUT=/tmp/rust.json cargo test -p qontinui-types \
        --test fleet_nouns_vocabulary hit_matrix_for_the_cross_engine_check
    python3 .github/scripts/fleet-nouns-portability.py /tmp/rust.json

Every engine applies the consumer contract in the file's header: find all
non-overlapping pattern matches, discard one that overlaps an exclude match,
and the text is a hit when one survives. Exit 1 on any disagreement, on a
missing/empty Rust matrix, or when the three engines saw different texts.
"""

import json
import re
import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
VOCAB = ROOT / "fleet-nouns.toml"
JS = Path(__file__).with_name("fleet-nouns-portability.cjs")


def all_texts(v):
    """Same order and dedup as the Rust test's `all_texts`."""
    out, seen = [], set()
    for c in v["class"]:
        for t in c["examples"] + c.get("literals", []):
            if t not in seen:
                seen.add(t)
                out.append(t)
    for t in v["lookalikes"] + [k["value"] for k in v["product_constant"]] + v["portability_probes"]:
        if t not in seen:
            seen.add(t)
            out.append(t)
    return out


def py_hits(c, text):
    excl = [m.span() for m in re.finditer(c["exclude"], text)] if c.get("exclude") else []
    return any(
        not any(m.start() < e and s < m.end() for s, e in excl)
        for m in re.finditer(c["pattern"], text)
    )


def main():
    if len(sys.argv) != 2:
        sys.exit("usage: fleet-nouns-portability.py <rust-matrix.json>")
    v = tomllib.loads(VOCAB.read_text(encoding="utf-8"))
    texts = all_texts(v)

    rust = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
    if not rust:
        sys.exit("the Rust hit matrix is empty — UNKNOWN, not agreement")
    py = [[t, [c["id"] for c in v["class"] if py_hits(c, t)]] for t in texts]
    payload = json.dumps({"class": v["class"], "texts": texts})
    js = json.loads(
        subprocess.run(["node", str(JS)], input=payload, capture_output=True,
                       text=True, check=True, encoding="utf-8").stdout
    )

    problems = []
    for name, m in (("rust", rust), ("js", js)):
        if [row[0] for row in m] != texts:
            problems.append(f"{name} evaluated a different text list than python ({len(m)} vs {len(texts)})")
    if not problems:
        for (t, p), (_, r), (_, j) in zip(py, rust, js):
            if not (p == r == j):
                problems.append(f"{t!r}: rust={r} python={p} js={j}")

    print(f"fleet-nouns portability: {len(texts)} texts x {len(v['class'])} classes, "
          f"{len(problems)} disagreement(s)")
    for p in problems:
        print(f"::error::{p}")
    sys.exit(1 if problems else 0)


if __name__ == "__main__":
    main()
