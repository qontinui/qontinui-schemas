/**
 * Auto-generated TypeScript types from qontinui-schemas
 * DO NOT EDIT - regenerate with: poetry run python scripts/generate_typescript.py
 */

export enum AccessibilityRole {
  APPLICATION = "application",
  DOCUMENT = "document",
  ARTICLE = "article",
  BANNER = "banner",
  COMPLEMENTARY = "complementary",
  CONTENTINFO = "contentinfo",
  FORM = "form",
  MAIN = "main",
  NAVIGATION = "navigation",
  REGION = "region",
  SEARCH = "search",
  BUTTON = "button",
  CHECKBOX = "checkbox",
  COMBOBOX = "combobox",
  DIALOG = "dialog",
  GRIDCELL = "gridcell",
  LINK = "link",
  LISTBOX = "listbox",
  MENU = "menu",
  MENUBAR = "menubar",
  MENUITEM = "menuitem",
  MENUITEMCHECKBOX = "menuitemcheckbox",
  MENUITEMRADIO = "menuitemradio",
  OPTION = "option",
  PROGRESSBAR = "progressbar",
  RADIO = "radio",
  RADIOGROUP = "radiogroup",
  SCROLLBAR = "scrollbar",
  SEARCHBOX = "searchbox",
  SLIDER = "slider",
  SPINBUTTON = "spinbutton",
  SWITCH = "switch",
  TAB = "tab",
  TABLIST = "tablist",
  TABPANEL = "tabpanel",
  TEXTBOX = "textbox",
  TOOLBAR = "toolbar",
  TOOLTIP = "tooltip",
  TREE = "tree",
  TREEGRID = "treegrid",
  TREEITEM = "treeitem",
  ALERT = "alert",
  ALERTDIALOG = "alertdialog",
  GRID = "grid",
  HEADING = "heading",
  IMG = "img",
  LIST = "list",
  LISTITEM = "listitem",
  LOG = "log",
  MARQUEE = "marquee",
  MATH = "math",
  NOTE = "note",
  SEPARATOR = "separator",
  STATUS = "status",
  TABLE = "table",
  CELL = "cell",
  COLUMNHEADER = "columnheader",
  ROW = "row",
  ROWGROUP = "rowgroup",
  ROWHEADER = "rowheader",
  TIMER = "timer",
  DEFINITION = "definition",
  DIRECTORY = "directory",
  FIGURE = "figure",
  GROUP = "group",
  PARAGRAPH = "paragraph",
  TERM = "term",
  GENERIC = "generic",
  STATIC_TEXT = "static_text",
  NONE = "none",
  UNKNOWN = "unknown",
  WINDOW = "window",
  PANE = "pane",
  TITLEBAR = "titlebar",
  EDIT = "edit",
  CUSTOM = "custom",
  DATAITEM = "dataitem",
  DATEPICKER = "datepicker",
  CALENDAR = "calendar",
  HYPERLINK = "hyperlink",
  SPLITBUTTON = "splitbutton",
}

export enum AccessibilityBackend {
  AUTO = "auto",
  CDP = "cdp",
  UIA = "uia",
  ATSPI = "atspi",
  AX = "ax",
  NONE = "none",
}

export interface AccessibilityState {
  /** Element has keyboard focus */
  is_focused?: boolean;
  /** Element is disabled/non-interactive */
  is_disabled?: boolean;
  /** Element is hidden from accessibility tree */
  is_hidden?: boolean;
  /** Expandable element expansion state */
  is_expanded?: boolean | null;
  /** Selectable element selection state */
  is_selected?: boolean | null;
  /** Checkable element checked state */
  is_checked?: boolean | null;
  /** Pressable element pressed state */
  is_pressed?: boolean | null;
  /** Element is read-only */
  is_readonly?: boolean;
  /** Element value is required */
  is_required?: boolean;
  /** Element allows multiple selections */
  is_multiselectable?: boolean;
  /** Element content can be edited */
  is_editable?: boolean;
  /** Element can receive focus */
  is_focusable?: boolean;
  /** Element is a modal dialog */
  is_modal?: boolean;
}

