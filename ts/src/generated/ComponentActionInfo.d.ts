/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with `just generate-types` or
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * Information about a single action exposed by a UI Bridge component.
 */
export interface ComponentActionInfo {
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
   * `click` is a delete button.
   *
   * Absent means **unclassified, not safe** — an action nobody has judged
   * must be treated as unknown rather than as `read`.
   */
  effect?: ("read" | "write" | "destructive") | null;
  /**
   * Unique action identifier within the component.
   */
  id: string;
  /**
   * Human-readable label.
   */
  label?: string | null;
  /**
   * Free-form declaration of the action's parameters, author-supplied on the
   * SDK side as `ComponentAction.paramSchema` and surfaced verbatim on
   * `/control/components`. Conventionally a small JSON Schema subset, but the
   * SDK does not constrain the shape, so it is carried through untyped — the
   * same treatment `ElementActionRequest::params` gets.
   */
  paramSchema?: {
    [k: string]: unknown;
  };
  /**
   * Fully-resolved invocation path for this action:
   * `/control/component/<componentId>/action/<actionId>`.
   *
   * **Server-annotated, not author-declared**: the SDK's
   * `annotateComponentWithInvocationPaths` computes it when serving the
   * component listing, so it is present on the wire but absent from anything
   * an app author writes.
   */
  path?: string | null;
}
