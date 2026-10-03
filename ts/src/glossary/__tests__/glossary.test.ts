import { describe, expect, it } from "vitest";

import {
  GLOSSARY,
  GLOSSARY_CONTENT_SHA256,
  GLOSSARY_TERMS,
  GLOSSARY_VERSION,
  glossaryEntry,
  isGlossaryTerm,
  lookupGlossaryTerm,
} from "../index";

describe("glossary table", () => {
  it("lists every keyed entry exactly once, in order", () => {
    expect([...GLOSSARY_TERMS]).toEqual(Object.keys(GLOSSARY));
    expect(new Set(GLOSSARY_TERMS).size).toBe(GLOSSARY_TERMS.length);
    expect(GLOSSARY_TERMS.length).toBeGreaterThanOrEqual(20);
  });

  it("keys each entry by its own id, with no dangling see_also", () => {
    for (const id of GLOSSARY_TERMS) {
      const e = GLOSSARY[id];
      expect(e.id).toBe(id);
      expect(e.term.trim()).not.toBe("");
      expect([...e.short].length).toBeLessThanOrEqual(160);
      expect([...e.long].length).toBeLessThanOrEqual(1200);
      for (const s of e.see_also) expect(isGlossaryTerm(s)).toBe(true);
      expect(e.since).toBeGreaterThanOrEqual(1);
      expect(e.since).toBeLessThanOrEqual(GLOSSARY_VERSION);
    }
  });

  it("carries its version and content digest", () => {
    expect(GLOSSARY_VERSION).toBeGreaterThanOrEqual(1);
    expect(GLOSSARY_CONTENT_SHA256).toMatch(/^[0-9a-f]{64}$/);
  });

  it("looks up by id and refuses what it does not define", () => {
    expect(glossaryEntry("gate").id).toBe("gate");
    expect(lookupGlossaryTerm("work_unit")?.term).toBe(GLOSSARY.work_unit.term);
    expect(lookupGlossaryTerm("no_such_term")).toBeNull();
    // Own keys only: an inherited property is not a term.
    expect(isGlossaryTerm("toString")).toBe(false);
    expect(isGlossaryTerm("__proto__")).toBe(false);
  });
});
