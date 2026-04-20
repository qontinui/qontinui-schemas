/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * ARIA/UIA accessibility roles. Based on WAI-ARIA 1.2 with extensions for
 * Windows UI Automation and other platform-specific accessibility APIs.
 */
type AccessibilityRole =
  | "application"
  | "document"
  | "article"
  | "banner"
  | "complementary"
  | "contentinfo"
  | "form"
  | "main"
  | "navigation"
  | "region"
  | "search"
  | "button"
  | "checkbox"
  | "combobox"
  | "dialog"
  | "gridcell"
  | "link"
  | "listbox"
  | "menu"
  | "menubar"
  | "menuitem"
  | "menuitemcheckbox"
  | "menuitemradio"
  | "option"
  | "progressbar"
  | "radio"
  | "radiogroup"
  | "scrollbar"
  | "searchbox"
  | "slider"
  | "spinbutton"
  | "switch"
  | "tab"
  | "tablist"
  | "tabpanel"
  | "textbox"
  | "toolbar"
  | "tooltip"
  | "tree"
  | "treegrid"
  | "treeitem"
  | "alert"
  | "alertdialog"
  | "grid"
  | "heading"
  | "img"
  | "list"
  | "listitem"
  | "log"
  | "marquee"
  | "math"
  | "note"
  | "separator"
  | "status"
  | "table"
  | "cell"
  | "columnheader"
  | "row"
  | "rowgroup"
  | "rowheader"
  | "timer"
  | "definition"
  | "directory"
  | "figure"
  | "group"
  | "paragraph"
  | "term"
  | "generic"
  | "static_text"
  | "none"
  | "unknown"
  | "window"
  | "pane"
  | "titlebar"
  | "edit"
  | "custom"
  | "dataitem"
  | "datepicker"
  | "calendar"
  | "hyperlink"
  | "splitbutton";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Accessibility capture backend. Each variant corresponds to a platform
 * accessibility API implementation.
 */
type AccessibilityBackend = "auto" | "cdp" | "uia" | "atspi" | "ax" | "none";

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Accessibility state flags for a node.
 *
 * These flags represent the current interactive state of an element.
 * Tri-state booleans use `Option<bool>` — `None` means "not applicable."
 */
interface AccessibilityState {
  /**
   * Checkable element's checked state.
   */
  isChecked?: boolean | null;
  /**
   * Element is disabled / non-interactive.
   */
  isDisabled: boolean;
  /**
   * Element content can be edited.
   */
  isEditable: boolean;
  /**
   * Expandable element's expansion state.
   */
  isExpanded?: boolean | null;
  /**
   * Element can receive focus.
   */
  isFocusable: boolean;
  /**
   * Element has keyboard focus.
   */
  isFocused: boolean;
  /**
   * Element is hidden from the accessibility tree.
   */
  isHidden: boolean;
  /**
   * Element is a modal dialog.
   */
  isModal: boolean;
  /**
   * Element allows multiple selections.
   */
  isMultiselectable: boolean;
  /**
   * Pressable element's pressed state.
   */
  isPressed?: boolean | null;
  /**
   * Element is read-only.
   */
  isReadonly: boolean;
  /**
   * Element value is required.
   */
  isRequired: boolean;
  /**
   * Selectable element's selection state.
   */
  isSelected?: boolean | null;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Bounding rectangle for an accessibility node. Coordinates are screen
 * pixels, typically absolute screen coordinates.
 */
interface AccessibilityBounds {
  height: number;
  width: number;
  x: number;
  y: number;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * A node in the accessibility tree.
 *
 * Each node represents an element in the accessibility hierarchy with its
 * role, name, value, state, and bounds. The `ref` field provides a stable
 * identifier for AI-driven automation (e.g., `@e1`, `@e2`).
 */
interface AccessibilityNode {
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
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Complete accessibility-tree snapshot.
 *
 * Full accessibility tree at a point in time, with metadata about the
 * capture source and summary statistics.
 */
interface AccessibilitySnapshot {
  backend: AccessibilityBackend;
  /**
   * Number of interactive nodes.
   */
  interactiveNodes: number;
  root: AccessibilityNode;
  /**
   * Unix timestamp of capture.
   */
  timestamp: number;
  /**
   * Page / window title.
   */
  title?: string | null;
  /**
   * Total number of nodes in the tree.
   */
  totalNodes: number;
  /**
   * Page URL (for web targets).
   */
  url?: string | null;
}

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Role criterion for [`AccessibilitySelector`] — either a single role or a
 * list of roles (any match).
 */
type RoleCriterion = AccessibilityRole | AccessibilityRole[];

/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */



/**
 * Selector for finding nodes in an accessibility tree.
 *
 * Provides flexible matching criteria for locating elements by role, name,
 * automation ID, or other attributes. Multiple criteria are combined with
 * AND logic.
 */
interface AccessibilitySelector {
  /**
   * Required ancestor selector.
   */
  ancestor?: AccessibilitySelector | null;
  /**
   * Match by automation / test ID.
   */
  automationId?: string | null;
  /**
   * Whether string matching is case-sensitive.
   */
  caseSensitive: boolean;
  /**
   * Match by CSS / control class name.
   */
  className?: string | null;
  /**
   * Match by HTML tag name.
   */
  htmlTag?: string | null;
  /**
   * Filter by interactivity.
   */
  isInteractive?: boolean | null;
  /**
   * Maximum tree depth to search.
   */
  maxDepth?: number | null;
  /**
   * Exact name match.
   */
  name?: string | null;
  /**
   * Partial name match (contains).
   */
  nameContains?: string | null;
  /**
   * Regex pattern for name matching.
   */
  namePattern?: string | null;
  /**
   * Match by role (single or list).
   */
  role?: RoleCriterion | null;
  /**
   * Required state flags (partial match).
   */
  state?: AccessibilityState | null;
  /**
   * Exact value match.
   */
  value?: string | null;
  /**
   * Partial value match (contains).
   */
  valueContains?: string | null;
}

export type { AccessibilityBackend, AccessibilityBounds, AccessibilityNode, AccessibilityRole, AccessibilitySelector, AccessibilitySnapshot, AccessibilityState };
