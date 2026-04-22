#!/usr/bin/env python3
"""Inject OpenAPI-style `discriminator` into schemars-emitted `oneOf` unions.

`schemars` v1 serialises `#[serde(tag = "...")]` Rust enums as `oneOf`
with each branch carrying `properties.<tag>.const = "..."` but does not
emit a sibling `discriminator` keyword. Without it, `datamodel-codegen`
generates `RootModel[Union[...]]` instead of a tagged
`Annotated[Union[...], Field(discriminator='<tag>')]`, which skips
Pydantic's fast-path discriminator validation.

This post-processor walks a `schemas.json` produced by the runner's
`export_schemas` binary and, for every `oneOf` whose branches share a
single `properties.<name>.const` discriminator, attaches
`discriminator: { propertyName: "<name>" }` next to the `oneOf`. The
output is rewritten in place (or to `--output`).

Idempotent: re-running on a schema that already has `discriminator`
leaves it unchanged.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any


def _find_discriminator(one_of: list[Any]) -> str | None:
    """Return the property name that discriminates this `oneOf`, or None.

    A branch qualifies if it has `properties.<name>` with a `const`
    string. The whole `oneOf` qualifies if every branch has the same
    `<name>` and its const values are pairwise distinct.
    """
    if not one_of or not all(isinstance(b, dict) for b in one_of):
        return None

    candidates: list[set[str]] = []
    for branch in one_of:
        props = branch.get("properties") or {}
        fields: set[str] = set()
        for name, spec in props.items():
            if isinstance(spec, dict) and isinstance(spec.get("const"), str):
                fields.add(name)
        candidates.append(fields)

    common = set.intersection(*candidates) if candidates else set()
    for name in sorted(common):
        consts = [b["properties"][name]["const"] for b in one_of]
        if len(set(consts)) == len(consts):
            return name
    return None


def _annotate(node: Any) -> int:
    """Recursively walk a schema node, injecting discriminator objects.

    Returns the number of unions annotated.
    """
    count = 0
    if isinstance(node, dict):
        one_of = node.get("oneOf")
        if isinstance(one_of, list) and one_of and "discriminator" not in node:
            prop = _find_discriminator(one_of)
            if prop is not None:
                node["discriminator"] = {"propertyName": prop}
                count += 1
        for v in node.values():
            count += _annotate(v)
    elif isinstance(node, list):
        for item in node:
            count += _annotate(item)
    return count


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", type=Path, help="schemas.json to rewrite")
    parser.add_argument(
        "--output",
        type=Path,
        default=None,
        help="write to this path instead of overwriting --input",
    )
    args = parser.parse_args()

    text = args.input.read_text(encoding="utf-8")
    data = json.loads(text)
    added = _annotate(data)

    out = args.output or args.input
    out.write_text(
        json.dumps(data, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )
    print(f"annotated {added} oneOf union(s) with discriminator", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
