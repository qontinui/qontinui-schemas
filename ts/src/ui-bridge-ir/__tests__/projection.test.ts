/**
 * Tests for `projectIRToBundledPage`.
 *
 * Covers:
 *   - Determinism (byte-identical output across two calls)
 *   - Round-trip-against-active.spec (key-field shape match)
 *   - Notes injection (descriptive paragraph append)
 *   - Empty cases (zero states, zero requiredElements, zero actions)
 *   - Criteria conversion (text -> textContent, ariaLabel -> accessibleName,
 *     attributes -> dataAttributes)
 */

import { describe, expect, it } from "vitest";

import type { IRDocument } from "../document";
import {
  projectIRToBundledPage,
  projectionVersion,
  type LegacySpec,
} from "../projection";

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/**
 * Slice of `qontinui-runner/src/specs/active.spec.uibridge.json` mapped back
 * to IR. Three to four states, transitions between them, a few criteria
 * shapes (role, textContent/text, accessibleName/ariaLabel, dataAttributes).
 */
const ACTIVE_PAGE_FIXTURE: IRDocument = {
  version: "1.0",
  id: "active",
  name: "Active Dashboard",
  description:
    "Active Dashboard page -- the real-time monitoring hub for running workflows.",
  metadata: {
    tags: ["active-dashboard", "monitoring", "real-time", "tier-1"],
  },
  initialState: "idle",
  states: [
    {
      id: "idle",
      name: "Idle",
      description: "No workflow is running.",
      requiredElements: [{ text: "No Active Workflow" }],
      isInitial: true,
    },
    {
      id: "running",
      name: "Running",
      description: "A workflow is actively executing.",
      requiredElements: [{ role: "button", text: "Stop" }],
      isInitial: false,
    },
    {
      id: "paused",
      name: "Paused",
      description: "The workflow is temporarily paused.",
      requiredElements: [{ role: "button", ariaLabel: "Play" }],
      isInitial: false,
    },
    {
      id: "breakpoint-paused",
      name: "Breakpoint Paused",
      description:
        "A workflow step with breakpoint: true has completed; execution is suspended.",
      requiredElements: [
        { text: "Breakpoint" },
        { attributes: { "content-label": "breakpoint step" } },
      ],
      isInitial: false,
    },
  ],
  transitions: [
    {
      id: "idle-to-running",
      name: "Start a workflow",
      fromStates: ["idle"],
      activateStates: ["running"],
      exitStates: ["idle"],
      actions: [
        {
          type: "click",
          target: { text: "Run Last Workflow" },
          waitAfter: { type: "idle", timeout: 5000 },
        },
      ],
    },
    {
      id: "running-to-paused",
      name: "Pause via play/pause button",
      fromStates: ["running"],
      activateStates: ["paused"],
      exitStates: ["running"],
      actions: [
        {
          type: "click",
          target: { role: "button", ariaLabel: "Pause" },
          waitAfter: { type: "idle", timeout: 2000 },
        },
      ],
    },
    {
      id: "paused-to-running",
      name: "Resume via play button",
      fromStates: ["paused"],
      activateStates: ["running"],
      exitStates: ["paused"],
      actions: [
        {
          type: "click",
          target: { role: "button", ariaLabel: "Play" },
          waitAfter: { type: "idle", timeout: 2000 },
        },
      ],
    },
  ],
};

// ---------------------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------------------

describe("projectIRToBundledPage — determinism", () => {
  it("produces byte-identical JSON across two calls with the same input", () => {
    const a = JSON.stringify(projectIRToBundledPage(ACTIVE_PAGE_FIXTURE));
    const b = JSON.stringify(projectIRToBundledPage(ACTIVE_PAGE_FIXTURE));
    expect(a).toBe(b);
  });

  it("produces byte-identical JSON regardless of insertion order of object keys in input", () => {
    // Two structurally identical IR docs with different in-memory key order.
    const reordered: IRDocument = {
      // Keys deliberately reordered.
      transitions: ACTIVE_PAGE_FIXTURE.transitions,
      states: ACTIVE_PAGE_FIXTURE.states,
      initialState: ACTIVE_PAGE_FIXTURE.initialState,
      metadata: ACTIVE_PAGE_FIXTURE.metadata,
      description: ACTIVE_PAGE_FIXTURE.description,
      name: ACTIVE_PAGE_FIXTURE.name,
      id: ACTIVE_PAGE_FIXTURE.id,
      version: ACTIVE_PAGE_FIXTURE.version,
    };

    const a = JSON.stringify(projectIRToBundledPage(ACTIVE_PAGE_FIXTURE));
    const b = JSON.stringify(projectIRToBundledPage(reordered));
    expect(a).toBe(b);
  });

  it("sorts keys lexicographically at the top level", () => {
    const out = projectIRToBundledPage(ACTIVE_PAGE_FIXTURE);
    const keys = Object.keys(out);
    expect(keys).toEqual([...keys].sort());
  });
});

