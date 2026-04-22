//! Geometric primitives for screen coordinates, regions, and multi-monitor
//! layouts.
//!
//! Mirrors `src/qontinui_schemas/config/models/geometry.py` and
//! `src/qontinui_schemas/config/models/monitors.py`. Rust is the source of
//! truth; the Python and TypeScript bindings are regenerated from the JSON
//! Schemas emitted here.
//!
//! Multi-monitor setups use three distinct coordinate systems — see
//! [`CoordinateSystem`] for the semantics.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ============================================================================
// CoordinateSystem
// ============================================================================

/// Coordinate system identifier. Always specify which system you are working
/// with to avoid confusion in multi-monitor setups.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CoordinateSystem {
    /// Absolute screen coordinates (OS-level). May be negative for monitors
    /// left of or above the primary. Used by pyautogui, mouse ops, and
    /// window positioning.
    Screen,
    /// Relative to the virtual desktop origin `(min_x, min_y)`. Always
    /// non-negative within the virtual desktop bounds. Used by screenshot
    /// pixel lookups and FIND match results.
    Virtual,
    /// Relative to a specific monitor's top-left corner. Requires
    /// `monitor_index`. Used by `searchRegion` in state configurations.
    MonitorRelative,
}

// ============================================================================
// Coordinates
// ============================================================================

/// X,Y coordinates with optional coordinate system specification.
///
/// Coordinates without an explicit `system` default to
/// [`CoordinateSystem::Screen`] for backward compatibility.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct Coordinates {
    /// X coordinate (horizontal position).
    #[serde(alias = "x")]
    pub x: i64,
    /// Y coordinate (vertical position).
    #[serde(alias = "y")]
    pub y: i64,
    /// Coordinate system. `None` defaults to `Screen`.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "system")]
    pub system: Option<CoordinateSystem>,
    /// Monitor index (required when `system` is `MonitorRelative`).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "monitor_index"
    )]
    pub monitor_index: Option<u32>,
}

// ============================================================================
// Region
// ============================================================================

/// Rectangular region on screen. Like [`Coordinates`], can optionally specify
/// which coordinate system the `x`/`y` are in.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct Region {
    /// X coordinate of the top-left corner.
    #[serde(alias = "x")]
    pub x: i64,
    /// Y coordinate of the top-left corner.
    #[serde(alias = "y")]
    pub y: i64,
    /// Width of the region (must be positive).
    #[serde(alias = "width")]
    pub width: u32,
    /// Height of the region (must be positive).
    #[serde(alias = "height")]
    pub height: u32,
    /// Coordinate system. `None` defaults to `Screen`.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "system")]
    pub system: Option<CoordinateSystem>,
    /// Monitor index (required when `system` is `MonitorRelative`).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "monitor_index"
    )]
    pub monitor_index: Option<u32>,
}

// ============================================================================
// Monitor
// ============================================================================

/// Spatial position of a monitor in a left-to-right layout, derived from the
/// X coordinate. Used for human-friendly display; use `index` for
/// programmatic operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum MonitorPosition {
    Left,
    Center,
    Right,
}

/// Standardized monitor information — a physical display with its position
/// in the virtual desktop and metadata for UI display.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct Monitor {
    /// OS-assigned monitor index (hardware enumeration order).
    #[serde(alias = "index")]
    pub index: u32,
    /// X position in absolute screen coordinates (can be negative).
    #[serde(alias = "x")]
    pub x: i64,
    /// Y position in absolute screen coordinates (can be negative).
    #[serde(alias = "y")]
    pub y: i64,
    /// Monitor width in pixels.
    #[serde(alias = "width")]
    pub width: u32,
    /// Monitor height in pixels.
    #[serde(alias = "height")]
    pub height: u32,
    /// Spatial position based on X coordinate (for UI display).
    #[serde(alias = "position")]
    pub position: MonitorPosition,
    /// Whether this is the primary/main monitor.
    #[serde(default, alias = "is_primary")]
    pub is_primary: bool,
    /// DPI scale factor (1.0 = 100%, 1.5 = 150%, 2.0 = 200%).
    #[serde(default = "default_scale_factor", alias = "scale_factor")]
    pub scale_factor: f32,
    /// Display name (e.g., "DELL U2720Q").
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "name")]
    pub name: Option<String>,
}

fn default_scale_factor() -> f32 {
    1.0
}

// ============================================================================
// VirtualDesktop
// ============================================================================

/// The combined coordinate space of all monitors.
///
/// - Origin: `(min_x, min_y)` across all monitors
/// - Size: bounding box containing all monitors
/// - Monitors may have gaps between them or different resolutions/DPI
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct VirtualDesktop {
    /// List of all monitors in the virtual desktop.
    #[serde(alias = "monitors")]
    pub monitors: Vec<Monitor>,
}
