/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { AccessibilityBounds } from './AccessibilityBounds';
import type { AccessibilityRole } from './AccessibilityRole';
import type { AccessibilityState } from './AccessibilityState';

/**
 * A node in the accessibility tree.
 *
 * Each node represents an element in the accessibility hierarchy with its
 * role, name, value, state, and bounds. The `ref` field provides a stable
 * identifier for AI-driven automation (e.g., `@e1`, `@e2`).
 */
export interface AccessibilityNode {
  /**
   * Automation ID / test-ID attribute.
   */
  automationId?: string | null;
  /**
   * Bounding rectangle in screen coordinates.
   */
  bounds?: AccessibilityBounds | null;
  /**
   * Child nodes in the tree.
   */
  children: AccessibilityNode[];
  /**
   * CSS class name or control class.
   */
  className?: string | null;
  /**
   * Accessible description (additional context).
   */
  description?: string | null;
  /**
   * HTML tag name (for web elements).
   */
  htmlTag?: string | null;
  /**
   * Whether the element accepts user interaction.
   */
  isInteractive: boolean;
  /**
   * Hierarchical level (for headings, tree items).
   */
  level?: number | null;
  /**
   * Accessible name (label).
   */
  name?: string | null;
  /**
   * Reference ID like `@e1`, `@e2` for AI interaction.
   */
  ref: string;
  role: AccessibilityRole;
  /**
   * Current state flags.
   */
  state?: AccessibilityState & {};
  /**
   * URL for link elements.
   */
  url?: string | null;
  /**
   * Current value (for inputs).
   */
  value?: string | null;
  [k: string]: unknown;
}
