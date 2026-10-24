#!/usr/bin/env python3
"""Generate the journey-query golden vectors from multistate's reference
implementation.

Plan 2026-09-20-ui-bridge-represents-the-users-path-and-the-passage-of-time,
D2 / Phase 2: the Rust journey queries (`rust/src/journey/query.rs`) must not
drift from the theory's reference implementation, multistate's
`multistate/testing/exploration/graph_utils.py`. This script runs that
module's `get_reachable_states`, `compute_shortest_distances` and
`find_terminal_states` -- each takes a multistate `PathTracker` and reads
`tracker.state_graph` -- over the fixture graphs below, and writes

    rust/tests/fixtures/journey-golden.json             (the vectors)
    rust/tests/fixtures/journey-golden.provenance.json  (multistate commit,
                                                        command, sha256)

`rust/tests/journey_golden.rs` asserts the Rust answers equal them.

How each fixture maps onto multistate
-------------------------------------
A fixture's `affordanceEdges` are the journey graph's NAVIGATION edges
(`navigationTrigger: affordance`, `outcome: changed`). They become the
multistate graph: one state per fixture node, one transition per edge.
`otherEdges` (pop / push / initial ...) are journey-only -- multistate has no
notion of how a transition was triggered -- so they are NOT given to
multistate; the Rust test still writes them to the ledger, which is how it
proves they do not leak into reachability.

`find_terminal_states` counts a self-loop as an outgoing transition, while a
journey dead end is "no navigation edge to a DIFFERENT node". So the terminal
vector is computed over the same graph with self-loops removed, and named
`terminalStatesWithoutSelfLoops` to say so.

The state graph handed to `PathTracker` is a duck-typed object exposing
exactly what `graph_utils.py` reads (`states[name].transitions[i].to_state`);
multistate's own StateGraph carries callables and config the reads never touch.

Fields under `handVerified` are NOT produced by multistate (it has no function
for them): `backOnly` per family, and the whole `prior_knowledge` family
(`requires_prior_knowledge`). Each is derived by hand in the comments beside
it.

Usage
-----
    python3 rust/tests/fixtures/journey/generate_golden.py \
        [--multistate-repo ../multistate] [--ref origin/main]

The multistate source is taken with `git archive <ref>` into a temporary
directory, so the checkout is never modified and the recorded commit is
exactly the code that ran. Output is deterministic (sorted keys); only
`generatedAt` in the provenance sidecar changes between runs at one commit.
"""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import io
import json
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path
from types import SimpleNamespace

HERE = Path(__file__).resolve().parent
FIXTURES = HERE.parent
REPO_ROOT = HERE.parents[3]
GOLDEN = FIXTURES / "journey-golden.json"
PROVENANCE = FIXTURES / "journey-golden.provenance.json"

# ---------------------------------------------------------------------------
# Fixture graphs (multistate-checked)
# ---------------------------------------------------------------------------
#
# otherEdges: {"from", "to", "trigger", "outcome"}. An isolated node is made
# observable by a reload of itself (`initial`, `no_change`).

