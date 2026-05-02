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
  projectLegacyToIR,
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

  it("projects a state with zero requiredElements to a group with empty assertions[]", () => {
    // Legacy specs ship architecture-only / documentation-only groups with
    // empty `assertions[]` (e.g. `graphql-resolver-inventory` in the runner's
    // `graphql-infrastructure.spec.uibridge.json`). The forward projection
    // must preserve that emptiness so the legacy<->IR round-trip is identity
    // for assertion counts; emitting a placeholder here silently inflates the
    // count and trips `check-spec-pairing` with an `assertionCount` mismatch.
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
    expect(group.assertions).toEqual([]);
    // The state still appears in the state-machine block — emptiness is local
    // to `groups[].assertions[]`, not state existence.
    expect(out.stateMachine.states).toHaveLength(1);
    expect(out.stateMachine.states[0]!.id).toBe("no-req");
  });

  it("preserves empty groups across a legacy -> IR -> legacy round-trip (regression: graphql-infrastructure)", () => {
    // Reproduces the on-disk shape of the four runner specs that previously
    // produced an `assertionCount` mismatch in `check-spec-pairing`: a group
    // with empty `assertions[]` and a matching state-machine state with
    // empty `elements[]`. Round-trip identity proves the placeholder
    // re-emission bug stays fixed.
    const legacy = {
      version: "1.0.0",
      description: "Architecture-only spec with empty groups",
      metadata: { component: "regression" },
      groups: [
        {
          id: "non-empty",
          name: "Has assertions",
          description: "",
          category: "element-presence",
          assertions: [
            {
              id: "a1",
              description: "First assertion",
              category: "element-presence",
              severity: "critical",
              assertionType: "exists",
              target: {
                type: "search",
                criteria: { textContent: "Run" },
                label: "Run button",
              },
              source: "ai-generated",
              reviewed: false,
              enabled: true,
            },
          ],
          source: "ai-generated",
        },
        {
          id: "empty",
          name: "Empty architecture group",
          description: "",
          category: "element-presence",
          assertions: [],
          source: "ai-generated",
        },
      ],
      stateMachine: {
        states: [
          {
            id: "non-empty",
            name: "Has assertions",
            description: "",
            elements: [{ textContent: "Run" }],
            isInitial: true,
            transitions: [],
          },
          {
            id: "empty",
            name: "Empty architecture group",
            description: "",
            elements: [],
            isInitial: false,
            transitions: [],
          },
        ],
      },
    } as unknown as Parameters<typeof projectLegacyToIR>[0];

    const ir = projectLegacyToIR(legacy, { docId: "regression" });
    const projected = projectIRToBundledPage(ir);

    expect(projected.groups).toHaveLength(2);
    expect(projected.groups[0]!.id).toBe("non-empty");
    expect(projected.groups[0]!.assertions).toHaveLength(1);
    expect(projected.groups[1]!.id).toBe("empty");
    expect(projected.groups[1]!.assertions).toEqual([]);

    // Total assertion count is identity across the round-trip.
    const legacyTotal = legacy.groups.reduce(
      (s: number, g: { assertions: unknown[] }) => s + g.assertions.length,
      0,
    );
    const projectedTotal = projected.groups.reduce(
      (s, g) => s + g.assertions.length,
      0,
    );
    expect(projectedTotal).toBe(legacyTotal);
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

// ---------------------------------------------------------------------------
// Inverse projection: legacy -> IR (Phase A2)
// ---------------------------------------------------------------------------

/**
 * Hand-authored minimal legacy fixture for the inverse direction. Mirrors a
 * trimmed `settings-general.spec.uibridge.json`-style shape with two groups,
 * a state-machine block, transitions with action sequences, and a
 * precondition on one assertion.
 */
const LEGACY_FIXTURE: LegacySpec = {
  version: "1.0.0",
  description: "Hand-authored test fixture for inverse projection.",
  groups: [
    {
      id: "alpha",
      name: "Alpha Group",
      description: "First group for the round-trip test.",
      category: "element-presence",
      assertions: [
        {
          id: "alpha-elem-0",
          description: "Heading exists",
          category: "element-presence",
          severity: "critical",
          assertionType: "exists",
          target: {
            type: "search",
            criteria: { role: "heading", textContent: "Alpha" },
            label: "Alpha heading",
          },
          source: "manual",
          reviewed: true,
          enabled: true,
        },
        {
          id: "alpha-elem-1",
          description: "Subtitle exists",
          category: "element-presence",
          severity: "warning",
          assertionType: "exists",
          target: {
            type: "search",
            criteria: { textContent: "Welcome" },
            label: "Subtitle",
          },
          source: "manual",
          reviewed: true,
          enabled: true,
          precondition: "Page is loaded.",
        },
      ],
      source: "manual",
    },
    {
      id: "beta",
      name: "Beta Group",
      description: "Second group, modal-style.",
      category: "element-presence",
      assertions: [
        {
          id: "beta-elem-0",
          description: "Dialog exists",
          category: "element-presence",
          severity: "critical",
          assertionType: "exists",
          target: {
            type: "search",
            criteria: { role: "dialog", accessibleName: "Confirm" },
            label: "Confirm dialog",
          },
          source: "manual",
          reviewed: true,
          enabled: true,
        },
      ],
      source: "manual",
    },
  ],
  stateMachine: {
    states: [
      {
        id: "alpha",
        name: "Alpha Group",
        description: "First group for the round-trip test.",
        elements: [
          { role: "heading", textContent: "Alpha" },
          { textContent: "Welcome" },
        ],
        isInitial: true,
        transitions: [
          {
            id: "alpha-to-beta",
            name: "Open Beta",
            activateStates: ["beta"],
            deactivateStates: [],
            staysVisible: true,
            process: [
              {
                action: "click",
                target: { role: "button", textContent: "Open" },
                waitAfter: { type: "idle", timeout: 1000 },
              },
            ],
          },
        ],
      },
      {
        id: "beta",
        name: "Beta Group",
        description: "Second group, modal-style.",
        elements: [{ role: "dialog", accessibleName: "Confirm" }],
        isInitial: false,
        transitions: [
          {
            id: "beta-to-alpha",
            name: "Close Beta",
            activateStates: ["alpha"],
            deactivateStates: ["beta"],
            staysVisible: false,
            process: [
              {
                action: "click",
                target: { textContent: "Cancel" },
              },
            ],
          },
        ],
      },
    ],
  },
  metadata: {
    component: "fixture-page",
    tags: ["fixture", "round-trip"],
  },
};

describe("projectLegacyToIR — basic shape", () => {
  it("converts metadata.component to ir.id and propagates tags", () => {
    const ir = projectLegacyToIR(LEGACY_FIXTURE);
    expect(ir.id).toBe("fixture-page");
    expect(ir.metadata?.tags).toEqual(["fixture", "round-trip"]);
  });

  it("uses opts.docId override when supplied", () => {
    const ir = projectLegacyToIR(LEGACY_FIXTURE, { docId: "override" });
    expect(ir.id).toBe("override");
  });

  it("emits one IR state per legacy group, preserving id + name + description", () => {
    const ir = projectLegacyToIR(LEGACY_FIXTURE);
    expect(ir.states).toHaveLength(2);
    expect(ir.states.map((s) => s.id)).toEqual(["alpha", "beta"]);
    const alpha = ir.states.find((s) => s.id === "alpha")!;
    expect(alpha.name).toBe("Alpha Group");
    expect(alpha.description).toBe("First group for the round-trip test.");
  });

  it("sources requiredElements from group.assertions[].target.criteria", () => {
    const ir = projectLegacyToIR(LEGACY_FIXTURE);
    const alpha = ir.states.find((s) => s.id === "alpha")!;
    expect(alpha.requiredElements).toHaveLength(2);
    expect(alpha.requiredElements[0]).toEqual({
      role: "heading",
      text: "Alpha",
    });
    expect(alpha.requiredElements[1]).toEqual({ text: "Welcome" });
  });

  it("inverts dataAttributes -> attributes and accessibleName passthrough", () => {
    const ir = projectLegacyToIR(LEGACY_FIXTURE);
    const beta = ir.states.find((s) => s.id === "beta")!;
    expect(beta.requiredElements[0]).toEqual({
      role: "dialog",
      accessibleName: "Confirm",
    });
  });

  it("lifts the first-found assertion.precondition to state.precondition", () => {
    const ir = projectLegacyToIR(LEGACY_FIXTURE);
    const alpha = ir.states.find((s) => s.id === "alpha")!;
    expect(alpha.precondition).toBe("Page is loaded.");
  });

  it("derives initialState from the first SM state with isInitial:true", () => {
    const ir = projectLegacyToIR(LEGACY_FIXTURE);
    expect(ir.initialState).toBe("alpha");
  });

  it("sets state.isInitial from the SM state", () => {
    const ir = projectLegacyToIR(LEGACY_FIXTURE);
    expect(ir.states.find((s) => s.id === "alpha")!.isInitial).toBe(true);
    expect(ir.states.find((s) => s.id === "beta")!.isInitial).toBe(false);
  });

  it("tags every state and transition with provenance.source = 'migrated'", () => {
    const ir = projectLegacyToIR(LEGACY_FIXTURE);
    for (const s of ir.states) {
      expect(s.provenance?.source).toBe("migrated");
    }
    for (const t of ir.transitions) {
      expect(t.provenance?.source).toBe("migrated");
    }
    expect(ir.provenance?.source).toBe("migrated");
  });

  it("collapses transitions in stateMachine.states[].transitions[] into one IRTransition", () => {
    const ir = projectLegacyToIR(LEGACY_FIXTURE);
    expect(ir.transitions).toHaveLength(2);
    const ids = ir.transitions.map((t) => t.id).sort();
    expect(ids).toEqual(["alpha-to-beta", "beta-to-alpha"]);
  });

  it("inverts process[] back into transition.actions with type/target/waitAfter", () => {
    const ir = projectLegacyToIR(LEGACY_FIXTURE);
    const t = ir.transitions.find((tt) => tt.id === "alpha-to-beta")!;
    expect(t.actions).toHaveLength(1);
    expect(t.actions[0]!.type).toBe("click");
    expect(t.actions[0]!.target).toEqual({ role: "button", text: "Open" });
    expect(t.actions[0]!.waitAfter).toEqual({ type: "idle", timeout: 1000 });
  });

  it("preserves modal-style empty deactivateStates as exitStates: []", () => {
    const ir = projectLegacyToIR(LEGACY_FIXTURE);
    const t = ir.transitions.find((tt) => tt.id === "alpha-to-beta")!;
    expect(t.exitStates).toEqual([]);
  });

  it("handles a legacy spec with no stateMachine block (synthesizes states from groups only)", () => {
    const noSM = {
      ...LEGACY_FIXTURE,
      stateMachine: { states: [] },
    } as LegacySpec;
    const ir = projectLegacyToIR(noSM);
    expect(ir.transitions).toEqual([]);
    expect(ir.states).toHaveLength(2);
    // No isInitial set when SM block is empty.
    expect(ir.initialState).toBeUndefined();
  });
});

// ---------------------------------------------------------------------------
// Round-trip tests
// ---------------------------------------------------------------------------

describe("round-trip — IR -> legacy -> IR (forward then inverse)", () => {
  it("preserves the active-page IR fixture's group/state/transition counts", () => {
    const legacy = projectIRToBundledPage(ACTIVE_PAGE_FIXTURE);
    const ir = projectLegacyToIR(legacy);
    expect(ir.states.length).toBe(ACTIVE_PAGE_FIXTURE.states.length);
    expect(ir.states.map((s) => s.id).sort()).toEqual(
      ACTIVE_PAGE_FIXTURE.states.map((s) => s.id).sort(),
    );
    expect(ir.transitions.length).toBe(ACTIVE_PAGE_FIXTURE.transitions.length);
    expect(ir.transitions.map((t) => t.id).sort()).toEqual(
      ACTIVE_PAGE_FIXTURE.transitions.map((t) => t.id).sort(),
    );
  });

  it("preserves the active-page IR fixture's initialState", () => {
    const legacy = projectIRToBundledPage(ACTIVE_PAGE_FIXTURE);
    const ir = projectLegacyToIR(legacy);
    expect(ir.initialState).toBe("idle");
  });

  it("preserves the active-page IR's tags", () => {
    const legacy = projectIRToBundledPage(ACTIVE_PAGE_FIXTURE);
    const ir = projectLegacyToIR(legacy);
    expect(ir.metadata?.tags).toEqual(ACTIVE_PAGE_FIXTURE.metadata?.tags);
  });

  it("preserves transition fromStates merging when the same transition is emitted under multiple SM states", () => {
    // Forward emits one copy of each transition under each fromState. The
    // active-page fixture only has one fromState per transition, but check
    // the dedup logic doesn't drop transitions when fromStates is single.
    const legacy = projectIRToBundledPage(ACTIVE_PAGE_FIXTURE);
    const ir = projectLegacyToIR(legacy);
    const orig = ACTIVE_PAGE_FIXTURE.transitions.find(
      (t) => t.id === "idle-to-running",
    )!;
    const round = ir.transitions.find((t) => t.id === "idle-to-running")!;
    expect(round.fromStates).toEqual(orig.fromStates);
    expect(round.activateStates).toEqual(orig.activateStates);
  });
});

describe("round-trip — legacy -> IR -> legacy (inverse then forward)", () => {
  it("preserves group count and group ids from a hand-authored legacy fixture", () => {
    const ir = projectLegacyToIR(LEGACY_FIXTURE);
    const back = projectIRToBundledPage(ir);
    expect(back.groups.length).toBe(LEGACY_FIXTURE.groups.length);
    expect(back.groups.map((g) => g.id)).toEqual(
      LEGACY_FIXTURE.groups.map((g) => g.id),
    );
  });

  it("preserves total assertion count", () => {
    const ir = projectLegacyToIR(LEGACY_FIXTURE);
    const back = projectIRToBundledPage(ir);
    const orig = LEGACY_FIXTURE.groups.reduce(
      (n, g) => n + g.assertions.length,
      0,
    );
    const round = back.groups.reduce((n, g) => n + g.assertions.length, 0);
    expect(round).toBe(orig);
  });

  it("preserves transition ids in the SM block", () => {
    const ir = projectLegacyToIR(LEGACY_FIXTURE);
    const back = projectIRToBundledPage(ir);
    const allTids: string[] = [];
    for (const s of back.stateMachine.states) {
      for (const t of s.transitions) {
        if (!allTids.includes(t.id)) allTids.push(t.id);
      }
    }
    const origTids: string[] = [];
    for (const s of LEGACY_FIXTURE.stateMachine.states) {
      for (const t of s.transitions) {
        if (!origTids.includes(t.id)) origTids.push(t.id);
      }
    }
    expect(allTids.sort()).toEqual(origTids.sort());
  });

  it("preserves transition action types", () => {
    const ir = projectLegacyToIR(LEGACY_FIXTURE);
    const back = projectIRToBundledPage(ir);
    const alphaSm = back.stateMachine.states.find((s) => s.id === "alpha")!;
    const t = alphaSm.transitions.find((tt) => tt.id === "alpha-to-beta")!;
    expect(t.process).toHaveLength(1);
    expect(t.process[0]!.action).toBe("click");
    expect(t.process[0]!.target).toEqual({ role: "button", textContent: "Open" });
  });

  it("preserves staysVisible on modal-style transitions (empty deactivateStates)", () => {
    const ir = projectLegacyToIR(LEGACY_FIXTURE);
    const back = projectIRToBundledPage(ir);
    const alphaSm = back.stateMachine.states.find((s) => s.id === "alpha")!;
    const t = alphaSm.transitions.find((tt) => tt.id === "alpha-to-beta")!;
    expect(t.staysVisible).toBe(true);
    expect(t.deactivateStates).toEqual([]);
  });

  it("preserves component + tags in metadata", () => {
    const ir = projectLegacyToIR(LEGACY_FIXTURE);
    const back = projectIRToBundledPage(ir);
    expect(back.metadata.component).toBe("fixture-page");
    expect(back.metadata.tags).toEqual(["fixture", "round-trip"]);
  });
});
