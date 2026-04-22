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
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct AccessibilityState {
    /// Element has keyboard focus.
    #[serde(default, alias = "is_focused")]
    pub is_focused: bool,
    /// Element is disabled / non-interactive.
    #[serde(default, alias = "is_disabled")]
    pub is_disabled: bool,
    /// Element is hidden from the accessibility tree.
    #[serde(default, alias = "is_hidden")]
    pub is_hidden: bool,
    /// Expandable element's expansion state.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "is_expanded"
    )]
    pub is_expanded: Option<bool>,
    /// Selectable element's selection state.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "is_selected"
    )]
    pub is_selected: Option<bool>,
    /// Checkable element's checked state.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "is_checked")]
    pub is_checked: Option<bool>,
    /// Pressable element's pressed state.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "is_pressed")]
    pub is_pressed: Option<bool>,
    /// Element is read-only.
    #[serde(default, alias = "is_readonly")]
    pub is_readonly: bool,
    /// Element value is required.
    #[serde(default, alias = "is_required")]
    pub is_required: bool,
    /// Element allows multiple selections.
    #[serde(default, alias = "is_multiselectable")]
    pub is_multiselectable: bool,
    /// Element content can be edited.
    #[serde(default, alias = "is_editable")]
    pub is_editable: bool,
    /// Element can receive focus.
    #[serde(default, alias = "is_focusable")]
    pub is_focusable: bool,
    /// Element is a modal dialog.
    #[serde(default, alias = "is_modal")]
    pub is_modal: bool,
}

// ============================================================================
// AccessibilityBounds
// ============================================================================

/// Bounding rectangle for an accessibility node. Coordinates are screen
/// pixels, typically absolute screen coordinates.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct AccessibilityBounds {
    #[serde(alias = "x")]
    pub x: i64,
    #[serde(alias = "y")]
    pub y: i64,
    #[serde(alias = "width")]
    pub width: i64,
    #[serde(alias = "height")]
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
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct AccessibilityNode {
    /// Reference ID like `@e1`, `@e2` for AI interaction.
    #[serde(rename = "ref")]
    pub ref_id: String,
    /// Accessibility role (button, textbox, etc.).
    #[serde(alias = "role")]
    pub role: AccessibilityRole,
    /// Accessible name (label).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "name")]
    pub name: Option<String>,
    /// Current value (for inputs).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "value")]
    pub value: Option<String>,
    /// Accessible description (additional context).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "description"
    )]
    pub description: Option<String>,
    /// Bounding rectangle in screen coordinates.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "bounds")]
    pub bounds: Option<AccessibilityBounds>,
    /// Current state flags.
    #[serde(default, alias = "state")]
    pub state: AccessibilityState,
    /// Whether the element accepts user interaction.
    #[serde(default, alias = "is_interactive")]
    pub is_interactive: bool,
    /// Hierarchical level (for headings, tree items).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "level")]
    pub level: Option<i64>,
    /// Automation ID / test-ID attribute.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "automation_id"
    )]
    pub automation_id: Option<String>,
    /// CSS class name or control class.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "class_name")]
    pub class_name: Option<String>,
    /// HTML tag name (for web elements).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "html_tag")]
    pub html_tag: Option<String>,
    /// URL for link elements.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "url")]
    pub url: Option<String>,
    /// Child nodes in the tree.
    #[serde(default, alias = "children")]
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
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct AccessibilitySnapshot {
    /// Root node of the accessibility tree.
    #[serde(alias = "root")]
    pub root: AccessibilityNode,
    /// Unix timestamp of capture.
    #[serde(alias = "timestamp")]
    pub timestamp: f64,
    /// Backend used for capture.
    #[serde(alias = "backend")]
    pub backend: AccessibilityBackend,
    /// Page URL (for web targets).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "url")]
    pub url: Option<String>,
    /// Page / window title.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "title")]
    pub title: Option<String>,
    /// Total number of nodes in the tree.
    #[serde(default, alias = "total_nodes")]
    pub total_nodes: i64,
    /// Number of interactive nodes.
    #[serde(default, alias = "interactive_nodes")]
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
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct AccessibilitySelector {
    /// Match by role (single or list).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "role")]
    pub role: Option<RoleCriterion>,
    /// Exact name match.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "name")]
    pub name: Option<String>,
    /// Partial name match (contains).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "name_contains"
    )]
    pub name_contains: Option<String>,
    /// Regex pattern for name matching.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "name_pattern"
    )]
    pub name_pattern: Option<String>,
    /// Exact value match.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "value")]
    pub value: Option<String>,
    /// Partial value match (contains).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "value_contains"
    )]
    pub value_contains: Option<String>,
    /// Match by automation / test ID.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "automation_id"
    )]
    pub automation_id: Option<String>,
    /// Match by CSS / control class name.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "class_name")]
    pub class_name: Option<String>,
    /// Match by HTML tag name.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "html_tag")]
    pub html_tag: Option<String>,
    /// Required state flags (partial match).
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "state")]
    pub state: Option<AccessibilityState>,
    /// Filter by interactivity.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "is_interactive"
    )]
    pub is_interactive: Option<bool>,
    /// Required ancestor selector.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "ancestor")]
    pub ancestor: Option<Box<AccessibilitySelector>>,
    /// Maximum tree depth to search.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "max_depth")]
    pub max_depth: Option<i64>,
    /// Whether string matching is case-sensitive.
    #[serde(default = "default_true", alias = "case_sensitive")]
    pub case_sensitive: bool,
}

fn default_true() -> bool {
    true
}
