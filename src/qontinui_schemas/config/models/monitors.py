"""
Monitor definitions for multi-monitor support.

This module provides standardized monitor types used across the Qontinui
ecosystem (library, runner, web) for consistent multi-monitor handling.

Monitor Indexing
----------------
Monitors have two different identification schemes that can cause confusion:

1. **OS Index** (`index` field): The index assigned by the operating system
   during hardware enumeration. This order is arbitrary and may not match
   physical layout. On Windows, this is the display adapter enumeration order.

2. **Spatial Position** (`position` field): A human-friendly identifier based
   on physical monitor arrangement (left, center, right). This is what users
   see in the UI and is determined by sorting monitors by X coordinate.

Always use `index` for programmatic operations and `position` for display.

Virtual Desktop
---------------
The virtual desktop is the combined coordinate space of all monitors:
- Origin: (min_x, min_y) across all monitors
- Size: Bounding box containing all monitors
- Monitors may have gaps between them
- Monitors may have different resolutions and DPI scales

Example Layout
--------------
Physical arrangement:
    [Monitor 1 (left)]  [Monitor 0 (center)]  [Monitor 2 (right)]
          1920x1080           1920x1080            1920x1080

Coordinates:
    Monitor 1: x=-1920, y=0  (index=1, position="left")
    Monitor 0: x=0, y=0      (index=0, position="center", primary)
    Monitor 2: x=1920, y=0   (index=2, position="right")

Virtual desktop: minX=-1920, minY=0, width=5760, height=1080
"""

from typing import List, Literal, Optional

from pydantic import BaseModel, Field, computed_field

from .geometry import CoordinateSystem, Region


class Monitor(BaseModel):
    """
    Standardized monitor information.

    Represents a physical display with its position in the virtual desktop
    and metadata for UI display.

    Examples
    --------
    Primary monitor at origin:
        >>> Monitor(
        ...     index=0, x=0, y=0, width=1920, height=1080,
        ...     position="center", is_primary=True
        ... )

    Left monitor with negative coordinates:
        >>> Monitor(
        ...     index=1, x=-1920, y=0, width=1920, height=1080,
        ...     position="left", is_primary=False
        ... )

    High-DPI monitor:
        >>> Monitor(
        ...     index=0, x=0, y=0, width=3840, height=2160,
        ...     position="center", is_primary=True, scale_factor=2.0
        ... )
    """

    index: int = Field(
        description="OS-assigned monitor index (hardware enumeration order)"
    )
    x: int = Field(
        description="X position in absolute screen coordinates (can be negative)"
    )
    y: int = Field(
        description="Y position in absolute screen coordinates (can be negative)"
    )
    width: int = Field(description="Monitor width in pixels", gt=0)
    height: int = Field(description="Monitor height in pixels", gt=0)
    position: Literal["left", "center", "right"] = Field(
        description="Spatial position based on X coordinate (for UI display)"
    )
    is_primary: bool = Field(
        default=False, description="Whether this is the primary/main monitor"
    )
    scale_factor: float = Field(
        default=1.0,
        description="DPI scale factor (1.0 = 100%, 1.5 = 150%, 2.0 = 200%)",
        gt=0,
    )
    name: Optional[str] = Field(
        default=None, description="Display name (e.g., 'DELL U2720Q')"
    )

    @computed_field
    @property
    def right(self) -> int:
        """X coordinate of the right edge."""
        return self.x + self.width

    @computed_field
    @property
    def bottom(self) -> int:
        """Y coordinate of the bottom edge."""
        return self.y + self.height

    def contains_point(self, screen_x: int, screen_y: int) -> bool:
        """Check if a screen coordinate point is within this monitor."""
        return self.x <= screen_x < self.right and self.y <= screen_y < self.bottom

    def to_region(self) -> Region:
        """Convert monitor bounds to a Region."""
        return Region(
            x=self.x,
            y=self.y,
            width=self.width,
            height=self.height,
            system=CoordinateSystem.SCREEN,
        )


