//! Accessibility tree models.
//!
//! Mirrors `src/qontinui_schemas/accessibility/models.py` +
//! `enums.py`. Rust is the source of truth; TS and Python bindings
//! regenerate from the JSON Schemas emitted here.
//!
//! The ref system (`@e1`, `@e2`, …) provides stable, AI-friendly identifiers
//! for interacting with elements without re-querying the accessibility tree.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ============================================================================
// Enums
// ============================================================================

/// ARIA/UIA accessibility roles. Based on WAI-ARIA 1.2 with extensions for
/// Windows UI Automation and other platform-specific accessibility APIs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AccessibilityRole {
    // Document structure
    Application,
    Document,
    Article,
    Banner,
    Complementary,
    Contentinfo,
    Form,
    Main,
    Navigation,
    Region,
    Search,

    // Widget roles
    Button,
    Checkbox,
    Combobox,
    Dialog,
    Gridcell,
    Link,
    Listbox,
    Menu,
    Menubar,
    Menuitem,
    Menuitemcheckbox,
    Menuitemradio,
    Option,
    Progressbar,
    Radio,
    Radiogroup,
    Scrollbar,
    Searchbox,
    Slider,
    Spinbutton,
    Switch,
    Tab,
    Tablist,
    Tabpanel,
    Textbox,
    Toolbar,
    Tooltip,
    Tree,
    Treegrid,
    Treeitem,

    // Structure roles
    Alert,
    Alertdialog,
    Grid,
    Heading,
    Img,
    List,
    Listitem,
    Log,
    Marquee,
    Math,
    Note,
    Separator,
    Status,
    Table,
    Cell,
    Columnheader,
    Row,
    Rowgroup,
    Rowheader,
    Timer,
    Definition,
    Directory,
    Figure,
    Group,
    Paragraph,
    Term,

    // Generic/fallback roles
    Generic,
    StaticText,
    None,
    Unknown,

    // Windows UIA specific
    Window,
    Pane,
    Titlebar,
    Edit,
    Custom,
    Dataitem,
    Datepicker,
    Calendar,
    Hyperlink,
    Splitbutton,
}

/// Accessibility capture backend. Each variant corresponds to a platform
/// accessibility API implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum AccessibilityBackend {
    /// Automatically detect the best available backend.
    Auto,
    /// Chrome DevTools Protocol (browsers, Electron, Tauri on Windows).
    Cdp,
    /// Windows UI Automation (native Windows apps).
    Uia,
    /// AT-SPI2 (Linux desktop accessibility).
    Atspi,
    /// macOS Accessibility API.
    Ax,
    /// Capture disabled.
    None,
}

// ============================================================================
// AccessibilityState
// ============================================================================

/// Accessibility state flags for a node.
///
/// These flags represent the current interactive state of an element.
/// Tri-state booleans use `Option<bool>` — `None` means "not applicable."
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct AccessibilityState {
    /// Element has keyboard focus.
    #[serde(default)]
    pub is_focused: bool,
    /// Element is disabled / non-interactive.
    #[serde(default)]
    pub is_disabled: bool,
    /// Element is hidden from the accessibility tree.
    #[serde(default)]
    pub is_hidden: bool,
    /// Expandable element's expansion state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_expanded: Option<bool>,
    /// Selectable element's selection state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_selected: Option<bool>,
    /// Checkable element's checked state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_checked: Option<bool>,
    /// Pressable element's pressed state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_pressed: Option<bool>,
    /// Element is read-only.
    #[serde(default)]
    pub is_readonly: bool,
    /// Element value is required.
    #[serde(default)]
    pub is_required: bool,
    /// Element allows multiple selections.
    #[serde(default)]
    pub is_multiselectable: bool,
    /// Element content can be edited.
    #[serde(default)]
    pub is_editable: bool,
    /// Element can receive focus.
    #[serde(default)]
    pub is_focusable: bool,
    /// Element is a modal dialog.
    #[serde(default)]
    pub is_modal: bool,
}

// ============================================================================
// AccessibilityBounds
// ============================================================================

/// Bounding rectangle for an accessibility node. Coordinates are screen
/// pixels, typically absolute screen coordinates.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AccessibilityBounds {
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
}

// ============================================================================
// AccessibilityNode
// ============================================================================

/// A node in the accessibility tree.
///
/// Each node represents an element in the accessibility hierarchy with its
/// role, name, value, state, and bounds. The `ref` field provides a stable
/// identifier for AI-driven automation (e.g., `@e1`, `@e2`).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AccessibilityNode {
    /// Reference ID like `@e1`, `@e2` for AI interaction.
    #[serde(rename = "ref")]
    pub ref_id: String,
    /// Accessibility role (button, textbox, etc.).
    pub role: AccessibilityRole,
    /// Accessible name (label).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Current value (for inputs).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Accessible description (additional context).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Bounding rectangle in screen coordinates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bounds: Option<AccessibilityBounds>,
    /// Current state flags.
    #[serde(default)]
    pub state: AccessibilityState,
    /// Whether the element accepts user interaction.
    #[serde(default)]
    pub is_interactive: bool,
    /// Hierarchical level (for headings, tree items).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level: Option<i64>,
    /// Automation ID / test-ID attribute.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub automation_id: Option<String>,
    /// CSS class name or control class.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub class_name: Option<String>,
    /// HTML tag name (for web elements).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html_tag: Option<String>,
    /// URL for link elements.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Child nodes in the tree.
    #[serde(default)]
    pub children: Vec<AccessibilityNode>,
}

// ============================================================================
// AccessibilitySnapshot
// ============================================================================

/// Complete accessibility-tree snapshot.
///
/// Full accessibility tree at a point in time, with metadata about the
/// capture source and summary statistics.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AccessibilitySnapshot {
    /// Root node of the accessibility tree.
    pub root: AccessibilityNode,
    /// Unix timestamp of capture.
    pub timestamp: f64,
    /// Backend used for capture.
    pub backend: AccessibilityBackend,
    /// Page URL (for web targets).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Page / window title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Total number of nodes in the tree.
    #[serde(default)]
    pub total_nodes: i64,
    /// Number of interactive nodes.
    #[serde(default)]
    pub interactive_nodes: i64,
}

// ============================================================================
// AccessibilitySelector
// ============================================================================

/// Role criterion for [`AccessibilitySelector`] — either a single role or a
/// list of roles (any match).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum RoleCriterion {
    Single(AccessibilityRole),
    Any(Vec<AccessibilityRole>),
}

/// Selector for finding nodes in an accessibility tree.
///
/// Provides flexible matching criteria for locating elements by role, name,
/// automation ID, or other attributes. Multiple criteria are combined with
/// AND logic.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct AccessibilitySelector {
    /// Match by role (single or list).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<RoleCriterion>,
    /// Exact name match.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Partial name match (contains).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name_contains: Option<String>,
    /// Regex pattern for name matching.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name_pattern: Option<String>,
    /// Exact value match.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Partial value match (contains).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_contains: Option<String>,
    /// Match by automation / test ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub automation_id: Option<String>,
    /// Match by CSS / control class name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub class_name: Option<String>,
    /// Match by HTML tag name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html_tag: Option<String>,
    /// Required state flags (partial match).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<AccessibilityState>,
    /// Filter by interactivity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_interactive: Option<bool>,
    /// Required ancestor selector.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ancestor: Option<Box<AccessibilitySelector>>,
    /// Maximum tree depth to search.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_depth: Option<i64>,
    /// Whether string matching is case-sensitive.
    #[serde(default = "default_true")]
    pub case_sensitive: bool,
}

fn default_true() -> bool {
    true
}
