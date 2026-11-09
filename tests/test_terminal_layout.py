"""The generated ``TerminalLayout`` model accepts what the runner serializes.

qontinui-web validates a relayed ``GET /terminal-layout`` body with this model
(plan ``2026-11-08-save-per-tab-runner-session-lists-on-sessions-page-kept-14-days``,
D3 and D9). These tests pin the two wire rules the Rust type documents:

- null means unknown or not applicable, never a stand-in: an optional key accepts
  ``null``, a missing optional key reads as ``None``, the runner's ``-1``
  sentinel is refused for ``order`` and ``zoneIndex``, and a required key is
  refused when missing, never defaulted.
- forward compatibility: a key added by a newer runner is ignored, never refused.

The example is the same body ``rust/tests/terminal_layout_wire.rs`` round-trips.
"""

import copy
import json
from typing import Any

import pytest
from pydantic import ValidationError

from qontinui_schemas.generated import (
    TerminalLayout,
    TerminalLayoutPage,
    TerminalLayoutRunner,
    TerminalLayoutSession,
)

FULL_EXAMPLE: dict[str, Any] = {
    "runner": {
        "instanceName": "primary",
        "port": 9876,
        "buildId": "20261108.1",
        "deviceId": "0b6f3c1e-8f7a-4c55-9d1e-2a4b6c8d0e1f",
    },
    "labelsRegistered": True,
    "capturedAt": "2026-11-08T09:30:00Z",
    "pages": [
        {
            "pageId": "page-a",
            "label": "Coord work",
            "order": 0,
            "sessions": [
                {
                    "terminalId": "term-1",
                    "title": "claude: layout wire type",
                    "titleWithheld": False,
                    "isAlive": True,
                    "spawnName": "plan-phase-1",
                    "claudeSessionId": "61a189b8-6516-459d-a981-9659f36b6743",
                    "accountLabel": "susanne",
                    "sessionName": "layout wire type",
                    "zoneIndex": 2,
                    "finished": True,
                    "finishedAt": "2026-11-08T09:00:00Z",
                    "finishReason": "unattended",
                },
                {
                    "terminalId": "term-2",
                    "title": None,
                    "titleWithheld": True,
                    "isAlive": False,
                    "spawnName": None,
                    "claudeSessionId": None,
                    "accountLabel": None,
                    "sessionName": None,
                    "zoneIndex": None,
                    "finished": None,
                    "finishedAt": None,
                    "finishReason": None,
                },
            ],
        },
        {"pageId": "page-b", "label": None, "order": None, "sessions": []},
    ],
}

SESSION_TRI_STATE_FIELDS = (
    "title",
    "spawnName",
    "claudeSessionId",
    "accountLabel",
    "sessionName",
    "zoneIndex",
    "finished",
    "finishedAt",
    "finishReason",
)


def test_full_example_validates_and_dumps_back_unchanged() -> None:
    layout = TerminalLayout.model_validate_json(json.dumps(FULL_EXAMPLE))
    assert layout.model_dump(mode="json") == FULL_EXAMPLE


def test_unknown_values_stay_none_not_a_stand_in() -> None:
    layout = TerminalLayout.model_validate(FULL_EXAMPLE)
    unknown = layout.pages[0].sessions[1]
    for field in SESSION_TRI_STATE_FIELDS:
        assert getattr(unknown, field) is None, field
    assert unknown.titleWithheld is True
    assert layout.pages[1].label is None
    assert layout.pages[1].order is None


def test_missing_optional_keys_read_as_none() -> None:
    session = TerminalLayoutSession.model_validate(
        {"terminalId": "term-3", "titleWithheld": False, "isAlive": True}
    )
    for field in SESSION_TRI_STATE_FIELDS:
        assert getattr(session, field) is None, field

    page = TerminalLayoutPage.model_validate({"pageId": "page-c", "sessions": []})
    assert page.label is None
    assert page.order is None

    runner = TerminalLayoutRunner.model_validate({})
    assert runner.model_dump() == {
        "instanceName": None,
        "port": None,
        "buildId": None,
        "deviceId": None,
    }


def test_keys_from_a_newer_runner_are_ignored_not_refused() -> None:
    body = copy.deepcopy(FULL_EXAMPLE)
    body["schemaVersion"] = 2
    body["runner"]["gitSha"] = "0123abcd"
    body["pages"][0]["color"] = "blue"
    body["pages"][0]["sessions"][0]["coordStatus"] = {"value": "finished", "read": "ok"}

    layout = TerminalLayout.model_validate(body)
    assert layout.model_dump(mode="json") == FULL_EXAMPLE


def test_a_negative_zone_index_is_refused() -> None:
    session = copy.deepcopy(FULL_EXAMPLE["pages"][0]["sessions"][0])
    session["zoneIndex"] = -1
    with pytest.raises(ValidationError):
        TerminalLayoutSession.model_validate(session)


def test_a_negative_order_is_refused() -> None:
    page = copy.deepcopy(FULL_EXAMPLE["pages"][0])
    page["order"] = -1
    with pytest.raises(ValidationError):
        TerminalLayoutPage.model_validate(page)


@pytest.mark.parametrize("missing", ["terminalId", "titleWithheld", "isAlive"])
def test_a_session_missing_a_required_key_is_refused(missing: str) -> None:
    session = copy.deepcopy(FULL_EXAMPLE["pages"][0]["sessions"][0])
    del session[missing]
    with pytest.raises(ValidationError):
        TerminalLayoutSession.model_validate(session)


@pytest.mark.parametrize(
    "missing", ["runner", "labelsRegistered", "capturedAt", "pages"]
)
def test_a_layout_missing_a_required_key_is_refused(missing: str) -> None:
    body = copy.deepcopy(FULL_EXAMPLE)
    del body[missing]
    with pytest.raises(ValidationError):
        TerminalLayout.model_validate(body)
