"""Recording-pipeline WS bridge command + event schemas.

The recording pipeline runs the multi-stage conversion from a
``CooccurrenceExport`` (produced by the UI Bridge SDK's
``RecordingSessionManager.stop()``) into a discovered state machine
with states + transitions + (optionally) a generated playbook. The
work runs **minutes** for non-trivial recordings, so this surface is
**async-with-progress**: the HTTP handler dispatches the command and
returns 202 immediately with a ``run_id``; the runner publishes a
terminal :class:`ProcessRecordingResult` event (and optionally
intermediate :class:`ProcessRecordingProgress` events) on
``runner:responses:{runner_id}``; the web side persists the result to
its ``recording_pipeline_runs`` PG table.

Wire shape (per command):

- Web -> runner: pubsub channel ``runner:commands:{runner_id}``,
  payload = ``ProcessRecordingRequest.model_dump(mode='json')`` (or the
  ``..WithPlaybook`` / ``..Merge`` siblings).
- Runner -> web: pubsub channel ``runner:responses:{runner_id}``,
  delivered as two distinct frame types:
  - ``recording_pipeline_progress``: intermediate progress events
    keyed by ``run_id`` (zero or more per run).
  - ``recording_pipeline_result``: terminal completion event keyed by
    ``run_id`` (exactly one per run, ``status="completed"`` on success
    or ``status="failed"`` on runner-side failure).
- An optional synchronous ``DispatchAck`` reply (matching ``request_id``,
  not ``run_id``) confirms the runner accepted the run. The runner-side
  handler should send this within seconds of receiving the command,
  before the long-running pipeline begins, so the HTTP handler's
  initial ``dispatch_and_wait`` does not time out waiting for the
  whole pipeline to complete.

Phase 4 of plan
``plans/2026-05-17-web-runner-ws-bridge-plan-b.md`` introduces this
module plus three commands (``recording_pipeline.process``,
``recording_pipeline.process_with_playbook``,
``recording_pipeline.merge``) and two event types
(``recording_pipeline_progress``, ``recording_pipeline_result``).
"""

from typing import Any, Literal
from uuid import UUID

from pydantic import BaseModel, Field

# =============================================================================
# Command literals
# =============================================================================

PROCESS_RECORDING_COMMAND: Literal["recording_pipeline.process"] = (
    "recording_pipeline.process"
)
PROCESS_RECORDING_WITH_PLAYBOOK_COMMAND: Literal[
    "recording_pipeline.process_with_playbook"
] = "recording_pipeline.process_with_playbook"
MERGE_RECORDING_COMMAND: Literal["recording_pipeline.merge"] = (
    "recording_pipeline.merge"
)

PROGRESS_EVENT_TYPE: Literal["recording_pipeline_progress"] = (
    "recording_pipeline_progress"
)
RESULT_EVENT_TYPE: Literal["recording_pipeline_result"] = "recording_pipeline_result"


# =============================================================================
# Request envelopes
# =============================================================================


class ProcessRecordingRequest(BaseModel):
    """Request envelope for ``recording_pipeline.process``.

    The ``request_id`` correlates the synchronous DispatchAck reply (so
    the HTTP handler's ``dispatch_and_wait`` returns promptly); the
    ``run_id`` correlates the eventual progress + result events (the
    long-lived run identifier persisted in the web-side
    ``recording_pipeline_runs`` row).
    """

    type: Literal["recording_pipeline.process"] = "recording_pipeline.process"
    command: Literal["recording_pipeline.process"] = "recording_pipeline.process"
    request_id: UUID = Field(
        description="Correlation id for the synchronous DispatchAck reply.",
    )
    run_id: UUID = Field(
        description=(
            "Long-lived run identifier; carried in every progress + result "
            "event so the web side can match them to the persisted row."
        ),
    )
    project_id: UUID | None = Field(
        default=None,
        description="Project UUID for downstream persistence (web-side).",
    )
    config_name: str | None = Field(
        default=None,
        description=(
            "Name for the resulting state config; defaults to "
            "``recording-{session_id}`` if omitted (web-side decides)."
        ),
    )
    recording_export: dict[str, Any] = Field(
        description=(
            "CooccurrenceExport JSON from the SDK's "
            "``RecordingSessionManager.stop()``."
        ),
    )
    config: dict[str, Any] = Field(
        default_factory=dict,
        description=(
            "Optional ``RecordingPipelineConfig`` overrides. Recognised "
            "keys mirror the dataclass fields (``min_confidence``, "
            "``treat_header_footer_as_global``, "
            "``dedupe_repeating_elements``, ``use_size_weighting``, "
            "``auto_detect_modal_states``). ``persist`` is forced false "
            "runner-side; persistence happens web-side."
        ),
    )