// ---------------------------------------------------------------------------
// Round-trip against active.spec
// ---------------------------------------------------------------------------

describe("projectIRToBundledPage — round-trip against active.spec", () => {
  const out = projectIRToBundledPage(ACTIVE_PAGE_FIXTURE);

  it("produces top-level shape with version, description, groups, stateMachine, metadata", () => {
    expect(out.version).toBe("1.0.0");
    expect(typeof out.description).toBe("string");
    expect(Array.isArray(out.groups)).toBe(true);
    expect(out.stateMachine).toBeDefined();
    expect(Array.isArray(out.stateMachine.states)).toBe(true);
    expect(out.metadata).toBeDefined();
  });

  it("produces one group per IR state, in input order", () => {
    expect(out.groups).toHaveLength(ACTIVE_PAGE_FIXTURE.states.length);
    expect(out.groups.map((g) => g.id)).toEqual(
      ACTIVE_PAGE_FIXTURE.states.map((s) => s.id),
    );
  });

  it("produces one assertion per requiredElement on each group", () => {
    const breakpointGroup = out.groups.find((g) => g.id === "breakpoint-paused");
    expect(breakpointGroup).toBeDefined();
    // breakpoint-paused has two requiredElements -> two assertions.
    expect(breakpointGroup!.assertions).toHaveLength(2);
    expect(breakpointGroup!.assertions[0]!.id).toBe("breakpoint-paused-elem-0");
    expect(breakpointGroup!.assertions[1]!.id).toBe("breakpoint-paused-elem-1");
  });

  it("converts text -> textContent in assertion targets", () => {
    const idleGroup = out.groups.find((g) => g.id === "idle")!;
    const assertion = idleGroup.assertions[0]!;
    expect(assertion.target.criteria).toEqual({ textContent: "No Active Workflow" });
  });

  it("converts ariaLabel -> accessibleName in assertion targets", () => {
    const pausedGroup = out.groups.find((g) => g.id === "paused")!;
    const assertion = pausedGroup.assertions[0]!;
    expect(assertion.target.criteria).toEqual({
      role: "button",
      accessibleName: "Play",
    });
  });

  it("converts attributes -> dataAttributes in assertion targets", () => {
    const bpGroup = out.groups.find((g) => g.id === "breakpoint-paused")!;
    // Second requiredElement uses attributes.
    const assertion = bpGroup.assertions[1]!;
    expect(assertion.target.criteria).toEqual({
      dataAttributes: { "content-label": "breakpoint step" },
    });
  });

  it("emits assertions with required legacy fields", () => {
    for (const group of out.groups) {
      for (const assertion of group.assertions) {
        expect(assertion.id).toBeTruthy();
        expect(assertion.description).toBeTruthy();
        expect(assertion.category).toBe("element-presence");
        expect(assertion.severity).toBe("critical");
        expect(assertion.assertionType).toBe("exists");
        expect(assertion.source).toBe("ai-generated");
        expect(assertion.reviewed).toBe(false);
        expect(assertion.enabled).toBe(true);
        expect(assertion.target.type).toBe("search");
        expect(assertion.target.label).toContain(
          // each target.label includes the state name
          group.name,
        );
      }
    }
  });

  it("places transitions under each fromState in stateMachine.states", () => {
    const idleSm = out.stateMachine.states.find((s) => s.id === "idle")!;
    expect(idleSm.transitions.map((t) => t.id)).toEqual(["idle-to-running"]);

    const runningSm = out.stateMachine.states.find((s) => s.id === "running")!;
    expect(runningSm.transitions.map((t) => t.id)).toEqual(["running-to-paused"]);

    const pausedSm = out.stateMachine.states.find((s) => s.id === "paused")!;
    expect(pausedSm.transitions.map((t) => t.id)).toEqual(["paused-to-running"]);
  });

  it("derives staysVisible from exitStates length (false when exitStates non-empty)", () => {
    const idleSm = out.stateMachine.states.find((s) => s.id === "idle")!;
    const t = idleSm.transitions[0]!;
    expect(t.deactivateStates).toEqual(["idle"]);
    expect(t.staysVisible).toBe(false);
  });

  it("renames action.type -> process[].action and converts target criteria", () => {
    const idleSm = out.stateMachine.states.find((s) => s.id === "idle")!;
    const proc = idleSm.transitions[0]!.process[0]!;
    expect(proc.action).toBe("click");
    expect(proc.target).toEqual({ textContent: "Run Last Workflow" });
    expect(proc.waitAfter).toEqual({ type: "idle", timeout: 5000 });
  });

  it("uses initialState fallback when state.isInitial is undefined", () => {
    const noExplicit: IRDocument = {
      version: "1.0",
      id: "x",
      name: "X",
      initialState: "first",
      states: [
        {
          id: "first",
          name: "First",
          requiredElements: [{ role: "heading" }],
        },
        {
          id: "second",
          name: "Second",
          requiredElements: [{ role: "button" }],
        },
      ],
      transitions: [],
    };
    const result = projectIRToBundledPage(noExplicit);
    expect(result.stateMachine.states.find((s) => s.id === "first")!.isInitial).toBe(
      true,
    );
    expect(
      result.stateMachine.states.find((s) => s.id === "second")!.isInitial,
    ).toBe(false);
  });

  it("propagates document tags into metadata.tags", () => {
    expect(out.metadata.tags).toEqual([
      "active-dashboard",
      "monitoring",
      "real-time",
      "tier-1",
    ]);
  });

  it("uses doc.id as metadata.component when no purpose is set", () => {
    expect(out.metadata.component).toBe("active");
  });

  it("uses metadata.purpose as metadata.component when present", () => {
    const withPurpose: IRDocument = {
      ...ACTIVE_PAGE_FIXTURE,
      metadata: { ...ACTIVE_PAGE_FIXTURE.metadata, purpose: "ActiveDashboardPage" },
    };
    const result = projectIRToBundledPage(withPurpose);
    expect(result.metadata.component).toBe("ActiveDashboardPage");
  });

  it("uses doc.description as the top-level description", () => {
    expect(out.description).toBe(ACTIVE_PAGE_FIXTURE.description);
  });
});