class VirtualDesktop(BaseModel):
    """
    The combined coordinate space of all monitors.

    Provides utilities for working with multi-monitor coordinate systems
    and converting between them.

    Examples
    --------
    Create from monitors:
        >>> monitors = [
        ...     Monitor(index=0, x=0, y=0, width=1920, height=1080, position="center"),
        ...     Monitor(index=1, x=-1920, y=0, width=1920, height=1080, position="left"),
        ... ]
        >>> desktop = VirtualDesktop(monitors=monitors)
        >>> desktop.min_x
        -1920
        >>> desktop.width
        3840
    """

    monitors: List[Monitor] = Field(
        description="List of all monitors in the virtual desktop"
    )

    @computed_field
    @property
    def min_x(self) -> int:
        """Minimum X coordinate across all monitors."""
        if not self.monitors:
            return 0
        return min(m.x for m in self.monitors)

    @computed_field
    @property
    def min_y(self) -> int:
        """Minimum Y coordinate across all monitors."""
        if not self.monitors:
            return 0
        return min(m.y for m in self.monitors)

    @computed_field
    @property
    def max_x(self) -> int:
        """Maximum X coordinate (right edge) across all monitors."""
        if not self.monitors:
            return 1920
        return max(m.right for m in self.monitors)

    @computed_field
    @property
    def max_y(self) -> int:
        """Maximum Y coordinate (bottom edge) across all monitors."""
        if not self.monitors:
            return 1080
        return max(m.bottom for m in self.monitors)

    @computed_field
    @property
    def width(self) -> int:
        """Total width of the virtual desktop."""
        return self.max_x - self.min_x

    @computed_field
    @property
    def height(self) -> int:
        """Total height of the virtual desktop."""
        return self.max_y - self.min_y

    def screen_to_virtual(self, screen_x: int, screen_y: int) -> tuple[int, int]:
        """
        Convert absolute screen coordinates to virtual desktop coordinates.

        Virtual coordinates have origin at (min_x, min_y), so they're always
        non-negative within the virtual desktop bounds.

        Parameters
        ----------
        screen_x : int
            Absolute screen X coordinate (can be negative)
        screen_y : int
            Absolute screen Y coordinate (can be negative)

        Returns
        -------
        tuple[int, int]
            (virtual_x, virtual_y) coordinates
        """
        return (screen_x - self.min_x, screen_y - self.min_y)

    def virtual_to_screen(self, virtual_x: int, virtual_y: int) -> tuple[int, int]:
        """
        Convert virtual desktop coordinates to absolute screen coordinates.

        Parameters
        ----------
        virtual_x : int
            Virtual desktop X coordinate (non-negative)
        virtual_y : int
            Virtual desktop Y coordinate (non-negative)

        Returns
        -------
        tuple[int, int]
            (screen_x, screen_y) absolute coordinates
        """
        return (virtual_x + self.min_x, virtual_y + self.min_y)

    def monitor_to_screen(
        self, monitor_x: int, monitor_y: int, monitor_index: int
    ) -> tuple[int, int]:
        """
        Convert monitor-relative coordinates to absolute screen coordinates.

        Parameters
        ----------
        monitor_x : int
            X coordinate relative to monitor's top-left
        monitor_y : int
            Y coordinate relative to monitor's top-left
        monitor_index : int
            Index of the monitor

        Returns
        -------
        tuple[int, int]
            (screen_x, screen_y) absolute coordinates

        Raises
        ------
        ValueError
            If monitor_index is not found
        """
        monitor = self.get_monitor_by_index(monitor_index)
        if monitor is None:
            raise ValueError(f"Monitor with index {monitor_index} not found")
        return (monitor_x + monitor.x, monitor_y + monitor.y)

    def screen_to_monitor(
        self, screen_x: int, screen_y: int, monitor_index: int
    ) -> tuple[int, int]:
        """
        Convert absolute screen coordinates to monitor-relative coordinates.

        Parameters
        ----------
        screen_x : int
            Absolute screen X coordinate
        screen_y : int
            Absolute screen Y coordinate
        monitor_index : int
            Index of the target monitor

        Returns
        -------
        tuple[int, int]
            (monitor_x, monitor_y) coordinates relative to monitor's top-left

        Raises
        ------
        ValueError
            If monitor_index is not found
        """
        monitor = self.get_monitor_by_index(monitor_index)
        if monitor is None:
            raise ValueError(f"Monitor with index {monitor_index} not found")
        return (screen_x - monitor.x, screen_y - monitor.y)

    def get_monitor_by_index(self, index: int) -> Optional[Monitor]:
        """Get a monitor by its OS-assigned index."""
        for monitor in self.monitors:
            if monitor.index == index:
                return monitor
        return None

    def get_monitor_at_point(self, screen_x: int, screen_y: int) -> Optional[Monitor]:
        """Get the monitor containing a screen coordinate point."""
        for monitor in self.monitors:
            if monitor.contains_point(screen_x, screen_y):
                return monitor
        return None

    def get_primary_monitor(self) -> Optional[Monitor]:
        """Get the primary monitor."""
        for monitor in self.monitors:
            if monitor.is_primary:
                return monitor
        return None

    def get_monitors_sorted_by_position(self) -> List[Monitor]:
        """Get monitors sorted by X coordinate (left to right)."""
        return sorted(self.monitors, key=lambda m: m.x)
