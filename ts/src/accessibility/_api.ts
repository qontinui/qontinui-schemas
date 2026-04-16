/**
 * Accessibility Tree Types
 *
 * Accessibility-tree snapshots captured from CDP, UIA, AT-SPI, or AX.
 * Source of truth: qontinui-schemas/rust/src/accessibility.rs.
 *
 * The `ref` system (`@e1`, `@e2`, …) provides stable, AI-friendly
 * identifiers for interacting with elements without re-querying the tree.
 */

export type { AccessibilityRole } from "../generated/AccessibilityRole";
export type { AccessibilityBackend } from "../generated/AccessibilityBackend";
export type { AccessibilityState } from "../generated/AccessibilityState";
export type { AccessibilityBounds } from "../generated/AccessibilityBounds";
export type { AccessibilityNode } from "../generated/AccessibilityNode";
export type { AccessibilitySnapshot } from "../generated/AccessibilitySnapshot";
export type { AccessibilitySelector } from "../generated/AccessibilitySelector";
