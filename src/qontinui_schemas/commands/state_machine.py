"""State-machine WS bridge command schemas.

These models define the request/reply payloads exchanged between
qontinui-web (HTTP handler) and qontinui-runner (Python dispatcher) for
state-machine discovery work that historically lived directly on the web
backend but now runs runner-side via the ``runner_command_ws`` relay.

Wire shape:

- Web -> runner: pubsub channel ``runner:commands:{runner_id}``,
  payload = ``DiscoverUIBridgeRequest.model_dump(mode='json')``.
- Runner -> web: pubsub channel ``runner:responses:{runner_id}``,
  payload = ``DiscoverUIBridgeResponse.model_dump(mode='json')`` on
  success, or ``DiscoverUIBridgeError.model_dump(mode='json')`` on
  failure. The web-side dispatcher filters by matching ``request_id``.

Phase 2 of plan
``plans/2026-05-17-web-runner-ws-bridge-plan-b.md`` introduces this
module + the single ``state_machine.discover_ui_bridge`` command.
Phases 3-7 extend it with additional command types.
"""

from typing import Any, Literal
from uuid import UUID

from pydantic import BaseModel, Field

# =============================================================================
# Command literal — shared by request + response for correlation
# =============================================================================

DISCOVER_UI_BRIDGE_COMMAND: Literal["state_machine.discover_ui_bridge"] = (
    "state_machine.discover_ui_bridge"
)


# =============================================================================
# Request / response / error envelopes
# =============================================================================


class DiscoverUIBridgeRequest(BaseModel):
    """Request envelope for ``state_machine.discover_ui_bridge``.

    Mirrors the existing ``UIBridgeDiscoverRequest`` shape from
    ``qontinui-web/backend/app/api/v1/endpoints/state_discovery.py``,
    augmented with the WS-bridge correlation fields (``command`` literal,
    ``request_id``). The ``type`` field is the runner-side WS dispatch
    key (matching the existing ``handle_relay_command`` ``msg_type``
    convention) and shadows the ``command`` literal so consumers may
    use whichever field they already key on.
    """

    type: Literal["state_machine.discover_ui_bridge"] = (
        "state_machine.discover_ui_bridge"
    )
    command: Literal["state_machine.discover_ui_bridge"] = (
        "state_machine.discover_ui_bridge"
    )
    request_id: UUID = Field(
        description="Correlation id; the response carries the same value.",
    )
    project_id: UUID | None = Field(
        default=None,
        description=(
            "Project that owns the discovery run (downstream logging only); "
            "may be omitted by endpoints that operate on raw render-logs "
            "without a project context."
        ),
    )
    renders: list[dict[str, Any]] = Field(
        default_factory=list,
        description="UI-bridge render-log entries (raw JSON dicts).",
    )
    include_html_ids: bool = Field(
        default=False,
        description="If True, HTML id attributes participate in state synthesis.",
    )
    cooccurrence_export: dict[str, Any] | None = Field(
        default=None,
        description=(
            "Optional co-occurrence export blob; when provided, the runner uses "
            "``StateDiscoveryService.discover_from_export`` instead of "
            "``discover_from_renders``."
        ),
    )
    strategy: Literal["auto", "fingerprint"] = Field(
        default="auto",
        description="Discovery strategy; maps to ``DiscoveryStrategyType``.",
    )


class DiscoverUIBridgeResponse(BaseModel):
    """Success envelope returned by the runner-side handler.

    Field names mirror the web-side ``UIBridgeDiscoveryResponse`` so the
    web handler can pass the inner payloads through with minimal
    transformation.
    """

    type: Literal["command_response"] = "command_response"
    command: Literal["state_machine.discover_ui_bridge"] = (
        "state_machine.discover_ui_bridge"
    )
    request_id: UUID
    states: list[dict[str, Any]] = Field(
        default_factory=list,
        description="Discovered states (``DiscoveredState.model_dump()`` shape).",
    )
    elements: list[dict[str, Any]] = Field(
        default_factory=list,
        description="Discovered elements (``DiscoveredElement.model_dump()`` shape).",
    )
    element_to_renders: dict[str, list[str]] = Field(
        default_factory=dict,
        description="Mapping from element id to render-log ids.",
    )
    render_count: int = 0
    unique_element_count: int = 0
    strategy_used: str = Field(
        default="fingerprint",
        description=(
            "Name of the ``DiscoveryStrategyType`` that produced the result."
        ),
    )
    strategy_metadata: dict[str, Any] = Field(
        default_factory=dict,
        description="Strategy-specific diagnostic metadata.",
    )


class DiscoverUIBridgeError(BaseModel):
    """Error envelope returned when the runner-side handler raises."""

    type: Literal["command_response"] = "command_response"
    command: Literal["state_machine.discover_ui_bridge"] = (
        "state_machine.discover_ui_bridge"
    )
    request_id: UUID
    error: Literal["qontinui_exception", "invalid_payload", "internal_error"]
    message: str
    traceback: str | None = None


__all__ = [
    "DISCOVER_UI_BRIDGE_COMMAND",
    "DiscoverUIBridgeRequest",
    "DiscoverUIBridgeResponse",
    "DiscoverUIBridgeError",
]
