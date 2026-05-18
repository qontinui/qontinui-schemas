"""Click-analysis WS bridge command schemas.

These models define the request/reply payloads exchanged between
qontinui-web (HTTP handler) and qontinui-runner (Python dispatcher) for
click-analysis tuning work that historically lived directly on the web
backend but now runs runner-side via the ``runner_command_ws`` relay.

Wire shape:

- Web -> runner: pubsub channel ``runner:commands:{runner_id}``,
  payload = ``TuneProfileRequest.model_dump(mode='json')``.
- Runner -> web: pubsub channel ``runner:responses:{runner_id}``,
  payload = ``TuneProfileResponse.model_dump(mode='json')`` on success
  or ``TuneProfileError.model_dump(mode='json')`` on failure. The
  web-side dispatcher filters by matching ``request_id``.

Phase 5 of plan
``plans/2026-05-17-web-runner-ws-bridge-plan-b.md`` introduces this
module + the single ``click_analysis.tune_profile`` command. The
runner-side handler downloads the supplied S3 screenshot URLs, calls
:class:`qontinui.discovery.click_analysis.ApplicationTuner` against the
decoded screenshots + ``known_elements`` ground truth, and returns the
``TuningResult`` payload.
"""

from typing import Any, Literal
from uuid import UUID

from pydantic import BaseModel, Field

# =============================================================================
# Command literals — shared by request + response for correlation
# =============================================================================

TUNE_PROFILE_COMMAND: Literal["click_analysis.tune_profile"] = (
    "click_analysis.tune_profile"
)


# =============================================================================
# Detection strategy + element type enum literals
# =============================================================================
#
# The runner-side qontinui module declares these as Python ``Enum`` types
# with lower_snake_case ``value`` fields. We mirror those values as a
# string literal union here so the wire protocol stays human-readable
# JSON (no enum-by-name lookups, no case sensitivity). The runner-side
# handler converts back into the qontinui enums by ``value`` lookup
# (``DetectionStrategy(s)`` / ``ElementType(t)``).

# From ``qontinui.discovery.click_analysis.models.DetectionStrategy``:
DetectionStrategyLiteral = Literal[
    "edge_based",
    "contour_based",
    "color_segmentation",
    "flood_fill",
    "gradient_based",
    "template_match",
    "fixed_size",
]

# From ``qontinui.discovery.click_analysis.models.ElementType``:
ElementTypeLiteral = Literal[
    "button",
    "icon",
    "text",
    "image",
    "checkbox",
    "radio",
    "input_field",
    "link",
    "menu_item",
    "tab",
    "unknown",
]


# =============================================================================
# Request / response / error envelopes
# =============================================================================


