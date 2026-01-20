"""Unit tests for accessibility schemas.

Tests cover:
- AccessibilityRole and AccessibilityBackend enums
- AccessibilityState model
- AccessibilityBounds model with center calculations
- AccessibilityNode model with nested children
- AccessibilitySnapshot with tree traversal methods
- AccessibilitySelector with various matching criteria
- AccessibilityConfig and AccessibilityCaptureOptions
- AccessibilityActionResult
"""

import time

from qontinui_schemas.accessibility import (
    AccessibilityActionResult,
    AccessibilityBackend,
    AccessibilityBounds,
    AccessibilityCaptureOptions,
    AccessibilityConfig,
    AccessibilityNode,
    AccessibilityRole,
    AccessibilitySelector,
    AccessibilitySnapshot,
    AccessibilityState,
)


class TestAccessibilityEnums:
    """Test AccessibilityRole and AccessibilityBackend enums."""

    def test_role_button_value(self) -> None:
        """Role enum values match expected strings."""
        assert AccessibilityRole.BUTTON.value == "button"
        assert AccessibilityRole.TEXTBOX.value == "textbox"
        assert AccessibilityRole.CHECKBOX.value == "checkbox"

    def test_role_from_string(self) -> None:
        """Role can be created from string value."""
        assert AccessibilityRole("button") == AccessibilityRole.BUTTON
        assert AccessibilityRole("textbox") == AccessibilityRole.TEXTBOX

    def test_backend_values(self) -> None:
        """Backend enum values match expected strings."""
        assert AccessibilityBackend.AUTO.value == "auto"
        assert AccessibilityBackend.CDP.value == "cdp"
        assert AccessibilityBackend.UIA.value == "uia"
        assert AccessibilityBackend.ATSPI.value == "atspi"
        assert AccessibilityBackend.AX.value == "ax"
        assert AccessibilityBackend.NONE.value == "none"

    def test_role_is_string_enum(self) -> None:
        """Role enum is a string enum for JSON serialization."""
        role = AccessibilityRole.BUTTON
        assert isinstance(role.value, str)
        # Access .value before the comparison to avoid MyPy type narrowing
        assert role.value == "button"
        # String enums inherit from str, so they can be used directly as strings
        assert role == "button"


class TestAccessibilityState:
    """Test AccessibilityState model."""

    def test_default_values(self) -> None:
        """All state flags have correct defaults."""
        state = AccessibilityState()
        assert state.is_focused is False
        assert state.is_disabled is False
        assert state.is_hidden is False
        assert state.is_expanded is None
        assert state.is_selected is None
        assert state.is_checked is None
        assert state.is_pressed is None
        assert state.is_readonly is False
        assert state.is_required is False
        assert state.is_multiselectable is False
        assert state.is_editable is False
        assert state.is_focusable is False
        assert state.is_modal is False

    def test_custom_values(self) -> None:
        """State can be created with custom values."""
        state = AccessibilityState(
            is_focused=True,
            is_checked=True,
            is_expanded=False,
            is_disabled=True,
        )
        assert state.is_focused is True
        assert state.is_checked is True
        assert state.is_expanded is False
        assert state.is_disabled is True

    def test_serialization_round_trip(self) -> None:
        """State serializes and deserializes correctly."""
        state = AccessibilityState(
            is_focused=True,
            is_selected=True,
            is_expanded=False,
        )
        data = state.model_dump()
        restored = AccessibilityState.model_validate(data)
        assert restored.is_focused is True
        assert restored.is_selected is True
        assert restored.is_expanded is False