FAMILIES = [
    {
        "name": "cycle_with_tail",
        "nodes": ["a", "b", "c", "d", "e"],
        "affordanceEdges": [["a", "b"], ["b", "c"], ["c", "a"], ["c", "d"], ["d", "e"]],
        "otherEdges": [],
        # e has no way out and no pop edge.
        "handVerified": {"backOnly": []},
    },
    {
        "name": "self_loops",
        "nodes": ["a", "b", "c"],
        "affordanceEdges": [["a", "a"], ["a", "b"], ["b", "b"], ["c", "c"]],
        "otherEdges": [],
        # b and c only loop onto themselves: dead ends, neither back-only.
        "handVerified": {"backOnly": []},
    },
    {
        "name": "disconnected_components",
        "nodes": ["a", "b", "c", "x", "y", "z"],
        "affordanceEdges": [["a", "b"], ["b", "c"], ["x", "y"], ["y", "x"]],
        "otherEdges": [
            # z is observed only by a reload: isolated in the navigation graph.
            {"from": "z", "to": "z", "trigger": "initial", "outcome": "no_change"},
            # A typed URL from c to x is not a path between the components.
            {"from": "c", "to": "x", "trigger": "push", "outcome": "changed"},
        ],
        # c and z are dead ends; neither has a pop edge.
        "handVerified": {"backOnly": []},
    },
    {
        "name": "pop_only_exit",
        "nodes": ["home", "list", "modal", "detail", "orphan"],
        "affordanceEdges": [
            ["home", "list"],
            ["list", "detail"],
            ["list", "modal"],
            ["detail", "list"],
        ],
        "otherEdges": [
            # modal can only be left with the browser's back button.
            {"from": "modal", "to": "list", "trigger": "pop", "outcome": "changed"},
            # orphan: reached by URL, left by back -- also back-only.
            {"from": "home", "to": "orphan", "trigger": "push", "outcome": "changed"},
            {"from": "orphan", "to": "home", "trigger": "pop", "outcome": "changed"},
        ],
        # Dead ends (no affordance exit): modal, orphan. Both have a pop edge
        # to a different node, so both are back-only.
        "handVerified": {"backOnly": ["modal", "orphan"]},
    },
    {
        "name": "diamond_with_shortcut",
        "nodes": ["a", "b", "c", "d", "e", "f"],
        "affordanceEdges": [
            ["a", "c"],
            ["a", "b"],
            ["b", "d"],
            ["c", "d"],
            ["d", "e"],
            ["b", "e"],
            ["e", "f"],
            ["f", "e"],
        ],
        "otherEdges": [],
        # The e <-> f cycle gives every node an exit: no dead ends at all.
        "handVerified": {"backOnly": []},
    },
]

# ---------------------------------------------------------------------------
# Hand-verified family: requires_prior_knowledge
# ---------------------------------------------------------------------------
#
# Definition (journey D2): nodes whose ONLY observed arrivals are
# initial | push | replace | hash, with no affordance edge into them from a
# different observed node. A pop arrival from a different node also
# disqualifies (the user had been there before by some other way).
#
#   home      no arrivals at all                      -> not listed
#   list      affordance from home                    -> disqualified
#   detail    affordance from list                    -> disqualified
#   deep      push x2 from home, initial (reload)     -> LISTED {initial:1, push:2}
#   settings  push from home AND affordance from home -> disqualified
#   hashpage  hash from list                          -> LISTED {hash:1}
#   backonly  replace from list, pop from detail      -> disqualified (pop)
#   selfclick push from home; affordance to itself    -> LISTED {push:1}
#             (a self-loop click is not a way in)
#   errored   push from home; affordance from list
#             that ERRORED but still landed there     -> disqualified

PRIOR_KNOWLEDGE = {
    "name": "prior_knowledge",
    "nodes": [
        "backonly",
        "deep",
        "detail",
        "errored",
        "hashpage",
        "home",
        "list",
        "selfclick",
        "settings",
    ],
    "affordanceEdges": [
        ["home", "list"],
        ["list", "detail"],
        ["home", "settings"],
        ["selfclick", "selfclick"],
    ],
    "otherEdges": [
        {"from": "home", "to": "deep", "trigger": "push", "outcome": "changed"},
        {"from": "home", "to": "deep", "trigger": "push", "outcome": "changed"},
        {"from": "deep", "to": "deep", "trigger": "initial", "outcome": "no_change"},
        {"from": "home", "to": "settings", "trigger": "push", "outcome": "changed"},
        {"from": "list", "to": "hashpage", "trigger": "hash", "outcome": "changed"},
        {"from": "list", "to": "backonly", "trigger": "replace", "outcome": "changed"},
        {"from": "detail", "to": "backonly", "trigger": "pop", "outcome": "changed"},
        {"from": "home", "to": "selfclick", "trigger": "push", "outcome": "changed"},
        {"from": "home", "to": "errored", "trigger": "push", "outcome": "changed"},
        {"from": "list", "to": "errored", "trigger": "affordance", "outcome": "error"},
    ],
    "handVerified": {
        "requiresPriorKnowledge": {
            "deep": {"initial": 1, "push": 2},
            "hashpage": {"hash": 1},
            "selfclick": {"push": 1},
        }
    },
}