class ProcessRecordingWithPlaybookRequest(BaseModel):
    """Request envelope for ``recording_pipeline.process_with_playbook``."""

    type: Literal[
        "recording_pipeline.process_with_playbook"
    ] = "recording_pipeline.process_with_playbook"
    command: Literal[
        "recording_pipeline.process_with_playbook"
    ] = "recording_pipeline.process_with_playbook"
    request_id: UUID
    run_id: UUID
    project_id: UUID | None = None
    config_name: str | None = None
    recording_export: dict[str, Any]
    config: dict[str, Any] = Field(default_factory=dict)
    variables: list[dict[str, Any]] = Field(
        default_factory=list,
        description="VariableCandidate dicts extracted from the session.",
    )
    app_name: str | None = Field(
        default=None,
        description="Application name (used in playbook frontmatter).",
    )
    app_url: str | None = Field(
        default=None,
        description="Application URL (used in playbook triggers).",
    )
    save_experience: bool = Field(
        default=True,
        description=(
            "Whether to save this session as experience memory "
            "(web-side post-result handling)."
        ),
    )
    playbook_template: dict[str, Any] | None = Field(
        default=None,
        description=(
            "Optional playbook-template overrides; reserved for future "
            "expansion (currently ignored by the runner-side generator)."
        ),
    )


class MergeRecordingRequest(BaseModel):
    """Request envelope for ``recording_pipeline.merge``.

    Merges a new recording into an existing state config. The web side
    supplies the two prior runs via ``run_ids`` (one is the
    already-persisted target config, the other is the new recording's
    ``run_id`` after a successful ``.process`` run) OR directly via
    ``existing_states`` + ``existing_transitions`` + ``recording_export``
    for the inline-merge path.
    """

    type: Literal["recording_pipeline.merge"] = "recording_pipeline.merge"
    command: Literal["recording_pipeline.merge"] = "recording_pipeline.merge"
    request_id: UUID
    run_id: UUID
    project_id: UUID | None = None
    config_id: UUID = Field(
        description="Target ``UIBridgeStateConfig`` UUID to merge into.",
    )
    recording_export: dict[str, Any] = Field(
        description="New recording's CooccurrenceExport JSON.",
    )
    existing_states: list[dict[str, Any]] = Field(
        default_factory=list,
        description=(
            "Serialised ``UIBridgeState`` payloads for the target "
            "config (web fetches these from PG and forwards them so "
            "the runner stays stateless w.r.t. persistence)."
        ),
    )
    existing_transitions: list[dict[str, Any]] = Field(
        default_factory=list,
        description=(
            "Serialised ``UIBridgeTransition`` payloads for the target "
            "config."
        ),
    )
    config: dict[str, Any] = Field(default_factory=dict)
    run_ids: list[UUID] = Field(
        default_factory=list,
        description=(
            "Optional: prior ``run_id`` values to associate with this "
            "merge for audit/logging. Not used by the merge algorithm "
            "itself."
        ),
    )


# =============================================================================
# Synchronous DispatchAck envelope (reply to dispatch_and_wait)
# =============================================================================


class DispatchAck(BaseModel):
    """Synchronous acknowledgement returned to the web-side
    ``dispatch_and_wait`` call.

    Confirms the runner received + queued the run. Progress + final
    result flow back as separate events keyed by ``run_id``.
    """

    type: Literal["command_response"] = "command_response"
    command: Literal[
        "recording_pipeline.process",
        "recording_pipeline.process_with_playbook",
        "recording_pipeline.merge",
    ]
    request_id: UUID
    run_id: UUID
    accepted: bool = Field(
        description=(
            "True if the runner accepted the run and will publish a "
            "terminal ``recording_pipeline_result`` event. False if the "
            "runner rejected it (see ``error`` for the reason)."
        ),
    )
    error: str | None = Field(
        default=None,
        description=(
            "If ``accepted`` is False, a short error literal "
            "(``invalid_payload``, ``internal_error``)."
        ),
    )
    message: str | None = Field(
        default=None,
        description="Human-readable error description, when ``accepted`` is False.",
    )


# =============================================================================
# Progress event envelope (zero or more per run)
# =============================================================================


ProgressStage = Literal[
    "initializing",
    "discovering_states",
    "building_transitions",
    "generating_playbook",
    "finalizing",
]