class TestAccessibilityBounds:
    """Test AccessibilityBounds model."""

    def test_basic_bounds(self) -> None:
        """Bounds stores coordinates correctly."""
        bounds = AccessibilityBounds(x=100, y=200, width=300, height=150)
        assert bounds.x == 100
        assert bounds.y == 200
        assert bounds.width == 300
        assert bounds.height == 150

    def test_center_x_calculation(self) -> None:
        """center_x property calculates correctly."""
        bounds = AccessibilityBounds(x=100, y=0, width=200, height=100)
        assert bounds.center_x == 200  # 100 + 200//2 = 200

    def test_center_y_calculation(self) -> None:
        """center_y property calculates correctly."""
        bounds = AccessibilityBounds(x=0, y=100, width=200, height=300)
        assert bounds.center_y == 250  # 100 + 300//2 = 250

    def test_center_with_odd_dimensions(self) -> None:
        """Center uses integer division for odd dimensions."""
        bounds = AccessibilityBounds(x=0, y=0, width=101, height=51)
        assert bounds.center_x == 50  # 0 + 101//2 = 50
        assert bounds.center_y == 25  # 0 + 51//2 = 25

    def test_zero_bounds(self) -> None:
        """Handles zero width/height gracefully."""
        bounds = AccessibilityBounds(x=50, y=50, width=0, height=0)
        assert bounds.center_x == 50
        assert bounds.center_y == 50

    def test_serialization_round_trip(self) -> None:
        """Bounds serializes and deserializes correctly."""
        bounds = AccessibilityBounds(x=10, y=20, width=100, height=50)
        data = bounds.model_dump()
        restored = AccessibilityBounds.model_validate(data)
        assert restored.x == 10
        assert restored.y == 20
        assert restored.width == 100
        assert restored.height == 50


class TestAccessibilityNode:
    """Test AccessibilityNode model."""

    def test_minimal_node(self) -> None:
        """Node with only required fields."""
        node = AccessibilityNode(ref="@e1", role=AccessibilityRole.BUTTON)
        assert node.ref == "@e1"
        assert node.role == AccessibilityRole.BUTTON
        assert node.name is None
        assert node.value is None
        assert node.children == []
        assert node.is_interactive is False

    def test_full_node(self) -> None:
        """Node with all optional fields populated."""
        bounds = AccessibilityBounds(x=0, y=0, width=100, height=50)
        state = AccessibilityState(is_focused=True)
        node = AccessibilityNode(
            ref="@e5",
            role=AccessibilityRole.TEXTBOX,
            name="Email",
            value="user@example.com",
            description="Enter your email address",
            bounds=bounds,
            state=state,
            is_interactive=True,
            level=None,
            automation_id="email-input",
            class_name="form-input",
            html_tag="input",
            url=None,
        )
        assert node.ref == "@e5"
        assert node.role == AccessibilityRole.TEXTBOX
        assert node.name == "Email"
        assert node.value == "user@example.com"
        assert node.bounds is not None
        assert node.bounds.width == 100
        assert node.state.is_focused is True
        assert node.is_interactive is True
        assert node.automation_id == "email-input"
        assert node.html_tag == "input"

    def test_nested_children(self) -> None:
        """Node can have nested children."""
        child1 = AccessibilityNode(
            ref="@e2", role=AccessibilityRole.LISTITEM, name="Item 1"
        )
        child2 = AccessibilityNode(
            ref="@e3", role=AccessibilityRole.LISTITEM, name="Item 2"
        )
        parent = AccessibilityNode(
            ref="@e1",
            role=AccessibilityRole.LIST,
            name="My List",
            children=[child1, child2],
        )
        assert len(parent.children) == 2
        assert parent.children[0].name == "Item 1"
        assert parent.children[1].name == "Item 2"

    def test_deeply_nested_structure(self) -> None:
        """Deeply nested tree structure works correctly."""
        leaf = AccessibilityNode(
            ref="@e4", role=AccessibilityRole.STATIC_TEXT, name="Leaf"
        )
        mid = AccessibilityNode(
            ref="@e3", role=AccessibilityRole.GROUP, children=[leaf]
        )
        inner = AccessibilityNode(
            ref="@e2", role=AccessibilityRole.REGION, children=[mid]
        )
        root = AccessibilityNode(
            ref="@e1", role=AccessibilityRole.DOCUMENT, children=[inner]
        )

        assert root.children[0].children[0].children[0].name == "Leaf"

    def test_serialization_with_children(self) -> None:
        """Node with children serializes correctly."""
        child = AccessibilityNode(ref="@e2", role=AccessibilityRole.BUTTON, name="OK")
        parent = AccessibilityNode(
            ref="@e1", role=AccessibilityRole.DIALOG, children=[child]
        )

        data = parent.model_dump()
        restored = AccessibilityNode.model_validate(data)

        assert len(restored.children) == 1
        assert restored.children[0].ref == "@e2"
        assert restored.children[0].name == "OK"


