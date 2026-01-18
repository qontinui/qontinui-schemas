"""Accessibility-related enumerations.

These enums are shared between:
- qontinui library (Python HAL implementation)
- qontinui-runner (TypeScript, via generated types)
- qontinui-web frontend (TypeScript, via generated types)

The AccessibilityRole enum follows ARIA roles with extensions for
Windows UI Automation and other platform-specific roles.
"""

from enum import Enum


class AccessibilityRole(str, Enum):
    """ARIA/UIA accessibility roles.

    Based on WAI-ARIA 1.2 roles with extensions for Windows UI Automation
    and other platform-specific accessibility APIs.
    """

    # Document structure
    APPLICATION = "application"
    DOCUMENT = "document"
    ARTICLE = "article"
    BANNER = "banner"
    COMPLEMENTARY = "complementary"
    CONTENTINFO = "contentinfo"
    FORM = "form"
    MAIN = "main"
    NAVIGATION = "navigation"
    REGION = "region"
    SEARCH = "search"

    # Widget roles
    BUTTON = "button"
    CHECKBOX = "checkbox"
    COMBOBOX = "combobox"
    DIALOG = "dialog"
    GRIDCELL = "gridcell"
    LINK = "link"
    LISTBOX = "listbox"
    MENU = "menu"
    MENUBAR = "menubar"
    MENUITEM = "menuitem"
    MENUITEMCHECKBOX = "menuitemcheckbox"
    MENUITEMRADIO = "menuitemradio"
    OPTION = "option"
    PROGRESSBAR = "progressbar"
    RADIO = "radio"
    RADIOGROUP = "radiogroup"
    SCROLLBAR = "scrollbar"
    SEARCHBOX = "searchbox"
    SLIDER = "slider"
    SPINBUTTON = "spinbutton"
    SWITCH = "switch"
    TAB = "tab"
    TABLIST = "tablist"
    TABPANEL = "tabpanel"
    TEXTBOX = "textbox"
    TOOLBAR = "toolbar"
    TOOLTIP = "tooltip"
    TREE = "tree"
    TREEGRID = "treegrid"
    TREEITEM = "treeitem"

    # Structure roles
    ALERT = "alert"
    ALERTDIALOG = "alertdialog"
    GRID = "grid"
    HEADING = "heading"
    IMG = "img"
    LIST = "list"
    LISTITEM = "listitem"
    LOG = "log"
    MARQUEE = "marquee"
    MATH = "math"
    NOTE = "note"
    SEPARATOR = "separator"
    STATUS = "status"
    TABLE = "table"
    CELL = "cell"
    COLUMNHEADER = "columnheader"
    ROW = "row"
    ROWGROUP = "rowgroup"
    ROWHEADER = "rowheader"
    TIMER = "timer"
    DEFINITION = "definition"
    DIRECTORY = "directory"
    FIGURE = "figure"
    GROUP = "group"
    PARAGRAPH = "paragraph"
    TERM = "term"

    # Generic/fallback roles
    GENERIC = "generic"
    STATIC_TEXT = "static_text"
    NONE = "none"
    UNKNOWN = "unknown"

    # Windows UIA specific
    WINDOW = "window"
    PANE = "pane"
    TITLEBAR = "titlebar"
    EDIT = "edit"
    CUSTOM = "custom"
    DATAITEM = "dataitem"
    DATEPICKER = "datepicker"
    CALENDAR = "calendar"
    HYPERLINK = "hyperlink"
    SPLITBUTTON = "splitbutton"


class AccessibilityBackend(str, Enum):
    """Accessibility capture backend types.

    AUTO: Automatically detect the best available backend.
    CDP: Chrome DevTools Protocol (browsers, Electron, Tauri on Windows).
    UIA: Windows UI Automation (native Windows apps).
    ATSPI: AT-SPI2 (Linux desktop accessibility).
    AX: macOS Accessibility API.
    NONE: No accessibility capture (disabled).
    """

    AUTO = "auto"
    CDP = "cdp"
    UIA = "uia"
    ATSPI = "atspi"
    AX = "ax"
    NONE = "none"


__all__ = [
    "AccessibilityRole",
    "AccessibilityBackend",
]
