"""Cross-language fixture round-trip tests for the generated Pydantic models.

These tests pin down that the JSON fixtures committed under ``tests/fixtures``
can be parsed by the generated ``qontinui_schemas.generated.models`` classes
and re-serialized without drift at the dict-equality level. They form the
Python half of the cross-language Rust/Python/TypeScript fixture contract
defined in Section 6 Layer 3 of the qontinui-types plan.
"""

import json
from pathlib import Path

import pytest
from pydantic import BaseModel, ValidationError

from qontinui_schemas.generated import (
    BoundedReadMeta,
    Constraint,
    ScheduledTask,
    UnifiedWorkflow,
)

FIXTURES = Path(__file__).parent / "fixtures"


def _roundtrip(model_cls: type[BaseModel], fixture_name: str) -> None:
    raw = (FIXTURES / fixture_name).read_text()
    parsed = model_cls.model_validate_json(raw)
    re_raw = parsed.model_dump_json(exclude_none=True, by_alias=True)
    # Compare as dicts so field ordering doesn't matter.
    assert json.loads(re_raw) == json.loads(raw), f"{fixture_name}: round-trip diverged"


def test_constraint_fixture_roundtrips() -> None:
    _roundtrip(Constraint, "constraint_sample.json")


def test_scheduled_task_fixture_roundtrips() -> None:
    _roundtrip(ScheduledTask, "scheduled_task_sample.json")


def test_unified_workflow_frame_fixture_roundtrips() -> None:
    _roundtrip(UnifiedWorkflow, "unified_workflow_frame_sample.json")


# ── Strict validation tests ──
#
# These pin down that the per-type Pydantic generation produces real
# `BaseModel`s with enum / literal validation, not `RootModel[Any]`
# pass-throughs (which was the bug with the old combined-schema codegen).


def test_constraint_validates_severity_enum() -> None:
    with pytest.raises(ValidationError):
        Constraint.model_validate_json(
            '{"id":"x","name":"y","description":"z",'
            '"check":{"type":"file_scope","allowed_paths":[]},'
            '"severity":"NOT_A_VALID_SEVERITY","enabled":true}'
        )


def test_constraint_validates_check_type_literal() -> None:
    with pytest.raises(ValidationError):
        Constraint.model_validate_json(
            '{"id":"x","name":"y","description":"z",'
            '"check":{"type":"not_a_real_check_kind"},'
            '"severity":"warn","enabled":true}'
        )


def test_scheduled_task_validates_status_enum() -> None:
    # `status` lives on the nested `last_run` (TaskExecutionRecord) field.
    # Corrupt it to prove the ScheduledTaskStatus enum is really validated
    # (not a `RootModel[Any]` pass-through).
    raw = (FIXTURES / "scheduled_task_sample.json").read_text()
    payload = json.loads(raw)
    payload["lastRun"]["status"] = "NOT_A_REAL_STATUS"
    with pytest.raises(ValidationError):
        ScheduledTask.model_validate(payload)


# ── DB round-trip validation ──
#
# These mirror the Rust integration tests in `rust/tests/round_trip.rs`. They
# stand in for DB-persisted workflow rows: the full fixture exercises every
# FullRunnerStep variant; the unknown-step fixture exercises the `list[Any]`
# fallback that lets the generated Pydantic model preserve runner-specific and
# forward-compatible step types verbatim.


def test_unified_workflow_full_fixture_roundtrips() -> None:
    _roundtrip(UnifiedWorkflow, "unified_workflow_full_sample.json")


def test_unified_workflow_unknown_step_fixture_roundtrips() -> None:
    _roundtrip(UnifiedWorkflow, "workflow_with_unknown_step_sample.json")


# ── BoundedReadMeta: the cross-language fixture contract (Python half) ──
#
# Plan ``2026-09-05-every-bounded-read-is-a-page-that-reads-as-a-corpus``
# Phase 1. Each ``bounded_read_meta_*.json`` fixture is stored in RFC 8785
# (JCS) canonical form. The Rust half (``rust/tests/round_trip.rs``,
# ``bounded_read_meta_fixtures_roundtrip_to_identical_jcs_bytes``) and this one
# both parse it into the typed model, re-serialize, canonicalize, and compare
# BYTES against the file — not dict equality.
#
# Unlike ``_roundtrip`` above, this does NOT pass ``exclude_none``: on this
# wire ``null`` is a value ("no count ran", "unknown", "no next page"), so every
# key must survive serialization.

BOUNDED_READ_META_KEYS = frozenset(
    {
        "available",
        "bound_kind",
        "count",
        "enumerate_via",
        "filter_narrowed",
        "limit",
        "next_cursor",
        "shown",
        "total",
        "truncated",
    }
)

BOUNDED_READ_META_FIXTURES = [
    "exact",
    "at_least",
    "complete",
    "unknown",
    "unavailable",
    "filter_narrowed",
    "ranked_not_pageable",
    # /memory/query with no capped arm: exact pool size, a ranking that cannot
    # page (truncated, no cursor, enumerate_via set).
    "ranked_exact",
]