class TestAccessibilitySnapshot:
    """Test AccessibilitySnapshot model."""

    def _create_sample_tree(self) -> AccessibilityNode:
        """Create a sample tree for testing."""
        button1 = AccessibilityNode(
            ref="@e2",
            role=AccessibilityRole.BUTTON,
            name="Submit",
            is_interactive=True,
        )
        button2 = AccessibilityNode(
            ref="@e3",
            role=AccessibilityRole.BUTTON,
            name="Cancel",
            is_interactive=True,
        )
        textbox = AccessibilityNode(
            ref="@e4",
            role=AccessibilityRole.TEXTBOX,
            name="Username",
            is_interactive=True,
        )
        static = AccessibilityNode(
            ref="@e5",
            role=AccessibilityRole.STATIC_TEXT,
            name="Welcome",
            is_interactive=False,
        )
        root = AccessibilityNode(
            ref="@e1",
            role=AccessibilityRole.DOCUMENT,
            name="Login Page",
            is_interactive=False,
            children=[button1, button2, textbox, static],
        )
        return root

    def test_basic_snapshot(self) -> None:
        """Snapshot with required fields."""
        root = AccessibilityNode(ref="@e1", role=AccessibilityRole.DOCUMENT)
        snapshot = AccessibilitySnapshot(
            root=root,
            timestamp=time.time(),
            backend=AccessibilityBackend.CDP,
        )
        assert snapshot.root.ref == "@e1"
        assert snapshot.backend == AccessibilityBackend.CDP
        assert snapshot.url is None
        assert snapshot.title is None
        assert snapshot.total_nodes == 0
        assert snapshot.interactive_nodes == 0

    def test_full_snapshot(self) -> None:
        """Snapshot with all optional fields."""
        root = self._create_sample_tree()
        snapshot = AccessibilitySnapshot(
            root=root,
            timestamp=1700000000.0,
            backend=AccessibilityBackend.CDP,
            url="https://example.com/login",
            title="Login Page",
            total_nodes=5,
            interactive_nodes=3,
        )
        assert snapshot.url == "https://example.com/login"
        assert snapshot.title == "Login Page"
        assert snapshot.total_nodes == 5
        assert snapshot.interactive_nodes == 3

    def test_get_node_by_ref_root(self) -> None:
        """get_node_by_ref finds root node."""
        root = self._create_sample_tree()
        snapshot = AccessibilitySnapshot(
            root=root, timestamp=time.time(), backend=AccessibilityBackend.CDP
        )
        found = snapshot.get_node_by_ref("@e1")
        assert found is not None
        assert found.ref == "@e1"
        assert found.name == "Login Page"

    def test_get_node_by_ref_child(self) -> None:
        """get_node_by_ref finds child nodes."""
        root = self._create_sample_tree()
        snapshot = AccessibilitySnapshot(
            root=root, timestamp=time.time(), backend=AccessibilityBackend.CDP
        )
        found = snapshot.get_node_by_ref("@e3")
        assert found is not None
        assert found.ref == "@e3"
        assert found.name == "Cancel"
        assert found.role == AccessibilityRole.BUTTON

    def test_get_node_by_ref_not_found(self) -> None:
        """get_node_by_ref returns None for missing ref."""
        root = self._create_sample_tree()
        snapshot = AccessibilitySnapshot(
            root=root, timestamp=time.time(), backend=AccessibilityBackend.CDP
        )
        found = snapshot.get_node_by_ref("@e999")
        assert found is None

    def test_get_interactive_nodes(self) -> None:
        """get_interactive_nodes returns all interactive elements."""
        root = self._create_sample_tree()
        snapshot = AccessibilitySnapshot(
            root=root, timestamp=time.time(), backend=AccessibilityBackend.CDP
        )
        interactive = snapshot.get_interactive_nodes()
        assert len(interactive) == 3
        refs = {node.ref for node in interactive}
        assert refs == {"@e2", "@e3", "@e4"}

    def test_get_interactive_nodes_empty_tree(self) -> None:
        """get_interactive_nodes handles tree with no interactive nodes."""
        root = AccessibilityNode(
            ref="@e1",
            role=AccessibilityRole.DOCUMENT,
            is_interactive=False,
            children=[
                AccessibilityNode(
                    ref="@e2", role=AccessibilityRole.STATIC_TEXT, is_interactive=False
                )
            ],
        )
        snapshot = AccessibilitySnapshot(
            root=root, timestamp=time.time(), backend=AccessibilityBackend.CDP
        )
        interactive = snapshot.get_interactive_nodes()
        assert len(interactive) == 0

    def test_serialization_round_trip(self) -> None:
        """Snapshot serializes and deserializes correctly."""
        root = self._create_sample_tree()
        snapshot = AccessibilitySnapshot(
            root=root,
            timestamp=1700000000.0,
            backend=AccessibilityBackend.CDP,
            url="https://example.com",
            title="Test Page",
            total_nodes=5,
            interactive_nodes=3,
        )
        data = snapshot.model_dump()
        restored = AccessibilitySnapshot.model_validate(data)

        assert restored.timestamp == 1700000000.0
        assert restored.url == "https://example.com"
        assert len(restored.root.children) == 4
        assert restored.get_node_by_ref("@e2") is not None


