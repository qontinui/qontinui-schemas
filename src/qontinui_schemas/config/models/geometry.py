"""
Geometric primitives for screen coordinates and regions.

This module provides basic geometric types used for positioning and
area definition in screen-based automation.

Coordinate Systems
------------------
Multi-monitor setups require careful handling of coordinate systems.
This module defines three distinct coordinate systems:

1. SCREEN (absolute): Raw screen coordinates as used by the OS.
   - Can be negative (monitors left of or above the primary monitor)
   - Used by pyautogui, mouse clicks, and OS-level operations
   - Example: (-1920, 0) for top-left of a left monitor

2. VIRTUAL (virtual desktop): Relative to the virtual desktop origin.
   - Origin is at (min_x, min_y) of all monitors combined
   - Always non-negative within the virtual desktop bounds
   - Used by screenshot pixel coordinates and FIND results
   - Example: (0, 0) is always top-left of the combined monitor space

3. MONITOR_RELATIVE: Relative to a specific monitor's top-left corner.
   - Always non-negative within that monitor
   - Requires specifying which monitor (by index)
   - Used by searchRegion in state configurations
   - Example: (100, 100) on monitor 1

Conversion Example
------------------
Given monitors: [left at x=-1920, primary at x=0]
- Screen point (-1920, 0) = Virtual point (0, 0) = Monitor 0 point (0, 0)
- Screen point (0, 0) = Virtual point (1920, 0) = Monitor 1 point (0, 0)
"""

from enum import Enum
from typing import Optional

from pydantic import BaseModel, Field, model_validator


class CoordinateSystem(str, Enum):
    """
    Coordinate system identifier.

    Always specify which coordinate system you're working with to avoid
    confusion in multi-monitor setups.
    """

    SCREEN = "screen"
    """
    Absolute screen coordinates (OS-level).
    Can be negative for monitors left of or above the primary monitor.
    Used by: pyautogui, mouse operations, window positioning.
    """

    VIRTUAL = "virtual"
    """
    Relative to virtual desktop origin (min_x, min_y of all monitors).
    Always non-negative within the virtual desktop bounds.
    Used by: screenshot pixel lookups, FIND match results.
    """

    MONITOR_RELATIVE = "monitor_relative"
    """
    Relative to a specific monitor's top-left corner (0, 0).
    Requires specifying monitor_index.
    Used by: searchRegion in state configurations.
    """


class Coordinates(BaseModel):
    """
    X,Y coordinates with optional coordinate system specification.

    For backward compatibility, coordinates without a system specified
    are assumed to be SCREEN coordinates.

    Examples
    --------
    Simple screen coordinate:
        >>> Coordinates(x=100, y=200)

    Explicit screen coordinate:
        >>> Coordinates(x=-1920, y=0, system=CoordinateSystem.SCREEN)

    Virtual desktop coordinate:
        >>> Coordinates(x=0, y=0, system=CoordinateSystem.VIRTUAL)

    Monitor-relative coordinate:
        >>> Coordinates(x=100, y=100, system=CoordinateSystem.MONITOR_RELATIVE, monitor_index=1)
    """

    x: int = Field(description="X coordinate (horizontal position)")
    y: int = Field(description="Y coordinate (vertical position)")
    system: Optional[CoordinateSystem] = Field(
        default=None,
        description="Coordinate system. None defaults to SCREEN for backward compatibility.",
    )
    monitor_index: Optional[int] = Field(
        default=None,
        description="Monitor index (required when system is MONITOR_RELATIVE)",
    )

    @model_validator(mode="after")
    def validate_monitor_index(self) -> "Coordinates":
        """Ensure monitor_index is provided for MONITOR_RELATIVE coordinates."""
        if (
            self.system == CoordinateSystem.MONITOR_RELATIVE
            and self.monitor_index is None
        ):
            raise ValueError(
                "monitor_index is required when system is MONITOR_RELATIVE"
            )
        return self

    @property
    def effective_system(self) -> CoordinateSystem:
        """Get the effective coordinate system (defaults to SCREEN)."""
        return self.system or CoordinateSystem.SCREEN


class Region(BaseModel):
    """
    Rectangular region on screen.

    Defines a rectangular area with position and dimensions.
    Like Coordinates, can optionally specify the coordinate system.

    Examples
    --------
    Simple region:
        >>> Region(x=100, y=100, width=200, height=150)

    Monitor-relative region:
        >>> Region(
        ...     x=0, y=0, width=1920, height=1080,
        ...     system=CoordinateSystem.MONITOR_RELATIVE,
        ...     monitor_index=0
        ... )
    """

    x: int = Field(description="X coordinate of top-left corner")
    y: int = Field(description="Y coordinate of top-left corner")
    width: int = Field(description="Width of the region", gt=0)
    height: int = Field(description="Height of the region", gt=0)
    system: Optional[CoordinateSystem] = Field(
        default=None,
        description="Coordinate system. None defaults to SCREEN for backward compatibility.",
    )
    monitor_index: Optional[int] = Field(
        default=None,
        description="Monitor index (required when system is MONITOR_RELATIVE)",
    )

    @model_validator(mode="after")
    def validate_monitor_index(self) -> "Region":
        """Ensure monitor_index is provided for MONITOR_RELATIVE regions."""
        if (
            self.system == CoordinateSystem.MONITOR_RELATIVE
            and self.monitor_index is None
        ):
            raise ValueError(
                "monitor_index is required when system is MONITOR_RELATIVE"
            )
        return self

    @property
    def effective_system(self) -> CoordinateSystem:
        """Get the effective coordinate system (defaults to SCREEN)."""
        return self.system or CoordinateSystem.SCREEN

    @property
    def right(self) -> int:
        """X coordinate of the right edge."""
        return self.x + self.width

    @property
    def bottom(self) -> int:
        """Y coordinate of the bottom edge."""
        return self.y + self.height

    def contains_point(self, x: int, y: int) -> bool:
        """Check if a point is within this region."""
        return self.x <= x < self.right and self.y <= y < self.bottom

    def overlaps(self, other: "Region") -> bool:
        """Check if this region overlaps with another region."""
        return not (
            self.right <= other.x
            or other.right <= self.x
            or self.bottom <= other.y
            or other.bottom <= self.y
        )
