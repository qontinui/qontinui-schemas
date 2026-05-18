"""Vision WS bridge command schemas.

These models define the request/reply payloads exchanged between
qontinui-web (HTTP handler) and qontinui-runner (Python dispatcher) for
visual-regression work that historically called
``qontinui.vision.comparison`` directly from the web service tier but
now routes runner-side via the ``runner_command_ws`` relay.

Wire shape (per command):

- Web -> runner: pubsub channel ``runner:commands:{runner_id}``,
  payload = ``<Request>.model_dump(mode='json')``.
- Runner -> web: pubsub channel ``runner:responses:{runner_id}``,
  payload = ``<Response>.model_dump(mode='json')`` on success or
  ``<Error>.model_dump(mode='json')`` on failure. The web-side
  dispatcher filters by matching ``request_id``.

Two commands ship in this module (Phase 7 of plan
``plans/2026-05-17-web-runner-ws-bridge-plan-b.md``):

- ``vision.compute_perceptual_hash`` — used by baseline creation
  (``app/services/visual_testing/baseline_image_processing.py``).
- ``vision.compare_screenshots`` — used by both
  ``comparison_service.py`` (the ``IgnoreRegion.from_dict`` consumer)
  and ``comparison_engine.py`` (the ``VisualComparator`` consumer).

Image bytes flow as base64-encoded payloads (PNG/JPEG already on the
web side from S3) rather than S3 URLs because the runner-side
``python-bridge`` does not currently carry an S3 client. The web side
already downloads bytes via its ``object_storage`` adapter; passing
the bytes through Redis pub/sub keeps the runner stateless w.r.t.
object storage. For ``compare_screenshots``, the runner returns the
diff PNG bytes (base64) and the web side uploads it via its existing
``_upload_diff_image`` helper.

The ``algorithm`` enum mirrors
:class:`qontinui.vision.comparison.ComparisonAlgorithm`
(``ssim`` / ``pixel_diff`` / ``perceptual_hash``). The
``hash_algorithm`` field on ``compute_perceptual_hash`` is reserved
for forward-compat: the runtime function currently always uses
``phash`` (via the ``imagehash`` library) and ignores any other value.
"""

from typing import Any, Literal
from uuid import UUID

from pydantic import BaseModel, Field

# =============================================================================
# Command literals — shared by request + response for correlation
# =============================================================================

COMPUTE_PERCEPTUAL_HASH_COMMAND: Literal["vision.compute_perceptual_hash"] = (
    "vision.compute_perceptual_hash"
)

COMPARE_SCREENSHOTS_COMMAND: Literal["vision.compare_screenshots"] = (
    "vision.compare_screenshots"
)


# =============================================================================
# vision.compute_perceptual_hash
# =============================================================================


class ComputePerceptualHashRequest(BaseModel):
    """Request envelope for ``vision.compute_perceptual_hash``.

    Computes a perceptual hash (phash) for a single image. The hash is
    stored alongside baselines for quick pre-comparison filtering. The
    runtime function is
    :meth:`qontinui.vision.comparison.VisualComparator.compute_perceptual_hash`,
    which returns an ``imagehash.ImageHash`` instance; we round-trip
    its hex repr (`str(hash)`).
    """

    type: Literal["vision.compute_perceptual_hash"] = "vision.compute_perceptual_hash"
    command: Literal["vision.compute_perceptual_hash"] = (
        "vision.compute_perceptual_hash"
    )
    request_id: UUID = Field(
        description="Correlation id; the response carries the same value.",
    )
    image_b64: str = Field(
        description=(
            "Base64-encoded image bytes (PNG/JPEG). The runner decodes "
            "to a numpy array and runs ``VisualComparator.compute_perceptual_hash``."
        ),
    )
    hash_algorithm: Literal["phash", "ahash", "dhash", "whash"] = Field(
        default="phash",
        description=(
            "Perceptual hash variant. Currently advisory — the runtime "
            "function always uses phash; reserved for forward-compat."
        ),
    )
    hash_size: int = Field(
        default=16,
        ge=2,
        le=64,
        description=(
            "Perceptual hash size (passed through to "
            "``VisualComparator.compute_perceptual_hash``). Larger = "
            "more precision."
        ),
    )


class ComputePerceptualHashResponse(BaseModel):
    """Success envelope returned by the runner-side handler."""

    type: Literal["command_response"] = "command_response"
    command: Literal["vision.compute_perceptual_hash"] = (
        "vision.compute_perceptual_hash"
    )
    request_id: UUID
    hash_hex: str = Field(
        description="Hex string representation of the perceptual hash.",
    )


class ComputePerceptualHashError(BaseModel):
    """Error envelope returned when the runner-side hash handler raises."""

    type: Literal["command_response"] = "command_response"
    command: Literal["vision.compute_perceptual_hash"] = (
        "vision.compute_perceptual_hash"
    )
    request_id: UUID
    error: Literal[
        "qontinui_exception",
        "invalid_payload",
        "internal_error",
        "imagehash_unavailable",
    ]
    message: str
    traceback: str | None = None


# =============================================================================
# vision.compare_screenshots
# =============================================================================


class IgnoreRegionPayload(BaseModel):
    """Wire shape for a single ignore region.

    Matches :class:`qontinui.vision.comparison.IgnoreRegion`'s
    ``from_dict`` contract: ``x`` / ``y`` / ``width`` / ``height`` are
    required, ``name`` is optional. Additional fields are tolerated
    for forward-compat (pydantic default of ``extra='ignore'``).
    """

    x: int = Field(description="Top-left x coordinate (pixels).")
    y: int = Field(description="Top-left y coordinate (pixels).")
    width: int = Field(ge=0, description="Region width (pixels).")
    height: int = Field(ge=0, description="Region height (pixels).")
    name: str | None = Field(
        default=None,
        description="Optional human-readable region name (e.g. 'timestamp').",
    )