class TestAccessibilitySelector:
    """Test AccessibilitySelector model."""

    def test_empty_selector(self) -> None:
        """Empty selector with all defaults."""
        selector = AccessibilitySelector()
        assert selector.role is None
        assert selector.name is None
        assert selector.name_contains is None
        assert selector.automation_id is None
        assert selector.is_interactive is None
        assert selector.case_sensitive is True

    def test_role_single(self) -> None:
        """Selector with single role."""
        selector = AccessibilitySelector(role=AccessibilityRole.BUTTON)
        assert selector.role == AccessibilityRole.BUTTON

    def test_role_list(self) -> None:
        """Selector with multiple roles."""
        selector = AccessibilitySelector(
            role=[AccessibilityRole.BUTTON, AccessibilityRole.LINK]
        )
        assert isinstance(selector.role, list)
        assert len(selector.role) == 2
        assert AccessibilityRole.BUTTON in selector.role

    def test_name_matching_options(self) -> None:
        """Selector with various name matching options."""
        selector = AccessibilitySelector(
            name="Submit",
            name_contains="ubmi",
            name_pattern=r"^Sub.*$",
            case_sensitive=False,
        )
        assert selector.name == "Submit"
        assert selector.name_contains == "ubmi"
        assert selector.name_pattern == r"^Sub.*$"
        assert selector.case_sensitive is False

    def test_automation_id(self) -> None:
        """Selector with automation ID."""
        selector = AccessibilitySelector(automation_id="btn-submit")
        assert selector.automation_id == "btn-submit"

    def test_interactive_filter(self) -> None:
        """Selector filtering by interactivity."""
        selector = AccessibilitySelector(is_interactive=True)
        assert selector.is_interactive is True

    def test_state_filter(self) -> None:
        """Selector with state requirements."""
        state = AccessibilityState(is_focused=True, is_disabled=False)
        selector = AccessibilitySelector(state=state)
        assert selector.state is not None
        assert selector.state.is_focused is True
        assert selector.state.is_disabled is False

    def test_ancestor_selector(self) -> None:
        """Selector with ancestor requirement."""
        ancestor = AccessibilitySelector(role=AccessibilityRole.DIALOG)
        selector = AccessibilitySelector(
            role=AccessibilityRole.BUTTON,
            ancestor=ancestor,
        )
        assert selector.ancestor is not None
        assert selector.ancestor.role == AccessibilityRole.DIALOG

    def test_max_depth(self) -> None:
        """Selector with max depth limit."""
        selector = AccessibilitySelector(max_depth=3)
        assert selector.max_depth == 3

    def test_serialization_round_trip(self) -> None:
        """Selector serializes and deserializes correctly."""
        selector = AccessibilitySelector(
            role=AccessibilityRole.TEXTBOX,
            name_contains="email",
            is_interactive=True,
            case_sensitive=False,
        )
        data = selector.model_dump()
        restored = AccessibilitySelector.model_validate(data)

        assert restored.role == AccessibilityRole.TEXTBOX
        assert restored.name_contains == "email"
        assert restored.is_interactive is True
        assert restored.case_sensitive is False


