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
