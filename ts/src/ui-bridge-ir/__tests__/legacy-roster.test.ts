/**
 * Legacy spec roster test (Phase A2 fixture gate).
 *
 * Walks the entire `qontinui_parent` monorepo, collects every shipping
 * `*.spec.uibridge.json`, and round-trips each through:
 *
 *     legacy --projectLegacyToIR-->  IR  --projectIRToBundledPage-->  legacy'
 *
 * For each fixture we assert (semantically — not byte-strict):
 *   - same number of groups
 *   - same group ids in input order
 *   - same total assertion count across all groups
 *
 * This is coarse but effective: it catches any data loss in the inverse
 * projection without forcing the inverse to preserve hand-edited assertion
 * ids / severity / category / etc. (which the forward direction regenerates
 * from defaults). Phase A4 will tighten the contract for hand-authored
 * fields.
 *
 * The walker fails fast if it finds < 100 fixtures (canary against a
 * misconfigured directory walk) — current expected count is 121 (96 runner +
 * 19 web + 5 mobile + 2 supervisor) as of 2026-04. The number is allowed to
 * grow; the lower bound just guards against accidentally testing nothing.
 */

import { readdirSync, readFileSync, statSync } from "node:fs";
import path from "node:path";

import { describe, expect, it } from "vitest";

import {
  projectIRToBundledPage,
  projectLegacyToIR,
  type LegacySpec,
} from "../projection";

// ---------------------------------------------------------------------------
// Discovery
// ---------------------------------------------------------------------------

/**
 * Resolve the monorepo root from the test file's location. The test file
 * lives at `qontinui-schemas/ts/src/ui-bridge-ir/__tests__/`, so five
 * `..` segments take us to `qontinui_parent/`.
 *
 * Use `process.cwd()` as a fallback only when the test runs from outside the
 * usual layout (e.g. CI checkout with renamed parent) — vitest sets cwd to
 * the package root by default.
 */
const MONOREPO_ROOT = path.resolve(
  __dirname,
  "..",
  "..",
  "..",
  "..",
  "..",
);

// Directories to skip. Note: do NOT include "build" — Next.js apps have route
// folders named `build/` (e.g., qontinui-web/frontend/src/app/(app)/build/)
// that legitimately contain spec fixtures.
const SKIP_DIRS = new Set([
  "node_modules",
  "target",
  "target-agent",
  "dist",
  ".git",
  ".next",
  ".venv",
  "__pycache__",
  "out",
]);

/**
 * Recursively walk `root`, collecting every `*.spec.uibridge.json` file.
 * Skips the directory names in `SKIP_DIRS`. Synchronous; the fixture set is
 * small (low hundreds) so we don't bother with worker threads.
 */
function collectSpecs(root: string): string[] {
  const out: string[] = [];
  const stack: string[] = [root];
  while (stack.length > 0) {
    const current = stack.pop()!;
    let entries: import("node:fs").Dirent[];
    try {
      entries = readdirSync(current, { withFileTypes: true });
    } catch {
      // Permission errors / vanished dirs — skip silently.
      continue;
    }
    for (const entry of entries) {
      if (SKIP_DIRS.has(entry.name)) continue;
      const full = path.join(current, entry.name);
      if (entry.isDirectory()) {
        stack.push(full);
      } else if (entry.isFile() && entry.name.endsWith(".spec.uibridge.json")) {
        out.push(full);
      } else if (entry.isSymbolicLink()) {
        // Resolve symlinks defensively — some plugin checkouts use them.
        try {
          const stat = statSync(full);
          if (stat.isDirectory()) {
            stack.push(full);
          } else if (stat.isFile() && entry.name.endsWith(".spec.uibridge.json")) {
            out.push(full);
          }
        } catch {
          // ignore
        }
      }
    }
  }
  // Sort for deterministic test ordering.
  out.sort();
  return out;
}

const SPECS = collectSpecs(MONOREPO_ROOT);

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe("legacy spec roster — discovery sanity", () => {
  it("finds at least 100 *.spec.uibridge.json files in the monorepo", () => {
    expect(SPECS.length).toBeGreaterThanOrEqual(100);
  });

  it("does not include node_modules / dist / target paths", () => {
    for (const f of SPECS) {
      expect(f).not.toMatch(/[\\/](?:node_modules|dist|target|\.git)[\\/]/);
    }
  });
});

describe("legacy spec roster — round-trip via IR", () => {
  // Use it.each so every fixture gets its own test name and failures pinpoint
  // the exact file. Pre-load each fixture as a tuple for the iterator.
  const cases: ReadonlyArray<readonly [string, string]> = SPECS.map((p) => [
    path.relative(MONOREPO_ROOT, p),
    p,
  ] as const);

  it.each(cases)("%s round-trips through IR without data loss", (_relpath, full) => {
    let raw = readFileSync(full, "utf-8");
    // Strip UTF-8 BOM if present — at least one shipping spec
    // (settings-security.spec.uibridge.json) was saved with a BOM.
    if (raw.charCodeAt(0) === 0xfeff) raw = raw.slice(1);
    const legacy = JSON.parse(raw) as LegacySpec;

    const ir = projectLegacyToIR(legacy);
    const projected = projectIRToBundledPage(ir);

    // 1. Same number of groups.
    expect(projected.groups.length).toBe(legacy.groups.length);

    // 2. Same group ids in input order.
    expect(projected.groups.map((g) => g.id)).toEqual(legacy.groups.map((g) => g.id));

    // 3. Same total assertion count across all groups (coarse data-loss check).
    const projectedAssertionCount = projected.groups.reduce(
      (sum, g) => sum + g.assertions.length,
      0,
    );
    const legacyAssertionCount = legacy.groups.reduce(
      (sum, g) => sum + (Array.isArray(g.assertions) ? g.assertions.length : 0),
      0,
    );
    // The forward projection emits a placeholder assertion for any state with
    // zero requiredElements. Real legacy fixtures never have zero-assertion
    // groups, but be defensive: allow projected >= legacy when legacy had
    // groups with zero assertions.
    const legacyHasEmptyGroup = legacy.groups.some(
      (g) => !Array.isArray(g.assertions) || g.assertions.length === 0,
    );
    if (legacyHasEmptyGroup) {
      expect(projectedAssertionCount).toBeGreaterThanOrEqual(legacyAssertionCount);
    } else {
      expect(projectedAssertionCount).toBe(legacyAssertionCount);
    }
  });
});