// ---------------------------------------------------------------------------
// Notes injection
// ---------------------------------------------------------------------------

describe("projectIRToBundledPage — notes injection", () => {
  it("appends notes as a separate paragraph (two-newline separator)", () => {
    const out = projectIRToBundledPage(
      ACTIVE_PAGE_FIXTURE,
      "Reviewed by Joshua on 2026-04-28.",
    );
    expect(out.description).toBe(
      `${ACTIVE_PAGE_FIXTURE.description}\n\nReviewed by Joshua on 2026-04-28.`,
    );
  });

  it("treats undefined notes as no-op (no trailing newlines)", () => {
    const out = projectIRToBundledPage(ACTIVE_PAGE_FIXTURE);
    expect(out.description).toBe(ACTIVE_PAGE_FIXTURE.description);
    expect(out.description.endsWith("\n")).toBe(false);
  });

  it("treats empty-string notes as no-op", () => {
    const out = projectIRToBundledPage(ACTIVE_PAGE_FIXTURE, "");
    expect(out.description).toBe(ACTIVE_PAGE_FIXTURE.description);
  });

  it("falls back to doc.name when doc.description is missing", () => {
    const noDesc: IRDocument = {
      version: "1.0",
      id: "p",
      name: "Page",
      states: [],
      transitions: [],
    };
    const out = projectIRToBundledPage(noDesc, "added later");
    expect(out.description).toBe("Page\n\nadded later");
  });
});

// ---------------------------------------------------------------------------
// Empty cases
// ---------------------------------------------------------------------------