class TuneProfileRequest(BaseModel):
    """Request envelope for ``click_analysis.tune_profile``.

    Mirrors the existing web-side ``TuningRequest`` shape from
    ``qontinui_schemas.template_capture.models.TuningRequest`` augmented
    with WS-bridge correlation fields (``command`` literal, ``request_id``)
    + the project/profile routing fields the web handler already has on
    its URL path. The runner downloads the screenshots from
    ``screenshot_urls`` directly (web-side does NOT pre-fetch bytes), so
    the runner-Python venv must have network access + (where applicable)
    S3 credentials matching the URL signing scheme.

    ``known_elements`` entries are the JSON dict shape of
    :class:`qontinui.discovery.click_analysis.models.InferredBoundingBox`
    — typically ``CandidateBoundingBox.model_dump()`` from the web side.
    The runner-side handler reconstructs ``InferredBoundingBox`` instances
    from these dicts before passing them to ``ApplicationTuner``.
    """

    type: Literal["click_analysis.tune_profile"] = "click_analysis.tune_profile"
    command: Literal["click_analysis.tune_profile"] = "click_analysis.tune_profile"
    request_id: UUID = Field(
        description="Correlation id; the response carries the same value.",
    )
    project_id: UUID | None = Field(
        default=None,
        description=(
            "Project that owns the application profile being tuned "
            "(downstream logging only; the runner is stateless w.r.t. the "
            "profile row). Optional — the existing tune_profile route does "
            "not carry a project_id in its URL path or body, so this field "
            "is permitted to be omitted by callers."
        ),
    )
    profile_name: str = Field(
        min_length=1,
        max_length=255,
        description=(
            "Name of the application profile being tuned. Forwarded to "
            "the runner for logging context only."
        ),
    )
    screenshot_urls: list[str] = Field(
        default_factory=list,
        description=(
            "URLs to sample screenshots the runner downloads directly. "
            "Typically presigned S3 URLs; HTTP(S) URLs that resolve "
            "without authentication are also valid."
        ),
    )
    known_elements: list[dict[str, Any]] = Field(
        default_factory=list,
        description=(
            "Optional ground-truth bounding boxes (one per screenshot or "
            "across screenshots). Each item is the JSON dict shape of "
            "``qontinui.discovery.click_analysis.models.InferredBoundingBox`` "
            "(``x``, ``y``, ``width``, ``height``, ``confidence``, "
            "``strategy_used``, ``element_type``, ``metadata``). The runner "
            "reconstructs ``InferredBoundingBox`` instances before invoking "
            "the tuner."
        ),
    )
    detection_strategies: list[DetectionStrategyLiteral] | None = Field(
        default=None,
        description=(
            "Optional override for the detection strategies the tuner "
            "should consider. When omitted, the tuner ranks its default "
            "strategy set. Values are the lower_snake_case ``value`` of "
            ":class:`qontinui.discovery.click_analysis.models.DetectionStrategy`."
        ),
    )
    target_element_types: list[ElementTypeLiteral] | None = Field(
        default=None,
        description=(
            "Optional hint to the tuner about which element types appear "
            "in the supplied screenshots. Advisory; the tuner does not "
            "currently filter by type but the field is reserved for "
            "future extension. Values are the lower_snake_case ``value`` "
            "of :class:`qontinui.discovery.click_analysis.models.ElementType`."
        ),
    )


class TuneProfileResponse(BaseModel):
    """Success envelope returned by the runner-side handler.

    ``tuning_result`` carries the JSON-dict shape of
    :class:`qontinui.discovery.click_analysis.application_profile.TuningResult`
    (i.e. the output of ``TuningResult.to_dict()``); the web handler
    re-validates it against ``qontinui_schemas.template_capture.TuningResult``
    before returning the HTTP response. A ``success=False`` result is a
    valid non-error outcome (e.g. zero screenshots provided); runner-side
    exceptions surface via :class:`TuneProfileError`.
    """

    type: Literal["command_response"] = "command_response"
    command: Literal["click_analysis.tune_profile"] = "click_analysis.tune_profile"
    request_id: UUID
    tuning_result: dict[str, Any] = Field(
        default_factory=dict,
        description=(
            "Serialised ``TuningResult`` payload. Keys: ``config`` "
            "(``InferenceConfig.to_dict()`` shape), ``strategy_rankings`` "
            "(list of ``(strategy_value, score)`` tuples), ``metrics`` "
            "(``TuningMetrics.to_dict()`` shape), ``success`` (bool), "
            "``error_message`` (str | None)."
        ),
    )


class TuneProfileError(BaseModel):
    """Error envelope returned when the runner-side handler raises."""

    type: Literal["command_response"] = "command_response"
    command: Literal["click_analysis.tune_profile"] = "click_analysis.tune_profile"
    request_id: UUID
    error: Literal[
        "qontinui_exception",
        "invalid_payload",
        "internal_error",
        "screenshot_download_failed",
    ]
    message: str
    traceback: str | None = None


__all__ = [
    "TUNE_PROFILE_COMMAND",
    "DetectionStrategyLiteral",
    "ElementTypeLiteral",
    "TuneProfileRequest",
    "TuneProfileResponse",
    "TuneProfileError",
]
