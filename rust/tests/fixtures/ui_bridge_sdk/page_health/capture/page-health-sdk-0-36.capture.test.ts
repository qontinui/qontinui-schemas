/**
 * Capture harness for the qontinui-schemas page-health fixtures named
 * sdk_0_36_0_page_health_*.json. Sibling of page-health.capture.test.ts, which
 * keeps reproducing the 0.30.0 captures at d6ff394 and fails on SDK >= 0.31.1: it
 * asserts that the recorded live snapshot is `measured`, and since 0.31.1
 * (ui-bridge 5c5d58f) that snapshot answers `unknown`.
 *
 * Not a test of this SDK. It drives the REAL producer through the call the
 * in-page handler itself makes at this SDK
 * (`pageHealth: async () => success(diagnoseSnapshotPageHealth(() =>
 * registry.createSnapshot()))`, packages/ui-bridge/src/server/handlers.ts), so
 * there is no restated input mapping to re-check, and writes the answer's wire
 * bytes (a JSON round-trip: what goes over HTTP as the `success` envelope's
 * `data`).
 *
 * Six captures, each a shape the 0.30.0 set cannot show or a re-record of one
 * it can:
 *   - ALL_INVISIBLE: over the recorded GENUINE live snapshot read from
 *     PAGE_HEALTH_SNAPSHOT_IN. Same input as the 0.30.0 `populated` capture,
 *     which rolled up CRITICAL; here it is `unknown{input_missing}`.
 *   - EMPTY_REGISTRY: over `new UIBridgeRegistry().createSnapshot()`.
 *   - LAID_OUT: over the REAL `createSnapshot()` of a laid-out registry. jsdom
 *     has no layout engine, so the only stubs are each element's
 *     `getBoundingClientRect` and the viewport size.
 *   - VISIBILITY_PARTLY_UNSTATED: the laid-out snapshot with `state.visible`
 *     deleted from one element. The producer still measures the rest and counts
 *     that one under `coverage.unmeasured[{dimension: "visibility"}]`, a
 *     dimension that does not exist at 0.30.0.
 *   - VISIBILITY_UNSTATED: the laid-out snapshot with `state.visible` deleted
 *     from every element, so nothing can be placed on screen:
 *     `unknown{input_missing}` that still carries that coverage.
 *   - ZERO_ELEMENTS_WITH_COMPONENTS: the recorded live snapshot with `elements`
 *     emptied and its components kept. The producer's `producer_not_run` branch
 *     needs zero components as well, so this one is still analyzed: a `measured`
 *     report over zero considered elements.
 *
 * ZERO_ELEMENTS_WITH_COMPONENTS and the two VISIBILITY captures edit the snapshot
 * and nothing else, and everything the producer writes is its own. The VISIBILITY
 * edits are needed because the core registry at this SDK always emits a boolean
 * `state.visible`; the producer's `visibilityStated` branch exists for a snapshot
 * that does not.
 *
 * The recorded snapshot is itself a whole-entry trim of a live page (3 of 160
 * elements, 2 of 6 components; see its own sidecar). What it shows is the
 * producer's answer over those entries, not the state of that live page.
 *
 * `evaluatedAt` is pinned (PAGE_HEALTH_EVALUATED_AT). The only bytes that vary
 * between runs are `observedAt` and `provenance.source.snapshotId`, which come
 * from the snapshot taken at capture; the two captures over the recorded snapshot
 * re-run byte-for-byte.
 *
 * Usage (from a ui-bridge checkout at the 0.36.0 publish commit, 76dc208, with
 * dependencies installed): copy this file to packages/ui-bridge/src/server/, then
 * from packages/ui-bridge run
 *   PAGE_HEALTH_SNAPSHOT_IN=<abs>.json PAGE_HEALTH_OUT_DIR=<abs dir> \
 *   PAGE_HEALTH_EVALUATED_AT=<RFC3339> \
 *   npx vitest run src/server/page-health-sdk-0-36.capture.test.ts
 */
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';
import { UIBridgeRegistry } from '../core/registry';
import { diagnoseSnapshotPageHealth } from './page-health';

type RawElement = { state?: Record<string, unknown> };
type RawSnapshot = { elements?: RawElement[] };

const PREFIX = 'sdk_0_36_0_page_health_';

function wire(value: unknown): string {
  return JSON.stringify(JSON.parse(JSON.stringify(value)), null, 2) + '\n';
}

const VIEWPORT = { width: 1280, height: 800 };

/** The same laid-out page as the 0.30.0 harness; boxes are CSS pixels. */
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
  {
    id: 'card-recent',
    tag: 'section',
    type: 'container',
    text: 'Recent runs: 12 passed, 1 failed',
    box: [260, 80, 480, 260],
  },
  {
    id: 'card-queue',
    tag: 'section',
    type: 'container',
    text: 'Loading queue…',
    box: [760, 80, 480, 260],
  },
  {
    id: 'card-summary',
    tag: 'section',
    type: 'container',
    text: 'Summary for this week',
    box: [260, 360, 980, 300],
  },
  {
    id: 'btn-rerun',
    tag: 'button',
    type: 'button',
    text: 'Re-run',
    box: [260, 680, 120, 40],
    disabled: true,
  },
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
  return registry.createSnapshot() as unknown as RawSnapshot;
}

/** A deep copy with `state.visible` deleted from the first `count` elements. */
function withoutVisibility(snapshot: RawSnapshot, count: number): RawSnapshot {
  const copy = JSON.parse(JSON.stringify(snapshot)) as RawSnapshot;
  for (const el of (copy.elements ?? []).slice(0, count)) {
    delete el.state?.visible;
  }
  return copy;
}

describe('page-health fixture capture, SDK 0.36.0', () => {
  it('writes the producer wire bytes for every shape', () => {
    const snapshotIn = process.env.PAGE_HEALTH_SNAPSHOT_IN;
    const outDir = process.env.PAGE_HEALTH_OUT_DIR;
    const evaluatedAt = process.env.PAGE_HEALTH_EVALUATED_AT;
    expect(snapshotIn && outDir && evaluatedAt, 'all three env vars must be set').toBeTruthy();
    mkdirSync(outDir!, { recursive: true });

    const capture = (name: string, status: string, takeSnapshot: () => unknown) => {
      const answer = diagnoseSnapshotPageHealth(takeSnapshot, { evaluatedAt: evaluatedAt! });
      expect(answer.status, name).toBe(status);
      writeFileSync(join(outDir!, `${PREFIX}${name}.json`), wire(answer));
    };

    const recorded = JSON.parse(readFileSync(snapshotIn!, 'utf8')) as RawSnapshot;
    capture('all_invisible', 'unknown', () => recorded);
    capture('zero_elements_with_components', 'measured', () => ({ ...recorded, elements: [] }));
    capture('empty_registry', 'unknown', () => new UIBridgeRegistry().createSnapshot());

    const laidOut = laidOutSnapshot();
    capture('laid_out', 'measured', () => laidOut);
    capture('visibility_partly_unstated', 'measured', () => withoutVisibility(laidOut, 1));
    capture('visibility_unstated', 'unknown', () =>
      withoutVisibility(laidOut, laidOut.elements?.length ?? 0)
    );
  });
});