def _jcs(value: object) -> str:
    """Minimal RFC 8785 (JCS) canonical serialization.

    Complete for the JSON these fixtures carry — objects, arrays, strings,
    integers, booleans and null — and it REFUSES floats rather than emit a
    number formatting it has not implemented (JCS requires ECMAScript
    Number-to-String, which ``repr(float)`` does not match in general). The
    Rust half uses ``serde_jcs``, the crate ``canonical_hash.rs`` hashes with.

    - object keys sorted by UTF-16 code units (RFC 8785 §3.2.3);
    - no insignificant whitespace;
    - strings escaped as JSON requires and nothing more (``ensure_ascii=False``;
      Python emits lowercase ``\\u00xx`` for the remaining control characters,
      as JCS does).
    """
    if value is None or isinstance(value, bool):
        return json.dumps(value)
    if isinstance(value, int):
        return str(value)
    if isinstance(value, float):
        raise TypeError("_jcs: floats are not supported by this minimal JCS")
    if isinstance(value, str):
        return json.dumps(value, ensure_ascii=False)
    if isinstance(value, list):
        return "[" + ",".join(_jcs(v) for v in value) + "]"
    if isinstance(value, dict):
        items = sorted(value.items(), key=lambda kv: kv[0].encode("utf-16-be"))
        return "{" + ",".join(f"{_jcs(k)}:{_jcs(v)}" for k, v in items) + "}"
    raise TypeError(f"_jcs: unsupported type {type(value).__name__}")


def test_jcs_helper_sorts_keys_by_utf16_and_drops_whitespace() -> None:
    # U+E000 sorts before U+1F600 by code point but AFTER it by UTF-16 code
    # unit (the surrogate 0xD83D < 0xE000) — the case a plain sort gets wrong.
    assert _jcs({"\ue000": 1, "\U0001f600": 2, "a": [True, None, "x\n"]}) == (
        '{"a":[true,null,"x\\n"],"\U0001f600":2,"\ue000":1}'
    )
    with pytest.raises(TypeError):
        _jcs({"f": 1.5})


@pytest.mark.parametrize("name", BOUNDED_READ_META_FIXTURES)
def test_bounded_read_meta_fixture_roundtrips_to_identical_jcs_bytes(
    name: str,
) -> None:
    raw = (FIXTURES / f"bounded_read_meta_{name}.json").read_text(encoding="utf-8")
    expected = raw.rstrip("\n")
    # The fixture must itself be canonical, or the comparison below would be
    # against a spelling neither language emits.
    assert _jcs(json.loads(raw)) == expected, f"{name}: fixture is not JCS"

    parsed = BoundedReadMeta.model_validate_json(raw)
    dumped = parsed.model_dump(mode="json", by_alias=True)
    assert set(dumped) == BOUNDED_READ_META_KEYS, f"{name}: key set diverged"
    assert _jcs(dumped) == expected, f"{name}: JCS bytes diverged"


def test_bounded_read_meta_rejects_an_unknown_bound_kind() -> None:
    raw = json.loads(
        (FIXTURES / "bounded_read_meta_exact.json").read_text(encoding="utf-8")
    )
    raw["bound_kind"] = "approximately"
    with pytest.raises(ValidationError):
        BoundedReadMeta.model_validate(raw)


# Keys the Rust type declares non-Option: the schema already requires them.
BOUNDED_READ_META_NON_NULLABLE_KEYS = [
    "available",
    "bound_kind",
    "count",
    "limit",
    "shown",
]
# Keys the Rust type declares Option<_>: `null` is a value, but the key must
# still be PRESENT — an absent `total` is "older server", not "no count ran".
BOUNDED_READ_META_NULLABLE_KEYS = [
    "enumerate_via",
    "filter_narrowed",
    "next_cursor",
    "total",
    "truncated",
]


def _exact_fixture_without(key: str) -> dict[str, object]:
    raw: dict[str, object] = json.loads(
        (FIXTURES / "bounded_read_meta_exact.json").read_text(encoding="utf-8")
    )
    del raw[key]
    return raw


def test_bounded_read_meta_key_partition_is_complete() -> None:
    assert (
        set(BOUNDED_READ_META_NON_NULLABLE_KEYS) | set(BOUNDED_READ_META_NULLABLE_KEYS)
        == BOUNDED_READ_META_KEYS
    )


@pytest.mark.parametrize("missing", BOUNDED_READ_META_NON_NULLABLE_KEYS)
def test_bounded_read_meta_rejects_a_missing_non_nullable_key(missing: str) -> None:
    with pytest.raises(ValidationError):
        BoundedReadMeta.model_validate(_exact_fixture_without(missing))


@pytest.mark.parametrize("missing", BOUNDED_READ_META_NULLABLE_KEYS)
def test_bounded_read_meta_rejects_a_missing_nullable_key(missing: str) -> None:
    with pytest.raises(ValidationError):
        BoundedReadMeta.model_validate(_exact_fixture_without(missing))
