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
Phase 3 extends it with two further commands:

- ``state_machine.ui_bridge.discover`` — discover-and-persist (used by
  ``POST /api/v1/projects/{project_id}/ui-bridge-discover``).
- ``state_machine.ui_bridge.pathfind`` — multistate pathfinding over a
  persisted UIBridgeRuntimeConfig (used by
  ``POST /api/v1/projects/{project_id}/ui-bridge-configs/{config_id}/pathfind``).

Phases 4-7 extend it with additional command types.
"""

from typing import Any, Literal
from uuid import UUID

from pydantic import BaseModel, Field

# =============================================================================
# Command literals — shared by request + response for correlation
# =============================================================================

DISCOVER_UI_BRIDGE_COMMAND: Literal["state_machine.discover_ui_bridge"] = (
    "state_machine.discover_ui_bridge"
)

UI_BRIDGE_DISCOVER_COMMAND: Literal["state_machine.ui_bridge.discover"] = (
    "state_machine.ui_bridge.discover"
)

UI_BRIDGE_PATHFIND_COMMAND: Literal["state_machine.ui_bridge.pathfind"] = (
    "state_machine.ui_bridge.pathfind"
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


# =============================================================================
# Phase 3 — state_machine.ui_bridge.discover (discover + persist)
# =============================================================================


class UIBridgeDiscoverRequest(BaseModel):
    """Request envelope for ``state_machine.ui_bridge.discover``.

    Mirrors the pre-#136 endpoint body shape from
    ``UIBridgeDiscoverAndSaveRequest`` in
    ``qontinui-web/backend/app/schemas/ui_bridge_state.py``: the runner
    performs the discovery; the web side persists the result to the
    ``ui_bridge_state_configs`` + ``ui_bridge_states`` rows after the
    response returns.

    Subtly different from
    :class:`DiscoverUIBridgeRequest` (Phase 2): this command targets a
    specific ``project_id`` for the persistence step, and the caller
    chooses the ``config_name`` for the resulting row.
    """

    type: Literal["state_machine.ui_bridge.discover"] = (
        "state_machine.ui_bridge.discover"
    )
    command: Literal["state_machine.ui_bridge.discover"] = (
        "state_machine.ui_bridge.discover"
    )
    request_id: UUID = Field(
        description="Correlation id; the response carries the same value.",
    )
    project_id: UUID = Field(
        description=(
            "Project that will own the persisted ``ui_bridge_state_configs`` "
            "row produced by the web side after the response returns."
        ),
    )
    renders: list[dict[str, Any]] = Field(
        default_factory=list,
        description="UI-bridge render-log entries (raw JSON dicts).",
    )
    config_name: str = Field(
        default="default",
        min_length=1,
        max_length=255,
        description="Name for the ``ui_bridge_state_configs`` row to create.",
    )
    config_description: str | None = Field(
        default=None,
        description="Optional description for the persisted config row.",
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


class UIBridgeDiscoverResponse(BaseModel):
    """Success envelope returned by the runner-side handler.

    Carries the runner-produced discovery result. Web persists the
    states + transitions to PG after receiving this; the runner is
    stateless w.r.t. the persistence step.
    """

    type: Literal["command_response"] = "command_response"
    command: Literal["state_machine.ui_bridge.discover"] = (
        "state_machine.ui_bridge.discover"
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


class UIBridgeDiscoverError(BaseModel):
    """Error envelope returned when the runner-side discover handler raises."""

    type: Literal["command_response"] = "command_response"
    command: Literal["state_machine.ui_bridge.discover"] = (
        "state_machine.ui_bridge.discover"
    )
    request_id: UUID
    error: Literal["qontinui_exception", "invalid_payload", "internal_error"]
    message: str
    traceback: str | None = None


# =============================================================================
# Phase 3 — state_machine.ui_bridge.pathfind
# =============================================================================


class UIBridgePathfindRequest(BaseModel):
    """Request envelope for ``state_machine.ui_bridge.pathfind``.

    The runner is stateless w.r.t. the persisted
    ``UIBridgeRuntimeConfig`` — the web side serialises the config rows
    (states + transitions) into the ``config`` field and the runner
    reconstructs a ``UIBridgeRuntime`` from it before invoking
    ``find_path``. This avoids a runner -> web -> runner round-trip for
    the config fetch.
    """

    type: Literal["state_machine.ui_bridge.pathfind"] = (
        "state_machine.ui_bridge.pathfind"
    )
    command: Literal["state_machine.ui_bridge.pathfind"] = (
        "state_machine.ui_bridge.pathfind"
    )
    request_id: UUID = Field(
        description="Correlation id; the response carries the same value.",
    )
    project_id: UUID = Field(
        description="Project that owns the runtime config (downstream logging only).",
    )
    config_id: UUID = Field(
        description=(
            "DB id of the ``ui_bridge_state_configs`` row whose serialised "
            "states + transitions are carried in ``config``."
        ),
    )
    from_states: list[str] = Field(
        default_factory=list,
        description=(
            "Currently-active state IDs (UIBridgeStateConfig state_id values, "
            "NOT DB UUIDs)."
        ),
    )
    target_states: list[str] = Field(
        default_factory=list,
        description="Target state IDs to reach.",
    )
    strategy: Literal["min_hops", "max_reliability", "balanced"] = Field(
        default="balanced",
        description=(
            "Pathfinding strategy hint. Currently advisory — the runner uses "
            "the runtime's configured ``SearchStrategy`` for the actual search."
        ),
    )
    config: dict[str, Any] = Field(
        default_factory=dict,
        description=(
            "Serialised ``UIBridgeRuntimeConfig`` payload: "
            '{"states": [{"state_id", "name", "element_ids"}, ...], '
            '"transitions": [{"transition_id", "name", "from_states", '
            '"activate_states", "exit_states", "actions", "path_cost", '
            '"stays_visible"}, ...]}.'
        ),
    )


class UIBridgePathfindStep(BaseModel):
    """One step in a pathfinding result.

    Matches the web-side ``PathfindingStep`` schema so the web handler
    can re-emit each step with minimal transformation.
    """

    transition_id: str
    transition_name: str
    from_states: list[str] = Field(default_factory=list)
    activate_states: list[str] = Field(default_factory=list)
    exit_states: list[str] = Field(default_factory=list)
    path_cost: float


class UIBridgePathfindResponse(BaseModel):
    """Success envelope returned by the runner-side pathfinding handler.

    ``found=False`` is a valid non-error outcome (no path exists); a
    runner-side exception surfaces via :class:`UIBridgePathfindError`.
    """

    type: Literal["command_response"] = "command_response"
    command: Literal["state_machine.ui_bridge.pathfind"] = (
        "state_machine.ui_bridge.pathfind"
    )
    request_id: UUID
    found: bool = False
    steps: list[UIBridgePathfindStep] = Field(default_factory=list)
    total_cost: float = 0.0
    error: str | None = Field(
        default=None,
        description=(
            "Non-exceptional reason for ``found=False`` (e.g. 'no path "
            "between specified states'). Runner-side exceptions surface "
            "via :class:`UIBridgePathfindError` instead."
        ),
    )


class UIBridgePathfindError(BaseModel):
    """Error envelope returned when the runner-side pathfind handler raises."""

    type: Literal["command_response"] = "command_response"
    command: Literal["state_machine.ui_bridge.pathfind"] = (
        "state_machine.ui_bridge.pathfind"
    )
    request_id: UUID
    error: Literal[
        "qontinui_exception",
        "invalid_payload",
        "internal_error",
        "multistate_unavailable",
    ]
    message: str
    traceback: str | None = None


__all__ = [
    "DISCOVER_UI_BRIDGE_COMMAND",
    "UI_BRIDGE_DISCOVER_COMMAND",
    "UI_BRIDGE_PATHFIND_COMMAND",
    "DiscoverUIBridgeRequest",
    "DiscoverUIBridgeResponse",
    "DiscoverUIBridgeError",
    "UIBridgeDiscoverRequest",
    "UIBridgeDiscoverResponse",
    "UIBridgeDiscoverError",
    "UIBridgePathfindRequest",
    "UIBridgePathfindStep",
    "UIBridgePathfindResponse",
    "UIBridgePathfindError",
]