def state_graph(nodes: list[str], edges: list[list[str]]) -> SimpleNamespace:
    """The duck-typed graph graph_utils.py reads."""
    states = {
        name: SimpleNamespace(
            transitions=[SimpleNamespace(to_state=t) for f, t in edges if f == name]
        )
        for name in nodes
    }
    return SimpleNamespace(states=states)


def run_multistate(family: dict, PathTracker, graph_utils) -> dict:
    nodes = family["nodes"]
    edges = family["affordanceEdges"]
    tracker = PathTracker(state_graph(nodes, edges), enable_screenshots=False)
    no_loops = [e for e in edges if e[0] != e[1]]
    tracker_no_loops = PathTracker(state_graph(nodes, no_loops), enable_screenshots=False)

    reachable = {}
    distances = {}
    for start in nodes:
        reachable[start] = sorted(graph_utils.get_reachable_states(tracker, start))
        distances[start] = dict(
            sorted(graph_utils.compute_shortest_distances(tracker, start).items())
        )
        if set(reachable[start]) != set(distances[start]):
            raise SystemExit(
                f"{family['name']}/{start}: multistate disagrees with itself "
                f"(reachable {reachable[start]} vs distances {sorted(distances[start])})"
            )
    return {
        "reachable": reachable,
        "distances": distances,
        "terminalStatesWithoutSelfLoops": sorted(
            graph_utils.find_terminal_states(tracker_no_loops)
        ),
    }


def git(repo: Path, *args: str) -> bytes:
    return subprocess.run(
        ["git", "-C", str(repo), *args], check=True, capture_output=True
    ).stdout


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument(
        "--multistate-repo",
        type=Path,
        default=REPO_ROOT.parent / "multistate",
        help="a git checkout of qontinui/multistate (read-only; default: the sibling)",
    )
    parser.add_argument("--ref", default="origin/main", help="multistate ref to run")
    args = parser.parse_args()

    commit = git(args.multistate_repo, "rev-parse", f"{args.ref}^{{commit}}").decode().strip()
    with tempfile.TemporaryDirectory() as tmp:
        archive = git(args.multistate_repo, "archive", commit, "src")
        with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
            tar.extractall(tmp, filter="data")
        sys.path.insert(0, str(Path(tmp) / "src"))
        from multistate.testing.exploration import graph_utils
        from multistate.testing.tracker import PathTracker

        families = []
        for family in FAMILIES:
            families.append(
                {
                    **family,
                    "source": "multistate",
                    "multistate": run_multistate(family, PathTracker, graph_utils),
                }
            )
    families.append({**PRIOR_KNOWLEDGE, "source": "hand-verified"})

    golden = {
        "about": (
            "Journey-query golden vectors. `multistate` blocks are output of "
            "multistate graph_utils.py at the commit in journey-golden.provenance.json; "
            "`handVerified` blocks are derived by hand in generate_golden.py. "
            "Regenerate with that script; never hand-edit."
        ),
        "families": families,
        "nodeSpecId": "golden",
    }
    text = json.dumps(golden, indent=2, sort_keys=True) + "\n"
    GOLDEN.write_text(text, encoding="utf-8")

    provenance = {
        "fixture": GOLDEN.name,
        "sha256": hashlib.sha256(text.encode("utf-8")).hexdigest(),
        "generator": "rust/tests/fixtures/journey/generate_golden.py",
        "generate_command": "python3 rust/tests/fixtures/journey/generate_golden.py"
        + (f" --ref {args.ref}" if args.ref != "origin/main" else ""),
        "multistate_repo": "qontinui/multistate",
        "multistate_ref": args.ref,
        "multistate_commit": commit,
        "multistate_module": "src/multistate/testing/exploration/graph_utils.py",
        "multistate_functions": [
            "compute_shortest_distances",
            "find_terminal_states",
            "get_reachable_states",
        ],
        "python": ".".join(map(str, sys.version_info[:3])),
        "generatedAt": dt.datetime.now(dt.timezone.utc).date().isoformat(),
    }
    PROVENANCE.write_text(json.dumps(provenance, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {GOLDEN} ({len(families)} families) at multistate {commit}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
