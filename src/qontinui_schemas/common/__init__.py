"""Common utilities and types for qontinui-schemas."""

from qontinui_schemas.common.metadata import (
    MatchLocation,
    RunnerMetadata,
    ScreenshotAnnotation,
    WorkflowMetadata,
)
from qontinui_schemas.common.stats import (
    CoverageData,
    ExecutionStats,
    ReliabilityStats,
    TransitionReliability,
)
from qontinui_schemas.common.time import (
    UTCDateTime,
    ensure_utc,
    from_iso,
    to_iso,
    to_utc,
    utc_now,
)

__all__ = [
    # Time utilities
    "UTCDateTime",
    "utc_now",
    "to_utc",
    "from_iso",
    "to_iso",
    "ensure_utc",
    # Metadata
    "RunnerMetadata",
    "WorkflowMetadata",
    "MatchLocation",
    "ScreenshotAnnotation",
    # Stats
    "ExecutionStats",
    "CoverageData",
    "ReliabilityStats",
    "TransitionReliability",
]
