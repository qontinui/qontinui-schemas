/**
 * Geometry Types
 *
 * Screen coordinates, regions, monitors, and virtual-desktop layouts.
 * Source of truth: `qontinui-schemas/rust/src/geometry.rs`.
 *
 * Multi-monitor setups use three distinct coordinate systems — see
 * `CoordinateSystem` for the semantics.
 */

export type { CoordinateSystem } from "../generated/CoordinateSystem";
export type { Coordinates } from "../generated/Coordinates";
export type { Region } from "../generated/Region";
export type { MonitorPosition } from "../generated/MonitorPosition";
export type { Monitor } from "../generated/Monitor";
export type { VirtualDesktop } from "../generated/VirtualDesktop";