class TestAccessibilityConfig:
    """Test AccessibilityConfig model."""

    def test_default_values(self) -> None:
        """Config has correct defaults."""
        config = AccessibilityConfig()
        assert config.backend == AccessibilityBackend.AUTO
        assert config.interactive_only is False
        assert config.include_hidden is False
        assert config.max_depth is None
        assert config.cdp_host == "localhost"
        assert config.cdp_port == 9222
        assert config.cdp_timeout == 30.0
        assert config.include_bounds is True
        assert config.include_value is True

    def test_custom_cdp_settings(self) -> None:
        """Config with custom CDP settings."""
        config = AccessibilityConfig(
            backend=AccessibilityBackend.CDP,
            cdp_host="192.168.1.100",
            cdp_port=9333,
            cdp_timeout=60.0,
        )
        assert config.backend == AccessibilityBackend.CDP
        assert config.cdp_host == "192.168.1.100"
        assert config.cdp_port == 9333
        assert config.cdp_timeout == 60.0

    def test_filtering_options(self) -> None:
        """Config with filtering options."""
        config = AccessibilityConfig(
            interactive_only=True,
            include_hidden=True,
            max_depth=5,
        )
        assert config.interactive_only is True
        assert config.include_hidden is True
        assert config.max_depth == 5

    def test_serialization_round_trip(self) -> None:
        """Config serializes and deserializes correctly."""
        config = AccessibilityConfig(
            backend=AccessibilityBackend.UIA,
            max_depth=10,
        )
        data = config.model_dump()
        restored = AccessibilityConfig.model_validate(data)
        assert restored.backend == AccessibilityBackend.UIA
        assert restored.max_depth == 10


class TestAccessibilityCaptureOptions:
    """Test AccessibilityCaptureOptions model."""

    def test_default_values(self) -> None:
        """Options has correct defaults."""
        options = AccessibilityCaptureOptions()
        assert options.target == "auto"
        assert options.include_screenshot is True
        assert options.config is not None
        assert options.config.backend == AccessibilityBackend.AUTO

    def test_custom_target(self) -> None:
        """Options with custom target."""
        options = AccessibilityCaptureOptions(
            target="web",
            include_screenshot=False,
        )
        assert options.target == "web"
        assert options.include_screenshot is False

    def test_with_config_override(self) -> None:
        """Options with custom config."""
        config = AccessibilityConfig(backend=AccessibilityBackend.CDP, cdp_port=9333)
        options = AccessibilityCaptureOptions(
            target="web",
            config=config,
        )
        assert options.config.backend == AccessibilityBackend.CDP
        assert options.config.cdp_port == 9333

    def test_serialization_round_trip(self) -> None:
        """Options serializes and deserializes correctly."""
        options = AccessibilityCaptureOptions(
            target="native",
            include_screenshot=False,
            config=AccessibilityConfig(backend=AccessibilityBackend.UIA),
        )
        data = options.model_dump()
        restored = AccessibilityCaptureOptions.model_validate(data)
        assert restored.target == "native"
        assert restored.config.backend == AccessibilityBackend.UIA


