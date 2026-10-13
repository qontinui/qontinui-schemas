/**
 * Capture harness for qontinui-schemas' SDK page-health fixtures
 * (rust/tests/fixtures/ui_bridge_sdk/page_health/*.json).
 *
 * Not a test of this SDK: it drives the REAL `diagnosePageHealth` producer
 * (`sdk/page-health`, what `/control/page-health` answers) with EXACTLY the
 * input mapping the in-page handler uses (`server/handlers.ts` `pageHealth`):
 * `elements` handed over as received, `registeredComponents` = the snapshot's
 * `components.length`, `observedAt` = its `timestamp`, `source` =
 * `{ snapshotId }`. It writes the answer's wire bytes (a JSON round-trip, i.e.
 * exactly what goes over HTTP inside the `success` envelope's `data`).
 *
 * Three captures:
 *   - PAGE_HEALTH_POPULATED_OUT: over a recorded genuine SDK snapshot read from
 *     PAGE_HEALTH_SNAPSHOT_IN (a fixture under rust/tests/fixtures/ui_bridge_sdk/).
 *   - PAGE_HEALTH_EMPTY_OUT: over `new UIBridgeRegistry().createSnapshot()` —
 *     an empty registry, zero elements and zero components.
 *   - PAGE_HEALTH_LAID_OUT_OUT: over the REAL `createSnapshot()` of a registry
 *     holding a laid-out page (sidebar, header, content cards, one of them
 *     still "Loading", one disabled button). jsdom has no layout engine, so the
 *     ONLY stub is each element's `getBoundingClientRect` (and the viewport
 *     size): visibility, `normalizedRect`, categories and every page-health
 *     check are the SDK's own code. This is the capture with VISIBLE,
 *     geometry-carrying elements — a measured report with `#` heatmap cells
 *     and a WARNING roll-up — which no recorded live snapshot provides.
 *
 * `evaluatedAt` is pinned (PAGE_HEALTH_EVALUATED_AT) so the capture is
 * reproducible apart from the empty registry's own `timestamp`/`snapshotId`.
 * The producer version is the SDK's own `__SDK_VERSION__` (vitest defines it
 * from package.json), i.e. the default producer, not an override.
 *
 * Usage (from a ui-bridge checkout with dependencies installed): copy this file
 * to packages/ui-bridge/src/server/, then from packages/ui-bridge run
 *   PAGE_HEALTH_SNAPSHOT_IN=<abs>.json PAGE_HEALTH_POPULATED_OUT=<abs>.json \
 *   PAGE_HEALTH_EMPTY_OUT=<abs>.json PAGE_HEALTH_LAID_OUT_OUT=<abs>.json \
 *   PAGE_HEALTH_EVALUATED_AT=<RFC3339> \
 *   npx vitest run src/server/page-health.capture.test.ts
 */