describe("projectIRToBundledPage — empty cases", () => {
  it("handles a document with zero states", () => {
    const empty: IRDocument = {
      version: "1.0",
      id: "empty",
      name: "Empty",
      states: [],
      transitions: [],
    };
    const out = projectIRToBundledPage(empty);
    expect(out.groups).toEqual([]);
    expect(out.stateMachine.states).toEqual([]);
    expect(out.version).toBe("1.0.0");
    expect(out.metadata.component).toBe("empty");
  });

  it("emits one placeholder assertion when a state has zero requiredElements", () => {
    const doc: IRDocument = {
      version: "1.0",
      id: "p",
      name: "P",
      states: [
        {
          id: "no-req",
          name: "No Required Elements",
          requiredElements: [],
        },
      ],
      transitions: [],
    };
    const out = projectIRToBundledPage(doc);
    expect(out.groups).toHaveLength(1);
    const group = out.groups[0]!;
    expect(group.assertions).toHaveLength(1);
    expect(group.assertions[0]!.id).toBe("no-req-elem-0");
    expect(group.assertions[0]!.target.criteria).toEqual({});
    expect(group.assertions[0]!.target.label).toBe(
      "Required element for No Required Elements",
    );
  });

  it("handles a state with empty-string text criteria (still emits an assertion)", () => {
    const doc: IRDocument = {
      version: "1.0",
      id: "p",
      name: "P",
      states: [
        {
          id: "empty-text",
          name: "Empty Text",
          requiredElements: [{ text: "" }],
        },
      ],
      transitions: [],
    };
    const out = projectIRToBundledPage(doc);
    expect(out.groups[0]!.assertions).toHaveLength(1);
    expect(out.groups[0]!.assertions[0]!.target.criteria).toEqual({ textContent: "" });
  });

  it("handles a transition with no actions (process: [])", () => {
    const doc: IRDocument = {
      version: "1.0",
      id: "p",
      name: "P",
      states: [
        { id: "a", name: "A", requiredElements: [{ role: "heading" }] },
        { id: "b", name: "B", requiredElements: [{ role: "heading" }] },
      ],
      transitions: [
        {
          id: "auto",
          name: "Auto transition",
          fromStates: ["a"],
          activateStates: ["b"],
          exitStates: ["a"],
          actions: [],
        },
      ],
    };
    const out = projectIRToBundledPage(doc);
    const aSm = out.stateMachine.states.find((s) => s.id === "a")!;
    expect(aSm.transitions[0]!.process).toEqual([]);
    expect(aSm.transitions[0]!.staysVisible).toBe(false);
  });

  it("derives staysVisible=true for a transition with no exitStates (modal-style)", () => {
    const doc: IRDocument = {
      version: "1.0",
      id: "p",
      name: "P",
      states: [
        { id: "base", name: "Base", requiredElements: [{ role: "heading" }] },
        { id: "modal", name: "Modal", requiredElements: [{ role: "dialog" }] },
      ],
      transitions: [
        {
          id: "open-modal",
          name: "Open modal",
          fromStates: ["base"],
          activateStates: ["modal"],
          // No exitStates -> modal-style; base stays visible underneath.
          actions: [{ type: "click", target: { role: "button" } }],
        },
      ],
    };
    const out = projectIRToBundledPage(doc);
    const baseSm = out.stateMachine.states.find((s) => s.id === "base")!;
    expect(baseSm.transitions[0]!.deactivateStates).toEqual([]);
    expect(baseSm.transitions[0]!.staysVisible).toBe(true);
  });
});

// ---------------------------------------------------------------------------
// Provenance + state-level fields
// ---------------------------------------------------------------------------

describe("projectIRToBundledPage — provenance + state metadata", () => {
  it("uses state.provenance.source for group.source when present", () => {
    const doc: IRDocument = {
      version: "1.0",
      id: "p",
      name: "P",
      states: [
        {
          id: "s",
          name: "S",
          requiredElements: [{ role: "heading" }],
          provenance: { source: "hand-authored", file: "p.tsx" },
        },
      ],
      transitions: [],
    };
    const out = projectIRToBundledPage(doc);
    expect(out.groups[0]!.source).toBe("hand-authored");
  });

  it("defaults group.source to ai-generated when provenance is missing", () => {
    const doc: IRDocument = {
      version: "1.0",
      id: "p",
      name: "P",
      states: [
        {
          id: "s",
          name: "S",
          requiredElements: [{ role: "heading" }],
        },
      ],
      transitions: [],
    };
    const out = projectIRToBundledPage(doc);
    expect(out.groups[0]!.source).toBe("ai-generated");
  });

  it("uses state.metadata.description as group.description when state.description is missing", () => {
    const doc: IRDocument = {
      version: "1.0",
      id: "p",
      name: "P",
      states: [
        {
          id: "s",
          name: "S",
          requiredElements: [{ role: "heading" }],
          metadata: { description: "From metadata" },
        },
      ],
      transitions: [],
    };
    const out = projectIRToBundledPage(doc);
    expect(out.groups[0]!.description).toBe("From metadata");
  });

  it("falls through to empty string when neither state.description nor state.metadata.description set", () => {
    const doc: IRDocument = {
      version: "1.0",
      id: "p",
      name: "P",
      states: [
        {
          id: "s",
          name: "S",
          requiredElements: [{ role: "heading" }],
        },
      ],
      transitions: [],
    };
    const out = projectIRToBundledPage(doc);
    expect(out.groups[0]!.description).toBe("");
  });
});

// ---------------------------------------------------------------------------
// Type / version export
// ---------------------------------------------------------------------------

describe("projectionVersion", () => {
  it('exports "1.0"', () => {
    expect(projectionVersion).toBe("1.0");
  });
});

// ---------------------------------------------------------------------------
// Type witness — confirms LegacySpec is a structural shape with the legacy fields
// ---------------------------------------------------------------------------

describe("LegacySpec — structural witness", () => {
  it("matches the legacy shape on a minimal projection", () => {
    const minimal: IRDocument = {
      version: "1.0",
      id: "m",
      name: "M",
      states: [],
      transitions: [],
    };
    const out: LegacySpec = projectIRToBundledPage(minimal);
    expect(out).toMatchObject({
      version: "1.0.0",
      description: "M",
      groups: [],
      stateMachine: { states: [] },
      metadata: { component: "m" },
    });
  });
});