class TestAccessibilityActionResult:
    """Test AccessibilityActionResult model."""

    def test_success_result(self) -> None:
        """Action result for successful action."""
        result = AccessibilityActionResult(
            success=True,
            ref="@e5",
            action="click",
            element_name="Submit",
            element_role="button",
        )
        assert result.success is True
        assert result.ref == "@e5"
        assert result.action == "click"
        assert result.error is None
        assert result.element_name == "Submit"
        assert result.element_role == "button"

    def test_failure_result(self) -> None:
        """Action result for failed action."""
        result = AccessibilityActionResult(
            success=False,
            ref="@e10",
            action="type",
            error="Element not found: @e10",
        )
        assert result.success is False
        assert result.ref == "@e10"
        assert result.action == "type"
        assert result.error == "Element not found: @e10"
        assert result.element_name is None

    def test_serialization_round_trip(self) -> None:
        """ActionResult serializes and deserializes correctly."""
        result = AccessibilityActionResult(
            success=True,
            ref="@e1",
            action="focus",
            element_name="Search",
            element_role="searchbox",
        )
        data = result.model_dump()
        restored = AccessibilityActionResult.model_validate(data)
        assert restored.success is True
        assert restored.ref == "@e1"
        assert restored.element_role == "searchbox"


class TestJsonSerialization:
    """Test JSON serialization for all models."""

    def test_snapshot_json_round_trip(self) -> None:
        """Complete snapshot survives JSON round-trip."""
        root = AccessibilityNode(
            ref="@e1",
            role=AccessibilityRole.DOCUMENT,
            name="Test Page",
            bounds=AccessibilityBounds(x=0, y=0, width=1920, height=1080),
            state=AccessibilityState(is_focused=True),
            children=[
                AccessibilityNode(
                    ref="@e2",
                    role=AccessibilityRole.BUTTON,
                    name="Click Me",
                    is_interactive=True,
                    bounds=AccessibilityBounds(x=100, y=100, width=120, height=40),
                )
            ],
        )
        snapshot = AccessibilitySnapshot(
            root=root,
            timestamp=1700000000.0,
            backend=AccessibilityBackend.CDP,
            url="https://example.com",
            title="Test Page",
            total_nodes=2,
            interactive_nodes=1,
        )

        # Serialize to JSON string
        json_str = snapshot.model_dump_json()

        # Deserialize from JSON string
        restored = AccessibilitySnapshot.model_validate_json(json_str)

        assert restored.url == "https://example.com"
        assert restored.root.name == "Test Page"
        assert len(restored.root.children) == 1
        assert restored.root.children[0].name == "Click Me"
        assert restored.root.bounds is not None
        assert restored.root.bounds.center_x == 960

    def test_selector_json_round_trip(self) -> None:
        """Selector with nested ancestor survives JSON round-trip."""
        selector = AccessibilitySelector(
            role=[AccessibilityRole.BUTTON, AccessibilityRole.LINK],
            name_contains="submit",
            is_interactive=True,
            ancestor=AccessibilitySelector(
                role=AccessibilityRole.FORM,
                automation_id="login-form",
            ),
        )

        json_str = selector.model_dump_json()
        restored = AccessibilitySelector.model_validate_json(json_str)

        assert isinstance(restored.role, list)
        assert len(restored.role) == 2
        assert restored.ancestor is not None
        assert restored.ancestor.role == AccessibilityRole.FORM
        assert restored.ancestor.automation_id == "login-form"