export interface AccessibilityBounds {
  /** X coordinate of top-left corner */
  x: number;
  /** Y coordinate of top-left corner */
  y: number;
  /** Width in pixels */
  width: number;
  /** Height in pixels */
  height: number;
}

export interface AccessibilityNode {
  /** Reference ID like @e1, @e2 for AI interaction */
  ref: string;
  /** Accessibility role (button, textbox, etc.) */
  role: AccessibilityRole;
  /** Accessible name (label) */
  name?: string | null;
  /** Current value (for inputs) */
  value?: string | null;
  /** Accessible description (additional context) */
  description?: string | null;
  /** Bounding rectangle in screen coordinates */
  bounds?: AccessibilityBounds | null;
  /** Current state flags */
  state?: AccessibilityState;
  /** Whether element accepts user interaction */
  is_interactive?: boolean;
  /** Hierarchical level (for headings, tree items) */
  level?: number | null;
  /** Automation ID / test ID attribute */
  automation_id?: string | null;
  /** CSS class name or control class */
  class_name?: string | null;
  /** HTML tag name (for web elements) */
  html_tag?: string | null;
  /** URL for link elements */
  url?: string | null;
  /** Child nodes in the tree */
  children?: AccessibilityNode[];
}

export interface AccessibilitySnapshot {
  /** Root node of the accessibility tree */
  root: AccessibilityNode;
  /** Unix timestamp of capture */
  timestamp: number;
  /** Backend used for capture */
  backend: AccessibilityBackend;
  /** Page URL (for web targets) */
  url?: string | null;
  /** Page/window title */
  title?: string | null;
  /** Total number of nodes in tree */
  total_nodes?: number;
  /** Number of interactive nodes */
  interactive_nodes?: number;
}

export interface AccessibilitySelector {
  /** Match by role (single or list) */
  role?: AccessibilityRole | AccessibilityRole[] | null;
  /** Exact name match */
  name?: string | null;
  /** Partial name match (contains) */
  name_contains?: string | null;
  /** Regex pattern for name matching */
  name_pattern?: string | null;
  /** Exact value match */
  value?: string | null;
  /** Partial value match (contains) */
  value_contains?: string | null;
  /** Match by automation/test ID */
  automation_id?: string | null;
  /** Match by CSS/control class name */
  class_name?: string | null;
  /** Match by HTML tag name */
  html_tag?: string | null;
  /** Required state flags (partial match) */
  state?: AccessibilityState | null;
  /** Filter by interactivity */
  is_interactive?: boolean | null;
  /** Required ancestor selector */
  ancestor?: AccessibilitySelector | null;
  /** Maximum tree depth to search */
  max_depth?: number | null;
  /** Whether string matching is case-sensitive */
  case_sensitive?: boolean;
}

export interface AccessibilityConfig {
  /** Accessibility backend to use (auto, cdp, uia, etc.) */
  backend?: AccessibilityBackend;
  /** Only capture interactive elements (reduces tree size) */
  interactive_only?: boolean;
  /** Include hidden/offscreen elements in capture */
  include_hidden?: boolean;
  /** Maximum tree depth to capture (None for unlimited) */
  max_depth?: number | null;
  /** CDP WebSocket host for browser connections */
  cdp_host?: string;
  /** CDP WebSocket port for browser connections */
  cdp_port?: number;
  /** Timeout in seconds for CDP operations */
  cdp_timeout?: number;
  /** Include bounding rectangles for each node */
  include_bounds?: boolean;
  /** Include current values for input elements */
  include_value?: boolean;
}

export interface AccessibilityCaptureOptions {
  /** Capture target: 'auto', 'web', 'native', 'tauri', or a window title/handle */
  target?: string;
  /** Also capture a screenshot alongside the tree */
  include_screenshot?: boolean;
  /** Capture configuration overrides */
  config?: AccessibilityConfig;
}

export interface AccessibilityActionResult {
  /** Whether the action succeeded */
  success: boolean;
  /** The ref that was acted upon */
  ref: string;
  /** The action performed (click, type, focus) */
  action: string;
  /** Error message if action failed */
  error?: string | null;
  /** Name of the element acted upon */
  element_name?: string | null;
  /** Role of the element acted upon */
  element_role?: string | null;
}
