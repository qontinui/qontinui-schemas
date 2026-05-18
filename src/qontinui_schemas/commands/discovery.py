"""Discovery WS bridge command schemas.

These models define the request/reply payloads exchanged between
qontinui-web (HTTP handler) and qontinui-runner (Python dispatcher) for
discovery-pipeline work that historically lived directly on the web
backend but now runs runner-side via the ``runner_command_ws`` relay.

Wire shape:

- Web -> runner: pubsub channel ``runner:commands:{runner_id}``,
  payload = ``BackgroundRemovalRequest.model_dump(mode='json')``.
- Runner -> web: pubsub channel ``runner:responses:{runner_id}``,
  payload = ``BackgroundRemovalResponse.model_dump(mode='json')`` on
  success, or ``BackgroundRemovalError.model_dump(mode='json')`` on
  failure. The web-side dispatcher filters by matching ``request_id``.

Phase 6 of plan
``plans/2026-05-17-web-runner-ws-bridge-plan-b.md`` introduces this
module with one command:

- ``discovery.background_removal`` — remove backgrounds from a batch of
  base64-encoded screenshots (used by ``POST /api/v1/remove-background``).
"""

from typing import Any, Literal
from uuid import UUID

from pydantic import BaseModel, Field

# =============================================================================
# Command literal — shared by request + response for correlation
# =============================================================================

BACKGROUND_REMOVAL_COMMAND: Literal["discovery.background_removal"] = (
    "discovery.background_removal"
)


# =============================================================================
# Request / response / error envelopes
# =============================================================================


class BackgroundRemovalRequest(BaseModel):
    """Request envelope for ``discovery.background_removal``.

    Mirrors the existing ``RemoveBackgroundRequest`` shape from
    ``qontinui-web/backend/app/api/v1/endpoints/background_removal.py``,
    augmented with the WS-bridge correlation fields (``command`` literal,
    ``request_id``). The ``type`` field is the runner-side WS dispatch
    key (matching the existing ``handle_relay_command`` ``msg_type``
    convention) and shadows the ``command`` literal so consumers may
    use whichever field they already key on.
    """

    type: Literal["discovery.background_removal"] = "discovery.background_removal"
    command: Literal["discovery.background_removal"] = "discovery.background_removal"
    request_id: UUID = Field(
        description="Correlation id; the response carries the same value.",
    )
    screenshots_b64: list[str] = Field(
        default_factory=list,
        description=(
            "Base64-encoded PNG/JPEG screenshot payloads to process. The "
            "runner decodes, analyses, and returns masked RGBA PNGs."
        ),
    )
    config: dict[str, Any] | None = Field(
        default=None,
        description=(
            "Optional ``BackgroundRemovalConfig`` payload (dataclass field "
            "names). When omitted, the runner uses dataclass defaults. "
            "See ``qontinui.discovery.background_removal.BackgroundRemovalConfig``."
        ),
    )
    debug: bool = Field(
        default=False,
        description=(
            "If True, the response statistics dict includes "
            "``background_mask_base64`` — a base64 PNG of the inferred "
            "background mask for diagnostics."
        ),
    )


class BackgroundRemovalResponse(BaseModel):
    """Success envelope returned by the runner-side handler.

    The runner invokes
    :func:`qontinui.discovery.background_removal.remove_backgrounds_from_base64`
    and returns the masked screenshots plus the analyser's statistics
    dict (pixel counts, percentages, image size, optional debug mask).
    Field names mirror the HTTP-layer ``RemoveBackgroundResponse``
    schema so the web handler can pass the inner payload through with
    minimal transformation.
    """

    type: Literal["command_response"] = "command_response"
    command: Literal["discovery.background_removal"] = "discovery.background_removal"
    request_id: UUID
    masked_screenshots_b64: list[str] = Field(
        default_factory=list,
        description=(
            "Masked RGBA base64-encoded PNGs (one per input screenshot, "
            "same order)."
        ),
    )
    statistics: dict[str, Any] = Field(
        default_factory=dict,
        description=(
            "Analyser statistics dict — keys include ``total_pixels``, "
            "``background_pixels``, ``foreground_pixels``, "
            "``background_percentage``, ``foreground_percentage``, "
            "``num_screenshots``, ``image_size``, and (when ``debug=True``) "
            "``background_mask_base64``."
        ),
    )


class BackgroundRemovalError(BaseModel):
    """Error envelope returned when the runner-side handler raises."""

    type: Literal["command_response"] = "command_response"
    command: Literal["discovery.background_removal"] = "discovery.background_removal"
    request_id: UUID
    error: Literal["qontinui_exception", "invalid_payload", "internal_error"]
    message: str
    traceback: str | None = None


__all__ = [
    "BACKGROUND_REMOVAL_COMMAND",
    "BackgroundRemovalRequest",
    "BackgroundRemovalResponse",
    "BackgroundRemovalError",
]