class ProcessRecordingProgress(BaseModel):
    """Progress event published on ``runner:responses:{runner_id}``.

    Zero or more per run; each carries a stage label, a 0..1 progress
    value within that stage, and optional diagnostic metrics.
    """

    type: Literal["recording_pipeline_progress"] = "recording_pipeline_progress"
    run_id: UUID
    stage: ProgressStage
    progress: float = Field(
        ge=0.0,
        le=1.0,
        description="Progress within the current stage, in [0..1].",
    )
    message: str | None = Field(
        default=None,
        description="Optional human-readable status message.",
    )
    partial_metrics: dict[str, float] = Field(
        default_factory=dict,
        description=(
            "Optional partial metrics (e.g. ``states_discovered``, "
            "``transitions_so_far``) emitted alongside the progress event."
        ),
    )


# =============================================================================
# Terminal result event envelope (exactly one per run)
# =============================================================================


class ProcessRecordingResult(BaseModel):
    """Terminal completion event for a recording-pipeline run.

    Exactly one per run. The web-side subscriber updates the matching
    ``recording_pipeline_runs`` row and (on success) persists the
    discovered states/transitions to the ``ui_bridge_state_configs``
    tree.

    On success, ``result`` carries a serialised ``RecordingPipelineResult``
    payload with the following keys (mirrors the runtime dataclass at
    ``qontinui/src/qontinui/state_machine/recording_pipeline.py:70``):

    - ``session_id`` (str): Recording session id from the export.
    - ``state_count`` (int): Number of discovered states.
    - ``transition_count`` (int): Number of detected transitions.
    - ``global_state_count`` (int): Subset of states flagged as global.
    - ``modal_state_count`` (int): Subset of states flagged as modal.
    - ``states`` (list[dict]): Serialised ``UIBridgeState`` payloads
      (``id``, ``name``, ``element_ids``, ``blocking``, ``blocks``,
      ``group``, ``path_cost``, ``metadata``).
    - ``transitions`` (list[dict]): Serialised ``UIBridgeTransition``
      payloads (``id``, ``name``, ``from_states``, ``activate_states``,
      ``exit_states``, ``actions``, ``activate_groups``, ``exit_groups``,
      ``path_cost``, ``stays_visible``, ``metadata``).
    - ``playbook_content`` (str, optional): Generated markdown body; only
      present for the ``..with_playbook`` command.

    On failure, ``error`` carries ``{error, message, traceback}``.
    """

    type: Literal["recording_pipeline_result"] = "recording_pipeline_result"
    run_id: UUID
    command: Literal[
        "recording_pipeline.process",
        "recording_pipeline.process_with_playbook",
        "recording_pipeline.merge",
    ]
    status: Literal["completed", "failed"]
    result: dict[str, Any] | None = Field(
        default=None,
        description=(
            "Serialised ``RecordingPipelineResult`` payload on success. "
            "See class docstring for the dict shape."
        ),
    )
    error: dict[str, Any] | None = Field(
        default=None,
        description=(
            "On failure, ``{error: str, message: str, traceback: str | None}``."
        ),
    )


# =============================================================================
# Inline-error envelopes (for the dispatch path; reuses Phase 2 patterns)
# =============================================================================


class RecordingPipelineError(BaseModel):
    """Error envelope returned by the runner-side handler.

    Surfaced when the runner cannot even start the run (payload
    validation, missing dependency, etc.). For runtime failures during
    the pipeline, the terminal :class:`ProcessRecordingResult` event
    carries ``status="failed"`` + an ``error`` dict instead.
    """

    type: Literal["command_response"] = "command_response"
    command: Literal[
        "recording_pipeline.process",
        "recording_pipeline.process_with_playbook",
        "recording_pipeline.merge",
    ]
    request_id: UUID
    run_id: UUID
    error: Literal["qontinui_exception", "invalid_payload", "internal_error"]
    message: str
    traceback: str | None = None


__all__ = [
    "PROCESS_RECORDING_COMMAND",
    "PROCESS_RECORDING_WITH_PLAYBOOK_COMMAND",
    "MERGE_RECORDING_COMMAND",
    "PROGRESS_EVENT_TYPE",
    "RESULT_EVENT_TYPE",
    "ProcessRecordingRequest",
    "ProcessRecordingWithPlaybookRequest",
    "MergeRecordingRequest",
    "DispatchAck",
    "ProgressStage",
    "ProcessRecordingProgress",
    "ProcessRecordingResult",
    "RecordingPipelineError",
]
