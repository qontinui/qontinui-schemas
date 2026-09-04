/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { ElementBbox } from "./ElementBbox";
import type { ElementIdentifier } from "./ElementIdentifier";
import type { ElementRect } from "./ElementRect";
import type { ElementState } from "./ElementState";

/**
 * A registered element in the UI Bridge registry.
 *
 * This is the serializable subset of the React `RegisteredElement`; it
 * includes identity, available actions, current state, and lifecycle info.
 */
export interface UIBridgeElement {
  /**
   * W3C "accessible name" per the accessible-name algorithm. Distinct from
   * aria_label because the algorithm may consult aria-labelledby, associated
   * label elements, title, or visible content. Source of truth for
   * IrElementCriteria.accessible_name.
   */
  accessibleName?: string | null;
  /**
   * Standard actions available on this element.
   */
  actions?: string[];
  /**
   * Computed aria-label (explicit attribute, falling back to aria-labelledby
   * reference resolution). Source of truth for IrElementCriteria.aria_label.
   */
  ariaLabel?: string | null;
  /**
   * Viewport-relative bounding box in CSS pixels, when the SDK has a
   * live DOM ref. Absent for elements registered without a ref or when
   * the snapshot is served from the DOM-fallback scanner.
   */
  bbox?: ElementBbox | null;
  /**
   * Custom (application-defined) actions.
   *
   * **Objects, not names.** These used to be a bare `Vec<String>` of action
   * names, which left an element-level custom action with nowhere to carry a
   * safety class — so the walker had no way to tell a `refresh` from a
   * `sendKeys` that writes raw bytes into a live agent PTY. Widening this to
   * [`ElementActionInfo`] is what makes `effect` REACHABLE from an element;
   * see that type's docs.
   *
   * Absent (`None`) means the element declares none. That is deliberately
   * distinct from an empty list, which would mean "declared, and empty".
   */
  customActions?: ElementActionInfo[] | null;
  /**
   * Unique element ID within the registry.
   */
  id: string;
  /**
   * Identifier bundle for locating the element.
   */
  identifier: ElementIdentifier;
  /**
   * Human-readable label for the element.
   */
  label?: string | null;
  /**
   * Whether the element's React component is currently mounted.
   */
  mounted: boolean;
  /**
   * Unix-epoch millisecond timestamp when the element was registered.
   */
  registeredAt: number;
  /**
   * ARIA role of the element (explicit `role=` or implicit per W3C ARIA-in-HTML).
   * Populated by the SDK's element walker. Source of truth for IrElementCriteria.role.
   */
  role?: string | null;
  /**
   * Current observable state.
   */
  state: ElementState;
  /**
   * HTML tag name in lowercase. Source of truth for IrElementCriteria.tag_name.
   */
  tagName?: string | null;
  /**
   * Visible text content with whitespace collapsed (DOM innerText-equivalent
   * on web; accessibilityLabel/text equivalent on native). Source of truth for
   * IrElementCriteria.text and text_contains. Distinct from state.text_content
   * which is a snapshot of the form-control value.
   */
  text?: string | null;
  /**
   * Element type (e.g. `"button"`, `"input"`, `"select"`).
   */
  type: string;
  /**
   * Cheap viewport-visibility signal derived by the SDK as
   * `bbox.width > 0 && bbox.height > 0`. Use `state.visible` for the
   * richer occlusion check.
   */
  visible?: boolean | null;
}
/**
 * Information about a single custom (application-defined) action exposed by a
 * UI Bridge **element**.
 *
 * The element-level twin of [`ComponentActionInfo`], and deliberately the same
 * field vocabulary so a consumer that walks both action planes reads one shape,
 * not two. One field is NOT carried over: `path`. A component action has a
 * resolved per-action invocation path because the SDK annotates one
 * (`/control/component/<componentId>/action/<actionId>`); an element action has
 * no analogue — every element action is invoked through the single
 * `/control/element/<elementId>/action` route with the action name in the body.
 * A field nothing would ever populate is dead schema surface, so it is absent
 * rather than perpetually `null`.
 *
 * ## Why this is an object rather than a name
 *
 * It exists so `effect` has somewhere to live. A bare list of action NAMES is
 * unannotatable: there is no slot on a name for a safety class, so an
 * element-level custom action could not declare itself `destructive` however
 * much its author wanted to. That is not hypothetical — the runner exposes
 * element custom actions that write raw bytes into a live agent PTY, and some
 * of them act on a pane with no visible view.
 *
 * The wire projection is widened FIRST and the annotation lands on top of it.
 * The reverse order ships an UNREACHABLE annotation, which is worse than an
 * absent one: the author believes the delete button is marked, and the walker
 * walks it anyway.
 */
export interface ElementActionInfo {
  /**
   * Longer description of what the action does.
   */
  description?: string | null;
  /**
   * Safety class of this action: `read`, `write` or `destructive`.
   *
   * **A safety annotation, not a hint.** An autonomous walk MUST NOT fire an
   * action annotated `destructive`; that is the same exclusion the IR already
   * applies to `destructive` transitions when generating auto-regressions.
   * Author-declared, because only the app author knows that a particular
   * custom action sends the keystrokes that end a session.
   *
   * Absent means **unclassified, not safe** — an action nobody has judged
   * must be treated as unknown rather than as `read`. `skip_serializing_if`
   * keeps an un-annotated action ABSENT on the wire rather than defaulting to
   * a class nobody chose.
   */
  effect?: ("read" | "write" | "destructive") | null;
  /**
   * Unique action identifier within the element — the key the action is
   * registered under, and the name `/control/element/<id>/action` is called
   * with.
   */
  id: string;
  /**
   * Human-readable label.
   */
  label?: string | null;
  /**
   * Free-form declaration of the action's parameters, author-supplied on the
   * SDK side and surfaced verbatim. Conventionally a small JSON Schema
   * subset, but the SDK does not constrain the shape, so it is carried
   * through untyped — the same treatment `ElementActionRequest::params` and
   * `ComponentActionInfo::param_schema` get.
   */
  paramSchema?: {
    [k: string]: unknown;
  };
}
