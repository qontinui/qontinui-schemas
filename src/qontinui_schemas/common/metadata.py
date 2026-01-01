"""Common metadata schemas used across execution tracking.

These schemas define the structure of metadata objects used by:
- qontinui-runner (sends to backend)
- qontinui-web backend (stores in database)
- qontinui-web frontend (displays to user)
"""

from typing import Any

from pydantic import BaseModel, Field


class RunnerMetadata(BaseModel):
    """Metadata about the runner environment.

    Captured at the start of each execution run to provide context
    about the execution environment.
    """

    runner_version: str = Field(..., description="Version of the Qontinui runner")
    os: str = Field(
        ..., description="Operating system (e.g., 'Windows 11', 'macOS 14')"
    )
    hostname: str = Field(..., description="Machine hostname")
    screen_resolution: str | None = Field(
        None, description="Screen resolution (e.g., '1920x1080')"
    )
    cpu_info: str | None = Field(None, description="CPU model/info")
    memory_mb: int | None = Field(None, description="Total memory in MB")
    extra: dict[str, Any] = Field(
        default_factory=dict,
        description="Additional runner-specific metadata",
    )


class WorkflowMetadata(BaseModel):
    """Metadata about the workflow being executed.

    Captured at the start of each execution run to provide context
    about what automation is being run.
    """

    workflow_id: str = Field(..., description="Unique workflow identifier")
    workflow_name: str = Field(..., description="Human-readable workflow name")
    workflow_version: str | None = Field(None, description="Workflow version string")
    total_states: int | None = Field(
        None, description="Total number of states in the workflow"
    )
    total_transitions: int | None = Field(
        None, description="Total number of transitions in the workflow"
    )
    tags: list[str] = Field(default_factory=list, description="Workflow tags")
    description: str | None = Field(None, description="Workflow description")
    initial_state_ids: list[str] = Field(
        default_factory=list,
        description="Initial active states when workflow starts",
    )


class MatchLocation(BaseModel):
    """Location of a pattern match on screen.

    Used for both action input (where to click) and output (where element was found).
    """

    x: int = Field(..., description="X coordinate (pixels)")
    y: int = Field(..., description="Y coordinate (pixels)")
    width: int | None = Field(None, description="Width of matched region")
    height: int | None = Field(None, description="Height of matched region")


class ScreenshotAnnotation(BaseModel):
    """Annotation overlay on a screenshot.

    Used to highlight regions of interest, matches, or errors.
    """

    type: str = Field(
        ..., description="Annotation type: 'box', 'circle', 'arrow', 'text'"
    )
    x: int = Field(..., description="X coordinate")
    y: int = Field(..., description="Y coordinate")
    width: int | None = Field(None, description="Width (for box/circle)")
    height: int | None = Field(None, description="Height (for box/circle)")
    label: str | None = Field(None, description="Text label for annotation")
    color: str | None = Field(None, description="Color (CSS format, e.g., '#ff0000')")


__all__ = [
    "RunnerMetadata",
    "WorkflowMetadata",
    "MatchLocation",
    "ScreenshotAnnotation",
]