import { readFileSync, writeFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { UIBridgeRegistry } from '../core/registry';
import { diagnosePageHealth } from './page-health';

type RawSnapshot = {
  elements?: unknown;
  components?: unknown;
  timestamp?: number;
  snapshotId?: unknown;
};

/**
 * The handler's input mapping, verbatim: a copy of the `pageHealth` handler's
 * `diagnosePageHealth({...})` argument in packages/ui-bridge/src/server/
 * handlers.ts (lines 5429-5438 at d6ff394). Re-check it against the handler
 * before a re-capture from a later SDK commit.
 */
function answer(raw: RawSnapshot, evaluatedAt: string) {
  return diagnosePageHealth(
    {
      elements: raw.elements,
      registeredComponents: Array.isArray(raw.components) ? raw.components.length : null,
      observedAt: typeof raw.timestamp === 'number' ? raw.timestamp : null,
      source: typeof raw.snapshotId === 'string' ? { snapshotId: raw.snapshotId } : null,
    },
    { evaluatedAt }
  );
}

function wire(value: unknown): string {
  return JSON.stringify(JSON.parse(JSON.stringify(value)), null, 2) + '\n';
}

const VIEWPORT = { width: 1280, height: 800 };

/** A laid-out page; boxes are CSS pixels in a 1280x800 viewport. */
const LAID_OUT: Array<{
  id: string;
  tag: string;
  type: string;
  text: string;
  box: [number, number, number, number];
  disabled?: boolean;
}> = [
  { id: 'nav-home', tag: 'a', type: 'link', text: 'Home', box: [0, 60, 220, 40] },
  { id: 'nav-runs', tag: 'a', type: 'link', text: 'Runs', box: [0, 100, 220, 40] },
  { id: 'nav-settings', tag: 'a', type: 'link', text: 'Settings', box: [0, 140, 220, 40] },
  { id: 'page-title', tag: 'h1', type: 'heading', text: 'Workflow runs', box: [240, 0, 600, 48] },
  { id: 'card-recent', tag: 'section', type: 'container', text: 'Recent runs: 12 passed, 1 failed', box: [260, 80, 480, 260] },
  { id: 'card-queue', tag: 'section', type: 'container', text: 'Loading queue…', box: [760, 80, 480, 260] },
  { id: 'card-summary', tag: 'section', type: 'container', text: 'Summary for this week', box: [260, 360, 980, 300] },
  { id: 'btn-rerun', tag: 'button', type: 'button', text: 'Re-run', box: [260, 680, 120, 40], disabled: true },
  { id: 'btn-new', tag: 'button', type: 'button', text: 'New run', box: [400, 680, 120, 40] },
];

function laidOutSnapshot(): RawSnapshot {
  Object.defineProperty(window, 'innerWidth', { configurable: true, value: VIEWPORT.width });
  Object.defineProperty(window, 'innerHeight', { configurable: true, value: VIEWPORT.height });
  const registry = new UIBridgeRegistry();
  for (const spec of LAID_OUT) {
    const el = document.createElement(spec.tag);
    el.textContent = spec.text;
    if (spec.disabled) el.setAttribute('disabled', '');
    const [x, y, width, height] = spec.box;
    el.getBoundingClientRect = () =>
      ({
        x,
        y,
        width,
        height,
        top: y,
        left: x,
        right: x + width,
        bottom: y + height,
        toJSON: () => ({}),
      }) as DOMRect;
    document.body.appendChild(el);
    registry.registerElement(spec.id, el, { type: spec.type as never, label: spec.text });
  }
  return registry.createSnapshot() as RawSnapshot;
}

describe('page-health fixture capture', () => {
  it('writes diagnosePageHealth() wire bytes for a recorded, an empty and a laid-out registry', () => {
    const snapshotIn = process.env.PAGE_HEALTH_SNAPSHOT_IN;
    const populatedOut = process.env.PAGE_HEALTH_POPULATED_OUT;
    const emptyOut = process.env.PAGE_HEALTH_EMPTY_OUT;
    const laidOutOut = process.env.PAGE_HEALTH_LAID_OUT_OUT;
    const evaluatedAt = process.env.PAGE_HEALTH_EVALUATED_AT;
    expect(
      snapshotIn && populatedOut && emptyOut && laidOutOut && evaluatedAt,
      'all five env vars must be set'
    ).toBeTruthy();

    const populated = answer(JSON.parse(readFileSync(snapshotIn!, 'utf8')) as RawSnapshot, evaluatedAt!);
    expect(populated.status).toBe('measured');
    writeFileSync(populatedOut!, wire(populated));

    const empty = answer(new UIBridgeRegistry().createSnapshot() as RawSnapshot, evaluatedAt!);
    expect(empty.status).toBe('unknown');
    writeFileSync(emptyOut!, wire(empty));

    const laidOut = answer(laidOutSnapshot(), evaluatedAt!);
    expect(laidOut.status).toBe('measured');
    writeFileSync(laidOutOut!, wire(laidOut));
  });
});
