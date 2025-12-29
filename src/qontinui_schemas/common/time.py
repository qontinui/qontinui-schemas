"""
Centralized time handling for Qontinui.

This module provides standardized time utilities to prevent timezone-related bugs
across all Qontinui services. All timestamps should be stored and transmitted in UTC.

Usage:
    from qontinui_schemas.common import UTCDateTime, utc_now, to_utc

    class MyModel(BaseModel):
        created_at: UTCDateTime = Field(default_factory=utc_now)
        updated_at: UTCDateTime | None = None

Key Principles:
    1. All timestamps are stored in UTC
    2. All timestamps are timezone-aware (never naive)
    3. Display conversion to local time is done only at the UI layer
    4. ISO 8601 format with 'Z' suffix for serialization
"""

from datetime import datetime, timezone
from typing import Annotated, Any

from pydantic import AfterValidator, PlainSerializer, WithJsonSchema

# =============================================================================
# Constants
# =============================================================================

UTC = timezone.utc
"""UTC timezone constant."""

ISO_FORMAT = "%Y-%m-%dT%H:%M:%S.%fZ"
"""Standard ISO 8601 format with Z suffix for UTC."""

ISO_FORMAT_NO_MICRO = "%Y-%m-%dT%H:%M:%SZ"
"""ISO 8601 format without microseconds."""


# =============================================================================
# Utility Functions
# =============================================================================


def utc_now() -> datetime:
    """
    Get current time in UTC with timezone info.

    Returns:
        datetime: Current UTC time, timezone-aware

    Example:
        >>> now = utc_now()
        >>> now.tzinfo == timezone.utc
        True
    """
    return datetime.now(UTC)


def to_utc(dt: datetime) -> datetime:
    """
    Convert a datetime to UTC.

    Args:
        dt: A datetime object (naive or aware)

    Returns:
        datetime: UTC datetime, timezone-aware

    Note:
        - If naive, assumes it's already UTC and adds tzinfo
        - If aware, converts to UTC

    Example:
        >>> from datetime import datetime, timezone
        >>> naive = datetime(2024, 1, 1, 12, 0, 0)
        >>> aware = to_utc(naive)
        >>> aware.tzinfo == timezone.utc
        True
    """
    if dt.tzinfo is None:
        # Naive datetime - assume it's UTC
        return dt.replace(tzinfo=UTC)
    else:
        # Aware datetime - convert to UTC
        return dt.astimezone(UTC)


def ensure_utc(dt: datetime) -> datetime:
    """
    Ensure a datetime is in UTC. Raises if conversion would be ambiguous.

    Args:
        dt: A datetime object

    Returns:
        datetime: UTC datetime, timezone-aware

    Raises:
        ValueError: If datetime is naive (ambiguous timezone)

    Example:
        >>> from datetime import datetime, timezone
        >>> aware = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        >>> ensure_utc(aware)
        datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
    """
    if dt.tzinfo is None:
        raise ValueError(
            f"Naive datetime '{dt}' cannot be safely converted to UTC. "
            "Use to_utc() if you want to assume UTC, or provide a timezone-aware datetime."
        )
    return dt.astimezone(UTC)


def from_iso(iso_string: str) -> datetime:
    """
    Parse an ISO 8601 string to UTC datetime.

    Args:
        iso_string: ISO 8601 formatted string

    Returns:
        datetime: UTC datetime, timezone-aware

    Handles:
        - 'Z' suffix (UTC)
        - '+00:00' offset (UTC)
        - Other timezone offsets (converted to UTC)
        - Naive strings (assumed UTC)

    Example:
        >>> dt = from_iso("2024-01-01T12:00:00Z")
        >>> dt.tzinfo == timezone.utc
        True
    """
    # Handle 'Z' suffix
    if iso_string.endswith("Z"):
        iso_string = iso_string[:-1] + "+00:00"

    try:
        dt = datetime.fromisoformat(iso_string)
    except ValueError:
        # Try parsing without microseconds
        try:
            dt = datetime.strptime(
                iso_string.replace("+00:00", ""), "%Y-%m-%dT%H:%M:%S"
            )
            dt = dt.replace(tzinfo=UTC)
        except ValueError as e:
            raise ValueError(f"Cannot parse ISO string: {iso_string}") from e

    return to_utc(dt)


