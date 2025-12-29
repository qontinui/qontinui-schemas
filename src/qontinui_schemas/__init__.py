"""Qontinui Schemas - Shared Pydantic models for Qontinui ecosystem.

This package provides schema definitions used across qontinui-web, qontinui-api,
and other Qontinui services. It has minimal dependencies (pydantic only) to avoid
pulling in heavy ML libraries.
"""

__version__ = "0.1.0"

# Re-export all schemas for convenience
from qontinui_schemas.api import *  # noqa: F401, F403

# Re-export common utilities
from qontinui_schemas.common import (  # noqa: F401
    UTCDateTime,
    ensure_utc,
    from_iso,
    to_iso,
    to_utc,
    utc_now,
)
from qontinui_schemas.config.models import *  # noqa: F401, F403
from qontinui_schemas.config.property_groups import *  # noqa: F401, F403
from qontinui_schemas.events import *  # noqa: F401, F403
from qontinui_schemas.rag.models import *  # noqa: F401, F403

__all__ = [
    # Version
    "__version__",
    # Time utilities
    "UTCDateTime",
    "utc_now",
    "to_utc",
    "from_iso",
    "to_iso",
    "ensure_utc",
]