class CompareScreenshotsRequest(BaseModel):
    """Request envelope for ``vision.compare_screenshots``.

    Runs :meth:`qontinui.vision.comparison.VisualComparator.compare`
    over two images. Image bytes flow as base64 (PNG/JPEG); ignore
    regions are converted on the runner via
    :meth:`IgnoreRegion.from_dict`. When ``generate_diff_image`` is
    True AND the comparison fails (``passed=False``), the runner also
    generates a diff PNG (base64-encoded) and returns it in the
    response so the web side can upload it via its existing
    ``_upload_diff_image`` helper.

    The ``algorithm`` literal mirrors
    :class:`qontinui.vision.comparison.ComparisonAlgorithm` values
    (``ssim`` / ``pixel_diff`` / ``perceptual_hash``).
    """

    type: Literal["vision.compare_screenshots"] = "vision.compare_screenshots"
    command: Literal["vision.compare_screenshots"] = "vision.compare_screenshots"
    request_id: UUID = Field(
        description="Correlation id; the response carries the same value.",
    )
    baseline_b64: str = Field(
        description=(
            "Base64-encoded baseline image bytes (PNG/JPEG). Decoded to "
            "an RGB numpy array runner-side."
        ),
    )
    current_b64: str = Field(
        description=(
            "Base64-encoded current (screenshot) image bytes (PNG/JPEG). "
            "Decoded to an RGB numpy array runner-side."
        ),
    )
    algorithm: Literal["ssim", "pixel_diff", "perceptual_hash"] = Field(
        default="ssim",
        description=(
            "Comparison algorithm; maps to "
            "``qontinui.vision.comparison.ComparisonAlgorithm``."
        ),
    )
    threshold: float | None = Field(
        default=None,
        description=(
            "Similarity threshold. ``None`` uses the algorithm default "
            "(0.95 for ssim, 0.99 for pixel_diff, 0.90 for perceptual_hash)."
        ),
    )
    ignore_regions: list[IgnoreRegionPayload] = Field(
        default_factory=list,
        description=(
            "Regions to mask out before comparison. Converted to "
            "``IgnoreRegion`` instances on the runner via ``from_dict``."
        ),
    )
    generate_diff_image: bool = Field(
        default=True,
        description=(
            "When True (default) and the comparison fails, the runner "
            "additionally generates a diff PNG and returns it as "
            "``diff_image_b64`` in the response."
        ),
    )


class CompareScreenshotsResult(BaseModel):
    """Inner comparison result payload.

    Mirrors :meth:`qontinui.vision.comparison.ComparisonResult.to_dict`
    output so the web side can pass it through with minimal
    transformation. Notably ``diff_mask`` is NOT serialised (it is a
    numpy array; kept runner-side and consumed there to render the
    diff PNG).
    """

    similarity_score: float = Field(
        description="0.0 to 1.0, where 1.0 is identical.",
    )
    passed: bool = Field(
        description="True if similarity_score >= threshold.",
    )
    threshold: float = Field(
        description="The threshold used for comparison.",
    )
    algorithm: Literal["ssim", "pixel_diff", "perceptual_hash"] = Field(
        description="Algorithm used (echoes the request value).",
    )
    execution_time_ms: int = Field(
        ge=0,
        description="Wall-clock comparison time in milliseconds.",
    )
    diff_regions: list[dict[str, Any]] = Field(
        default_factory=list,
        description=(
            "Bounding-box regions where differences were detected; "
            "``DiffRegion.to_dict()`` shape."
        ),
    )
    error: str | None = Field(
        default=None,
        description=(
            "Algorithm-internal error message (e.g. 'scikit-image not "
            "installed'). Note: this is the qontinui-side soft error; "
            "runner dispatcher failures surface via "
            ":class:`CompareScreenshotsError` instead."
        ),
    )


class CompareScreenshotsResponse(BaseModel):
    """Success envelope returned by the runner-side handler."""

    type: Literal["command_response"] = "command_response"
    command: Literal["vision.compare_screenshots"] = "vision.compare_screenshots"
    request_id: UUID
    result: CompareScreenshotsResult = Field(
        description="The comparison result (similarity, regions, etc.).",
    )
    diff_image_b64: str | None = Field(
        default=None,
        description=(
            "Base64-encoded diff PNG bytes. Populated when "
            "``generate_diff_image=True`` AND the comparison failed AND "
            "a diff mask was producible. ``None`` otherwise."
        ),
    )


class CompareScreenshotsError(BaseModel):
    """Error envelope returned when the runner-side compare handler raises."""

    type: Literal["command_response"] = "command_response"
    command: Literal["vision.compare_screenshots"] = "vision.compare_screenshots"
    request_id: UUID
    error: Literal[
        "qontinui_exception",
        "invalid_payload",
        "internal_error",
    ]
    message: str
    traceback: str | None = None


__all__ = [
    "COMPUTE_PERCEPTUAL_HASH_COMMAND",
    "COMPARE_SCREENSHOTS_COMMAND",
    "ComputePerceptualHashRequest",
    "ComputePerceptualHashResponse",
    "ComputePerceptualHashError",
    "IgnoreRegionPayload",
    "CompareScreenshotsRequest",
    "CompareScreenshotsResult",
    "CompareScreenshotsResponse",
    "CompareScreenshotsError",
]