def to_iso(dt: datetime) -> str:
    """
    Convert datetime to ISO 8601 string in UTC with 'Z' suffix.

    Args:
        dt: A datetime object (naive or aware)

    Returns:
        str: ISO 8601 formatted string with 'Z' suffix

    Example:
        >>> from datetime import datetime, timezone
        >>> dt = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        >>> to_iso(dt)
        '2024-01-01T12:00:00.000000Z'
    """
    utc_dt = to_utc(dt)
    return utc_dt.strftime(ISO_FORMAT)


def to_iso_compact(dt: datetime) -> str:
    """
    Convert datetime to compact ISO 8601 string (no microseconds).

    Args:
        dt: A datetime object (naive or aware)

    Returns:
        str: Compact ISO 8601 formatted string with 'Z' suffix

    Example:
        >>> from datetime import datetime, timezone
        >>> dt = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        >>> to_iso_compact(dt)
        '2024-01-01T12:00:00Z'
    """
    utc_dt = to_utc(dt)
    return utc_dt.strftime(ISO_FORMAT_NO_MICRO)


# =============================================================================
# Pydantic Validators and Serializers
# =============================================================================


def _validate_utc_datetime(v: Any) -> datetime:
    """Pydantic validator for UTCDateTime field."""
    if isinstance(v, datetime):
        return to_utc(v)
    if isinstance(v, str):
        return from_iso(v)
    if isinstance(v, (int, float)):
        # Unix timestamp
        return datetime.fromtimestamp(v, tz=UTC)
    raise ValueError(f"Cannot convert {type(v).__name__} to datetime")


def _serialize_utc_datetime(dt: datetime) -> str:
    """Pydantic serializer for UTCDateTime field."""
    return to_iso(dt)


# =============================================================================
# Annotated Type for Pydantic Models
# =============================================================================

UTCDateTime = Annotated[
    datetime,
    AfterValidator(_validate_utc_datetime),
    PlainSerializer(_serialize_utc_datetime, return_type=str),
    WithJsonSchema({"type": "string", "format": "date-time"}),
]
"""
Annotated datetime type that ensures UTC timezone.

Use this type in Pydantic models for all timestamp fields:

    class MyModel(BaseModel):
        created_at: UTCDateTime
        updated_at: UTCDateTime | None = None

Features:
    - Automatically converts naive datetimes to UTC
    - Converts timezone-aware datetimes to UTC
    - Parses ISO 8601 strings
    - Parses Unix timestamps
    - Serializes to ISO 8601 with 'Z' suffix
"""


# =============================================================================
# Duration Utilities
# =============================================================================


def duration_seconds(start: datetime, end: datetime) -> int:
    """
    Calculate duration in seconds between two datetimes.

    Both datetimes are converted to UTC before calculation.

    Args:
        start: Start datetime
        end: End datetime

    Returns:
        int: Duration in seconds (always positive)

    Example:
        >>> from datetime import datetime, timezone, timedelta
        >>> start = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        >>> end = start + timedelta(hours=1)
        >>> duration_seconds(start, end)
        3600
    """
    start_utc = to_utc(start)
    end_utc = to_utc(end)
    delta = end_utc - start_utc
    return abs(int(delta.total_seconds()))


def duration_ms(start: datetime, end: datetime) -> int:
    """
    Calculate duration in milliseconds between two datetimes.

    Args:
        start: Start datetime
        end: End datetime

    Returns:
        int: Duration in milliseconds (always positive)
    """
    start_utc = to_utc(start)
    end_utc = to_utc(end)
    delta = end_utc - start_utc
    return abs(int(delta.total_seconds() * 1000))
